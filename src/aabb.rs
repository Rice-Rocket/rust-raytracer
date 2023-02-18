#[path = "material.rs"] mod material;
pub use material::*;


#[derive(Clone)]
pub struct AABB {
    pub minimum: Point3,
    pub maximum: Point3,
}

impl AABB {
    pub fn new(a: Point3, b: Point3) -> Self {
        Self {
            minimum: a,
            maximum: b
        }
    }
    pub fn empty() -> Self {
        Self {
            minimum: Point3::origin(),
            maximum: Point3::origin()
        }
    }
    pub fn surrounding_box(box0: Self, box1: Self) -> Self {
        let small = Point3::new(
            box0.minimum.x.min(box1.minimum.x),
            box0.minimum.y.min(box1.minimum.y),
            box0.minimum.z.min(box1.minimum.z)
        );
        let big = Point3::new(
            box0.maximum.x.max(box1.maximum.x),
            box0.maximum.y.max(box1.maximum.y),
            box0.maximum.z.max(box1.maximum.z)
        );
        return Self::new(small, big);
    }
    pub fn intersect(&self, r: Ray, mut t_min: f32, mut t_max: f32) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / r.direction[a];
            let t0 = (self.minimum[a] - r.origin[a]) * inv_d;
            let t1 = (self.maximum[a] - r.origin[a]) * inv_d;
            if inv_d < 0.0 {
                let (t0, t1) = (t1, t0);
                t_min = if t0 > t_min { t0 } else { t_min };
                t_max = if t1 < t_max { t1 } else { t_max };
            }
            if t_max <= t_min {
                return false;
            }
        };
        return true;
    }
}

