use crate::{math::Vec3, raytracing::Camera};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortalView {
    Exterior,
    Interior,
    DesertPavilion,
    MahavaipulyaChamber,
    LuyangAcademy,
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
            Self::DesertPavilion => Camera::orbital(
                Vec3::new(0.0, 1.35, -3.25),
                7.2,
                0.0,
                5.0_f32.to_radians(),
                45.0,
                aspect_ratio,
            ),
            Self::MahavaipulyaChamber => Camera::orbital(
                Vec3::new(0.0, 1.1, -2.35),
                7.6,
                0.0,
                7.0_f32.to_radians(),
                47.0,
                aspect_ratio,
            ),
            Self::LuyangAcademy => Camera::orbital(
                Vec3::new(0.0, 1.2, -2.8),
                7.4,
                0.0,
                6.0_f32.to_radians(),
                46.0,
                aspect_ratio,
            ),
        }
    }
}
