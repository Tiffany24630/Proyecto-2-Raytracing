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
    _objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<crate::game::ExhibitionId> {
    let pointer = (u * 576.0, (1.0 - v) * 324.0);
    [
        (crate::math::Vec3::new(-3.4, 1.1, -2.0), crate::game::ExhibitionId::LuyangAcademy),
        (crate::math::Vec3::new(3.4, 1.1, -2.0), crate::game::ExhibitionId::MahavaipulyaChamber),
        (crate::math::Vec3::new(0.0, 1.1, 2.2), crate::game::ExhibitionId::DesertPavilion),
    ]
    .into_iter()
    .filter_map(|(position, exhibition)| {
        camera.project_to_screen(position, 576, 324).map(|(x, y)| (x as f32, y as f32, exhibition))
    })
    .filter_map(|(x, y, exhibition)| {
        let distance = (pointer.0 - x).hypot(pointer.1 - y);
        (distance <= 98.0).then_some((distance, exhibition))
    })
    .min_by(|left, right| left.0.total_cmp(&right.0))
    .map(|(_, exhibition)| exhibition)
}

pub fn academy_target_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<AcademyTarget> {
    let pointer = (u * 576.0, (1.0 - v) * 324.0);
    if let Some((x, y)) = camera.project_to_screen(crate::math::Vec3::new(0.0, -0.10, -1.65), 576, 324)
        && (pointer.0 - x as f32).hypot(pointer.1 - y as f32) <= 52.0
    {
        return Some(AcademyTarget::ConfirmButton);
    }
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
