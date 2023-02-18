#[path = "texture.rs"] mod texture;
pub use texture::*;
use std::sync::Arc;



#[derive(Clone)]
pub struct HitRecord {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material + Send + Sync>,
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
            material: Arc::new(Lambertian::new(Arc::new(SolidColor::new(Rgb::origin())))),
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

pub trait Material {
    fn scatter(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool;
    fn emitted(&self, _u: f32, _v: f32, _point: Point3) -> Rgb {
        Rgb::origin()
    }
}

pub struct Lambertian {
    pub albedo: Arc<dyn Texture + Send + Sync>
}

impl Lambertian {
    pub fn new(albedo: Arc<dyn Texture + Send + Sync>) -> Self {
        Self {
            albedo: albedo
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        let mut scatter_dir = rec.normal + random_unit_vec3();

        if scatter_dir.near_zero() {
            scatter_dir = rec.normal;
        }

        scattered.reset(rec.point, scatter_dir, r_in.time);
        attenuation.set_to(self.albedo.get_color(rec.u, rec.v, rec.point));
        return true;
    }
}


pub struct Glossy {
    pub albedo: Rgb,
    pub fuzz: f32
}

impl Glossy {
    pub fn new(albedo: Rgb, fuzz: f32) -> Self {
        Self {
            albedo: albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 }
        }
    }
}

impl Material for Glossy {
    fn scatter(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        let reflected = reflect(r_in.direction.normalize(), rec.normal);
        scattered.reset(rec.point, reflected + random_in_unit_sphere() * self.fuzz, r_in.time);
        attenuation.set_to(self.albedo);
        return scattered.direction.dot(rec.normal) > 0.0;
    }
}


pub struct Dielectric {
    pub refraction_index: f32
}

impl Dielectric {
    pub fn new(index_of_refraction: f32) -> Self {
        Self {
            refraction_index: index_of_refraction
        }
    }
    pub fn reflectance(&self, cosin: f32, ref_idx: f32) -> f32 {
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        return r0 + (1.0 - r0) * (1.0 - cosin).powi(5);
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: Ray, attenuation: &mut Rgb, rec: HitRecord, scattered: &mut Ray) -> bool {
        attenuation.set_to(Vec3::new(1.0, 1.0, 1.0));
        let refraction_ratio = if rec.front_face { 1.0 / self.refraction_index } else { self.refraction_index };

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
}


pub struct Emissive {
    pub color: Arc<SolidColor>
}

impl Emissive {
    pub fn new(color: Rgb) -> Self {
        Self {
            color: Arc::new(SolidColor::new(color))
        }
    }
}

impl Material for Emissive {
    fn scatter(&self, _r_in: Ray, _attenuation: &mut Rgb, _rec: HitRecord, _scattered: &mut Ray) -> bool {
        return false;
    }
    fn emitted(&self, u: f32, v: f32, point: Point3) -> Rgb {
        self.color.get_color(u, v, point)
    }
}