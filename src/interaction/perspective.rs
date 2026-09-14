use crate::{raytracing::Camera, world::MEMORY_CORE_CENTER};

use super::puzzle::{CAMERA_TOLERANCE, Puzzle};

pub const CAMERA_POSITION_TOLERANCE: f32 = 0.40;
pub const TARGET_RADIUS: f32 = 7.4;
pub const TARGET_YAW: f32 = 7.5_f32.to_radians();
pub const TARGET_PITCH: f32 = 10.0_f32.to_radians();

#[derive(Clone, Copy, Debug)]
pub struct PerspectiveCheck {
    pub position_error: f32,
    pub target_error: f32,
    pub orientation_error: f32,
    pub aligned: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerspectiveStatus {
    Searching,
    CameraAligned,
    Solved,
}

impl PerspectiveStatus {
    pub fn from(puzzle: Puzzle, check: PerspectiveCheck) -> Self {
        if puzzle.is_solved(check.aligned) {
            Self::Solved
        } else if check.aligned {
            Self::CameraAligned
        } else {
            Self::Searching
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Searching => "SEARCHING",
            Self::CameraAligned => "PERSPECTIVE CORRECT",
            Self::Solved => "MEMORY ALIGNED",
        }
    }
}

pub fn target_camera(aspect_ratio: f32) -> Camera {
    Camera::orbital(
        MEMORY_CORE_CENTER,
        TARGET_RADIUS,
        TARGET_YAW,
        TARGET_PITCH,
        42.0,
        aspect_ratio,
    )
}

pub fn check_perspective(camera: &Camera, aspect_ratio: f32) -> PerspectiveCheck {
    let expected = target_camera(aspect_ratio);
    let position_error = (camera.position() - expected.position()).length();
    let target_error = (camera.target() - expected.target()).length();
    let orientation_error = camera
        .forward()
        .dot(expected.forward())
        .clamp(-1.0, 1.0)
        .acos();
    PerspectiveCheck {
        position_error,
        target_error,
        orientation_error,
        aligned: position_error <= CAMERA_POSITION_TOLERANCE
            && target_error <= CAMERA_POSITION_TOLERANCE
            && orientation_error <= CAMERA_TOLERANCE,
    }
}

#[cfg(test)]
mod tests {
    use super::{check_perspective, target_camera};
    use crate::{math::Vec3, raytracing::Camera};

    const ASPECT_RATIO: f32 = 16.0 / 9.0;

    #[test]
    fn target_camera_is_aligned_but_wrong_pose_is_not() {
        let target = target_camera(ASPECT_RATIO);
        assert!(check_perspective(&target, ASPECT_RATIO).aligned);

        let wrong = Camera::orbital(
            Vec3::new(0.0, 1.25, -1.25),
            9.0,
            1.0,
            0.0,
            42.0,
            ASPECT_RATIO,
        );
        assert!(!check_perspective(&wrong, ASPECT_RATIO).aligned);
    }

    #[test]
    fn target_is_reachable_from_interior_with_discrete_controls() {
        let mut camera = Camera::orbital(
            Vec3::new(0.0, 1.0, -1.25),
            5.0,
            0.0,
            10.0_f32.to_radians(),
            48.0,
            ASPECT_RATIO,
        );
        camera.orbit(7.5_f32.to_radians(), 0.0);
        camera.zoom(2.4);

        assert!(check_perspective(&camera, ASPECT_RATIO).aligned);
    }
}
