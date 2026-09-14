use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn metal() -> Material {
    Material {
        kind: MaterialKind::Metal,
        albedo: Vec3::new(0.57, 0.64, 0.72),
        specular: 0.95,
        reflectivity: 0.78,
        transparency: 0.0,
        refractive_index: 1.0,
        emission: Vec3::new(0.0, 0.0, 0.0),
        texture_scale: 2.2,
        texture_weight: 0.82,
    }
}
