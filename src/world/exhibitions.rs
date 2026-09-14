use crate::{
    geometry::{Cube, Object},
    materials::{Material, crystal, ink, metal, stone, wood},
    math::Vec3,
};

pub(super) fn add_exhibitions(objects: &mut Vec<Box<dyn Object>>) {
    add_luyang(objects);
    add_mahavaipulya(objects);
    add_desert_pavilion(objects);
}

fn add_luyang(objects: &mut Vec<Box<dyn Object>>) {
    let wood = wood();
    let ink = ink();
    let crystal = crystal();
    let mut paper = stone();
    paper.albedo = Vec3::new(0.76, 0.66, 0.48);
    paper.texture_weight = 0.30;

    for (x, canvas) in [(-3.55, ink), (-2.65, crystal)] {
        add_cube(
            objects,
            "luyang painting frame",
            Vec3::new(x, 1.20, -1.78),
            Vec3::new(0.82, 1.34, 0.16),
            wood,
        );
        add_cube(
            objects,
            "luyang painting",
            Vec3::new(x, 1.20, -1.68),
            Vec3::new(0.62, 1.12, 0.08),
            canvas,
        );
    }
    for x in [-3.42, -2.72] {
        add_cube(
            objects,
            "luyang scroll",
            Vec3::new(x, 0.34, -0.35),
            Vec3::new(0.52, 0.07, 0.78),
            paper,
        );
    }
    add_cube(
        objects,
        "luyang ink stone",
        Vec3::new(-3.0, 0.43, -0.62),
        Vec3::new(0.28, 0.22, 0.28),
        ink,
    );
}

fn add_mahavaipulya(objects: &mut Vec<Box<dyn Object>>) {
    let wood = wood();
    let ink = ink();
    let crystal = crystal();
    let metal = metal();
    let mut paper = stone();
    paper.albedo = Vec3::new(0.78, 0.72, 0.58);
    paper.texture_weight = 0.30;

    for x in [2.05, 3.95] {
        add_cube(
            objects,
            "mahavaipulya shelf upright",
            Vec3::new(x, 1.08, -1.72),
            Vec3::new(0.18, 2.45, 0.32),
            wood,
        );
    }
    for y in [0.12, 0.90, 1.68, 2.30] {
        add_cube(
            objects,
            "mahavaipulya shelf",
            Vec3::new(3.0, y, -1.72),
            Vec3::new(2.10, 0.14, 0.34),
            wood,
        );
    }
    for (x, y, material) in [
        (2.40, 0.48, ink),
        (2.83, 0.50, crystal),
        (3.28, 0.47, metal),
        (3.65, 1.25, ink),
    ] {
        add_cube(
            objects,
            "mahavaipulya book",
            Vec3::new(x, y, -1.50),
            Vec3::new(0.28, 0.56, 0.24),
            material,
        );
    }
    add_cube(
        objects,
        "mahavaipulya page",
        Vec3::new(3.0, 0.34, -0.32),
        Vec3::new(0.92, 0.06, 0.72),
        paper,
    );
}

fn add_desert_pavilion(objects: &mut Vec<Box<dyn Object>>) {
    let mut sand = stone();
    sand.albedo = Vec3::new(0.72, 0.52, 0.27);
    sand.texture_weight = 0.28;
    let metal = metal();

    add_cube(
        objects,
        "desert sand platform",
        Vec3::new(-2.85, -0.42, 2.25),
        Vec3::new(2.65, 0.48, 2.15),
        sand,
    );
    for x in [-3.72, -1.98] {
        add_cube(
            objects,
            "desert pavilion pillar",
            Vec3::new(x, 0.92, 2.15),
            Vec3::new(0.34, 2.68, 0.38),
            sand,
        );
    }
    add_cube(
        objects,
        "desert pavilion arch",
        Vec3::new(-2.85, 2.25, 2.15),
        Vec3::new(2.15, 0.34, 0.44),
        sand,
    );
    add_cube(
        objects,
        "desert sculpture base",
        Vec3::new(-2.85, 0.02, 2.52),
        Vec3::new(0.78, 0.42, 0.78),
        sand,
    );
    add_cube(
        objects,
        "desert sculpture",
        Vec3::new(-2.85, 0.66, 2.52),
        Vec3::new(0.38, 0.90, 0.38),
        metal,
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
