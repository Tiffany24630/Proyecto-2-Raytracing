use super::Vec3;

pub fn rotate_y(vector: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    Vec3::new(
        vector.x * cos + vector.z * sin,
        vector.y,
        -vector.x * sin + vector.z * cos,
    )
}

#[cfg(test)]
mod tests {
    use super::rotate_y;
    use crate::math::Vec3;

    #[test]
    fn quarter_turn_rotates_x_toward_negative_z() {
        let rotated = rotate_y(Vec3::new(1.0, 0.0, 0.0), std::f32::consts::FRAC_PI_2);
        assert!(rotated.x.abs() < 1e-6);
        assert!((rotated.z + 1.0).abs() < 1e-6);
    }
}
