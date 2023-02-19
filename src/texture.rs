#[path = "perlin.rs"] mod perlin;
pub use perlin::*;

use std::{fs::File, path::Path, io::BufReader};

use image::{self, GenericImageView, DynamicImage};


pub trait Texture {
    fn get_color(&self, u: f32, v: f32, point: Point3) -> Rgb;
}

pub struct SolidColor {
    pub color: Rgb
}

impl SolidColor {
    pub fn new(color: Rgb) -> Self {
        Self {
            color: color
        }
    }
}

impl Texture for SolidColor {
    fn get_color(&self, _u: f32, _v: f32, _point: Point3) -> Rgb {
        self.color
    }
}


pub struct Checkered {
    pub odd_color: Rgb,
    pub even_color: Rgb
}

impl Checkered {
    pub fn new(odd: Rgb, even: Rgb) -> Self {
        Self {
            odd_color: odd,
            even_color: even
        }
    }
}

impl Texture for Checkered {
    fn get_color(&self, u: f32, v: f32, point: Point3) -> Rgb {
        let sines = (10.0 * point.x).sin() * (10.0 * point.y).sin() * (10.0 * point.z).sin();
        if sines < 0.0 {
            self.odd_color
        }
        else {
            self.even_color
        }
    }
}


pub struct NoiseTexture {
    noise: Perlin,
    pub scale: f32,
    pub turb: usize
}

impl NoiseTexture {
    pub fn new(scale: f32, turb: usize) -> Self {
        Self {
            noise: Perlin::new(),
            scale: scale,
            turb: turb
        }
    }
}

impl Texture for NoiseTexture {
    fn get_color(&self, _u: f32, _v: f32, point: Point3) -> Rgb {
        Rgb::new(1.0, 1.0, 1.0) * 0.5 * (1.0 + (self.scale * point.z + 10.0 * self.noise.turb(point, self.turb)).sin())
    }
    // fn get_color(&self, _u: f32, _v: f32, point: Point3) -> Rgb {
    //     Rgb::new(1.0, 1.0, 1.0) * self.noise.turb(point * self.scale, self.turb)
    // }
}


pub struct ImageTexture {
    pub bytes_per_scanline: u32,
    pub width: u32,
    pub height: u32,
    pub data: DynamicImage
}

impl ImageTexture {
    pub fn load(path: &str) -> Self {
        let bytes_per_pixel = 3;
        let data = image::open(&Path::new(&path)).unwrap();
        let (width, height) = data.dimensions();
        let bytes_per_scanline = bytes_per_pixel * width;
        Self {
            bytes_per_scanline: bytes_per_scanline,
            data: data,
            width: width,
            height: height
        }
    }
}

impl Texture for ImageTexture {
    fn get_color(&self, u_in: f32, v_in: f32, _point: Point3) -> Rgb {
        let u = clamp(u_in, 0.0, 1.0);
        let v = 1.0 - clamp(v_in, 0.0, 1.0);
        let mut i = (u * self.width as f32) as u32;
        let mut j = (v * self.height as f32) as u32;
        if i >= self.width {i = self.width - 1};
        if j >= self.height {j = self.height - 1};
        let color_scale = 1.0 / 255.0;
        let pixel = self.data.get_pixel(i, j);
        return Rgb::new(pixel.0[0] as f32 * color_scale, pixel.0[1] as f32 * color_scale, pixel.0[2] as f32 * color_scale);
    }
}