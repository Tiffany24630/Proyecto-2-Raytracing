use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn stone() -> Material {
    Material {
        kind: MaterialKind::Stone,
        albedo: Vec3::new(0.82, 0.84, 0.88),
        specular: 0.24,
        reflectivity: 0.05,
        transparency: 0.0,
        refractive_index: 1.0,
        emission: Vec3::new(0.0, 0.0, 0.0),
        texture_scale: 1.15,
        texture_weight: 0.76,
    }
}
