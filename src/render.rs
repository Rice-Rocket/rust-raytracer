use image::{self, ImageBuffer};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::SystemTime;
use std::sync::{Arc, Mutex};
use linya::{Bar, Progress};
use num_cpus;
use math::round::half_up;
#[path = "camera.rs"] mod camera;
pub use camera::*;
use rayon::prelude::*;



pub fn ray_color(r: Ray, background: Rgb, scene: &SceneColliders, depth: usize) -> Rgb {
    if depth <= 0 {
        return Rgb::origin();
    }

    let hit = scene.intersect(r, 0.001, f32::MAX);
    match hit {
        Some(rec) => {
            let mut scattered = Ray::new(Vec3::origin(), Vec3::origin(), 0.0);
            let mut attenuation = Vec3::origin();
            let emitted = rec.material.emitted(rec.u, rec.v, rec.point, &scene.atlas);
            match rec.material.scatter(r, &mut attenuation, rec.clone(), &mut scattered, &scene.atlas) {
                true => return emitted + attenuation * ray_color(scattered, background, scene, depth - 1),
                false => return emitted
            }
        },
        None => { return background }
    };
}

pub fn write_color(imbuf: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>, x: u32, y: u32, pixel_color: Rgb, samples_per_pixel: usize) {
    let mut r = pixel_color.x;
    let mut g = pixel_color.y;
    let mut b  = pixel_color.z;
    
    let scale = 1.0 / samples_per_pixel as f32;
    r = (scale * r).sqrt();
    g = (scale * g).sqrt();
    b = (scale * b).sqrt();
    
    let ir = (256.0 * clamp(r, 0.0, 0.999)) as u8;
    let ig = (256.0 * clamp(g, 0.0, 0.999)) as u8;
    let ib = (256.0 * clamp(b, 0.0, 0.999)) as u8;

    imbuf.put_pixel(x, y, image::Rgb([ir, ig, ib]));
}

pub fn render(mut imgbuf: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>, scene: &SceneColliders, cam: &Camera, background: Rgb, max_depth: usize, samples_per_pixel: usize) {
    let img_width = imgbuf.width();
    let img_height = imgbuf.height();
    let bar = ProgressBar::new(img_width as u64 * img_height as u64);
    bar.set_style(ProgressStyle::with_template(
        "Rendering... | {percent}% | Elapsed: {elapsed} | ETA: {eta} | Scanlines Remaining: {msg}\n{bar:100.white/white}"
    ).unwrap());
    
    for j in (0..img_height).rev() {
        for i in 0..img_width {
            let mut pixel_color = Rgb::origin();
            
            for _ in 0..samples_per_pixel {
                let u = (i as f32 + random()) / (img_width as f32 - 1.0);
                let v = (j as f32 + random()) / (img_height as f32 - 1.0);
                let r = cam.get_ray(u, v);
                pixel_color = pixel_color + ray_color(r, background, &scene, max_depth);
            }

            write_color(&mut imgbuf, i, img_height - j - 1, pixel_color, samples_per_pixel);
            bar.inc(1);
        }
        bar.set_message(format!("{}", j));
    }
    bar.finish();
}



pub fn render_worker(background: Rgb, img_width: u32, img_height: u32, samples_per_thread: usize, thread_cam: &Camera, thread_scene: &SceneColliders, max_depth: usize, n_threads: usize, bar: &Bar, progress: &Mutex<Progress>) -> ImageBuffer<image::Rgb<f32>, Vec<f32>> {
    let mut subimage = ImageBuffer::new(img_width, img_height);
    for j in (0..img_height).rev() {
        for i in 0..img_width {
            let mut pixel_color = Rgb::origin();
            
            for _ in 0..samples_per_thread {
                let u = (i as f32 + random()) / (img_width as f32 - 1.0);
                let v = (j as f32 + random()) / (img_height as f32 - 1.0);
                let r = thread_cam.get_ray(u, v);
                pixel_color = pixel_color + ray_color(r, background, &thread_scene, max_depth);
            }

            let mut r = pixel_color.x;
            let mut g = pixel_color.y;
            let mut b  = pixel_color.z;
            
            let scale = 1.0 / samples_per_thread as f32;
            r = (scale * r).sqrt();
            g = (scale * g).sqrt();
            b = (scale * b).sqrt();
            
            let ir = 256.0 * clamp(r, 0.0, 0.999);
            let ig = 256.0 * clamp(g, 0.0, 0.999);
            let ib = 256.0 * clamp(b, 0.0, 0.999);

            subimage.put_pixel(i, img_height - j - 1, image::Rgb([ir / (n_threads - 1) as f32, ig / (n_threads - 1) as f32, ib / (n_threads - 1) as f32]));
        }
        progress.lock().unwrap().inc_and_draw(bar, 1);
    }
    return subimage;
}



