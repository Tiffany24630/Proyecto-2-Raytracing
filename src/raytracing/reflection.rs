use crate::math::Vec3;

pub const MAX_DEPTH: u32 = 4;

pub fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - normal * (2.0 * incident.dot(normal))
}
