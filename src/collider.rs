#[path = "aabb.rs"] mod aabb;
pub use aabb::*;
use std::sync::Arc;


pub trait Geometry {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB>;
}


pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Arc<dyn Material + Send + Sync>
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
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        return Some(AABB::new(
            self.center - Vec3::new(self.radius, self.radius, self.radius),
            self.center + Vec3::new(self.radius, self.radius, self.radius)
        ));
    }
}

impl Sphere {
    pub fn new(center: Point3, radius: f32, material: Arc<dyn Material + Send + Sync>) -> Self {
        Self {
            center: center,
            radius: radius,
            material: material
        }
    }
}


pub struct MovingSphere {
    pub center_0: Point3,
    pub center_1: Point3,
    pub time_0: f32,
    pub time_1: f32,
    pub radius: f32,
    pub material: Arc<dyn Material + Send + Sync>
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
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        let box0 = AABB::new(
            self.center(time_0) - Vec3::new(self.radius, self.radius, self.radius),
            self.center(time_0) + Vec3::new(self.radius, self.radius, self.radius),
        );
        let box1 = AABB::new(
            self.center(time_1) - Vec3::new(self.radius, self.radius, self.radius),
            self.center(time_1) + Vec3::new(self.radius, self.radius, self.radius),
        );
        return Some(AABB::surrounding_box(box0, box1));
    }
}

impl MovingSphere {
    pub fn new(center0: Point3, center1: Point3, time0: f32, time1: f32, radius: f32, material: Arc<dyn Material + Send + Sync>) -> Self {
        Self {
            center_0: center0,
            center_1: center1,
            time_0: time0,
            time_1: time1,
            radius: radius,
            material: material
        }
    }
    pub fn center(&self, time: f32) -> Point3 {
        self.center_0 + (self.center_1 - self.center_0) * ((time - self.time_0) / (self.time_1 - self.time_0))
    }
}


pub struct SceneColliders {
    pub objects: Vec<Arc<dyn Geometry + Sync + Send>>,
}

impl SceneColliders {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
    pub fn add(&mut self, object: Arc<dyn Geometry + Sync + Send>) {
        self.objects.push(object);
    }
    pub fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut hit_anything = false;
        let mut closest = t_max;
        let mut hit_rec = HitRecord::new(Point3::origin(), 0.0);
        
        for obj in self.objects.iter() {
            let hit = obj.intersect(r, t_min, closest);
            match hit {
                Some(rec) => {
                    hit_anything = true;
                    closest = rec.t;
                    hit_rec = rec;
                },
                None => {}
            }
        };

        return match hit_anything {
            true => Some(hit_rec),
            false => None
        };
    }
    pub fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        if self.objects.is_empty() {
            return None;
        }
        let mut temp_box = Some(AABB::empty());
        let mut first_box = true;
        let mut out_box = None;

        for obj in self.objects.iter() {
            temp_box = obj.bounding_box(time_0, time_1);
            if temp_box.is_some() {
                return None;
            }
            out_box = if first_box { temp_box.clone() } else { Some(AABB::surrounding_box(out_box.unwrap().clone(), temp_box.unwrap().clone())) };
            first_box = false;
        }
        return out_box;
    }
}