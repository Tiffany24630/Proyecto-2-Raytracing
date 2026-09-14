use crate::{math::Vec3, raytracing::Camera};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalView {
    Exterior,
    Interior,
}

impl PortalView {
    pub fn camera(self, aspect_ratio: f32) -> Camera {
        match self {
            Self::Exterior => Camera::orbital(
                Vec3::new(0.0, 1.35, 8.65),
                9.5,
                0.0,
                10.0_f32.to_radians(),
                50.0,
                aspect_ratio,
            ),
            Self::Interior => Camera::orbital(
                Vec3::new(0.0, 1.0, -1.25),
                8.1,
                0.0,
                9.0_f32.to_radians(),
                50.0,
                aspect_ratio,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PortalView;

    #[test]
    fn portal_changes_camera_side_and_can_return() {
        let exterior = PortalView::Exterior.camera(16.0 / 9.0);
        let interior = PortalView::Interior.camera(16.0 / 9.0);

        assert!(exterior.position().z > 8.65);
        assert!(interior.position().z < 8.65);
    }

    #[test]
    fn interior_orbit_stays_inside_the_enlarged_hall() {
        let mut camera = PortalView::Interior.camera(16.0 / 9.0);
        camera.zoom(100.0);
        assert!((camera.radius() - 9.5).abs() < f32::EPSILON);

        for _ in 0..48 {
            let position = camera.position();
            assert!(position.x.abs() < 10.4);
            assert!(position.z > -12.4 && position.z < 8.4);
            camera.orbit(7.5_f32.to_radians(), 0.0);
        }
    }
}
