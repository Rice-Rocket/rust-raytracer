#[path = "perlin.rs"] mod perlin;
pub use perlin::*;

use std::{fs::File, path::Path, io::BufReader, sync::Arc, sync::Mutex};

use image::{self, GenericImageView, DynamicImage};


pub struct ImageTextureAtlas {
    pub images: Vec<DynamicImage>,
}

impl ImageTextureAtlas {
    pub fn new() -> Self {
        Self {
            images: Vec::new()
        }
    }
    pub fn load(&mut self, path: &str) -> usize {
        self.images.push(image::open(&Path::new(&path)).unwrap());
        return self.images.len() - 1;
    }
}



#[derive(Clone)]
pub enum TextureType {
    SolidColor(Rgb),
    Checkered(Rgb, Rgb),
    NoiseTexture(Perlin, f32, usize),
    ImageTexture(u32, u32, u32, usize),
}

#[derive(Clone)]
pub struct Texture {
    pub texture_type: TextureType,
}

impl Texture {
    pub fn solid_color(color: Rgb) -> Self {
        Self {
            texture_type: TextureType::SolidColor(color),
        }
    }
    pub fn checkered(odd_color: Rgb, even_color: Rgb) -> Self {
        Self {
            texture_type: TextureType::Checkered(odd_color, even_color),
        }
    }
    pub fn noise(scale: f32, turb: usize) -> Self {
        Self {
            texture_type: TextureType::NoiseTexture(Perlin::new(), scale, turb),
        }
    }
    pub fn image(atlas: &Arc<Mutex<ImageTextureAtlas>>, img_data_idx: usize) -> Self {
        let bytes_per_pixel = 3;
        let (width, height) = atlas.lock().unwrap().images[img_data_idx].dimensions();
        let bytes_per_scanline = bytes_per_pixel * width;

        Self {
            texture_type: TextureType::ImageTexture(bytes_per_pixel, width, height, img_data_idx),
        }
    }

    fn get_solid_color(&self, color: &Rgb, _u: f32, _v: f32, _point: Point3) -> Rgb {
        *color
    }
    fn get_checkered_color(&self, odd_color: &Rgb, even_color: &Rgb, _u: f32, _v: f32, point: Point3) -> Rgb {
        let sines = (10.0 * point.x).sin() * (10.0 * point.y).sin() * (10.0 * point.z).sin();
        if sines < 0.0 {
            *odd_color
        }
        else {
            *even_color
        }
    }
    fn get_noise_color(&self, noise: &Perlin, scale: &f32, turb: &usize, _u: f32, _v: f32, point: Point3) -> Rgb {
        Rgb::new(1.0, 1.0, 1.0) * 0.5 * (1.0 + (*scale * point.z + 10.0 * noise.turb(point, *turb)).sin())
    }
    // fn get_noise_color(&self, _u: f32, _v: f32, point: Point3) -> Rgb {
    //     Rgb::new(1.0, 1.0, 1.0) * self.noise.turb(point * self.scale, self.turb)
        // }
    fn get_image_color(&self, width: &u32, height: &u32, img_data_idx: &usize, u_in: f32, v_in: f32, _point: Point3, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> Rgb {
        let u = clamp(u_in, 0.0, 1.0);
        let v = 1.0 - clamp(v_in, 0.0, 1.0);
        let mut i = (u * *width as f32) as u32;
        let mut j = (v * *height as f32) as u32;
        if i >= *width {i = *width - 1};
        if j >= *height {j = *height - 1};
        let color_scale = 1.0 / 255.0;
        let pixel = atlas.lock().unwrap().images[*img_data_idx].get_pixel(i, j);
        return Rgb::new(pixel.0[0] as f32 * color_scale, pixel.0[1] as f32 * color_scale, pixel.0[2] as f32 * color_scale);
    }

    pub fn get_color(&self, u: f32, v: f32, point: Point3, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> Rgb {
        match &self.texture_type {
            TextureType::SolidColor(color) => self.get_solid_color(color, u, v, point),
            TextureType::Checkered(odd_color, even_color) => self.get_checkered_color(odd_color, even_color, u, v, point),
            TextureType::NoiseTexture(noise, scale, turb) => self.get_noise_color(noise, scale, turb, u, v, point),
            TextureType::ImageTexture(_bytes_per_scanline, width, height, img_data_idx) => self.get_image_color(width, height, img_data_idx, u, v, point, atlas),
        }
    }
}


