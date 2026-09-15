use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, crystal, metal, stone},
    math::Vec3,
};

pub(super) fn add_exterior(objects: &mut Vec<Box<dyn Object>>) {
    let stone = stone();
    let metal = metal();
    let mut grass = stone;
    grass.albedo = Vec3::new(0.22, 0.42, 0.16);
    grass.specular = 0.06;
    grass.reflectivity = 0.01;
    grass.texture_scale = 0.22;
    grass.texture_weight = 0.92;
    let mut ruin_stone = stone;
    ruin_stone.albedo = Vec3::new(0.52, 0.59, 0.64);
    ruin_stone.texture_weight = 0.58;
    let mut rock = stone;
    rock.albedo = Vec3::new(0.30, 0.38, 0.43);
    rock.specular = 0.10;
    rock.reflectivity = 0.02;
    rock.texture_scale = 0.72;
    rock.texture_weight = 0.42;
    let mut wind = crystal();
    wind.reflectivity = 0.05;
    wind.transparency = 0.18;
    wind.emission = Vec3::new(0.06, 0.18, 0.24);
    wind.texture_weight = 0.32;

    add_grass_terrain(objects, grass);
    add_path(objects, stone);
    add_ruins(objects, ruin_stone, metal);
    add_rocks(objects, rock);
    add_wind_fragments(objects, wind);
}

fn add_grass_terrain(objects: &mut Vec<Box<dyn Object>>, grass: Material) {
    objects.push(Box::new(Plane::new(
        "exterior grass",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        grass,
    )));
    for x in [-4.45, 4.45] {
        add_cube(
            objects,
            "exterior grass verge",
            Vec3::new(x, -0.91, 13.4),
            Vec3::new(4.55, 0.14, 7.2),
            grass,
        );
    }
}

fn add_path(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for z in [10.7, 12.9, 15.1] {
        add_cube(
            objects,
            "exterior path",
            Vec3::new(0.0, -0.87, z),
            Vec3::new(4.2, 0.18, 1.86),
            stone,
        );
    }
}

fn add_ruins(objects: &mut Vec<Box<dyn Object>>, stone: Material, metal: Material) {
    for (x, z, height) in [
        (-4.65, 11.2, 3.25),
        (4.85, 11.8, 2.35),
        (-5.25, 14.5, 1.95),
        (5.10, 15.0, 2.85),
    ] {
        add_cube(
            objects,
            "exterior ruin base",
            Vec3::new(x, -0.69, z),
            Vec3::new(1.35, 0.58, 1.35),
            stone,
        );
        add_cube(
            objects,
            "exterior ruin pillar",
            Vec3::new(x, -0.40 + height * 0.5, z),
            Vec3::new(0.72, height, 0.72),
            stone,
        );
    }

    for (x, z, y, size) in [
        (-3.55, 11.2, 2.72, Vec3::new(2.65, 0.30, 0.62)),
        (4.15, 15.0, 2.18, Vec3::new(2.25, 0.28, 0.58)),
    ] {
        add_cube(
            objects,
            "exterior ruin lintel",
            Vec3::new(x, y, z),
            size,
            metal,
        );
    }
}

fn add_rocks(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for (x, y, z, size, yaw) in [
        (-3.45, -0.42, 12.0, Vec3::new(1.80, 1.28, 1.35), 18.0_f32),
        (3.65, -0.53, 13.3, Vec3::new(1.48, 1.02, 1.82), -24.0),
        (-4.65, -0.54, 15.5, Vec3::new(1.28, 1.06, 1.05), 34.0),
        (2.85, -0.50, 10.5, Vec3::new(1.42, 1.02, 1.18), 27.0),
        (5.55, -0.55, 15.7, Vec3::new(1.32, 0.94, 1.60), -32.0),
    ] {
        add_rotated_cube(
            objects,
            "exterior rock",
            Vec3::new(x, y, z),
            size,
            yaw.to_radians(),
            stone,
        );
    }
}

fn add_wind_fragments(objects: &mut Vec<Box<dyn Object>>, wind: Material) {
    for (x, y, z, size) in [
        (-2.40, 0.35, 11.1, 0.20),
        (2.55, 1.18, 11.8, 0.17),
        (-3.25, 1.85, 13.0, 0.23),
        (2.05, 2.30, 14.0, 0.16),
        (3.85, 0.65, 15.1, 0.21),
        (-1.55, 1.42, 16.0, 0.18),
    ] {
        add_cube(
            objects,
            "wind fragment",
            Vec3::new(x, y, z),
            Vec3::new(size, size * 2.8, size),
            wind,
        );
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

fn add_rotated_cube(
    objects: &mut Vec<Box<dyn Object>>,
    name: &'static str,
    center: Vec3,
    size: Vec3,
    yaw: f32,
    material: Material,
) {
    objects.push(Box::new(Cube::from_center_rotated(
        name, center, size, yaw, material,
    )));
}
