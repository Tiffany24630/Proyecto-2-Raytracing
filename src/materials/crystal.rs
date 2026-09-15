use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn crystal() -> Material {
    Material {
        kind: MaterialKind::Crystal,
        albedo: Vec3::new(0.48, 0.78, 0.96),
        specular: 1.0,
        reflectivity: 0.18,
        transparency: 0.88,
        refractive_index: 1.5,
        emission: Vec3::new(0.025, 0.065, 0.105),
        texture_scale: 1.10,
        texture_weight: 0.70,
    }
}
