use crate::math::Vec3;

pub fn refract(incident: Vec3, normal: Vec3, eta_ratio: f32) -> Option<Vec3> {
    let cos_theta = (-incident).dot(normal).min(1.0);
    let perpendicular = (incident + normal * cos_theta) * eta_ratio;
    let parallel_length_squared = 1.0 - perpendicular.length_squared();

    if parallel_length_squared < 0.0 {
        return None;
    }

    let parallel = normal * -parallel_length_squared.sqrt();
    Some((perpendicular + parallel).normalized())
}

pub fn schlick_reflectance(cosine: f32, first_ior: f32, second_ior: f32) -> f32 {
    let ratio = (first_ior - second_ior) / (first_ior + second_ior);
    let r0 = ratio * ratio;
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}
