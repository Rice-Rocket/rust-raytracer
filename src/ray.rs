#[path = "vec3.rs"] mod vec3;
pub use vec3::*;


#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub time: f32,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3, time: f32) -> Self {
        Self {
            origin: origin,
            direction: direction,
            time: time
        }
    }
    pub fn reset(&mut self, origin: Point3, direction: Vec3, time: f32) {
        self.origin = origin;
        self.direction = direction;
        self.time = time;
    }
    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }
}


