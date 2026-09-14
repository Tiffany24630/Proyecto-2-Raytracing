use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn ink() -> Material {
    Material {
        kind: MaterialKind::Ink,
        albedo: Vec3::new(0.10, 0.025, 0.16),
        specular: 0.48,
        reflectivity: 0.12,
        transparency: 0.08,
        refractive_index: 1.2,
        emission: Vec3::new(0.30, 0.015, 0.48),
        texture_scale: 1.1,
        texture_weight: 0.82,
    }
}
