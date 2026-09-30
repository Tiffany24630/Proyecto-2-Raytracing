use super::Vec3;

pub fn rotate_y(vector: Vec3, angle: f32) -> Vec3 {
    let (sin, cos) = angle.sin_cos();
    Vec3::new(
        vector.x * cos + vector.z * sin,
        vector.y,
        -vector.x * sin + vector.z * cos,
    )
}
