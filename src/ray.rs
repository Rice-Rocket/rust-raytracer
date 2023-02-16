#[path = "vec3.rs"] mod vec3;
pub use vec3::*;


#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self {
            origin: origin,
            direction: direction
        }
    }
    pub fn reset(&mut self, origin: Point3, direction: Vec3) {
        self.origin = origin;
        self.direction = direction;
    }
    pub fn at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }
}


