use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, crystal, ink, metal, stone, wood},
    math::Vec3,
    raytracing::Light,
};

use super::Scene;

pub fn build_academy_room(order: [u8; 3], selected: Option<u8>) -> Scene {
    let stone = stone();
    let wood = wood();
    let metal = metal();
    let crystal = crystal();
    let ink = ink();
    let mut floor = stone;
    floor.albedo = Vec3::new(0.35, 0.25, 0.20);
    floor.texture_weight = 0.62;

    let mut objects: Vec<Box<dyn Object>> = vec![Box::new(Plane::new(
        "academy gallery floor",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        floor,
    ))];

    add_cube(
        &mut objects,
        "academy gallery wall",
        Vec3::new(0.0, 1.8, -4.85),
        Vec3::new(9.4, 5.6, 0.34),
        stone,
    );
    for x in [-4.35, 4.35] {
        add_cube(
            &mut objects,
            "academy pavilion post",
            Vec3::new(x, 1.45, -3.75),
            Vec3::new(0.42, 4.9, 0.42),
            wood,
        );
    }
    add_cube(
        &mut objects,
        "academy painting cornice",
        Vec3::new(0.0, 3.35, -4.48),
        Vec3::new(6.15, 0.30, 0.30),
        metal,
    );

    let slot_x = [-1.62, 0.0, 1.62];
    for (slot, fragment) in order.into_iter().enumerate() {
        let material = match fragment {
            0 => crystal,
            1 => ink,
            _ => metal,
        };
        let name = match fragment {
            0 => "academy fragment A",
            1 => "academy fragment B",
            _ => "academy fragment C",
        };
        add_cube(
            &mut objects,
            name,
            Vec3::new(slot_x[slot], 1.65, -4.42),
            Vec3::new(1.46, 2.72, 0.16),
            material,
        );
        add_cube(
            &mut objects,
            "academy painting motif",
            Vec3::new(slot_x[slot], 1.15 + fragment as f32 * 0.48, -4.31),
            Vec3::new(0.62 + fragment as f32 * 0.14, 0.22, 0.08),
            wood,
        );
    }

    if let Some(fragment) = selected {
        let slot = order
            .iter()
            .position(|candidate| *candidate == fragment)
            .expect("selected painting fragment must exist");
        for y in [0.22, 3.08] {
            add_cube(
                &mut objects,
                "academy selection halo",
                Vec3::new(slot_x[slot], y, -4.24),
                Vec3::new(1.68, 0.08, 0.08),
                crystal,
            );
        }
    }

    add_cube(
        &mut objects,
        "academy confirm pedestal",
        Vec3::new(0.0, -0.55, -1.65),
        Vec3::new(1.45, 0.75, 1.25),
        stone,
    );
    add_cube(
        &mut objects,
        "academy confirm button",
        Vec3::new(0.0, -0.12, -1.65),
        Vec3::new(0.62, 0.14, 0.62),
        crystal,
    );

    Scene {
        objects,
        light: Light::new(
            Vec3::new(-3.2, 7.0, 2.2),
            Vec3::new(1.0, 0.69, 0.43),
            1.08,
        ),
        camera_target: Vec3::new(0.0, 1.2, -2.8),
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
    use super::build_academy_room;

    #[test]
    fn gallery_contains_three_fragments_and_confirmation_button() {
        let room = build_academy_room([2, 0, 1], None);
        assert_eq!(
            room.objects
                .iter()
                .filter(|object| object.name().starts_with("academy fragment"))
                .count(),
            3
        );
        assert!(room
            .objects
            .iter()
            .any(|object| object.name() == "academy confirm button"));
        assert!(room.objects.len() <= 18);
    }
}
