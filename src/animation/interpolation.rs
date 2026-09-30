use crate::math::Vec3;

pub fn lerp(start: f32, end: f32, t: f32) -> f32 {
    match t {
        t if t <= 0.0 => start,
        t if t >= 1.0 => end,
        t => start + (end - start) * t,
    }
}

pub fn lerp_vec3(start: Vec3, end: Vec3, t: f32) -> Vec3 {
    match t {
        t if t <= 0.0 => start,
        t if t >= 1.0 => end,
        t => start + (end - start) * t,
    }
}

pub fn lerp_angle(start: f32, end: f32, t: f32) -> f32 {
    if t <= 0.0 {
        return start;
    }
    if t >= 1.0 {
        return end;
    }
    let tau = std::f32::consts::TAU;
    let difference = (end - start + std::f32::consts::PI).rem_euclid(tau) - std::f32::consts::PI;
    start + difference * t
}