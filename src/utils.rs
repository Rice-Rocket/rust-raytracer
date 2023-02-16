pub use std::f32::consts::{PI, TAU};
use rand::{Rng, thread_rng};


pub fn to_radians(degrees: f32) -> f32 {
    degrees * PI / 180.0
}

pub fn random() -> f32 {
    return thread_rng().gen_range(0f32..1f32);
}

pub fn randrange(min: f32, max: f32) -> f32 {
    return thread_rng().gen_range(min..max);
}

pub fn clamp(x: f32, min: f32, max: f32) -> f32{
    if x < min {
        return min;
    }
    if x > max {
        return max;
    }
    return x;
}