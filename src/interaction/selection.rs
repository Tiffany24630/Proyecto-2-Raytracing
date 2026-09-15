use crate::{geometry::Object, raytracing::Camera};

const SELECTION_RADIUS_UV: f32 = 0.012;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcademyTarget {
    Fragment(u8),
    ConfirmButton,
}

pub(super) fn puzzle_piece_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<&'static str> {
    let mut nearest: Option<crate::raytracing::HitRecord> = None;
    for (offset_u, offset_v) in [
        (0.0, 0.0),
        (-SELECTION_RADIUS_UV, 0.0),
        (SELECTION_RADIUS_UV, 0.0),
        (0.0, -SELECTION_RADIUS_UV),
        (0.0, SELECTION_RADIUS_UV),
    ] {
        let ray = camera.ray(
            (u + offset_u).clamp(0.0, 1.0),
            (v + offset_v).clamp(0.0, 1.0),
        );
        for hit in objects
            .iter()
            .filter(|object| object.name().starts_with("puzzle piece"))
            .filter_map(|object| object.hit(&ray, 0.001, f32::INFINITY))
        {
            if nearest.is_none_or(|current| hit.t < current.t) {
                nearest = Some(hit);
            }
        }
    }
    nearest.map(|hit| hit.object_name)
}

pub fn exhibition_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<crate::game::ExhibitionId> {
    let mut nearest: Option<crate::raytracing::HitRecord> = None;
    for (offset_u, offset_v) in [
        (0.0, 0.0),
        (-SELECTION_RADIUS_UV, 0.0),
        (SELECTION_RADIUS_UV, 0.0),
        (0.0, -SELECTION_RADIUS_UV),
        (0.0, SELECTION_RADIUS_UV),
    ] {
        let ray = camera.ray(
            (u + offset_u).clamp(0.0, 1.0),
            (v + offset_v).clamp(0.0, 1.0),
        );
        for hit in objects
            .iter()
            .filter(|object| {
                object.name().starts_with("desert ")
                    || object.name().starts_with("mahavaipulya ")
                    || object.name().starts_with("luyang ")
            })
            .filter_map(|object| object.hit(&ray, 0.001, f32::INFINITY))
        {
            if nearest.is_none_or(|current| hit.t < current.t) {
                nearest = Some(hit);
            }
        }
    }
    let hit = nearest?;
    if hit.object_name.starts_with("desert ") {
        Some(crate::game::ExhibitionId::DesertPavilion)
    } else if hit.object_name.starts_with("mahavaipulya ") {
        Some(crate::game::ExhibitionId::MahavaipulyaChamber)
    } else {
        Some(crate::game::ExhibitionId::LuyangAcademy)
    }
}

pub fn academy_target_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<AcademyTarget> {
    let mut nearest: Option<crate::raytracing::HitRecord> = None;
    for (offset_u, offset_v) in [
        (0.0, 0.0),
        (-SELECTION_RADIUS_UV, 0.0),
        (SELECTION_RADIUS_UV, 0.0),
        (0.0, -SELECTION_RADIUS_UV),
        (0.0, SELECTION_RADIUS_UV),
    ] {
        let ray = camera.ray(
            (u + offset_u).clamp(0.0, 1.0),
            (v + offset_v).clamp(0.0, 1.0),
        );
        for hit in objects
            .iter()
            .filter(|object| {
                object.name().starts_with("academy fragment ")
                    || object.name() == "academy confirm button"
            })
            .filter_map(|object| object.hit(&ray, 0.001, f32::INFINITY))
        {
            if nearest.is_none_or(|current| hit.t < current.t) {
                nearest = Some(hit);
            }
        }
    }
    match nearest?.object_name {
        "academy fragment A" => Some(AcademyTarget::Fragment(0)),
        "academy fragment B" => Some(AcademyTarget::Fragment(1)),
        "academy fragment C" => Some(AcademyTarget::Fragment(2)),
        "academy confirm button" => Some(AcademyTarget::ConfirmButton),
        _ => None,
    }
}

