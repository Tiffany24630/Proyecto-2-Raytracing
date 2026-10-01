use crate::{
    geometry::{Cube, Object},
    materials::{Material, ink},
    math::Vec3,
    raytracing::Light,
};

use super::memory_core::MemoryCoreState;

const MELANTA_X: f32 = 0.0;
const MELANTA_Z: f32 = -3.55;

pub const fn melanta_light() -> Light {
    Light::new(Vec3::new(0.0, 7.2, 2.0), Vec3::new(1.0, 0.10, 0.07), 1.08)
}

pub fn melanta_transition_light(corruption: f32) -> Light {
    let t = corruption.clamp(0.0, 1.0);
    let restored = MemoryCoreState::Restored.light();
    let corrupted = melanta_light();
    Light::new(
        restored.position + (corrupted.position - restored.position) * t,
        restored.color + (corrupted.color - restored.color) * t,
        restored.intensity + (corrupted.intensity - restored.intensity) * t,
    )
}

/// AÃ±ade la figura corrupta de Melanta a la escena. `time` (segundos, reloj
/// determinista de la animaciÃ³n ambiental) hace que la figura entera respire
/// (se mece verticalmente muy despacio), que el ojo parpadee, que las
/// partÃ­culas floten cada una con su propia fase y que las vetas de
/// corrupciÃ³n del suelo pulsen. Con `time == 0.0` se obtiene exactamente la
/// pose estÃ¡tica original.
pub fn add_melanta_event(objects: &mut Vec<Box<dyn Object>>, time: f32) {
    let corruption = corrupted_ink();
    let mut flare = corruption;
    flare.albedo = Vec3::new(0.72, 0.015, 0.035);
    flare.emission = Vec3::new(0.58, 0.012, 0.035);
    flare.reflectivity = 0.08;
    flare.texture_weight = 0.30;

    // La figura entera respira muy despacio; todas sus partes comparten el
    // mismo desplazamiento vertical para que no se desarticule.
    let breath = 0.07 * (time * 0.85).sin();
    let sway = 0.05 * (time * 0.55).sin();

    // A monumental abstract Watcher placed above the core so it reads immediately.
    add_cube(
        objects,
        "melanta torso",
        Vec3::new(MELANTA_X + sway, 2.10 + breath, MELANTA_Z),
        Vec3::new(1.02, 2.70, 0.62),
        corruption,
    );
    add_cube(
        objects,
        "melanta head",
        Vec3::new(MELANTA_X + sway, 3.78 + breath, MELANTA_Z),
        Vec3::new(0.82, 0.82, 0.72),
        corruption,
    );
    for (index, (x, yaw)) in [(-1.05, -0.34), (1.05, 0.34)].into_iter().enumerate() {
        let flex = 0.10 * (time * 1.1 + index as f32 * std::f32::consts::PI).sin();
        objects.push(Box::new(Cube::from_center_rotated(
            "melanta arm",
            Vec3::new(MELANTA_X + x + sway, 2.66 + breath, MELANTA_Z + 0.05),
            Vec3::new(1.55, 0.26, 0.34),
            yaw + flex,
            corruption,
        )));
    }
    for x in [-0.34, 0.0, 0.34] {
        add_cube(
            objects,
            "melanta crown",
            Vec3::new(
                MELANTA_X + x + sway,
                4.35 + x.abs() * 0.42 + breath,
                MELANTA_Z,
            ),
            Vec3::new(0.16, 0.72, 0.16),
            corruption,
        );
    }

    for side in [-1.0_f32, 1.0] {
        for tier in 0..3 {
            let distance = 1.55 + tier as f32 * 0.82;
            let flutter = 0.05 * (time * 1.4 + tier as f32 * 1.1 + side).sin();
            add_cube(
                objects,
                "melanta spatial wing",
                Vec3::new(
                    MELANTA_X + side * distance + sway,
                    3.28 - tier as f32 * 0.38 + breath + flutter,
                    MELANTA_Z - 0.13,
                ),
                Vec3::new(1.18, 0.34, 0.24),
                corruption,
            );
        }
    }

    // El ojo parpadea: su emisiÃ³n sube y baja de intensidad con el tiempo, en
    // vez de quedarse fija, para que la figura se sienta observando.
    let eye_pulse = 0.55 + 0.45 * (time * 2.3).sin().max(0.0).powf(3.0);
    let mut eye = flare;
    eye.emission = flare.emission * (0.6 + 0.9 * eye_pulse);
    add_cube(
        objects,
        "melanta eye",
        Vec3::new(MELANTA_X + sway, 3.80 + breath, MELANTA_Z + 0.43),
        Vec3::new(0.38, 0.28, 0.10),
        eye,
    );

    // Cada partÃ­cula flota en su propia Ã³rbita pequeÃ±a y parpadea con una
    // fase distinta, asÃ­ que el conjunto se ve como ceniza/energÃ­a a la
    // deriva en vez de motas estÃ¡ticas.
    for (index, offset) in [
        (-2.65, 0.55, -2.90),
        (-2.20, 2.35, -3.55),
        (-1.72, 3.42, -2.72),
        (-1.30, 0.38, -1.72),
        (-0.82, 3.68, -1.55),
        (-0.45, 2.30, -2.82),
        (0.48, 3.18, -2.42),
        (0.76, 0.48, -2.15),
        (1.18, 2.65, -1.72),
        (1.62, 3.55, -3.15),
        (2.12, 1.15, -2.82),
        (2.68, 2.52, -3.62),
    ]
    .into_iter()
    .enumerate()
    {
        let (x, y, z) = offset;
        let phase = index as f32 * 0.83;
        let drift_y = 0.16 * (time * 1.2 + phase).sin();
        let drift_x = 0.09 * (time * 0.9 + phase * 1.4).cos();
        let flicker = 0.5 + 0.5 * (time * 3.1 + phase * 2.0).sin();
        let size = 0.10 + (index % 3) as f32 * 0.04;
        let mut particle = flare;
        particle.emission = flare.emission * (0.5 + flicker);
        add_cube(
            objects,
            "melanta particle",
            Vec3::new(x + drift_x, y + drift_y, z),
            Vec3::new(size, size * 1.8, size),
            particle,
        );
    }

    for (index, (center, size)) in [
        (Vec3::new(-2.25, -0.58, -1.15), Vec3::new(2.25, 0.07, 0.16)),
        (Vec3::new(2.18, -0.57, -2.05), Vec3::new(2.10, 0.08, 0.18)),
        (Vec3::new(0.0, -0.56, -3.52), Vec3::new(2.65, 0.06, 0.14)),
    ]
    .into_iter()
    .enumerate()
    {
        let pulse = 0.6 + 0.4 * (time * 1.15 + index as f32 * 2.05).sin();
        let mut vein = flare;
        vein.emission = flare.emission * pulse;
        add_cube(objects, "melanta corruption vein", center, size, vein);
    }
}

fn corrupted_ink() -> Material {
    Material {
        albedo: Vec3::new(0.055, 0.002, 0.010),
        emission: Vec3::new(0.14, 0.003, 0.025),
        reflectivity: 0.18,
        texture_weight: 0.68,
        ..ink()
    }
}

fn add_cube(
    objects: &mut Vec<Box<dyn Object>>,
    name: &'static str,
    center: Vec3,
    size: Vec3,
    material: Material,
) {
    objects.push(Box::new(Cube::from_center(name, center, size, material)));
}
