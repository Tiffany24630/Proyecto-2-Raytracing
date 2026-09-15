use crate::{geometry::Object, raytracing::Camera};

pub(super) fn puzzle_piece_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<&'static str> {
    let ray = camera.ray(u, v);
    objects
        .iter()
        .filter(|object| object.name().starts_with("puzzle piece"))
        .filter_map(|object| object.hit(&ray, 0.001, f32::INFINITY))
        .min_by(|left, right| left.t.total_cmp(&right.t))
        .map(|hit| hit.object_name)
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
}
