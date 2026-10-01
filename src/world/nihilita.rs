use crate::{
    geometry::{Cube, Object},
    materials::{Material, crystal, metal, stone},
    math::Vec3,
};

const NIHILITA_X: f32 = 2.65;
const NIHILITA_Z: f32 = -2.85;

/// AÃ±ade el epÃ­logo de Nihilita a la escena. `time` (segundos) hace que la
/// figura entera flote suavemente sobre el pedestal (que se queda fijo en el
/// suelo), que las alas de memoria destellen con fases distintas y que el
/// sigilo restaurado lata como un corazÃ³n sereno. Con `time == 0.0` se
/// obtiene exactamente la pose estÃ¡tica original.
pub fn add_nihilita_epilogue(objects: &mut Vec<Box<dyn Object>>, time: f32) {
    let mut robes = stone();
    robes.albedo = Vec3::new(0.56, 0.69, 0.82);
    robes.emission = Vec3::new(0.06, 0.11, 0.15);
    robes.texture_weight = 0.28;

    let mut memory_light = crystal();
    memory_light.albedo = Vec3::new(0.22, 0.78, 0.92);
    memory_light.emission = Vec3::new(0.18, 0.64, 0.78);
    memory_light.transparency = 0.28;
    memory_light.reflectivity = 0.16;

    let mut gold = metal();
    gold.albedo = Vec3::new(0.82, 0.60, 0.20);
    gold.reflectivity = 0.52;

    // El pedestal se queda fijo (es parte del mobiliario del templo); el
    // resto de la figura flota sereno sobre Ã©l, con una leve deriva lateral.
    let float = 0.06 * (time * 0.6).sin();
    let sway = 0.035 * (time * 0.4).sin();

    add_cube(
        objects,
        "nihilita memory dais",
        Vec3::new(NIHILITA_X, -0.50, NIHILITA_Z),
        Vec3::new(1.55, 0.18, 1.25),
        gold,
    );
    add_cube(
        objects,
        "nihilita lower robes",
        Vec3::new(NIHILITA_X + sway, 0.28 + float, NIHILITA_Z),
        Vec3::new(1.08, 1.45, 0.70),
        robes,
    );
    add_cube(
        objects,
        "nihilita torso",
        Vec3::new(NIHILITA_X + sway, 1.33 + float, NIHILITA_Z),
        Vec3::new(0.72, 0.82, 0.54),
        robes,
    );
    add_cube(
        objects,
        "nihilita head",
        Vec3::new(NIHILITA_X + sway, 2.08 + float, NIHILITA_Z),
        Vec3::new(0.54, 0.58, 0.52),
        memory_light,
    );
    for (side, yaw) in [(-1.0_f32, -0.34), (1.0, 0.34)] {
        objects.push(Box::new(Cube::from_center_rotated(
            "nihilita arm",
            Vec3::new(NIHILITA_X + side * 0.58 + sway, 1.30 + float, NIHILITA_Z),
            Vec3::new(0.76, 0.18, 0.22),
            yaw,
            robes,
        )));
    }
    for x in [-0.32, 0.0, 0.32] {
        add_cube(
            objects,
            "nihilita celestial crown",
            Vec3::new(
                NIHILITA_X + x + sway,
                2.55 + x.abs() * 0.28 + float,
                NIHILITA_Z,
            ),
            Vec3::new(0.10, 0.58, 0.10),
            gold,
        );
    }
    for (index, (side, height)) in [(-1.0_f32, 1.70), (1.0, 1.70), (-1.0, 1.18), (1.0, 1.18)]
        .into_iter()
        .enumerate()
    {
        // Cada ala destella con su propia fase: un shimmer sereno, no un
        // parpadeo caÃ³tico como el de Melanta.
        let shimmer = 0.65 + 0.35 * (time * 0.9 + index as f32 * 1.3).sin();
        let mut wing = memory_light;
        wing.emission = memory_light.emission * shimmer;
        add_cube(
            objects,
            "nihilita memory wing",
            Vec3::new(
                NIHILITA_X + side * 0.86 + sway,
                height + float,
                NIHILITA_Z - 0.10,
            ),
            Vec3::new(0.72, 0.14, 0.26),
            wing,
        );
    }

    // El sigilo late como un corazÃ³n en calma: un pulso lento y suave.
    let heartbeat = 0.75 + 0.25 * (time * 1.3).sin();
    let mut sigil = memory_light;
    sigil.emission = memory_light.emission * heartbeat;
    add_cube(
        objects,
        "nihilita restored sigil",
        Vec3::new(NIHILITA_X + sway, 1.34 + float, NIHILITA_Z + 0.31),
        Vec3::new(0.24, 0.24, 0.08),
        sigil,
    );
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
