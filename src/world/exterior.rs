use crate::{
    geometry::{Cube, Object},
    materials::{Material, crystal, metal, stone},
    math::Vec3,
};

pub(super) fn add_exterior(objects: &mut Vec<Box<dyn Object>>) {
    let stone = stone();
    let metal = metal();
    let mut wind = crystal();
    wind.reflectivity = 0.05;
    wind.transparency = 0.24;
    wind.texture_weight = 0.40;

    add_path(objects, stone);
    add_ruins(objects, stone, metal);
    add_rocks(objects, stone);
    add_wind_fragments(objects, wind);
}

fn add_path(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for z in [7.2, 9.3, 11.4] {
        add_cube(
            objects,
            "exterior path",
            Vec3::new(0.0, -0.87, z),
            Vec3::new(3.4, 0.18, 1.82),
            stone,
        );
    }
}

fn add_ruins(objects: &mut Vec<Box<dyn Object>>, stone: Material, metal: Material) {
    for (x, z, height) in [
        (-4.7, 7.8, 2.4),
        (4.8, 8.5, 1.7),
        (-5.4, 11.1, 1.4),
        (5.2, 11.8, 2.1),
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

    for (x, z, y) in [(-4.7, 7.8, 1.69), (5.2, 11.8, 1.40)] {
        add_cube(
            objects,
            "exterior ruin capital",
            Vec3::new(x, y, z),
            Vec3::new(1.18, 0.30, 1.18),
            metal,
        );
    }
}

fn add_rocks(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for (x, y, z, size) in [
        (-2.8, -0.62, 9.0, Vec3::new(1.10, 0.75, 0.85)),
        (3.1, -0.70, 10.4, Vec3::new(0.82, 0.58, 1.15)),
        (-3.8, -0.73, 12.5, Vec3::new(0.70, 0.52, 0.68)),
        (2.5, -0.68, 7.4, Vec3::new(0.86, 0.64, 0.74)),
        (6.1, -0.74, 9.7, Vec3::new(0.62, 0.50, 0.82)),
    ] {
        add_cube(objects, "exterior rock", Vec3::new(x, y, z), size, stone);
    }
}

fn add_wind_fragments(objects: &mut Vec<Box<dyn Object>>, wind: Material) {
    for (x, y, z, size) in [
        (-2.0, 0.25, 8.4, 0.18),
        (2.4, 1.10, 8.9, 0.15),
        (-3.1, 1.75, 10.0, 0.22),
        (1.8, 2.15, 10.8, 0.14),
        (3.6, 0.55, 12.0, 0.19),
        (-1.4, 1.30, 12.5, 0.16),
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
