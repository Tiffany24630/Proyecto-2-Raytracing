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
