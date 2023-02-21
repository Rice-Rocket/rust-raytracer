#[path = "aabb.rs"] mod aabb;
pub use aabb::*;
use std::sync::Arc;
use std::cmp::Ordering;


fn box_compare(a: &Geometry, b: &Geometry, axis: usize) -> Ordering {
    let box_a = a.bounding_box(0.0, 0.0).unwrap();
    let box_b = b.bounding_box(0.0, 0.0).unwrap();

    if box_a.minimum[axis] < box_b.minimum[axis] {
        return Ordering::Less;
    } else if box_a.minimum[axis] > box_b.minimum[axis] {
        return Ordering::Greater;
    } else {
        return Ordering::Equal;
    }
}

fn box_x_compare(a: &Geometry, b: &Geometry) -> Ordering {
    box_compare(a, b, 0)
}
fn box_y_compare(a: &Geometry, b: &Geometry) -> Ordering {
    box_compare(a, b, 1)
}
fn box_z_compare(a: &Geometry, b: &Geometry) -> Ordering {
    box_compare(a, b, 2)
}


#[derive(Clone)]
pub enum GeometryType {
    Sphere,
    MovingSphere,
    XYRect,
    XZRect,
    YZRect,
    Cuboid,
    ConstantMedium,
    TranslateInstance,
    YRotationInstance,
    BVHNode
}

#[derive(Clone)]
pub struct Geometry {
    pub geometry_type: GeometryType,
    pub material: Arc<Material>,
    
    pub time_0: Option<f32>,
    pub time_1: Option<f32>,

    pub center: Option<Point3>,
    pub center1: Option<Point3>,
    pub radius: Option<f32>,

    pub x0: Option<f32>,
    pub x1: Option<f32>,
    pub y0: Option<f32>,
    pub y1: Option<f32>,
    pub z0: Option<f32>,
    pub z1: Option<f32>,
    pub k: Option<f32>,

    pub p0: Option<Point3>,
    pub p1: Option<Point3>,
    pub sides: Option<SceneColliders>,

    pub boundary: Option<Box<Geometry>>,
    pub neg_inv_density: Option<f32>,

    pub node_left: Option<Box<Geometry>>,
    pub node_right: Option<Box<Geometry>>,
    pub node_bounding_box: Option<AABB>,
    pub node_axis: Option<usize>,

    pub instance_geometry: Option<Box<Geometry>>,
    pub trans_displacement: Option<Vec3>,

    pub y_rot_angle: Option<f32>,
    pub y_rot_sin_theta: Option<f32>,
    pub y_rot_cos_theta: Option<f32>,
    pub y_rot_has_box: Option<bool>,
    pub y_rot_bbox: Option<AABB>
}

impl Default for Geometry {
    fn default() -> Self {
        Self {
            geometry_type: GeometryType::Sphere,
            material: Arc::new(Material::lambertian(Texture::solid_color(Rgb::origin()))),
            time_0: None,
            time_1: None,
            center: None,
            center1: None,
            radius: None,
            x0: None,
            x1: None,
            y0: None,
            y1: None,
            z0: None,
            z1: None,
            k: None,
            p0: None,
            p1: None,
            sides: None,
            boundary: None,
            neg_inv_density: None,
            node_left: None,
            node_right: None,
            node_bounding_box: None,
            node_axis: None,
            instance_geometry: None,
            trans_displacement: None,
            y_rot_angle: None,
            y_rot_sin_theta: None,
            y_rot_cos_theta: None,
            y_rot_has_box: None,
            y_rot_bbox: None
        }
    }
}

