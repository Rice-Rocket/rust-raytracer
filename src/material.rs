#[path = "texture.rs"] mod texture;
pub use texture::*;
use std::{sync::{Arc, Mutex}, fs::DirEntry};



#[derive(Clone)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Material,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub front_face: bool
}

impl HitRecord {
    pub fn new(point: Point3, t: f32) -> Self {
        Self {
            point: point,
            normal: Vec3::new(0.0, 0.0, 0.0),
            material: Material::lambertian(Texture::solid_color(Rgb::origin())),
            t: t,
            u: 0.0,
            v: 0.0,
            front_face: false
        }
    }
    pub fn set_face_normal(&mut self, r: Ray, out_normal: Vec3) {
        self.front_face = r.direction.dot(out_normal) < 0.0;
        self.normal = if self.front_face { out_normal } else { -out_normal };
    }
    pub fn set_uv(&mut self, uv: (f32, f32)) {
        self.u = uv.0;
        self.v = uv.1;
    }
}


#[derive(Clone)]
pub struct ONB {
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3
}

impl ONB {
    pub fn build_from_w(n: Vec3) -> Self {
        let w = n.normalize();
        let a = if w.x.abs() > 0.9 { Vec3::new(0., 1., 0.) } else { Vec3::new(1., 0., 0.) };
        let v = w.cross(a).normalize();
        let u = w.cross(v);
        Self {
            u: u,
            v: v,
            w: w
        }
    }
    pub fn local(&self, a: Vec3) -> Vec3 {
        self.u * a.x + self.v * a.y + self.w * a.z
    }
}



#[derive(Clone)]
pub enum MaterialType {
    Lambertian(Texture),
    Glossy(Rgb, f32),
    Dielectric(f32),
    Isotropic(Texture),
    Emissive(Texture)
}


#[derive(Clone)]
pub struct Material {
    pub mat_type: MaterialType,
}

impl Material {
    pub fn lambertian(albedo: Texture) -> Self {
        Self {
            mat_type: MaterialType::Lambertian(albedo),
        }
    }
    pub fn glossy(albedo: Rgb, fuzz: f32) -> Self {
        Self {
            mat_type: MaterialType::Glossy(albedo, if fuzz < 1.0 {fuzz} else {1.0}),
        }
    }
    pub fn dielectric(refraction_index: f32) -> Self {
        Self {
            mat_type: MaterialType::Dielectric(refraction_index),
        }
    }
    pub fn isotropic(color: Rgb) -> Self {
        Self {
            mat_type: MaterialType::Isotropic(Texture::solid_color(color)),
        }
    }
    pub fn emissive(color: Rgb) -> Self {
        Self {
            mat_type: MaterialType::Emissive(Texture::solid_color(color)),
        }
    }

    
    fn scatter_lambertian(&self, albedo: &Texture, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray, pdf: &mut f32, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> bool {
        let uvw = ONB::build_from_w(rec.normal);
        let dir = uvw.local(random_cosin_direction());
        scattered.reset(rec.point, dir.normalize(), r_in.time);
        attenuation.set_to(albedo.get_color(rec.u, rec.v, rec.point, atlas));
        *pdf = uvw.w.dot(scattered.direction) / PI;
        return true;
    }
    // fn scatter_lambertian(&self, albedo: &Texture, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray, pdf: &mut f32, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> bool {
    //     let mut scatter_dir = rec.normal + random_unit_vec3();
    //     if scatter_dir.near_zero() {
    //         scatter_dir = rec.normal;
    //     }
        
    //     scattered.reset(rec.point, scatter_dir.normalize(), r_in.time);
    //     attenuation.set_to(albedo.get_color(rec.u, rec.v, rec.point, atlas));
    //     *pdf = rec.normal.dot(scattered.direction) / PI;
    //     return true;
    // }
    // fn scatter_lambertian(&self, albedo: &Texture, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray, pdf: &mut f32, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> bool {
    //     let scatter_dir = random_in_hemisphere(rec.normal);
    //     scattered.reset(rec.point, scatter_dir.normalize(), r_in.time);
    //     attenuation.set_to(albedo.get_color(rec.u, rec.v, rec.point, atlas));
    //     *pdf = 0.5 / PI;
    //     return true;
    // }
    fn scattering_pdf_lambertian(&self, r_in: Ray, rec: HitRecord, scattered: &mut Ray) -> f32 {
        let cosin = rec.normal.dot(scattered.direction.normalize());
        return if cosin < 0.0 { 0.0 } else { cosin / PI };
    }

    fn reflectance(&self, cosin: f32, ref_idx: f32) -> f32 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        return r0 + (1.0 - r0) * (1.0 - cosin).powi(5);
    }
    fn scatter_glossy(&self, color: &Rgb, fuzz: &f32, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        let reflected = reflect(r_in.direction.normalize(), rec.normal);
        scattered.reset(rec.point, reflected + random_in_unit_sphere() * *fuzz, r_in.time);
        attenuation.set_to(*color);
        return scattered.direction.dot(rec.normal) > 0.0;
    }
    
    fn scatter_dielectric(&self, refraction_index: &f32, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        attenuation.set_to(Vec3::new(1.0, 1.0, 1.0));
        let refraction_ratio = if rec.front_face { 1.0 / *refraction_index } else { *refraction_index };

        let unit_dir = r_in.direction.normalize();
        let cos_theta = (-unit_dir).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let dir;

        if cannot_refract || (self.reflectance(cos_theta, refraction_ratio) > random()) {
            dir = reflect(unit_dir, rec.normal);
        }
        else {
            dir = refract(unit_dir, rec.normal, refraction_ratio);
        }

        scattered.reset(rec.point, dir, r_in.time);
        return true;
    }

    fn scatter_isotropic(&self, albedo: &Texture, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> bool {
        scattered.reset(rec.point, random_in_unit_sphere(), r_in.time);
        attenuation.set_to(albedo.get_color(rec.u, rec.v, rec.point, atlas));
        return true;
    }

    fn scatter_emissive(&self, _r_in: Ray, _attenuation: &mut Rgb, _rec: HitRecord, _scattered: &mut Ray) -> bool {
        return false;
    }


    pub fn scatter(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray, pdf: &mut f32, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> bool {
        match &self.mat_type {
            MaterialType::Lambertian(albedo) => self.scatter_lambertian(albedo, r_in, attenuation, rec, scattered, pdf, atlas),
            MaterialType::Glossy(color, fuzz) => self.scatter_glossy(color, fuzz, r_in, attenuation, rec, scattered),
            MaterialType::Dielectric(refraction_index) => self.scatter_dielectric(refraction_index, r_in, attenuation, rec, scattered),
            MaterialType::Isotropic(albedo) => self.scatter_isotropic(albedo, r_in, attenuation, rec, scattered, atlas),
            MaterialType::Emissive(_albedo) => self.scatter_emissive(r_in, attenuation, rec, scattered)
        }
    }
    pub fn scattering_pdf(&self, r_in: Ray, rec: HitRecord, scattered: &mut Ray) -> f32 {
        match &self.mat_type {
            MaterialType::Lambertian(albedo) => self.scattering_pdf_lambertian(r_in, rec, scattered),
            _ => 0.0
        }
    }
    pub fn emitted(&self, u: f32, v: f32, point: Point3, atlas: &Arc<Mutex<ImageTextureAtlas>>) -> Rgb {
        match &self.mat_type {
            MaterialType::Emissive(albedo) => albedo.get_color(u, v, point, atlas),
            _ => Rgb::origin()
        }
    }
}

