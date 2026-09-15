use crate::{geometry::Object, raytracing::Camera};

const SELECTION_RADIUS_UV: f32 = 0.012;

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

#[cfg(test)]
mod tests {
    use super::puzzle_piece_at;
    use crate::{
        geometry::{Cube, Object},
        materials::{crystal, stone},
        math::Vec3,
        raytracing::Camera,
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
}
