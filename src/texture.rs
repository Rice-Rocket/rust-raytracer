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
    SolidColor,
    Checkered,
    NoiseTexture,
    ImageTexture
}

#[derive(Clone)]
pub struct Texture {
    pub texture_type: TextureType,

    pub color: Option<Rgb>,
    pub color2: Option<Rgb>,

    pub noise: Option<Perlin>,
    pub scale: Option<f32>,
    pub turb: Option<usize>,
    
    pub bytes_per_scanline: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub img_data_idx: Option<usize>
}

impl Texture {
    pub fn solid_color(color: Rgb) -> Self {
        Self {
            texture_type: TextureType::SolidColor,
            color: Some(color),
            color2: None,
            noise: None,
            scale: None,
            turb: None,
            bytes_per_scanline: None,
            width: None,
            height: None,
            img_data_idx: None
        }
    }
    pub fn checkered(odd_color: Rgb, even_color: Rgb) -> Self {
        Self {
            texture_type: TextureType::Checkered,
            color: Some(odd_color),
            color2: Some(even_color),
            noise: None,
            scale: None,
            turb: None,
            bytes_per_scanline: None,
            width: None,
            height: None,
            img_data_idx: None
        }
    }
    pub fn noise(scale: f32, turb: usize) -> Self {
        Self {
            texture_type: TextureType::NoiseTexture,
            color: None,
            color2: None,
            noise: Some(Perlin::new()),
            scale: Some(scale),
            turb: Some(turb),
            bytes_per_scanline: None,
            width: None,
            height: None,
            img_data_idx: None
        }
    }
    pub fn image(atlas: &Arc<Mutex<ImageTextureAtlas>>, img_data_idx: usize) -> Self {
        let bytes_per_pixel = 3;
        let (width, height) = atlas.lock().unwrap().images[img_data_idx].dimensions();
        let bytes_per_scanline = bytes_per_pixel * width;

        Self {
            texture_type: TextureType::ImageTexture,
            color: None,
            color2: None,
            noise: None,
            scale: None,
            turb: None,
            bytes_per_scanline: Some(bytes_per_scanline),
            width: Some(width),
            height: Some(height),
            img_data_idx: Some(img_data_idx)
        }
    }

    fn get_solid_color(&self, _u: f32, _v: f32, _point: Point3) -> Rgb {
        self.color.unwrap()
    }
    fn get_checkered_color(&self, _u: f32, _v: f32, point: Point3) -> Rgb {
        let sines = (10.0 * point.x).sin() * (10.0 * point.y).sin() * (10.0 * point.z).sin();
        if sines < 0.0 {
            self.color.unwrap()
        }
        else {
            self.color2.unwrap()
        }
    }
    fn get_noise_color(&self, _u: f32, _v: f32, point: Point3) -> Rgb {
        Rgb::new(1.0, 1.0, 1.0) * 0.5 * (1.0 + (self.scale.unwrap() * point.z + 10.0 * self.noise.as_ref().unwrap().turb(point, self.turb.unwrap())).sin())
    }
    // fn get_noise_color(&self, _u: f32, _v: f32, point: Point3) -> Rgb {
    //     Rgb::new(1.0, 1.0, 1.0) * self.noise.turb(point * self.scale, self.turb)
        // }
    fn get_image_color(&self, u_in: f32, v_in: f32, _point: Point3, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> Rgb {
        let u = clamp(u_in, 0.0, 1.0);
        let v = 1.0 - clamp(v_in, 0.0, 1.0);
        let mut i = (u * self.width.unwrap() as f32) as u32;
        let mut j = (v * self.height.unwrap() as f32) as u32;
        if i >= self.width.unwrap() {i = self.width.unwrap() - 1};
        if j >= self.height.unwrap() {j = self.height.unwrap() - 1};
        let color_scale = 1.0 / 255.0;
        let pixel = atlas.lock().unwrap().images[self.img_data_idx.unwrap()].get_pixel(i, j);
        return Rgb::new(pixel.0[0] as f32 * color_scale, pixel.0[1] as f32 * color_scale, pixel.0[2] as f32 * color_scale);
    }

    pub fn get_color(&self, u: f32, v: f32, point: Point3, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> Rgb {
        match self.texture_type {
            TextureType::SolidColor => self.get_solid_color(u, v, point),
            TextureType::Checkered => self.get_checkered_color(u, v, point),
            TextureType::NoiseTexture => self.get_noise_color(u, v, point),
            TextureType::ImageTexture => self.get_image_color(u, v, point, atlas),
        }
    }
}


