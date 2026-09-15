use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn wood() -> Material {
    Material {
        kind: MaterialKind::Wood,
        albedo: Vec3::new(0.48, 0.16, 0.08),
        specular: 0.30,
        reflectivity: 0.05,
        transparency: 0.0,
        refractive_index: 1.0,
        emission: Vec3::new(0.0, 0.0, 0.0),
        texture_scale: 0.85,
        texture_weight: 0.78,
    }
}
