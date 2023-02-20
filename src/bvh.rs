#[path = "instances.rs"] mod instances;
use std::{sync::Arc};
use std::cmp::Ordering;
pub use instances::*;


fn box_compare(a: &Arc<dyn Geometry + Send + Sync>, b: &Arc<dyn Geometry + Send + Sync>, axis: usize) -> Ordering {
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

fn box_x_compare(a: &Arc<dyn Geometry + Send + Sync>, b: &Arc<dyn Geometry + Send + Sync>) -> Ordering {
    box_compare(a, b, 0)
}
fn box_y_compare(a: &Arc<dyn Geometry + Send + Sync>, b: &Arc<dyn Geometry + Send + Sync>) -> Ordering {
    box_compare(a, b, 1)
}
fn box_z_compare(a: &Arc<dyn Geometry + Send + Sync>, b: &Arc<dyn Geometry + Send + Sync>) -> Ordering {
    box_compare(a, b, 2)
}


pub struct BVHNode {
    pub left: Arc<dyn Geometry + Send + Sync>,
    pub right: Arc<dyn Geometry + Send + Sync>,
    pub bounding_box: Option<AABB>,
    pub axis: usize
}

impl Geometry for BVHNode {
    fn intersect(&self, r: Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match &self.bounding_box {
            Some(bounding_box) => {
                if !bounding_box.intersect(r, t_min, t_max) {
                    return None;
                }
            }
            None => (),
        }

        if r.direction[self.axis] >= 0.0 {
            match self.left.intersect(r, t_min, t_max) {
                Some(left_hit) => {
                    let right_hit = self.right.intersect(r, t_min, left_hit.t);
                    match right_hit {
                        Some(right_hit) => Some(right_hit),
                        None => Some(left_hit),
                    }
                }
                None => {
                    let right_hit = self.right.intersect(r, t_min, t_max);
                    match right_hit {
                        Some(right_hit) => Some(right_hit),
                        None => None,
                    }
                }
            }
        } else {
            match self.right.intersect(r, t_min, t_max) {
                Some(right_hit) => {
                    let left_hit = self.left.intersect(r, t_min, right_hit.t);
                    match left_hit {
                        Some(left_hit) => Some(left_hit),
                        _ => Some(right_hit),
                    }
                }
                _ => {
                    let left_hit = self.left.intersect(r, t_min, t_max);
                    match left_hit {
                        Some(left_hit) => Some(left_hit),
                        _ => None,
                    }
                }
            }
        }
    }
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        return self.bounding_box.clone();
    }
}

impl BVHNode {
    pub fn new(colliders: &Vec<Arc<dyn Geometry + Send + Sync>>, time_0: f32, time_1: f32, start: usize, end: usize) -> Self {
        let axis = randuint(0, 2);
        let comparator = if axis == 0 { box_x_compare } else if axis == 2 { box_y_compare } else { box_z_compare };
        let obj_span = end - start;
        let mut temp_colliders = colliders.clone();
        let left: Arc<dyn Geometry + Send + Sync>;
        let right: Arc<dyn Geometry + Send + Sync>;
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
            left = Arc::new(BVHNode::new(temp_colliders.clone().as_ref(), time_0, time_1, start, mid));
            right = Arc::new(BVHNode::new(temp_colliders.clone().as_ref(), time_0, time_1, mid, end));
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
            left: left,
            right: right,
            bounding_box: bounding_box,
            axis: axis
        }
    }
}