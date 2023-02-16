#[path = "collider.rs"] mod collider;
pub use collider::*;


pub struct Camera {
    pub look_from: Point3,
    pub look_at: Point3,
    pub vup: Vec3,
    pub theta: f32,
    pub h: f32,
    pub aspect_ratio: f32,
    pub aperture: f32,
    pub focus_dist: f32,
    pub lens_radius: f32,

    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,

    pub viewport_height: f32,
    pub viewport_width: f32,
    pub focal_length: f32,
    
    pub origin: Point3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub lower_left: Vec3
}

impl Camera {
    pub fn new(look_from: Point3, look_at: Point3, vup: Vec3, vfov: f32, aperture: f32, focus_dist: f32, aspect_ratio: f32) -> Self {
        let theta = to_radians(vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let w = (look_from - look_at).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let origin = look_from;
        let horizontal = u * focus_dist * viewport_width;
        let vertical = v * focus_dist * viewport_height;
        let lens_radius = aperture / 2.0;
        Self {
            look_at: look_at,
            look_from: look_from,
            vup: vup,
            theta: theta,
            h: h,
            aspect_ratio: aspect_ratio,
            aperture: aperture,
            focus_dist: focus_dist,
            lens_radius: lens_radius,
            viewport_height: viewport_height,
            viewport_width: viewport_width,
            focal_length: focal_length,

            u: u,
            v: v,
            w: w,
    
            origin: origin,
            horizontal: horizontal,
            vertical: vertical,
            lower_left: origin - horizontal / 2.0 - vertical / 2.0 - w * focus_dist,
        }
    }
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        let rd = random_in_unit_disk() * self.lens_radius;
        let offset = self.u * rd.x + self.v * rd.y;
        Ray::new(self.origin + offset, self.lower_left + self.horizontal * u + self.vertical * v - self.origin - offset)
    }
}