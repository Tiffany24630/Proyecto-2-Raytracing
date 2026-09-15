use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, crystal, metal, stone},
    math::Vec3,
    raytracing::Light,
};

use super::Scene;

pub fn build_desert_room_with_seal(seal_yaw: f32) -> Scene {
    let mut sand = stone();
    sand.albedo = Vec3::new(0.66, 0.40, 0.17);
    sand.texture_scale = 0.34;
    sand.texture_weight = 0.72;

    let mut sandstone = stone();
    sandstone.albedo = Vec3::new(0.73, 0.52, 0.27);
    sandstone.texture_weight = 0.48;

    let mut gold = metal();
    gold.albedo = Vec3::new(0.78, 0.49, 0.12);
    gold.reflectivity = 0.42;

    let mut memory = crystal();
    memory.albedo = Vec3::new(0.10, 0.66, 0.88);
    memory.emission = Vec3::new(0.06, 0.24, 0.34);

    let mut objects: Vec<Box<dyn Object>> = vec![Box::new(Plane::new(
        "desert room sand",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        sand,
    ))];

    for x in [-3.65, 3.65] {
        add_cube(
            &mut objects,
            "desert room pillar",
            Vec3::new(x, 1.45, -3.2),
            Vec3::new(0.82, 4.9, 0.82),
            sandstone,
        );
        add_cube(
            &mut objects,
            "desert room capital",
            Vec3::new(x, 4.02, -3.2),
            Vec3::new(1.34, 0.28, 1.34),
            gold,
        );
    }

    add_cube(
        &mut objects,
        "desert room celestial lintel",
        Vec3::new(0.0, 4.18, -3.2),
        Vec3::new(8.2, 0.32, 0.74),
        gold,
    );

    for (level, size) in [(0, 4.8), (1, 3.7), (2, 2.6), (3, 1.5)] {
        add_cube(
            &mut objects,
            "desert room stepped shrine",
            Vec3::new(0.0, -0.78 + level as f32 * 0.34, -3.8),
            Vec3::new(size, 0.34, size),
            sandstone,
        );
    }

    objects.push(Box::new(Cube::from_center_rotated(
        "desert room memory seal",
        Vec3::new(0.0, 2.15, -3.8),
        Vec3::new(0.34, 2.65, 1.12),
        seal_yaw,
        memory,
    )));
    add_cube(
        &mut objects,
        "desert room return gate",
        Vec3::new(0.0, 1.3, 4.6),
        Vec3::new(3.4, 4.4, 0.34),
        gold,
    );

    Scene {
        objects,
        light: Light::new(Vec3::new(-3.5, 7.5, 3.5), Vec3::new(1.0, 0.78, 0.48), 1.08),
        camera_target: Vec3::new(0.0, 1.35, -3.25),
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

#[cfg(test)]
mod tests {
    use super::build_desert_room_with_seal;

    #[test]
    fn provisional_room_is_independent_and_keeps_a_small_object_budget() {
        let room = build_desert_room_with_seal(0.0);
        assert!(
            room.objects
                .iter()
                .any(|object| object.name() == "desert room sand")
        );
        assert!(
            room.objects
                .iter()
                .any(|object| object.name() == "desert room return gate")
        );
        assert!(room.objects.len() <= 16);
    }
}