impl Geometry {
    pub fn sphere(center: Point3, radius: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::Sphere,
            material: Arc::new(material),
            center: Some(center),
            radius: Some(radius),
            ..Self::default()
        }
    }
    pub fn moving_sphere(center0: Point3, center1: Point3, time0: f32, time1: f32, radius: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::MovingSphere,
            material: Arc::new(material),
            center: Some(center0),
            center1: Some(center1),
            time_0: Some(time0),
            time_1: Some(time1),
            radius: Some(radius),
            ..Self::default()
        }
    }
    pub fn xyrect(x0: f32, x1: f32, y0: f32, y1: f32, k: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::XYRect,
            x0: Some(x0), 
            x1: Some(x1), 
            y0: Some(y0), 
            y1: Some(y1), 
            k: Some(k), 
            material: Arc::new(material),
            ..Self::default()
        }
    }
    pub fn xzrect(x0: f32, x1: f32, z0: f32, z1: f32, k: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::XZRect,
            x0: Some(x0), 
            x1: Some(x1), 
            z0: Some(z0), 
            z1: Some(z1), 
            k: Some(k), 
            material: Arc::new(material),
            ..Self::default()
        }
    }
    pub fn yzrect(y0: f32, y1: f32, z0: f32, z1: f32, k: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::YZRect,
            z0: Some(z0), 
            z1: Some(z1), 
            y0: Some(y0), 
            y1: Some(y1), 
            k: Some(k), 
            material: Arc::new(material),
            ..Self::default()
        }
    }
    pub fn cuboid(p0: Point3, p1: Point3, material: Material) -> Self {
        let mut sides = SceneColliders::new();
        sides.add(Self::xyrect(p0.x, p1.x, p0.y, p1.y, p1.z, material.clone()));
        sides.add(Self::xyrect(p0.x, p1.x, p0.y, p1.y, p0.z, material.clone()));

        sides.add(Self::xzrect(p0.x, p1.x, p0.z, p1.z, p1.y, material.clone()));
        sides.add(Self::xzrect(p0.x, p1.x, p0.z, p1.z, p0.y, material.clone()));

        sides.add(Self::yzrect(p0.y, p1.y, p0.z, p1.z, p1.x, material.clone()));
        sides.add(Self::yzrect(p0.y, p1.y, p0.z, p1.z, p0.x, material.clone()));

        Self {
            geometry_type: GeometryType::Cuboid,
            p0: Some(p0),
            p1: Some(p1),
            material: Arc::new(material),
            sides: Some(sides),
            ..Self::default()
        }
    }
    pub fn constant_medium(boundary: Geometry, density: f32, color: Rgb) -> Self {
        Self {
            geometry_type: GeometryType::ConstantMedium,
            boundary: Some(Box::new(boundary)),
            neg_inv_density: Some(-1.0 / density),
            material: Arc::new(Material::isotropic(color)),
            ..Self::default()
        }
    }
    pub fn instance_translation(geometry: Geometry, displacement: Vec3) -> Self {
        Self {
            geometry_type: GeometryType::TranslateInstance,
            instance_geometry: Some(Box::new(geometry)), 
            trans_displacement: Some(displacement),
            ..Default::default()
        }
    }
    pub fn instance_y_rotation(geometry: Geometry, angle: f32) -> Self {
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
            geometry_type: GeometryType::YRotationInstance,
            instance_geometry: Some(Box::new(geometry)),
            y_rot_angle: Some(angle),
            y_rot_sin_theta: Some(sin_theta),
            y_rot_cos_theta: Some(cos_theta),
            y_rot_has_box: Some(has_box),
            y_rot_bbox: Some(AABB::new(min, max)),
            ..Self::default()
        }
    }
    pub fn bvh_node(colliders: &Vec<Geometry>, time_0: f32, time_1: f32, start: usize, end: usize) -> Self {
        let axis = randuint(0, 2);
        let comparator = if axis == 0 { box_x_compare } else if axis == 2 { box_y_compare } else { box_z_compare };
        let obj_span = end - start;
        let mut temp_colliders = colliders.clone();
        let left: Geometry;
        let right: Geometry;
        if obj_span == 1 {
            left = temp_colliders[start].clone();
            right = temp_colliders[start].clone();
        } else if obj_span == 2 {
            if comparator(&temp_colliders[start], &temp_colliders[start + 1]).is_le() {
                left = temp_colliders[start].clone();
                right = temp_colliders[start + 1].clone();
            } else {
                left = temp_colliders[start + 1].clone();
                right = temp_colliders[start].clone();
            }
        } else {
            temp_colliders[start..end].sort_by(|a, b| {
                comparator(a, b)
            });
            let mid = start + obj_span / 2;
            left = Geometry::bvh_node(temp_colliders.clone().as_ref(), time_0, time_1, start, mid);
            right = Geometry::bvh_node(temp_colliders.clone().as_ref(), time_0, time_1, mid, end);
        }

        let box_left = left.bounding_box(time_0, time_1);
        let box_right = right.bounding_box(time_0, time_1);
        let bounding_box: Option<AABB> = match (box_left, box_right) {
            (Some(box_left), Some(box_right)) => Some(AABB::surrounding_box(box_left, box_right)),
            (Some(box_left), None) => Some(box_left),
            (None, Some(box_right)) => Some(box_right),
            (None, None) => None,
        };
        Self {
            geometry_type: GeometryType::BVHNode,
            node_left: Some(Box::new(left)),
            node_right: Some(Box::new(right)),
            node_bounding_box: bounding_box,
            node_axis: Some(axis),
            ..Self::default()
        }
    }

    fn moving_sphere_center(&self, time: f32) -> Point3 {
        self.center.unwrap() + (self.center1.unwrap() - self.center.unwrap()) * ((time - self.time_0.unwrap()) / (self.time_1.unwrap() - self.time_0.unwrap()))
    }
    fn sphere_get_uv(&self, point: Point3) -> (f32, f32) {
        let theta = (-point.y).acos();
        let phi = (-point.z).atan2(point.x) + PI;
        (phi / TAU, theta / PI)
    }

    fn intersect_sphere(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.center.unwrap();
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - self.radius.unwrap() * self.radius.unwrap();

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
        let out_normal = (point - self.center.unwrap()) / self.radius.unwrap();
        rec.set_face_normal(r, out_normal);
        rec.material = self.material.clone();
        rec.set_uv(self.sphere_get_uv(out_normal));
        return Some(rec);
    }
    fn bounding_box_sphere(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        return Some(AABB::new(
            self.center.unwrap() - Vec3::new(self.radius.unwrap(), self.radius.unwrap(), self.radius.unwrap()),
            self.center.unwrap() + Vec3::new(self.radius.unwrap(), self.radius.unwrap(), self.radius.unwrap())
        ));
    }

    fn intersect_moving_sphere(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.moving_sphere_center(r.time);
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - self.radius.unwrap() * self.radius.unwrap();

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
        let out_normal = (point - self.moving_sphere_center(r.time)) / self.radius.unwrap();
        rec.set_face_normal(r, out_normal);
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_moving_sphere(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        let box0 = AABB::new(
            self.moving_sphere_center(time_0) - Vec3::new(self.radius.unwrap(), self.radius.unwrap(), self.radius.unwrap()),
            self.moving_sphere_center(time_0) + Vec3::new(self.radius.unwrap(), self.radius.unwrap(), self.radius.unwrap()),
        );
        let box1 = AABB::new(
            self.moving_sphere_center(time_1) - Vec3::new(self.radius.unwrap(), self.radius.unwrap(), self.radius.unwrap()),
            self.moving_sphere_center(time_1) + Vec3::new(self.radius.unwrap(), self.radius.unwrap(), self.radius.unwrap()),
        );
        return Some(AABB::surrounding_box(box0, box1));
    }

    fn intersect_xyrect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k.unwrap() - r.origin.z) / r.direction.z;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let x = r.origin.x + t * r.direction.x;
        let y = r.origin.y + t * r.direction.y;
        if (x < self.x0.unwrap()) || (x > self.x1.unwrap()) || (y < self.y0.unwrap()) || (y > self.y1.unwrap()) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((x - self.x0.unwrap()) / (self.x1.unwrap() - self.x0.unwrap()), (y - self.y0.unwrap()) / (self.y1.unwrap() - self.y0.unwrap())));
        rec.set_face_normal(r, Vec3::new(0.0, 0.0, 1.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_xyrect(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0.unwrap(), self.y0.unwrap(), self.k.unwrap() - 0.0001),
            Point3::new(self.x1.unwrap(), self.y1.unwrap(), self.k.unwrap() + 0.0001)
        ))
    }

    fn intersect_xzrect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k.unwrap() - r.origin.y) / r.direction.y;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let x = r.origin.x + t * r.direction.x;
        let z = r.origin.z + t * r.direction.z;
        if (x < self.x0.unwrap()) || (x > self.x1.unwrap()) || (z < self.z0.unwrap()) || (z > self.z1.unwrap()) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((x - self.x0.unwrap()) / (self.x1.unwrap() - self.x0.unwrap()), (z - self.z0.unwrap()) / (self.z1.unwrap() - self.z0.unwrap())));
        rec.set_face_normal(r, Vec3::new(0.0, 1.0, 0.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_xzrect(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0.unwrap(), self.k.unwrap() - 0.0001, self.z0.unwrap()),
            Point3::new(self.x1.unwrap(), self.k.unwrap() + 0.0001, self.z1.unwrap())
        ))
    }

    fn intersect_yzrect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k.unwrap() - r.origin.x) / r.direction.x;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let y = r.origin.y + t * r.direction.y;
        let z = r.origin.z + t * r.direction.z;
        if (y < self.y0.unwrap()) || (y > self.y1.unwrap()) || (z < self.z0.unwrap()) || (z > self.z1.unwrap()) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((y - self.y0.unwrap()) / (self.y1.unwrap() - self.y0.unwrap()), (z - self.z0.unwrap()) / (self.z1.unwrap() - self.z0.unwrap())));
        rec.set_face_normal(r, Vec3::new(1.0, 0.0, 0.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_yzrect(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.k.unwrap() - 0.0001, self.y0.unwrap(), self.z0.unwrap()),
            Point3::new(self.k.unwrap() + 0.0001, self.y1.unwrap(), self.z1.unwrap())
        ))
    }

    fn intersect_cuboid(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        return self.sides.as_ref().unwrap().intersect(r, t_min, t_max);
    }
    fn bounding_box_cuboid(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        return Some(AABB::new(self.p0.unwrap(), self.p1.unwrap()));
    }

    fn intersect_constant_medium(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut rec1;
        match self.boundary.as_ref().unwrap().intersect(r, f32::MIN, f32::MAX) {
            Some(rec) => rec1 = rec,
            None => return None
        };
        
        let mut rec2;
        match self.boundary.as_ref().unwrap().intersect(r, rec1.t + 0.0001, f32::MAX) {
            Some(rec) => rec2 = rec,
            None => return None
        };

        if rec1.t < t_min {
            rec1.t = t_min;
        }
        if rec2.t > t_max {
            rec2.t = t_max;
        }
        if rec1.t >= rec2.t {
            return None;
        }
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let ray_length = r.direction.length();
        let dist_inside_bound = (rec2.t - rec1.t) * ray_length;
        let hit_dist = self.neg_inv_density.unwrap() * random().log10();

        if hit_dist > dist_inside_bound {
            return None;
        }

        let rec_t = rec1.t + hit_dist / ray_length;
        let mut rec = HitRecord::new(r.at(rec_t), rec_t);
        rec.normal = Vec3::new(1.0, 0.0, 0.0);
        rec.front_face = true;
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_constant_medium(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        self.boundary.as_ref().unwrap().bounding_box(time_0, time_1)
    }

    fn intersect_translate_instance(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_r = Ray::new(r.origin - self.trans_displacement.unwrap(), r.direction, r.time);
        match self.instance_geometry.as_ref().unwrap().intersect(moved_r, t_min, t_max) {
            Some(mut rec) => {
                rec.point = rec.point + self.trans_displacement.unwrap();
                rec.set_face_normal(moved_r, rec.normal);
                return Some(rec);
            }
            None => None
        }
    }
    fn bounding_box_translate_instance(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        match self.instance_geometry.as_ref().unwrap().bounding_box(time_0, time_1) {
            Some(aabb) => Some(AABB::new(aabb.minimum + self.trans_displacement.unwrap(), aabb.maximum + self.trans_displacement.unwrap())),
            None => None
        }
    }

    fn intersect_y_rot_instance(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = r.origin.clone();
        let mut dir = r.direction.clone();

        origin.x = self.y_rot_cos_theta.unwrap() * r.origin.x - self.y_rot_sin_theta.unwrap() * r.origin.z;
        origin.z = self.y_rot_sin_theta.unwrap() * r.origin.x + self.y_rot_cos_theta.unwrap() * r.origin.z;

        dir.x = self.y_rot_cos_theta.unwrap() * r.direction.x - self.y_rot_sin_theta.unwrap() * r.direction.z;
        dir.z = self.y_rot_sin_theta.unwrap() * r.direction.x + self.y_rot_cos_theta.unwrap() * r.direction.z;

        let rotated_ray = Ray::new(origin, dir, r.time);
        
        match self.instance_geometry.as_ref().unwrap().intersect(rotated_ray, t_min, t_max) {
            Some(mut rec) => {
                let mut p = rec.point.clone();
                let mut norm = rec.normal.clone();
            
                p.x = self.y_rot_cos_theta.unwrap() * rec.point.x + self.y_rot_sin_theta.unwrap() * rec.point.z;
                p.z = -self.y_rot_sin_theta.unwrap() * rec.point.x + self.y_rot_cos_theta.unwrap() * rec.point.z;

                norm.x = self.y_rot_cos_theta.unwrap() * rec.normal.x + self.y_rot_sin_theta.unwrap() * rec.normal.z;
                norm.z = -self.y_rot_sin_theta.unwrap() * rec.normal.x + self.y_rot_cos_theta.unwrap() * rec.normal.z;

                rec.point = p;
                rec.set_face_normal(rotated_ray, norm);
                return Some(rec);
            }
            None => None
        }
    }
    fn bounding_box_y_rot_instance(&self, _time_0: f32, _time_1: f32) -> Option<AABB> {
        match self.y_rot_has_box.unwrap() {
            true => Some(self.y_rot_bbox.as_ref().unwrap().clone()),
            false => None
        }
    }

    fn intersect_bvh(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match &self.node_bounding_box {
            Some(bounding_box) => {
                if !bounding_box.intersect(r, t_min, t_max) {
                    return None;
                }
            }
            None => (),
        }

        if r.direction[self.node_axis.unwrap()] >= 0.0 {
            match self.node_left.as_ref().unwrap().intersect(r, t_min, t_max) {
                Some(left_hit) => {
                    let right_hit = self.node_right.as_ref().unwrap().intersect(r, t_min, left_hit.t);
                    match right_hit {
                        Some(right_hit) => Some(right_hit),
                        None => Some(left_hit),
                    }
                }
                None => {
                    let right_hit = self.node_right.as_ref().unwrap().intersect(r, t_min, t_max);
                    match right_hit {
                        Some(right_hit) => Some(right_hit),
                        None => None,
                    }
                }
            }
        } else {
            match self.node_right.as_ref().unwrap().intersect(r, t_min, t_max) {
                Some(right_hit) => {
                    let left_hit = self.node_left.as_ref().unwrap().intersect(r, t_min, right_hit.t);
                    match left_hit {
                        Some(left_hit) => Some(left_hit),
                        _ => Some(right_hit),
                    }
                }
                _ => {
                    let left_hit = self.node_left.as_ref().unwrap().intersect(r, t_min, t_max);
                    match left_hit {
                        Some(left_hit) => Some(left_hit),
                        _ => None,
                    }
                }
            }
        }
    }
    fn bounding_box_bvh(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        return self.node_bounding_box.clone();
    }

    pub fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self.geometry_type {
            GeometryType::Sphere => self.intersect_sphere(r, t_min, t_max),
            GeometryType::MovingSphere => self.intersect_moving_sphere(r, t_min, t_max),
            GeometryType::XYRect => self.intersect_xyrect(r, t_min, t_max),
            GeometryType::XZRect => self.intersect_xzrect(r, t_min, t_max),
            GeometryType::YZRect => self.intersect_yzrect(r, t_min, t_max),
            GeometryType::Cuboid => self.intersect_cuboid(r, t_min, t_max),
            GeometryType::ConstantMedium => self.intersect_constant_medium(r, t_min, t_max),
            GeometryType::TranslateInstance => self.intersect_translate_instance(r, t_min, t_max),
            GeometryType::YRotationInstance => self.intersect_y_rot_instance(r, t_min, t_max),
            GeometryType::BVHNode => self.intersect_bvh(r, t_min, t_max)
        }
    }
    pub fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        match self.geometry_type {
            GeometryType::Sphere => self.bounding_box_sphere(time_0, time_1),
            GeometryType::MovingSphere => self.bounding_box_moving_sphere(time_0, time_1),
            GeometryType::XYRect => self.bounding_box_xyrect(time_0, time_1),
            GeometryType::XZRect => self.bounding_box_xzrect(time_0, time_1),
            GeometryType::YZRect => self.bounding_box_yzrect(time_0, time_1),
            GeometryType::Cuboid => self.bounding_box_cuboid(time_0, time_1),
            GeometryType::ConstantMedium => self.bounding_box_constant_medium(time_0, time_1),
            GeometryType::TranslateInstance => self.bounding_box_translate_instance(time_0, time_1),
            GeometryType::YRotationInstance => self.bounding_box_y_rot_instance(time_0, time_1),
            GeometryType::BVHNode => self.bounding_box_bvh(time_0, time_1)
        }
    }
}




#[derive(Clone)]
pub struct SceneColliders {
    pub objects: Vec<Geometry>,
}

impl SceneColliders {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
    pub fn add(&mut self, object: Geometry) {
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