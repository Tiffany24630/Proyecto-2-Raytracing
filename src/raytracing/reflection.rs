use crate::math::Vec3;

pub const MAX_DEPTH: u32 = 4;

pub fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * (2.0 * incident.dot(normal))
}

#[cfg(test)]
mod tests {
    use super::reflect;
    use crate::math::Vec3;

    #[test]
    fn reflection_preserves_angle_and_length() {
        let incident = Vec3::new(1.0, -1.0, 0.0).normalized();
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let reflected = reflect(incident, normal);

        assert!((reflected - Vec3::new(1.0, 1.0, 0.0).normalized()).length() < 1e-6);
        assert!((reflected.length() - incident.length()).abs() < 1e-6);
        assert!((incident.dot(normal) + reflected.dot(normal)).abs() < 1e-6);
    }
}
