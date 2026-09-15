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

    for (x, canvas) in [(-7.05, ink), (-5.45, crystal)] {
        add_cube(
            objects,
            "luyang painting frame",
            Vec3::new(x, 1.42, -5.78),
            Vec3::new(1.36, 1.95, 0.18),
            wood,
        );
        add_cube(
            objects,
            "luyang painting",
            Vec3::new(x, 1.42, -5.66),
            Vec3::new(1.08, 1.64, 0.08),
            canvas,
        );
    }
    for x in [-6.72, -5.68] {
        add_cube(
            objects,
            "luyang scroll",
            Vec3::new(x, 0.26, -3.95),
            Vec3::new(0.76, 0.07, 1.08),
            paper,
        );
    }
    add_cube(
        objects,
        "luyang ink stone",
        Vec3::new(-6.2, 0.33, -4.25),
        Vec3::new(0.38, 0.22, 0.38),
        ink,
    );
    for x in [-7.85, -4.55] {
        add_cube(
            objects,
            "luyang pavilion post",
            Vec3::new(x, 1.55, -4.25),
            Vec3::new(0.18, 3.25, 0.18),
            wood,
        );
    }
    add_cube(
        objects,
        "luyang pavilion eave",
        Vec3::new(-6.2, 3.12, -4.25),
        Vec3::new(4.05, 0.20, 0.48),
        wood,
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

    for x in [4.85, 7.55] {
        add_cube(
            objects,
            "mahavaipulya shelf upright",
            Vec3::new(x, 1.45, -5.72),
            Vec3::new(0.20, 3.20, 0.38),
            wood,
        );
    }
    for y in [0.08, 0.92, 1.76, 2.58] {
        add_cube(
            objects,
            "mahavaipulya shelf",
            Vec3::new(6.2, y, -5.72),
            Vec3::new(2.90, 0.14, 0.38),
            wood,
        );
    }
    for (x, y, material) in [
        (5.30, 0.46, ink),
        (5.82, 0.48, crystal),
        (6.36, 0.45, metal),
        (6.98, 1.31, ink),
    ] {
        add_cube(
            objects,
            "mahavaipulya book",
            Vec3::new(x, y, -5.48),
            Vec3::new(0.36, 0.62, 0.26),
            material,
        );
    }
    add_cube(
        objects,
        "mahavaipulya page",
        Vec3::new(6.2, 0.24, -3.95),
        Vec3::new(1.25, 0.06, 0.88),
        paper,
    );
    for (x, y, z) in [
        (5.15, 1.18, -3.75),
        (6.35, 1.72, -4.05),
        (7.25, 1.08, -3.70),
    ] {
        add_cube(
            objects,
            "mahavaipulya floating page",
            Vec3::new(x, y, z),
            Vec3::new(0.78, 0.05, 0.52),
            paper,
        );
    }
}

fn add_desert_pavilion(objects: &mut Vec<Box<dyn Object>>) {
    let mut sand = stone();
    sand.albedo = Vec3::new(0.72, 0.52, 0.27);
    sand.texture_weight = 0.28;
    let metal = metal();

    add_cube(
        objects,
        "desert sand platform",
        Vec3::new(-6.2, -0.36, 3.85),
        Vec3::new(4.05, 0.48, 3.65),
        sand,
    );
    for x in [-7.55, -4.85] {
        add_cube(
            objects,
            "desert pavilion pillar",
            Vec3::new(x, 1.22, 3.95),
            Vec3::new(0.42, 3.35, 0.46),
            sand,
        );
    }
    add_cube(
        objects,
        "desert pavilion arch",
        Vec3::new(-6.2, 2.86, 3.95),
        Vec3::new(3.25, 0.38, 0.52),
        sand,
    );
    add_cube(
        objects,
        "desert sculpture base",
        Vec3::new(-6.2, 0.02, 3.95),
        Vec3::new(1.15, 0.46, 1.15),
        sand,
    );
    add_cube(
        objects,
        "desert sculpture",
        Vec3::new(-6.2, 0.86, 3.95),
        Vec3::new(0.48, 1.22, 0.48),
        metal,
    );
    for (tier, size) in [(0, 1.55), (1, 1.12), (2, 0.70)] {
        add_cube(
            objects,
            "desert pyramid tier",
            Vec3::new(-6.2, 0.18 + tier as f32 * 0.24, 5.05),
            Vec3::new(size, 0.24, size),
            sand,
        );
    }
    for x in [-7.15, -5.25] {
        add_cube(
            objects,
            "desert obelisk",
            Vec3::new(x, 0.72, 4.85),
            Vec3::new(0.25, 1.55, 0.25),
            metal,
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
