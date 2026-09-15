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

#[cfg(test)]
mod tests {
    use super::{lerp, lerp_angle};

    #[test]
    fn interpolation_clamps_and_angle_uses_shortest_path() {
        assert_eq!(lerp(2.0, 6.0, 0.5), 4.0);
        assert_eq!(lerp(2.0, 6.0, 2.0), 6.0);

        let angle = lerp_angle(350.0_f32.to_radians(), 10.0_f32.to_radians(), 0.5);
        assert!((angle.to_degrees() - 360.0).abs() < 1e-3);
    }
}
