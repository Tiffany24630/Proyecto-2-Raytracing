use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, crystal, ink, metal, stone, wood},
    math::Vec3,
    raytracing::Light,
};

use super::Scene;

pub fn build_library_room(collected_pages: [bool; 3], corruption: f32) -> Scene {
    let corruption = corruption.clamp(0.0, 1.0);
    let wood = wood();
    let metal = metal();
    let ink = ink();
    let mut floor = stone();
    floor.albedo = Vec3::new(0.20, 0.27, 0.34);
    floor.texture_weight = 0.58;
    let mut paper = stone();
    paper.albedo = Vec3::new(0.86, 0.73, 0.49);
    paper.texture_scale = 0.40;
    paper.texture_weight = 0.28;
    let mut memory = crystal();
    memory.albedo = Vec3::new(0.18, 0.64, 0.92);
    memory.emission = Vec3::new(0.05, 0.20, 0.34);

    let mut objects: Vec<Box<dyn Object>> = vec![Box::new(Plane::new(
        "library floor",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        floor,
    ))];

    for x in [-4.3, 4.3] {
        for y in [-0.55, 0.65, 1.85, 3.05] {
            add_cube(
                &mut objects,
                "library shelf",
                Vec3::new(x, y, -3.7),
                Vec3::new(2.2, 0.18, 0.72),
                wood,
            );
        }
        for y in [0.0, 1.2, 2.4] {
            add_cube(
                &mut objects,
                "library books",
                Vec3::new(x, y, -3.30),
                Vec3::new(1.72, 0.62, 0.24),
                ink,
            );
        }
    }

    add_cube(
        &mut objects,
        "library giant book pedestal",
        Vec3::new(0.0, -0.58, -3.25),
        Vec3::new(3.3, 0.72, 2.25),
        metal,
    );
    for x in [-0.82, 0.82] {
        add_cube(
            &mut objects,
            "library giant book",
            Vec3::new(x, 0.08, -3.25),
            Vec3::new(1.56, 0.18, 2.05),
            paper,
        );
    }

    for (index, (position, size)) in [
        (Vec3::new(-2.55, 0.35, -0.65), Vec3::new(0.92, 0.08, 0.66)),
        (Vec3::new(2.45, 1.48, -1.85), Vec3::new(0.82, 0.08, 0.62)),
        (Vec3::new(0.0, 2.32, -4.05), Vec3::new(0.96, 0.08, 0.68)),
    ]
    .into_iter()
    .enumerate()
    {
        if !collected_pages[index] {
            let name = match index {
                0 => "library page A",
                1 => "library page B",
                _ => "library page C",
            };
            add_cube(&mut objects, name, position, size, paper);
        } else {
            add_cube(
                &mut objects,
                "library restored page",
                Vec3::new(-0.52 + index as f32 * 0.52, 0.22, -3.25),
                Vec3::new(0.42, 0.05, 0.72),
                memory,
            );
        }
    }

    if corruption >= 0.35 {
        for (index, x) in [-3.1, -1.55, 0.0, 1.55, 3.1].into_iter().enumerate() {
            if corruption >= 0.7 || index % 2 == 0 {
                add_cube(
                    &mut objects,
                    "library melanta echo",
                    Vec3::new(x, 0.4 + (index % 2) as f32 * 1.25, -5.15),
                    Vec3::new(0.12, 1.15, 0.12),
                    ink,
                );
            }
        }
    }

    let calm_color = Vec3::new(0.54, 0.78, 1.0);
    let danger_color = Vec3::new(1.0, 0.08, 0.06);
    Scene {
        objects,
        light: Light::new(
            Vec3::new(-2.8, 6.8, 2.4),
            calm_color + (danger_color - calm_color) * corruption,
            1.05 + corruption * 0.10,
        ),
        camera_target: Vec3::new(0.0, 1.1, -2.35),
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
    use super::build_library_room;

    #[test]
    fn room_has_three_pages_and_a_bounded_object_budget() {
        let room = build_library_room([false; 3], 0.0);
        assert_eq!(
            room.objects
                .iter()
                .filter(|object| object.name().starts_with("library page"))
                .count(),
            3
        );
        assert!(room.objects.len() <= 28);
    }

    #[test]
    fn collected_page_moves_into_the_giant_book() {
        let room = build_library_room([true, false, false], 0.0);
        assert!(
            !room
                .objects
                .iter()
                .any(|object| object.name() == "library page A")
        );
        assert!(
            room.objects
                .iter()
                .any(|object| object.name() == "library restored page")
        );
    }

    #[test]
    fn approaching_melanta_changes_light_and_adds_only_cheap_echoes() {
        let calm = build_library_room([false; 3], 0.0);
        let imminent = build_library_room([false; 3], 0.72);
        assert!(imminent.light.color.x > calm.light.color.x);
        assert!(
            imminent
                .objects
                .iter()
                .any(|object| object.name() == "library melanta echo")
        );
        assert!(imminent.objects.len() <= 32);
    }
}