pub fn render_multi(scene: SceneColliders, cam: Camera, background: Rgb, max_depth: usize, samples_per_pixel: usize, img_width: u32, img_height: u32) -> ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let n_threads = num_cpus::get();
    let samples_per_thread = samples_per_pixel / n_threads;
    let global_buf: Mutex<ImageBuffer<image::Rgb<f32>, Vec<f32>>> = Mutex::new(ImageBuffer::new(img_width, img_height));
    let time_start = SystemTime::now();
    let progress = Mutex::new(Progress::new());

    (0..n_threads).into_par_iter().for_each(|i| {
        let bar = progress.lock().unwrap().bar(img_height as usize, format!("Rendering [Thread {}]", i + 1));
        let subimage = render_worker(
            background, img_width, img_height, samples_per_thread, &cam, 
            &scene, max_depth, n_threads, &bar, &progress
        );
        let mut imgdata = global_buf.lock().unwrap();
        for j in 0..img_width {
            for k in 0..img_height {
                let p = imgdata.get_pixel_mut(j, k);
                let pix = *subimage.get_pixel(j, k);
                p.0[0] += pix.0[0];
                p.0[1] += pix.0[1];
                p.0[2] += pix.0[2];
            }
        }
    });

    let imgbuf_f32 = global_buf.lock().unwrap().clone();
    let mut finalbuf = ImageBuffer::new(img_width, img_height);

    for (x, y, pixel) in finalbuf.enumerate_pixels_mut() {
        let pix = imgbuf_f32.get_pixel(x, y);
        *pixel = image::Rgb([pix.0[0] as u8, pix.0[1] as u8, pix.0[2] as u8]);
    }

    let render_time = SystemTime::now().duration_since(time_start).unwrap().as_secs();
    println!("\nRendered in {}s ({}m)", render_time, half_up(render_time as f64 / 60.0, 2));

    return finalbuf;
}




pub fn render_frame_worker(background: Rgb, img_width: u32, img_height: u32, samples_per_thread: usize, thread_cam: &Camera, thread_scene: &SceneColliders, max_depth: usize, n_threads: usize) -> ImageBuffer<image::Rgb<f32>, Vec<f32>> {
    let mut subimage = ImageBuffer::new(img_width, img_height);
    for j in (0..img_height).rev() {
        for i in 0..img_width {
            let mut pixel_color = Rgb::origin();
            
            for _ in 0..samples_per_thread {
                let u = (i as f32 + random()) / (img_width as f32 - 1.0);
                let v = (j as f32 + random()) / (img_height as f32 - 1.0);
                let r = thread_cam.get_ray(u, v);
                pixel_color = pixel_color + ray_color(r, background, &thread_scene, max_depth);
            }

            let mut r = pixel_color.x;
            let mut g = pixel_color.y;
            let mut b  = pixel_color.z;
            
            let scale = 1.0 / samples_per_thread as f32;
            r = (scale * r).sqrt();
            g = (scale * g).sqrt();
            b = (scale * b).sqrt();
            
            let ir = clamp(r, 0.0, 0.999);
            let ig = clamp(g, 0.0, 0.999);
            let ib = clamp(b, 0.0, 0.999);

            subimage.put_pixel(i, img_height - j - 1, image::Rgb([ir / n_threads as f32, ig / n_threads as f32, ib / n_threads as f32]));
        }
    }
    return subimage;
}


pub fn render_frame_multi(scene: SceneColliders, cam: Camera, background: Rgb, max_depth: usize, samples_per_pixel: usize, img_width: u32, img_height: u32) -> ImageBuffer<image::Rgb<f32>, Vec<f32>> {
    let n_threads = num_cpus::get();
    let samples_per_thread = samples_per_pixel / n_threads;
    let global_buf: Mutex<ImageBuffer<image::Rgb<f32>, Vec<f32>>> = Mutex::new(ImageBuffer::new(img_width, img_height));

    (0..n_threads).into_par_iter().for_each(|i| {
        let subimage = render_frame_worker(
            background, img_width, img_height, samples_per_thread, &cam, 
            &scene, max_depth, n_threads
        );
        let mut imgdata = global_buf.lock().unwrap();
        for j in 0..img_width {
            for k in 0..img_height {
                let p = imgdata.get_pixel_mut(j, k);
                let pix = *subimage.get_pixel(j, k);
                p.0[0] += pix.0[0];
                p.0[1] += pix.0[1];
                p.0[2] += pix.0[2];
            }
        }
    });

    let imgbuf_f32 = global_buf.lock().unwrap().clone();
    return imgbuf_f32;
}