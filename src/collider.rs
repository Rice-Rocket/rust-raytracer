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
        rec.set_uv(self.get_uv(out_normal));
        return Some(rec);
    }
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
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
    pub fn get_uv(&self, point: Point3) -> (f32, f32) {
        let theta = (-point.y).acos();
        let phi = (-point.z).atan2(point.x) + PI;
        (phi / TAU, theta / PI)
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


pub struct XYRect {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub k: f32,
    pub material: Arc<dyn Material + Send + Sync>
}

impl XYRect {
    pub fn new(x0: f32, x1: f32, y0: f32, y1: f32, k: f32, material: Arc<dyn Material + Send + Sync>) -> Self {
        Self {
            x0, x1, y0, y1, k, 
            material: material
        }
    }
}

impl Geometry for XYRect {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - r.origin.z) / r.direction.z;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let x = r.origin.x + t * r.direction.x;
        let y = r.origin.y + t * r.direction.y;
        if (x < self.x0) || (x > self.x1) || (y < self.y0) || (y > self.y1) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((x - self.x0) / (self.x1 - self.x0), (y - self.y0) / (self.y1 - self.y0)));
        rec.set_face_normal(r, Vec3::new(0.0, 0.0, 1.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0, self.y0, self.k - 0.0001),
            Point3::new(self.x1, self.y1, self.k + 0.0001)
        ))
    }
}


pub struct XZRect {
    pub x0: f32,
    pub x1: f32,
    pub z0: f32,
    pub z1: f32,
    pub k: f32,
    pub material: Arc<dyn Material + Send + Sync>
}

impl XZRect {
    pub fn new(x0: f32, x1: f32, z0: f32, z1: f32, k: f32, material: Arc<dyn Material + Send + Sync>) -> Self {
        Self {
            x0, x1, z0, z1, k, 
            material: material
        }
    }
}

impl Geometry for XZRect {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - r.origin.y) / r.direction.y;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let x = r.origin.x + t * r.direction.x;
        let z = r.origin.z + t * r.direction.z;
        if (x < self.x0) || (x > self.x1) || (z < self.z0) || (z > self.z1) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((x - self.x0) / (self.x1 - self.x0), (z - self.z0) / (self.z1 - self.z0)));
        rec.set_face_normal(r, Vec3::new(0.0, 1.0, 0.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0, self.k - 0.0001, self.z0),
            Point3::new(self.x1, self.k + 0.0001, self.z1)
        ))
    }
}


pub struct YZRect {
    pub z0: f32,
    pub z1: f32,
    pub y0: f32,
    pub y1: f32,
    pub k: f32,
    pub material: Arc<dyn Material + Send + Sync>
}

impl YZRect {
    pub fn new(y0: f32, y1: f32, z0: f32, z1: f32, k: f32, material: Arc<dyn Material + Send + Sync>) -> Self {
        Self {
            z0, z1, y0, y1, k, 
            material: material
        }
    }
}

impl Geometry for YZRect {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - r.origin.x) / r.direction.x;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let y = r.origin.y + t * r.direction.y;
        let z = r.origin.z + t * r.direction.z;
        if (y < self.y0) || (y > self.y1) || (z < self.z0) || (z > self.z1) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((y - self.y0) / (self.y1 - self.y0), (z - self.z0) / (self.z1 - self.z0)));
        rec.set_face_normal(r, Vec3::new(1.0, 0.0, 0.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.k - 0.0001, self.y0, self.z0),
            Point3::new(self.k + 0.0001, self.y1, self.z1)
        ))
    }
}


pub struct Cuboid {
    pub p0: Point3,
    pub p1: Point3,
    pub material: Arc<dyn Material + Send + Sync>,
    pub sides: SceneColliders
}

impl Cuboid {
    pub fn new(p0: Point3, p1: Point3, material: Arc<dyn Material + Send + Sync>) -> Self {
        let mut sides = SceneColliders::new();
        sides.add(Arc::new(XYRect::new(p0.x, p1.x, p0.y, p1.y, p1.z, material.clone())));
        sides.add(Arc::new(XYRect::new(p0.x, p1.x, p0.y, p1.y, p0.z, material.clone())));

        sides.add(Arc::new(XZRect::new(p0.x, p1.x, p0.z, p1.z, p1.y, material.clone())));
        sides.add(Arc::new(XZRect::new(p0.x, p1.x, p0.z, p1.z, p0.y, material.clone())));

        sides.add(Arc::new(YZRect::new(p0.y, p1.y, p0.z, p1.z, p1.x, material.clone())));
        sides.add(Arc::new(YZRect::new(p0.y, p1.y, p0.z, p1.z, p0.x, material.clone())));

        Self {
            p0: p0,
            p1: p1,
            material: material,
            sides: sides
        }
    }
}

impl Geometry for Cuboid {
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        return Some(AABB::new(self.p0, self.p1));
    }
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        return self.sides.intersect(r, t_min, t_max);
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