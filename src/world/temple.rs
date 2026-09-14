use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, crystal, ink, metal, stone, wood},
    math::Vec3,
    raytracing::Light,
};

use super::Scene;
use super::exhibitions::add_exhibitions;
use super::exterior::add_exterior;

pub fn build_temple() -> Scene {
    let stone = stone();
    let wood = wood();
    let metal = metal();
    let crystal = crystal();
    let ink = ink();
    for material in [stone, wood, metal, crystal, ink] {
        material.validate();
    }

    let mut objects: Vec<Box<dyn Object>> = Vec::new();
    objects.push(Box::new(Plane::new(
        "exterior floor",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        stone,
    )));
    add_exterior(&mut objects);

    add_cube(
        &mut objects,
        "temple floor",
        Vec3::new(0.0, -0.80, 0.0),
        Vec3::new(10.0, 0.35, 11.0),
        stone,
    );
    add_stairs(&mut objects, stone);
    add_walls(&mut objects, stone);
    add_columns(&mut objects, stone, metal);
    add_roof(&mut objects, stone, wood);
    add_platforms(&mut objects, stone, wood);
    add_exhibitions(&mut objects);
    add_portal(&mut objects, stone, metal, crystal);
    add_memory_pedestal(&mut objects, stone, metal, crystal, ink);

    Scene {
        objects,
        light: Light::new(Vec3::new(-4.0, 7.5, 4.5), Vec3::new(1.0, 0.91, 0.78), 1.35),
        camera_target: Vec3::new(0.0, 1.0, -0.25),
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

fn add_stairs(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for step in 0..6 {
        add_cube(
            objects,
            "temple stair",
            Vec3::new(0.0, -0.95 + step as f32 * 0.06, 7.15 - step as f32 * 0.36),
            Vec3::new(3.6, 0.14, 0.52),
            stone,
        );
    }
}

fn add_walls(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for x in [-5.0, 5.0] {
        add_cube(
            objects,
            "side wall",
            Vec3::new(x, -0.05, -1.3),
            Vec3::new(0.35, 1.30, 7.2),
            stone,
        );
    }
    for x in [-3.45, 3.45] {
        add_cube(
            objects,
            "rear wall",
            Vec3::new(x, 1.0, -5.0),
            Vec3::new(3.1, 3.4, 0.35),
            stone,
        );
    }
}

fn add_columns(objects: &mut Vec<Box<dyn Object>>, stone: Material, metal: Material) {
    for x in [-4.0, 4.0] {
        for z in [-3.75, 3.35] {
            add_cube(
                objects,
                "column base",
                Vec3::new(x, -0.48, z),
                Vec3::new(1.15, 0.55, 1.15),
                stone,
            );
            add_cube(
                objects,
                "column shaft",
                Vec3::new(x, 1.45, z),
                Vec3::new(0.65, 3.35, 0.65),
                stone,
            );
            add_cube(
                objects,
                "column capital",
                Vec3::new(x, 3.30, z),
                Vec3::new(1.20, 0.38, 1.20),
                metal,
            );
        }
    }
}

fn add_roof(objects: &mut Vec<Box<dyn Object>>, stone: Material, wood: Material) {
    for z in [-4.15, 0.0, 4.15] {
        add_cube(
            objects,
            "roof beam",
            Vec3::new(0.0, 4.05, z),
            Vec3::new(10.4, 0.38, 0.48),
            stone,
        );
    }
    for x in [-4.65, 0.0, 4.65] {
        add_cube(
            objects,
            "roof rafter",
            Vec3::new(x, 4.24, 0.0),
            Vec3::new(0.42, 0.28, 8.7),
            wood,
        );
    }
}

fn add_platforms(objects: &mut Vec<Box<dyn Object>>, stone: Material, wood: Material) {
    for x in [-3.0, 3.0] {
        add_cube(
            objects,
            "exhibition platform",
            Vec3::new(x, -0.46, -0.45),
            Vec3::new(2.55, 0.52, 2.85),
            stone,
        );
        add_cube(
            objects,
            "exhibition table",
            Vec3::new(x, 0.05, -0.45),
            Vec3::new(1.75, 0.50, 1.65),
            wood,
        );
    }
}

fn add_portal(
    objects: &mut Vec<Box<dyn Object>>,
    stone: Material,
    metal: Material,
    crystal: Material,
) {
    for x in [-1.9, 1.9] {
        add_cube(
            objects,
            "portal pillar",
            Vec3::new(x, 1.20, 4.80),
            Vec3::new(0.48, 4.05, 0.70),
            stone,
        );
    }
    add_cube(
        objects,
        "portal lintel",
        Vec3::new(0.0, 3.20, 4.80),
        Vec3::new(4.28, 0.48, 0.70),
        metal,
    );
    add_cube(
        objects,
        "portal membrane",
        Vec3::new(0.0, 1.17, 4.78),
        Vec3::new(3.30, 3.35, 0.16),
        crystal,
    );
}

fn add_memory_pedestal(
    objects: &mut Vec<Box<dyn Object>>,
    stone: Material,
    metal: Material,
    crystal: Material,
    ink: Material,
) {
    add_cube(
        objects,
        "pedestal base",
        Vec3::new(0.0, -0.38, -1.25),
        Vec3::new(2.55, 0.65, 2.55),
        stone,
    );
    add_cube(
        objects,
        "pedestal tier",
        Vec3::new(0.0, 0.02, -1.25),
        Vec3::new(1.82, 0.34, 1.82),
        metal,
    );
    add_cube(
        objects,
        "memory core",
        Vec3::new(0.0, 1.25, -1.25),
        Vec3::new(0.92, 0.92, 0.92),
        crystal,
    );

    for x in [-0.78, 0.78] {
        for z in [-2.03, -0.47] {
            add_cube(
                objects,
                "memory cage",
                Vec3::new(x, 1.35, z),
                Vec3::new(0.16, 3.10, 0.16),
                metal,
            );
        }
    }
    add_cube(
        objects,
        "memory cage crown",
        Vec3::new(0.0, 2.90, -1.25),
        Vec3::new(1.75, 0.18, 1.75),
        metal,
    );
    add_cube(
        objects,
        "preserved echo",
        Vec3::new(0.0, 0.55, -3.35),
        Vec3::new(0.72, 1.38, 0.72),
        ink,
    );
}

#[cfg(test)]
mod tests {
    use super::build_temple;

    #[test]
    fn temple_contains_every_required_architectural_group() {
        let temple = build_temple();
        let names: Vec<_> = temple.objects.iter().map(|object| object.name()).collect();

        for required in [
            "temple floor",
            "column shaft",
            "side wall",
            "exhibition platform",
            "temple stair",
            "pedestal base",
            "roof beam",
            "portal membrane",
            "memory core",
            "exterior path",
            "exterior ruin pillar",
            "exterior rock",
            "wind fragment",
            "luyang painting",
            "mahavaipulya book",
            "desert pavilion arch",
        ] {
            assert!(names.contains(&required), "missing '{required}'");
        }
        assert!((85..=105).contains(&temple.objects.len()));
    }
}
