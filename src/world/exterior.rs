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
        (-5.4, 11.0, 2.9),
        (5.5, 11.8, 2.1),
        (-6.2, 14.5, 1.8),
        (6.0, 15.2, 2.6),
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

    for (x, z, y) in [(-5.4, 11.0, 2.15), (6.0, 15.2, 1.75)] {
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
        (-3.4, -0.62, 12.2, Vec3::new(1.20, 0.82, 0.92)),
        (3.7, -0.70, 13.7, Vec3::new(0.92, 0.64, 1.25)),
        (-4.5, -0.73, 16.0, Vec3::new(0.78, 0.58, 0.74)),
        (3.0, -0.68, 10.4, Vec3::new(0.92, 0.70, 0.80)),
        (6.8, -0.74, 13.2, Vec3::new(0.72, 0.56, 0.92)),
    ] {
        add_cube(objects, "exterior rock", Vec3::new(x, y, z), size, stone);
    }
}

fn add_wind_fragments(objects: &mut Vec<Box<dyn Object>>, wind: Material) {
    for (x, y, z, size) in [
        (-2.4, 0.25, 11.3, 0.18),
        (2.8, 1.10, 11.9, 0.15),
        (-3.6, 1.75, 13.2, 0.22),
        (2.1, 2.15, 14.0, 0.14),
        (4.2, 0.55, 15.3, 0.19),
        (-1.7, 1.30, 16.0, 0.16),
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
