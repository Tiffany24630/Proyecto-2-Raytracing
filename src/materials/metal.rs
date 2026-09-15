use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn metal() -> Material {
    Material {
        kind: MaterialKind::Metal,
        albedo: Vec3::new(0.82, 0.68, 0.34),
        specular: 0.95,
        reflectivity: 0.62,
        transparency: 0.0,
        refractive_index: 1.0,
        emission: Vec3::new(0.0, 0.0, 0.0),
        texture_scale: 1.45,
        texture_weight: 0.74,
    }
}
