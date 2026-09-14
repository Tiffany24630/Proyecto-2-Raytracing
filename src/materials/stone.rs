use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn stone() -> Material {
    Material {
        kind: MaterialKind::Stone,
        albedo: Vec3::new(0.40, 0.44, 0.52),
        specular: 0.15,
        reflectivity: 0.04,
        transparency: 0.0,
        refractive_index: 1.0,
        emission: Vec3::new(0.0, 0.0, 0.0),
        texture_scale: 1.4,
    }
}
