use crate::{
    geometry::{Cube, Object},
    materials::{Material, ink},
    math::Vec3,
    raytracing::Light,
};

pub const fn melanta_light() -> Light {
    Light::new(Vec3::new(1.8, 5.2, 1.5), Vec3::new(1.0, 0.16, 0.10), 1.45)
}

pub fn add_melanta_event(objects: &mut Vec<Box<dyn Object>>) {
    let corruption = corrupted_ink();

    // A deliberately abstract observer: silhouette, crown and extended arms.
    add_cube(
        objects,
        "melanta torso",
        Vec3::new(0.0, 1.55, -4.28),
        Vec3::new(0.72, 2.45, 0.52),
        corruption,
    );
    add_cube(
        objects,
        "melanta head",
        Vec3::new(0.0, 3.02, -4.28),
        Vec3::new(0.58, 0.58, 0.58),
        corruption,
    );
    for (x, yaw) in [(-0.82, -0.34), (0.82, 0.34)] {
        objects.push(Box::new(Cube::from_center_rotated(
            "melanta arm",
            Vec3::new(x, 1.92, -4.24),
            Vec3::new(1.18, 0.22, 0.30),
            yaw,
            corruption,
        )));
    }
    for x in [-0.34, 0.0, 0.34] {
        add_cube(
            objects,
            "melanta crown",
            Vec3::new(x, 3.53 + x.abs() * 0.45, -4.28),
            Vec3::new(0.13, 0.62, 0.13),
            corruption,
        );
    }

    // Static particles keep this phase deterministic and inexpensive.
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
        let size = 0.10 + (index % 3) as f32 * 0.04;
        add_cube(
            objects,
            "melanta particle",
            Vec3::new(x, y, z),
            Vec3::new(size, size * 1.8, size),
            corruption,
        );
    }

    for (center, size) in [
        (Vec3::new(-2.25, -0.58, -1.15), Vec3::new(2.25, 0.07, 0.16)),
        (Vec3::new(2.18, -0.57, -2.05), Vec3::new(2.10, 0.08, 0.18)),
        (Vec3::new(0.0, -0.56, -3.52), Vec3::new(2.65, 0.06, 0.14)),
    ] {
        add_cube(objects, "melanta corruption vein", center, size, corruption);
    }
}

fn corrupted_ink() -> Material {
    Material {
        albedo: Vec3::new(0.24, 0.004, 0.015),
        emission: Vec3::new(0.68, 0.006, 0.025),
        reflectivity: 0.22,
        texture_weight: 0.58,
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

#[cfg(test)]
mod tests {
    use super::{add_melanta_event, melanta_light};

    #[test]
    fn event_adds_figure_particles_and_red_light() {
        let mut objects = Vec::new();
        add_melanta_event(&mut objects);
        let names: Vec<_> = objects.iter().map(|object| object.name()).collect();

        assert!(names.contains(&"melanta torso"));
        assert!(names.contains(&"melanta particle"));
        assert!(objects.len() >= 20);
        assert!(melanta_light().color.x > melanta_light().color.y * 4.0);
    }
}
