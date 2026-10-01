use std::f32::consts::TAU;

use crate::{
    geometry::{Cube, Object},
    materials::crystal,
    math::Vec3,
    raytracing::Light,
};

use super::{MEMORY_CORE_CENTER, MemoryCorePose};

pub const SHARD_COUNT: usize = 6;

/// Pose del Memory Core en el instante `time` (segundos): gira despacio sobre
/// su eje y sube y baja unos centÃ­metros. Las partÃ­culas del nÃºcleo usan la
/// misma pose, asÃ­ que orbitan con Ã©l.
pub fn ambient_core_pose(time: f32) -> MemoryCorePose {
    MemoryCorePose {
        offset: Vec3::new(0.0, 0.07 * (time * 1.4).sin(), 0.0),
        yaw: time * 0.35,
    }
}

/// Fragmentos de cristal que flotan en Ã³rbita alrededor del Memory Core. Cada
/// uno gira sobre sÃ­ mismo, sube y baja con su propia fase y pulsa su emisiÃ³n.
pub fn add_ambient_shards(objects: &mut Vec<Box<dyn Object>>, time: f32) {
    for index in 0..SHARD_COUNT {
        let phase = index as f32 / SHARD_COUNT as f32 * TAU;
        let angle = phase + time * 0.45;
        let radius = 2.15 + 0.15 * (time * 0.7 + phase * 2.0).sin();
        let height =
            1.35 + 0.40 * (time * 0.9 + phase * 1.7).sin() + 0.25 * (index % 2) as f32;
        let pulse = 0.5 + 0.5 * (time * 2.0 + phase).sin();

        let mut material = crystal();
        material.transparency = 0.62;
        material.reflectivity = 0.14;
        material.texture_weight = 0.45;
        material.emission = Vec3::new(0.05, 0.14, 0.22) * (0.6 + 0.8 * pulse);

        let center = Vec3::new(
            MEMORY_CORE_CENTER.x + radius * angle.cos(),
            height,
            MEMORY_CORE_CENTER.z + radius * angle.sin(),
        );
        objects.push(Box::new(Cube::from_center_rotated(
            "ambient shard",
            center,
            Vec3::new(0.16, 0.34, 0.16),
            angle * 2.0 + time * 0.8,
            material,
        )));
    }
}

/// Luz cÃ¡lida que orbita junto al Memory Core y pulsa suavemente. Complementa
/// a la luz principal: tiÃ±e de dorado la piedra y el metal cercanos al nÃºcleo.
pub fn ambient_accent_light(time: f32) -> Light {
    let angle = time * 0.6;
    let position = Vec3::new(
        MEMORY_CORE_CENTER.x + 1.35 * angle.cos(),
        MEMORY_CORE_CENTER.y + 1.10 + 0.15 * (time * 1.3).sin(),
        MEMORY_CORE_CENTER.z + 1.35 * angle.sin(),
    );
    let pulse = 0.5 + 0.5 * (time * 1.7).sin();
    Light::new(position, Vec3::new(1.0, 0.74, 0.38), 1.05 + 0.45 * pulse)
}