pub fn library_page_at(camera: &Camera, objects: &[Box<dyn Object>], u: f32, v: f32) -> Option<u8> {
    let mut nearest: Option<crate::raytracing::HitRecord> = None;
    for (offset_u, offset_v) in [
        (0.0, 0.0),
        (-SELECTION_RADIUS_UV, 0.0),
        (SELECTION_RADIUS_UV, 0.0),
        (0.0, -SELECTION_RADIUS_UV),
        (0.0, SELECTION_RADIUS_UV),
    ] {
        let ray = camera.ray(
            (u + offset_u).clamp(0.0, 1.0),
            (v + offset_v).clamp(0.0, 1.0),
        );
        for hit in objects
            .iter()
            .filter(|object| object.name().starts_with("library page "))
            .filter_map(|object| object.hit(&ray, 0.001, f32::INFINITY))
        {
            if nearest.is_none_or(|current| hit.t < current.t) {
                nearest = Some(hit);
            }
        }
    }
    let hit = nearest?;
    match hit.object_name {
        "library page A" => Some(0),
        "library page B" => Some(1),
        "library page C" => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{AcademyTarget, academy_target_at, exhibition_at, library_page_at, puzzle_piece_at};
    use crate::{
        game::ExhibitionId,
        geometry::{Cube, Object},
        materials::{crystal, stone},
        math::Vec3,
        raytracing::Camera,
        world::{PortalView, build_academy_room, build_library_room, build_temple},
    };

    #[test]
    fn temple_geometry_does_not_steal_a_puzzle_selection() {
        let objects: Vec<Box<dyn Object>> = vec![
            Box::new(Cube::from_center(
                "memory cage",
                Vec3::new(0.0, 0.0, 2.0),
                Vec3::new(1.0, 1.0, 0.4),
                stone(),
            )),
            Box::new(Cube::from_center(
                "puzzle piece A",
                Vec3::default(),
                Vec3::new(0.65, 1.4, 0.22),
                crystal(),
            )),
        ];
        let camera = Camera::orbital(Vec3::default(), 6.0, 0.0, 0.0, 45.0, 16.0 / 9.0);

        assert_eq!(
            puzzle_piece_at(&camera, &objects, 0.5, 0.5),
            Some("puzzle piece A")
        );
    }

    #[test]
    fn selection_assists_a_nearby_cursor_without_targeting_scenery() {
        let objects: Vec<Box<dyn Object>> = vec![Box::new(Cube::from_center(
            "puzzle piece A",
            Vec3::default(),
            Vec3::new(0.65, 1.4, 0.22),
            crystal(),
        ))];
        let camera = Camera::orbital(Vec3::default(), 6.0, 0.0, 0.0, 45.0, 16.0 / 9.0);

        assert_eq!(
            puzzle_piece_at(&camera, &objects, 0.545, 0.5),
            Some("puzzle piece A")
        );
        assert_eq!(puzzle_piece_at(&camera, &objects, 0.75, 0.5), None);
    }

    #[test]
    fn desert_miniature_is_selectable_in_the_real_temple_scene() {
        let scene = build_temple();
        let mut camera = PortalView::Interior.camera(16.0 / 9.0);

        for _ in 0..72 {
            if let Some((x, y)) = camera.project_to_screen(Vec3::new(-6.2, 0.4, 3.95), 576, 324) {
                let u = x as f32 / 576.0;
                let v = 1.0 - y as f32 / 324.0;
                if exhibition_at(&camera, &scene.objects, u, v)
                    == Some(ExhibitionId::DesertPavilion)
                {
                    return;
                }
            }
            camera.orbit(5.0_f32.to_radians(), 0.0);
        }

        panic!("desert miniature must be reachable with normal orbit controls");
    }

    #[test]
    fn luyang_miniature_is_selectable_with_normal_orbit_controls() {
        let scene = build_temple();
        let mut camera = PortalView::Interior.camera(16.0 / 9.0);

        for _ in 0..72 {
            if let Some((x, y)) = camera.project_to_screen(Vec3::new(-6.2, 1.42, -5.66), 576, 324)
                && exhibition_at(
                    &camera,
                    &scene.objects,
                    x as f32 / 576.0,
                    1.0 - y as f32 / 324.0,
                ) == Some(ExhibitionId::LuyangAcademy)
            {
                return;
            }
            camera.orbit(5.0_f32.to_radians(), 0.0);
        }

        panic!("Luyang miniature must be reachable with normal orbit controls");
    }

    #[test]
    fn mahavaipulya_miniature_is_selectable_with_normal_orbit_controls() {
        let scene = build_temple();
        let mut camera = PortalView::Interior.camera(16.0 / 9.0);

        for _ in 0..72 {
            if let Some((x, y)) = camera.project_to_screen(Vec3::new(6.2, 1.45, -5.48), 576, 324)
                && exhibition_at(
                    &camera,
                    &scene.objects,
                    x as f32 / 576.0,
                    1.0 - y as f32 / 324.0,
                ) == Some(ExhibitionId::MahavaipulyaChamber)
            {
                return;
            }
            camera.orbit(5.0_f32.to_radians(), 0.0);
        }

        panic!("Mahavaipulya miniature must be reachable with normal orbit controls");
    }

    #[test]
    fn all_library_pages_are_selectable_from_normal_orbit_views() {
        let scene = build_library_room([false; 3], 0.0);
        let mut camera = PortalView::MahavaipulyaChamber.camera(16.0 / 9.0);
        let mut found = [false; 3];

        for _ in 0..72 {
            for (page, position) in [
                (0, Vec3::new(-2.55, 0.35, -0.65)),
                (1, Vec3::new(2.45, 1.48, -1.85)),
                (2, Vec3::new(0.0, 2.32, -4.05)),
            ] {
                if let Some((x, y)) = camera.project_to_screen(position, 576, 324) {
                    let u = x as f32 / 576.0;
                    let v = 1.0 - y as f32 / 324.0;
                    found[page] |=
                        library_page_at(&camera, &scene.objects, u, v) == Some(page as u8);
                }
            }
            camera.orbit(5.0_f32.to_radians(), 0.0);
        }

        assert_eq!(found, [true; 3]);
    }

    #[test]
    fn academy_fragments_and_button_are_selectable_from_the_entry_view() {
        let scene = build_academy_room([2, 0, 1], None);
        let camera = PortalView::LuyangAcademy.camera(16.0 / 9.0);

        for (position, expected) in [
            (Vec3::new(-1.62, 1.65, -4.42), AcademyTarget::Fragment(2)),
            (Vec3::new(0.0, 1.65, -4.42), AcademyTarget::Fragment(0)),
            (Vec3::new(1.62, 1.65, -4.42), AcademyTarget::Fragment(1)),
            (Vec3::new(0.0, -0.12, -1.65), AcademyTarget::ConfirmButton),
        ] {
            let (x, y) = camera
                .project_to_screen(position, 576, 324)
                .expect("academy target must be visible from the entry view");
            assert_eq!(
                academy_target_at(
                    &camera,
                    &scene.objects,
                    x as f32 / 576.0,
                    1.0 - y as f32 / 324.0,
                ),
                Some(expected)
            );
        }
    }
}
