#[path = "aabb.rs"] mod aabb;
pub use aabb::*;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::sync::{Arc, Mutex};
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
    Sphere(Point3, f32),
    MovingSphere(Point3, Point3, f32, f32, f32),
    XYRect(f32, f32, f32, f32, f32),
    XZRect(f32, f32, f32, f32, f32),
    YZRect(f32, f32, f32, f32, f32),
    Cuboid(Point3, Point3, SceneColliders),
    ConstantMedium(Box<Geometry>, f32),
    TranslateInstance(Box<Geometry>, Vec3),
    YRotationInstance(Box<Geometry>, Axis, f32, f32, f32, bool, AABB),
    BVHNode(Box<Geometry>, Box<Geometry>, Option<AABB>, usize),
    Triangle(Point3, Point3, Point3, Vec3),
}

#[derive(Clone)]
pub struct Geometry {
    pub geometry_type: GeometryType,
    pub material: Material,
}

impl Geometry {
    pub fn sphere(center: Point3, radius: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::Sphere(center, radius),
            material: material,
        }
    }
    pub fn moving_sphere(center0: Point3, center1: Point3, time0: f32, time1: f32, radius: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::MovingSphere(center0, center1, time0, time1, radius),
            material: material,
        }
    }
    pub fn xyrect(x0: f32, x1: f32, y0: f32, y1: f32, k: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::XYRect(x0, x1, y0, y1, k),
            material: material,
        }
    }
    pub fn xzrect(x0: f32, x1: f32, z0: f32, z1: f32, k: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::XZRect(x0, x1, z0, z1, k),
            material: material,
        }
    }
    pub fn yzrect(y0: f32, y1: f32, z0: f32, z1: f32, k: f32, material: Material) -> Self {
        Self {
            geometry_type: GeometryType::YZRect(z0, z1, y0, y1, k),
            material: material,
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
            geometry_type: GeometryType::Cuboid(p0, p1, sides),
            material: material,
        }
    }
    pub fn constant_medium(boundary: Geometry, density: f32, color: Rgb) -> Self {
        Self {
            geometry_type: GeometryType::ConstantMedium(Box::new(boundary), -1.0 / density),
            material: Material::isotropic(color),
        }
    }
    pub fn instance_translation(geometry: Geometry, displacement: Vec3) -> Self {
        Self {
            material: geometry.material.clone(),
            geometry_type: GeometryType::TranslateInstance(Box::new(geometry), displacement),
        }
    }
    pub fn instance_rotation(geometry: Geometry, axis: Axis, angle: f32) -> Self {
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

                    let tester;
                    match axis {
                        Axis::Y => {
                            let newx = cos_theta * x + sin_theta * z;
                            let newz = -sin_theta * x + cos_theta * z;
                            tester = Vec3::new(newx, y, newz);
                        },
                        Axis::X => {
                            let newy = cos_theta * y + sin_theta * z;
                            let newz = -sin_theta * y + cos_theta * z;
                            tester = Vec3::new(x, newy, newz);
                        },
                        Axis::Z => {
                            let newx = cos_theta * x + sin_theta * y;
                            let newy = -sin_theta * x + cos_theta * y;
                            tester = Vec3::new(newx, newy, z);
                        }
                    }

                    for c in 0..3 {
                        min[c] = min[c].min(tester[c]);
                        max[c] = max[c].max(tester[c]);
                    }
                }
            }
        }

        Self {
            material: geometry.material.clone(),
            geometry_type: GeometryType::YRotationInstance(Box::new(geometry), axis, angle, sin_theta, cos_theta, has_box, AABB::new(min, max)),
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
            material: left.material.clone(),
            geometry_type: GeometryType::BVHNode(Box::new(left), Box::new(right), bounding_box, axis),
        }
    }
    pub fn triangle(p0: Point3, p1: Point3, p2: Point3, material: Material) -> Self {
        let v0v1 = p1 - p0;
        let v0v2 = p2 - p0;
        Self {
            geometry_type: GeometryType::Triangle(p0, p1, p2, v0v1.cross(v0v2).normalize()),
            material: material,
        }
    }
    pub fn load_obj(path: &str, scale: f32, material: Material) -> Self {
        let mut vertices = Vec::new();
        let mut triangles: Vec<(usize, usize, usize)> = Vec::new();

        let file = File::open(path).unwrap();
        for line in BufReader::new(file).lines() {
            let res_line = line.unwrap();
            let mut splitted = res_line.split_whitespace();
            
            let action_option = splitted.next();
            let action;
            match action_option {
                Some(val) => action = val,
                None => continue
            }

            if action.starts_with("vt"){
            }
            else if action.starts_with("v") {
                vertices.push(Vec3::new(splitted.next().unwrap().parse::<f32>().unwrap() * scale, splitted.next().unwrap().parse::<f32>().unwrap() * scale, splitted.next().unwrap().parse::<f32>().unwrap() * scale))
            }
            else if action.starts_with("f") {
                if splitted.clone().next().unwrap().split("/").count() == 1 {
                    triangles.push((splitted.next().unwrap().parse().unwrap(), splitted.next().unwrap().parse().unwrap(), splitted.next().unwrap().parse().unwrap()))
                }
                else {
                    let mut p1 = splitted.next().unwrap().split("/");
                    let mut p2 = splitted.next().unwrap().split("/");
                    let mut p3 = splitted.next().unwrap().split("/");
                    triangles.push((p1.nth(0).unwrap().parse().unwrap(), p2.nth(0).unwrap().parse().unwrap(), p3.nth(0).unwrap().parse().unwrap()))
                }
            }
        };

        let mut tris = Vec::new();
        for t in triangles.iter() {
            tris.push(Geometry::triangle(vertices[t.0 - 1], vertices[t.1 - 1], vertices[t.2 - 1], material.clone()));
        };

        Geometry::bvh_node(&tris, 0., 1., 0, tris.len())
    }

    fn moving_sphere_center(&self, center0: Point3, center1: Point3, time0: f32, time1: f32, time: f32) -> Point3 {
        center0 + (center1 - center0) * ((time - time0) / (time1 - time0))
    }
    fn sphere_get_uv(&self, point: Point3) -> (f32, f32) {
        let theta = (-point.y).acos();
        let phi = (-point.z).atan2(point.x) + PI;
        (phi / TAU, theta / PI)
    }

    fn intersect_sphere(&self, center: &Point3, radius: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - *center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - radius * radius;

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
        let out_normal = (point - *center) / radius;
        rec.set_face_normal(r, out_normal);
        rec.material = self.material.clone();
        rec.set_uv(self.sphere_get_uv(out_normal));
        return Some(rec);
    }
    fn bounding_box_sphere(&self, center: &Point3, radius: f32, _time_0: f32, _time_1: f32) -> Option<AABB> {
        return Some(AABB::new(
            *center - Vec3::new(radius, radius, radius),
            *center + Vec3::new(radius, radius, radius)
        ));
    }

    fn intersect_moving_sphere(&self, center0: &Point3, center1: &Point3, time0: f32, time1: f32, radius: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = r.origin - self.moving_sphere_center(*center0, *center1, time0, time1, r.time);
        let a = r.direction.length_squared();
        let half_b = oc.dot(r.direction);
        let c = oc.length_squared() - radius * radius;

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
        let out_normal = (point - self.moving_sphere_center(*center0, *center1, time0, time1, r.time)) / radius;
        rec.set_face_normal(r, out_normal);
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_moving_sphere(&self, center0: &Point3, center1: &Point3, time0: f32, time1: f32, radius: f32, time_0: f32, time_1: f32) -> Option<AABB> {
        let box0 = AABB::new(
            self.moving_sphere_center(*center0, *center1, time0, time1, time_0) - Vec3::new(radius, radius, radius),
            self.moving_sphere_center(*center0, *center1, time0, time1, time_0) + Vec3::new(radius, radius, radius),
        );
        let box1 = AABB::new(
            self.moving_sphere_center(*center0, *center1, time0, time1, time_1) - Vec3::new(radius, radius, radius),
            self.moving_sphere_center(*center0, *center1, time0, time1, time_1) + Vec3::new(radius, radius, radius),
        );
        return Some(AABB::surrounding_box(box0, box1));
    }

    fn intersect_xyrect(&self, x0: f32, x1: f32, y0: f32, y1: f32, k: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (k - r.origin.z) / r.direction.z;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let x = r.origin.x + t * r.direction.x;
        let y = r.origin.y + t * r.direction.y;
        if (x < x0) || (x > x1) || (y < y0) || (y > y1) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((x - x0) / (x1 - x0), (y - y0) / (y1 - y0)));
        rec.set_face_normal(r, Vec3::new(0.0, 0.0, 1.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_xyrect(&self, x0: f32, x1: f32, y0: f32, y1: f32, k: f32, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(x0, y0, k - 0.0001),
            Point3::new(x1, y1, k + 0.0001)
        ))
    }

    fn intersect_xzrect(&self, x0: f32, x1: f32, z0: f32, z1: f32, k: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (k - r.origin.y) / r.direction.y;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let x = r.origin.x + t * r.direction.x;
        let z = r.origin.z + t * r.direction.z;
        if (x < x0) || (x > x1) || (z < z0) || (z > z1) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((x - x0) / (x1 - x0), (z - z0) / (z1 - z0)));
        rec.set_face_normal(r, Vec3::new(0.0, 1.0, 0.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_xzrect(&self, x0: f32, x1: f32, z0: f32, z1: f32, k: f32, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(x0, k - 0.0001, z0),
            Point3::new(x1, k + 0.0001, z1)
        ))
    }

    fn intersect_yzrect(&self, z0: f32, z1: f32, y0: f32, y1: f32, k: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (k - r.origin.x) / r.direction.x;
        if (t < t_min) || (t > t_max) {
            return None;
        }
        let y = r.origin.y + t * r.direction.y;
        let z = r.origin.z + t * r.direction.z;
        if (y < y0) || (y > y1) || (z < z0) || (z > z1) {
            return None;
        }
        let mut rec = HitRecord::new(r.at(t), t);
        rec.set_uv(((y - y0) / (y1 - y0), (z - z0) / (z1 - z0)));
        rec.set_face_normal(r, Vec3::new(1.0, 0.0, 0.0));
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_yzrect(&self, y0: f32, y1: f32, z0: f32, z1: f32, k: f32, _time_0: f32, _time_1: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(k - 0.0001, y0, z0),
            Point3::new(k + 0.0001, y1, z1)
        ))
    }

    fn intersect_cuboid(&self, sides: &SceneColliders, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        return sides.intersect(r, t_min, t_max);
    }
    fn bounding_box_cuboid(&self, p0: &Point3, p1: &Point3, _time_0: f32, _time_1: f32) -> Option<AABB> {
        return Some(AABB::new(*p0, *p1));
    }

    fn intersect_constant_medium(&self, boundary: &Box<Geometry>, neg_inv_density: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut rec1;
        match boundary.intersect(r, f32::MIN, f32::MAX) {
            Some(rec) => rec1 = rec,
            None => return None
        };
        
        let mut rec2;
        match boundary.intersect(r, rec1.t + 0.0001, f32::MAX) {
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
        let hit_dist = neg_inv_density * random().log10();

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
    fn bounding_box_constant_medium(&self, boundary: &Box<Geometry>, time_0: f32, time_1: f32) -> Option<AABB> {
        boundary.bounding_box(time_0, time_1)
    }

    fn intersect_translate_instance(&self, geometry: &Box<Geometry>, displacement: &Vec3, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_r = Ray::new(r.origin - *displacement, r.direction, r.time);
        match geometry.intersect(moved_r, t_min, t_max) {
            Some(mut rec) => {
                rec.point = rec.point + *displacement;
                rec.set_face_normal(moved_r, rec.normal);
                return Some(rec);
            }
            None => None
        }
    }
    fn bounding_box_translate_instance(&self, geometry: &Box<Geometry>, displacement: &Vec3, time_0: f32, time_1: f32) -> Option<AABB> {
        match geometry.bounding_box(time_0, time_1) {
            Some(aabb) => Some(AABB::new(aabb.minimum + *displacement, aabb.maximum + *displacement)),
            None => None
        }
    }

    fn intersect_rot_instance(&self, geometry: &Box<Geometry>, axis: &Axis, sin_theta: f32, cos_theta: f32, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = r.origin.clone();
        let mut dir = r.direction.clone();
        let others = axis.others();

        origin[others.0] = cos_theta * r.origin[others.0] - sin_theta * r.origin[others.1];
        origin[others.1] = sin_theta * r.origin[others.0] + cos_theta * r.origin[others.1];

        dir[others.0] = cos_theta * r.direction[others.0] - sin_theta * r.direction[others.1];
        dir[others.1] = sin_theta * r.direction[others.0] + cos_theta * r.direction[others.1];

        let rotated_ray = Ray::new(origin, dir, r.time);
        
        match geometry.intersect(rotated_ray, t_min, t_max) {
            Some(mut rec) => {
                let mut p = rec.point.clone();
                let mut norm = rec.normal.clone();
            
                p[others.0] = cos_theta * rec.point[others.0] + sin_theta * rec.point[others.1];
                p[others.1] = -sin_theta * rec.point[others.0] + cos_theta * rec.point[others.1];

                norm[others.0] = cos_theta * rec.normal[others.0] + sin_theta * rec.normal[others.1];
                norm[others.1] = -sin_theta * rec.normal[others.0] + cos_theta * rec.normal[others.1];

                rec.point = p;
                rec.set_face_normal(rotated_ray, norm);
                return Some(rec);
            }
            None => None
        }
    }
    fn bounding_box_rot_instance(&self, has_box: bool, bbox: &AABB, _time_0: f32, _time_1: f32) -> Option<AABB> {
        match has_box {
            true => Some(bbox.clone()),
            false => None
        }
    }

    fn intersect_bvh(&self, left: &Box<Geometry>, right: &Box<Geometry>, bounding_box: &Option<AABB>, axis: usize, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match &bounding_box {
            Some(bounding_box) => {
                if !bounding_box.intersect(r, t_min, t_max) {
                    return None;
                }
            }
            None => (),
        }

        if r.direction[axis] >= 0.0 {
            match left.intersect(r, t_min, t_max) {
                Some(left_hit) => {
                    let right_hit = right.intersect(r, t_min, left_hit.t);
                    match right_hit {
                        Some(right_hit) => Some(right_hit),
                        None => Some(left_hit),
                    }
                }
                None => {
                    let right_hit = left.intersect(r, t_min, t_max);
                    match right_hit {
                        Some(right_hit) => Some(right_hit),
                        None => None,
                    }
                }
            }
        } else {
            match right.intersect(r, t_min, t_max) {
                Some(right_hit) => {
                    let left_hit = left.intersect(r, t_min, right_hit.t);
                    match left_hit {
                        Some(left_hit) => Some(left_hit),
                        _ => Some(right_hit),
                    }
                }
                _ => {
                    let left_hit = left.intersect(r, t_min, t_max);
                    match left_hit {
                        Some(left_hit) => Some(left_hit),
                        _ => None,
                    }
                }
            }
        }
    }
    fn bounding_box_bvh(&self, bounding_box: &Option<AABB>, _time_0: f32, _time_1: f32) -> Option<AABB> {
        return bounding_box.clone();
    }

    fn intersect_triangle(&self, p0: Point3, p1: Point3, p2: Point3, plane_normal: Vec3, r: Ray, _t_min: f32, _t_max: f32) -> Option<HitRecord> {
        let n_dot_dir = plane_normal.dot(r.direction);
        if n_dot_dir.abs() < 0.000001 {
            return None;
        }

        let d = -plane_normal.dot(p0);
        let dist = -(plane_normal.dot(r.origin) + d) / n_dot_dir;

        if dist < 0.0 {
            return None;
        }

        let hit_pos = r.origin + r.direction * dist;

        let edge0 = p1 - p0;
        let vp0 = hit_pos - p0;
        let plane_perp = edge0.cross(vp0);
        if plane_normal.dot(plane_perp) < 0.0 {
            return None;
        }
        
        let edge1 = p2 - p1;
        let vp1 = hit_pos - p1;
        let plane_perp = edge1.cross(vp1);
        if plane_normal.dot(plane_perp) < 0.0 {
            return None;
        }
        
        let edge2 = p0 - p2;
        let vp2 = hit_pos - p2;
        let plane_perp = edge2.cross(vp2);
        if plane_normal.dot(plane_perp) < 0.0 {
            return None;
        }

        let mut rec = HitRecord::new(hit_pos, dist);
        rec.set_face_normal(r, plane_normal);
        rec.material = self.material.clone();
        return Some(rec);
    }
    fn bounding_box_triangle(&self, p0: &Point3, p1: &Point3, p2: &Point3, _time_0: f32, _time_1: f32) -> Option<AABB> {
        let maxp = Point3::new(
            p0.x.max(p1.x.max(p2.x)),
            p0.y.max(p1.y.max(p2.y)),
            p0.z.max(p1.z.max(p2.z)),
        );
        let minp = Point3::new(
            p0.x.min(p1.x.min(p2.x)),
            p0.y.min(p1.y.min(p2.y)),
            p0.z.min(p1.z.min(p2.z)),
        );
        return Some(AABB::new(minp, maxp));
    }

    pub fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match &self.geometry_type {
            GeometryType::Sphere(center, radius) => self.intersect_sphere(center, *radius, r, t_min, t_max),
            GeometryType::MovingSphere(center0, center1, time0, time1, radius) => self.intersect_moving_sphere(center0, center1, *time0, *time1, *radius, r, t_min, t_max),
            GeometryType::XYRect(x0, x1, y0, y1, k) => self.intersect_xyrect(*x0, *x1, *y0, *y1, *k, r, t_min, t_max),
            GeometryType::XZRect(x0, x1, z0, z1, k) => self.intersect_xzrect(*x0, *x1, *z0, *z1, *k, r, t_min, t_max),
            GeometryType::YZRect(z0, z1, y0, y1, k) => self.intersect_yzrect(*z0, *z1, *y0, *y1, *k, r, t_min, t_max),
            GeometryType::Cuboid(_p0, _p1, sides) => self.intersect_cuboid(sides, r, t_min, t_max),
            GeometryType::ConstantMedium(boundary, neg_inv_density) => self.intersect_constant_medium(boundary, *neg_inv_density, r, t_min, t_max),
            GeometryType::TranslateInstance(geometry, displacement) => self.intersect_translate_instance(geometry, displacement, r, t_min, t_max),
            GeometryType::YRotationInstance(geometry, axis, angle, sin_theta, cos_theta, has_box, aabb) => self.intersect_rot_instance(geometry, axis, *sin_theta, *cos_theta, r, t_min, t_max),
            GeometryType::BVHNode(left, right, bounding_box, axis) => self.intersect_bvh(left, right, bounding_box, *axis, r, t_min, t_max),
            GeometryType::Triangle(p0, p1, p2, plane_normal) => self.intersect_triangle(*p0, *p1, *p2, *plane_normal, r, t_min, t_max)
        }
    }
    pub fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        match &self.geometry_type {
            GeometryType::Sphere(center, radius) => self.bounding_box_sphere(center, *radius, time_0, time_1),
            GeometryType::MovingSphere(center0, center1, time0, time1, radius) => self.bounding_box_moving_sphere(center0, center1, *time0, *time1, *radius, time_0, time_1),
            GeometryType::XYRect(x0, x1, y0, y1, k) => self.bounding_box_xyrect(*x0, *x1, *y0, *y1, *k, time_0, time_1),
            GeometryType::XZRect(x0, x1, z0, z1, k) => self.bounding_box_xzrect(*x0, *x1, *z0, *z1, *k, time_0, time_1),
            GeometryType::YZRect(z0, z1, y0, y1, k) => self.bounding_box_yzrect(*y0, *y1, *z0, *z1, *k, time_0, time_1),
            GeometryType::Cuboid(p0, p1, _sides) => self.bounding_box_cuboid(p0, p1, time_0, time_1),
            GeometryType::ConstantMedium(boundary, _neg_inv_density) => self.bounding_box_constant_medium(boundary, time_0, time_1),
            GeometryType::TranslateInstance(geometry, displacement) => self.bounding_box_translate_instance(geometry, displacement, time_0, time_1),
            GeometryType::YRotationInstance(_geometry, _axis, _angle, _sin_theta, _cos_theta, has_box, aabb) => self.bounding_box_rot_instance(*has_box, aabb, time_0, time_1),
            GeometryType::BVHNode(_left, _right, bounding_box, _axis) => self.bounding_box_bvh(bounding_box, time_0, time_1),
            GeometryType::Triangle(p0, p1, p2, _plane_normal) => self.bounding_box_triangle(p0, p1, p2, time_0, time_1),
        }
    }
}




#[derive(Clone)]
pub struct SceneColliders {
    pub objects: Vec<Geometry>,
    pub atlas: Arc<Mutex<ImageTextureAtlas>>,
}

impl SceneColliders {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            atlas: Arc::new(Mutex::new(ImageTextureAtlas::new()))
        }
    }
    pub fn add(&mut self, object: Geometry) {
        self.objects.push(object);
    }
    pub fn load_image(&mut self, path: &str) -> Texture {
        let idx = self.atlas.lock().unwrap().load(path);
        Texture::image(&self.atlas, idx)
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