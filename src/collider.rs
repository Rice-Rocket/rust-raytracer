#[path = "material.rs"] mod material;
pub use material::*;


pub trait Geometry {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}


pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
}

impl Geometry for Sphere {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 { return None };
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if (root < t_min) || (t_max < root) {
            root = (-half_b + sqrtd) / a;
            if (root < t_min) || (t_max < root) { return None };
        }

        let point = r.at(root);
        let mut rec = HitRecord::new(point, root);
        let out_normal = (point - self.center) / self.radius;
        rec.set_face_normal(r, out_normal);
        return Some(rec);
    }
}

impl Sphere {
    pub fn new(center: Point3, radius: f32) -> Self {
        Self {
            center: center,
            radius: radius,
        }
    }
}


pub struct MovingSphere {
    pub center_0: Point3,
    pub center_1: Point3,
    pub time_0: f32,
    pub time_1: f32,
    pub radius: f32,
}

impl Geometry for MovingSphere {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.center(r.time);
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 { return None };
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if (root < t_min) || (t_max < root) {
            root = (-half_b + sqrtd) / a;
            if (root < t_min) || (t_max < root) { return None };
        }

        let point = r.at(root);
        let mut rec = HitRecord::new(point, root);
        let out_normal = (point - self.center(r.time)) / self.radius;
        rec.set_face_normal(r, out_normal);
        return Some(rec);
    }
}

impl MovingSphere {
    pub fn new(center0: Point3, center1: Point3, time0: f32, time1: f32, radius: f32) -> Self {
        Self {
            center_0: center0,
            center_1: center1,
            time_0: time0,
            time_1: time1,
            radius: radius
        }
    }
    pub fn center(&self, time: f32) -> Point3 {
        self.center_0 + (self.center_1 - self.center_0) * ((time - self.time_0) / (self.time_1 - self.time_0))
    }
}


pub struct SceneColliders {
    pub objects: Vec<Box<dyn Geometry + Sync + Send>>,
    pub materials: Vec<Box<dyn Material + Sync + Send>>
}

impl SceneColliders {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            materials: Vec::new()
        }
    }
    pub fn add(&mut self, object: Box<dyn Geometry + Sync + Send>, material: Box<dyn Material + Sync + Send>) {
        self.objects.push(object);
        self.materials.push(material);
    }
    pub fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<(HitRecord, usize)> {
        let mut hit_anything = false;
        let mut closest = t_max;
        let mut hit_rec = HitRecord::new(Point3::origin(), 0.0);
        let mut hit_mat = 0;
        
        for (i, obj) in self.objects.iter().enumerate() {
            let hit = obj.intersect(r, t_min, closest);
            match hit {
                Some(rec) => {
                    hit_anything = true;
                    closest = rec.t;
                    hit_rec = rec;
                    hit_mat = i;
                },
                None => {}
            }
        };

        return match hit_anything {
            true => Some((hit_rec, hit_mat)),
            false => None
        };
    }
}