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
                Vec3::new(0.0, 1.0, 4.8),
                13.0,
                0.0,
                12.0_f32.to_radians(),
                48.0,
                aspect_ratio,
            ),
            Self::Interior => Camera::orbital(
                Vec3::new(0.0, 1.0, -1.25),
                5.0,
                0.0,
                10.0_f32.to_radians(),
                48.0,
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

        assert!(exterior.position().z > 4.8);
        assert!(interior.position().z < 4.8);
    }
}
