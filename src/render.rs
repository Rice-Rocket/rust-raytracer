use image::{self, ImageBuffer};
use indicatif::{ProgressBar, ProgressStyle};
#[path = "camera.rs"] mod camera;
pub use camera::*;


pub fn ray_color(r: Ray, scene: &SceneColliders, depth: usize) -> Rgb {
    if depth <= 0 {
        return Rgb::origin();
    }

    let hit = scene.intersect(r, 0.001, f32::MAX);
    match hit {
        Some((mut rec, mat_idx)) => {
            let mut scattered = Ray::new(Vec3::origin(), Vec3::origin());
            let mut attenuation = Vec3::origin();
            if scene.materials[mat_idx].scatter(r, &mut attenuation, &mut rec, &mut scattered) {
                return attenuation * ray_color(scattered, scene, depth - 1);
            }
            return Rgb::origin();
        },
        None => {}
    };

    let norm = r.direction.normalize();
    let t = 0.5 * (norm.y + 1.0);
    return Rgb::new(1.0, 1.0, 1.0) * (1.0 - t) + Rgb::new(0.5, 0.7, 1.0) * t;
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


pub fn render(mut imgbuf: &mut ImageBuffer<image::Rgb<u8>, Vec<u8>>, scene: &SceneColliders, cam: &Camera, max_depth: usize, samples_per_pixel: usize) {
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
                pixel_color = pixel_color + ray_color(r, &scene, max_depth);
            }

            write_color(&mut imgbuf, i, img_height - j - 1, pixel_color, samples_per_pixel);
            bar.inc(1);
        }
        bar.set_message(format!("{}", j));
    }
    bar.finish_and_clear();
}