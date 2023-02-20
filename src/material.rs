#[path = "texture.rs"] mod texture;
pub use texture::*;
use std::sync::Arc;



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
pub enum MaterialType {
    Lambertian,
    Glossy,
    Dielectric,
    Isotropic,
    Emissive
}


#[derive(Clone)]
pub struct Material {
    pub mat_type: MaterialType,

    pub albedo: Option<Texture>,

    pub color: Option<Rgb>,
    pub fuzz: Option<f32>,

    pub refraction_index: Option<f32>,
}

impl Material {
    pub fn lambertian(albedo: Texture) -> Self {
        Self {
            mat_type: MaterialType::Lambertian,
            albedo: Some(albedo),
            color: None,
            fuzz: None,
            refraction_index: None,
        }
    }
    pub fn glossy(albedo: Rgb, fuzz: f32) -> Self {
        Self {
            mat_type: MaterialType::Glossy,
            albedo: None,
            color: Some(albedo),
            fuzz: Some(if fuzz < 1.0 {fuzz} else {1.0}),
            refraction_index: None
        }
    }
    pub fn dielectric(refraction_index: f32) -> Self {
        Self {
            mat_type: MaterialType::Dielectric,
            albedo: None,
            color: None,
            fuzz: None,
            refraction_index: Some(refraction_index)
        }
    }
    pub fn isotropic(color: Rgb) -> Self {
        Self {
            mat_type: MaterialType::Isotropic,
            albedo: Some(Texture::solid_color(color)),
            color: None,
            fuzz: None,
            refraction_index: None
        }
    }
    pub fn emissive(color: Rgb) -> Self {
        Self {
            mat_type: MaterialType::Emissive,
            albedo: Some(Texture::solid_color(color)),
            color: None,
            fuzz: None,
            refraction_index: None
        }
    }

    fn reflectance(&self, cosin: f32, ref_idx: f32) -> f32 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        return r0 + (1.0 - r0) * (1.0 - cosin).powi(5);
    }
    fn scatter_lambertian(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        let mut scatter_dir = rec.normal + random_unit_vec3();

        if scatter_dir.near_zero() {
            scatter_dir = rec.normal;
        }

        scattered.reset(rec.point, scatter_dir, r_in.time);
        attenuation.set_to(self.albedo.as_ref().unwrap().get_color(rec.u, rec.v, rec.point));
        return true;
    }
    fn scatter_glossy(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        let reflected = reflect(r_in.direction.normalize(), rec.normal);
        scattered.reset(rec.point, reflected + random_in_unit_sphere() * self.fuzz.unwrap(), r_in.time);
        attenuation.set_to(self.color.unwrap());
        return scattered.direction.dot(rec.normal) > 0.0;
    }
    fn scatter_dielectric(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        attenuation.set_to(Vec3::new(1.0, 1.0, 1.0));
        let refraction_ratio = if rec.front_face { 1.0 / self.refraction_index.unwrap() } else { self.refraction_index.unwrap() };

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
    fn scatter_isotropic(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        scattered.reset(rec.point, random_in_unit_sphere(), r_in.time);
        attenuation.set_to(self.albedo.as_ref().unwrap().get_color(rec.u, rec.v, rec.point));
        return true;
    }
    fn scatter_emissive(&self, _r_in: Ray, _attenuation: &mut Rgb, _rec: HitRecord, _scattered: &mut Ray) -> bool {
        return false;
    }

    pub fn scatter(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        match self.mat_type {
            MaterialType::Lambertian => self.scatter_lambertian(r_in, attenuation, rec, scattered),
            MaterialType::Glossy => self.scatter_glossy(r_in, attenuation, rec, scattered),
            MaterialType::Dielectric => self.scatter_dielectric(r_in, attenuation, rec, scattered),
            MaterialType::Isotropic => self.scatter_isotropic(r_in, attenuation, rec, scattered),
            MaterialType::Emissive => self.scatter_emissive(r_in, attenuation, rec, scattered)
        }
    }
    pub fn emitted(&self, u: f32, v: f32, point: Point3) -> Rgb {
        match self.mat_type {
            MaterialType::Lambertian => Rgb::origin(),
            MaterialType::Glossy => Rgb::origin(),
            MaterialType::Dielectric => Rgb::origin(),
            MaterialType::Isotropic => Rgb::origin(),
            MaterialType::Emissive => self.albedo.as_ref().unwrap().get_color(u, v, point)
        }
    }
}

