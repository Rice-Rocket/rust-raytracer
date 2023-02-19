#[path = "collider.rs"] mod collider;
pub use collider::*;
use std::sync::Arc;



pub struct TranslateInstance {
    pub geometry: Arc<dyn Geometry + Send + Sync>,
    pub displacement: Vec3,
}

impl TranslateInstance {
    pub fn new(geometry: Arc<dyn Geometry + Send + Sync>, displacement: Vec3) -> Self {
        Self {
            geometry, displacement
        }
    }
}

impl Geometry for TranslateInstance {
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        match self.geometry.bounding_box(time_0, time_1) {
            Some(aabb) => Some(AABB::new(aabb.minimum + self.displacement, aabb.maximum + self.displacement)),
            None => None
        }
    }
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_r = Ray::new(r.origin - self.displacement, r.direction, r.time);
        match self.geometry.intersect(moved_r, t_min, t_max) {
            Some(mut rec) => {
                rec.point = rec.point + self.displacement;
                rec.set_face_normal(moved_r, rec.normal);
                return Some(rec);
            }
            None => None
        }
    }
}

pub struct YRotationInstance {
    pub geometry: Arc<dyn Geometry + Send + Sync>,
    pub angle: f32,
    pub sin_theta: f32,
    pub cos_theta: f32,
    pub has_box: bool,
    pub bbox: AABB
}

impl YRotationInstance {
    pub fn new(geometry: Arc<dyn Geometry + Send + Sync>, angle: f32) -> Self {
        let radians = to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox_option = geometry.bounding_box(0.0, 1.0);
        let has_box: bool;
        let bbox = match bbox_option {
            Some(aabb) => {
                has_box = true;
                aabb
            }
            None => {
                has_box = false;
                AABB::empty()
            }
        };

        let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f32 * bbox.maximum.x + (1 - i) as f32 * bbox.minimum.x;
                    let y = j as f32 * bbox.maximum.y + (1 - j) as f32 * bbox.minimum.y;
                    let z = k as f32 * bbox.maximum.z + (1 - k) as f32 * bbox.minimum.z;

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);
                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }

        Self {
            geometry: geometry,
            angle: angle,
            sin_theta: sin_theta,
            cos_theta: cos_theta,
            has_box: has_box,
            bbox: AABB::new(min, max)
        }
    }
}

impl Geometry for YRotationInstance {
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        match self.has_box {
            true => Some(self.bbox.clone()),
            false => None
        }
    }
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = r.origin.clone();
        let mut dir = r.direction.clone();

        origin.x = self.cos_theta * r.origin.x - self.sin_theta * r.origin.z;
        origin.z = self.sin_theta * r.origin.x + self.cos_theta * r.origin.z;

        dir.x = self.cos_theta * r.direction.x - self.sin_theta * r.direction.z;
        dir.z = self.sin_theta * r.direction.x + self.cos_theta * r.direction.z;

        let rotated_ray = Ray::new(origin, dir, r.time);
        
        match self.geometry.intersect(rotated_ray, t_min, t_max) {
            Some(mut rec) => {
                let mut p = rec.point.clone();
                let mut norm = rec.normal.clone();
            
                p.x = self.cos_theta * rec.point.x + self.sin_theta * rec.point.z;
                p.z = -self.sin_theta * rec.point.x + self.cos_theta * rec.point.z;

                norm.x = self.cos_theta * rec.normal.x + self.sin_theta * rec.normal.z;
                norm.z = -self.sin_theta * rec.normal.x + self.cos_theta * rec.normal.z;

                rec.point = p;
                rec.set_face_normal(rotated_ray, norm);
                return Some(rec);
            }
            None => None
        }
    }
}