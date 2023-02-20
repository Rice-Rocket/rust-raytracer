#[path = "collider.rs"] mod collider;
pub use collider::*;
use std::sync::Arc;



impl Geometry for YRotationInstance {
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        match self.y_rot_has_box {
            true => Some(self.y_rot_bbox.clone()),
            false => None
        }
    }
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = r.origin.clone();
        let mut dir = r.direction.clone();

        origin.x = self.y_rot_cos_theta * r.origin.x - self.y_rot_sin_theta * r.origin.z;
        origin.z = self.y_rot_sin_theta * r.origin.x + self.y_rot_cos_theta * r.origin.z;

        dir.x = self.y_rot_cos_theta * r.direction.x - self.y_rot_sin_theta * r.direction.z;
        dir.z = self.y_rot_sin_theta * r.direction.x + self.y_rot_cos_theta * r.direction.z;

        let rotated_ray = Ray::new(origin, dir, r.time);
        
        match self.instance_geometry.intersect(rotated_ray, t_min, t_max) {
            Some(mut rec) => {
                let mut p = rec.point.clone();
                let mut norm = rec.normal.clone();
            
                p.x = self.y_rot_cos_theta * rec.point.x + self.y_rot_sin_theta * rec.point.z;
                p.z = -self.y_rot_sin_theta * rec.point.x + self.y_rot_cos_theta * rec.point.z;

                norm.x = self.y_rot_cos_theta * rec.normal.x + self.y_rot_sin_theta * rec.normal.z;
                norm.z = -self.y_rot_sin_theta * rec.normal.x + self.y_rot_cos_theta * rec.normal.z;

                rec.point = p;
                rec.set_face_normal(rotated_ray, norm);
                return Some(rec);
            }
            None => None
        }
    }
}