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

#[cfg(test)]
mod tests {
    use super::{refract, schlick_reflectance};
    use crate::math::Vec3;

    #[test]
    fn air_to_glass_bends_ray_toward_normal() {
        let incident = Vec3::new(0.6, -0.8, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let refracted = refract(incident, normal, 1.0 / 1.5).unwrap();

        assert!(refracted.x.abs() < incident.x.abs());
        assert!(refracted.y < 0.0);
        assert!((refracted.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn glass_to_air_can_produce_total_internal_reflection() {
        let incident = Vec3::new(3.0_f32.sqrt() * 0.5, 0.5, 0.0);
        let inward_normal = Vec3::new(0.0, -1.0, 0.0);

        assert!(refract(incident, inward_normal, 1.5).is_none());
    }

    #[test]
    fn fresnel_reflection_is_about_four_percent_at_normal_incidence() {
        let reflectance = schlick_reflectance(1.0, 1.0, 1.5);
        assert!((reflectance - 0.04).abs() < 1e-6);
    }
}
