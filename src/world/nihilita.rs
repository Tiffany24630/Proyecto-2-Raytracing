use crate::{
    geometry::{Cube, Object},
    materials::{Material, crystal, metal, stone},
    math::Vec3,
};

const NIHILITA_X: f32 = 2.65;
const NIHILITA_Z: f32 = -2.85;

pub fn add_nihilita_epilogue(objects: &mut Vec<Box<dyn Object>>) {
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
        Vec3::new(NIHILITA_X, 0.28, NIHILITA_Z),
        Vec3::new(1.08, 1.45, 0.70),
        robes,
    );
    add_cube(
        objects,
        "nihilita torso",
        Vec3::new(NIHILITA_X, 1.33, NIHILITA_Z),
        Vec3::new(0.72, 0.82, 0.54),
        robes,
    );
    add_cube(
        objects,
        "nihilita head",
        Vec3::new(NIHILITA_X, 2.08, NIHILITA_Z),
        Vec3::new(0.54, 0.58, 0.52),
        memory_light,
    );
    for (side, yaw) in [(-1.0_f32, -0.34), (1.0, 0.34)] {
        objects.push(Box::new(Cube::from_center_rotated(
            "nihilita arm",
            Vec3::new(NIHILITA_X + side * 0.58, 1.30, NIHILITA_Z),
            Vec3::new(0.76, 0.18, 0.22),
            yaw,
            robes,
        )));
    }
    for x in [-0.32, 0.0, 0.32] {
        add_cube(
            objects,
            "nihilita celestial crown",
            Vec3::new(NIHILITA_X + x, 2.55 + x.abs() * 0.28, NIHILITA_Z),
            Vec3::new(0.10, 0.58, 0.10),
            gold,
        );
    }
    for (side, height) in [(-1.0_f32, 1.70), (1.0, 1.70), (-1.0, 1.18), (1.0, 1.18)] {
        add_cube(
            objects,
            "nihilita memory wing",
            Vec3::new(NIHILITA_X + side * 0.86, height, NIHILITA_Z - 0.10),
            Vec3::new(0.72, 0.14, 0.26),
            memory_light,
        );
    }
    add_cube(
        objects,
        "nihilita restored sigil",
        Vec3::new(NIHILITA_X, 1.34, NIHILITA_Z + 0.31),
        Vec3::new(0.24, 0.24, 0.08),
        memory_light,
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

#[cfg(test)]
mod tests {
    use super::add_nihilita_epilogue;

    #[test]
    fn epilogue_figure_is_distinct_and_keeps_a_small_object_budget() {
        let mut objects = Vec::new();
        add_nihilita_epilogue(&mut objects);
        assert!(objects.iter().any(|object| object.name() == "nihilita head"));
        assert!(objects
            .iter()
            .any(|object| object.name() == "nihilita restored sigil"));
        assert!(objects.len() <= 16);
    }
}
