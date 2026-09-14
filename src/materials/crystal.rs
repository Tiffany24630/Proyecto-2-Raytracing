use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn crystal() -> Material {
    Material {
        kind: MaterialKind::Crystal,
        albedo: Vec3::new(0.22, 0.68, 0.86),
        specular: 1.0,
        reflectivity: 0.18,
        transparency: 0.88,
        refractive_index: 1.5,
        emission: Vec3::new(0.015, 0.055, 0.085),
        texture_scale: 1.25,
        texture_weight: 0.82,
    }
}
