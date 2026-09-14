use crate::math::Vec3;

use super::{Material, MaterialKind};

pub const fn wood() -> Material {
    Material {
        kind: MaterialKind::Wood,
        albedo: Vec3::new(0.58, 0.27, 0.12),
        specular: 0.22,
        reflectivity: 0.05,
        transparency: 0.0,
        refractive_index: 1.0,
        emission: Vec3::new(0.0, 0.0, 0.0),
        texture_scale: 1.0,
    }
}
