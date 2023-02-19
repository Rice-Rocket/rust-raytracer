use std::ops::{Add, Sub, Mul, Div, Neg, Index, IndexMut};
#[path = "utils.rs"] mod utils;
pub use utils::*;



#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, scaler: f32) -> Self::Output {
        Self {
            x: self.x * scaler,
            y: self.y * scaler,
            z: self.z * scaler
        }
    }
}

impl Mul for Vec3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, scaler: f32) -> Self::Output {
        Self {
            x: self.x / scaler,
            y: self.y / scaler,
            z: self.z / scaler
        }
    }
}

impl Div for Vec3 {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Cannot index into Vec3 with {}", index)
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Cannot index into Vec3 with {}", index)
        }
    }
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x,
            y: y,
            z: z
        }
    }
    pub fn origin() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    }
    pub fn random() -> Self {
        Self {
            x: random(),
            y: random(),
            z: random()
        }
    }
    pub fn randrange(min: f32, max: f32) -> Self {
        Self {
            x: randrange(min, max),
            y: randrange(min, max),
            z: randrange(min, max)
        }
    }
    pub fn set_to(&mut self, other: Self) {
        self.x = other.x;
        self.y = other.y;
        self.z = other.z;
    }
    pub fn near_zero(&self) -> bool {
        let s = 0.00000001;
        (self.x.abs() < s) && (self.y.abs() < s) && (self.z.abs() < s)
    }
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }
    pub fn normalize(&self) -> Self {
        self.clone() / self.length()
    }
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x
        }
    }
}


pub type Rgb = Vec3;
pub type Point3 = Vec3;


pub fn random_in_unit_sphere() -> Vec3 {
    loop {
        let p = Vec3::randrange(-1.0, 1.0);
        if p.length_squared() >= 1.0 {
            continue;
        }
        return p;
    }
}

pub fn random_unit_vec3() -> Vec3 {
    random_in_unit_sphere().normalize()
}

pub fn random_in_hemisphere(normal: Vec3) -> Vec3 {
    let in_unit_sphere = random_in_unit_sphere();
    if in_unit_sphere.dot(normal) > 0.0 {
        return in_unit_sphere;
    }
    else {
        return -in_unit_sphere;
    }
}

pub fn random_in_unit_disk() -> Vec3 {
    loop {
        let p = Vec3::new(randrange(-1.0, 1.0), randrange(-1.0, 1.0), 0.0);
        if p.length_squared() >= 1.0 {
            continue;
        }
        return p;
    }
}

pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - n * 2.0 * v.dot(n)
}

pub fn refract(uv: Vec3, n: Vec3, etai_over_etat: f32) -> Vec3 {
    let cos_theta = -uv.dot(n).min(1.0);
    let r_out_perp = (uv + n * cos_theta) * etai_over_etat;
    let r_out_parralel = n * -((1.0 - r_out_perp.length_squared()).abs().sqrt());
    return r_out_perp + r_out_parralel;
}