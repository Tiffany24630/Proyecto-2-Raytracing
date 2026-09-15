use crate::{raytracing::Camera, world::MEMORY_CORE_CENTER};

use super::puzzle::{CAMERA_TOLERANCE, Puzzle};

pub const CAMERA_POSITION_TOLERANCE: f32 = 0.75;
pub const TARGET_RADIUS: f32 = 7.3;
pub const TARGET_YAW: f32 = 15.0_f32.to_radians();
pub const TARGET_PITCH: f32 = 9.0_f32.to_radians();

#[derive(Clone, Copy, Debug)]
pub struct PerspectiveCheck {
    pub aligned: bool,
    pub progress: f32,
    pub stage: PerspectiveStage,
    pub guidance: CameraGuidance,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerspectiveStage {
    Searching,
    Approaching,
    AlmostAligned,
    Aligned,
}

impl PerspectiveStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Searching => "BUSCANDO",
            Self::Approaching => "ACERCANDOTE",
            Self::AlmostAligned => "CASI ALINEADA",
            Self::Aligned => "CAMARA ALINEADA",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraGuidance {
    TurnLeft,
    TurnRight,
    TiltUp,
    TiltDown,
    ZoomIn,
    ZoomOut,
    FineTune,
    Correct,
}

impl CameraGuidance {
    pub const fn label(self) -> &'static str {
        match self {
            Self::TurnLeft => "GIRA A LA IZQUIERDA CON A",
            Self::TurnRight => "GIRA A LA DERECHA CON D",
            Self::TiltUp => "ELEVA LA VISTA CON W",
            Self::TiltDown => "BAJA LA VISTA CON S",
            Self::ZoomIn => "ACERCA LA CAMARA CON +",
            Self::ZoomOut => "ALEJA LA CAMARA CON -",
            Self::FineTune => "CASI ALINEADA: AJUSTE FINO",
            Self::Correct => "PERSPECTIVA CORRECTA",
        }
    }
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
    let aligned = position_error <= CAMERA_POSITION_TOLERANCE
        && target_error <= CAMERA_POSITION_TOLERANCE
        && orientation_error <= CAMERA_TOLERANCE;

    let yaw_error = signed_angle_delta(camera.yaw(), TARGET_YAW);
    let pitch_error = TARGET_PITCH - camera.pitch();
    let radius_error = TARGET_RADIUS - camera.radius();
    let yaw_ratio = (yaw_error.abs() / 30.0_f32.to_radians()).clamp(0.0, 1.0);
    let pitch_ratio = (pitch_error.abs() / 30.0_f32.to_radians()).clamp(0.0, 1.0);
    let radius_ratio = (radius_error.abs() / 1.6).clamp(0.0, 1.0);
    let progress = if aligned {
        1.0
    } else {
        1.0 - (yaw_ratio + pitch_ratio + radius_ratio) / 3.0
    };

    let guidance = if aligned {
        CameraGuidance::Correct
    } else if yaw_ratio < 0.05 && pitch_ratio < 0.05 && radius_ratio < 0.05 {
        CameraGuidance::FineTune
    } else if yaw_ratio >= pitch_ratio && yaw_ratio + 0.001 >= radius_ratio {
        if yaw_error > 0.0 {
            CameraGuidance::TurnRight
        } else {
            CameraGuidance::TurnLeft
        }
    } else if radius_ratio >= pitch_ratio {
        if radius_error < 0.0 {
            CameraGuidance::ZoomIn
        } else {
            CameraGuidance::ZoomOut
        }
    } else if pitch_error > 0.0 {
        CameraGuidance::TiltUp
    } else {
        CameraGuidance::TiltDown
    };
    let stage = if aligned {
        PerspectiveStage::Aligned
    } else if progress >= 0.82 {
        PerspectiveStage::AlmostAligned
    } else if progress >= 0.50 {
        PerspectiveStage::Approaching
    } else {
        PerspectiveStage::Searching
    };

    PerspectiveCheck {
        aligned,
        progress,
        stage,
        guidance,
    }
}

fn signed_angle_delta(current: f32, target: f32) -> f32 {
    (target - current + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::{CameraGuidance, PerspectiveStage, check_perspective, target_camera};
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
            8.1,
            0.0,
            9.0_f32.to_radians(),
            50.0,
            ASPECT_RATIO,
        );
        for _ in 0..3 {
            camera.orbit(5.0_f32.to_radians(), 0.0);
        }
        for _ in 0..2 {
            camera.zoom(-0.4);
        }

        assert!(check_perspective(&camera, ASPECT_RATIO).aligned);
    }

    #[test]
    fn feedback_improves_as_normal_controls_approach_the_target() {
        let mut camera = Camera::orbital(
            Vec3::new(0.0, 1.0, -1.25),
            8.1,
            0.0,
            9.0_f32.to_radians(),
            50.0,
            ASPECT_RATIO,
        );
        let initial = check_perspective(&camera, ASPECT_RATIO);
        assert_eq!(initial.stage, PerspectiveStage::Approaching);
        assert_eq!(initial.guidance, CameraGuidance::TurnRight);

        camera.orbit(15.0_f32.to_radians(), 0.0);
        let turned = check_perspective(&camera, ASPECT_RATIO);
        assert!(turned.progress > initial.progress);
        assert_eq!(turned.guidance, CameraGuidance::ZoomIn);

        camera.zoom(-0.8);
        let aligned = check_perspective(&camera, ASPECT_RATIO);
        assert_eq!(aligned.progress, 1.0);
        assert_eq!(aligned.stage, PerspectiveStage::Aligned);
        assert_eq!(aligned.guidance, CameraGuidance::Correct);
    }

    #[test]
    fn almost_aligned_feedback_keeps_the_final_direction_visible() {
        let mut camera = Camera::orbital(
            Vec3::new(0.0, 1.0, -1.25),
            7.3,
            8.0_f32.to_radians(),
            9.0_f32.to_radians(),
            50.0,
            ASPECT_RATIO,
        );
        let almost = check_perspective(&camera, ASPECT_RATIO);
        assert_eq!(almost.stage, PerspectiveStage::AlmostAligned);
        assert_eq!(almost.guidance, CameraGuidance::TurnRight);

        camera.orbit(5.0_f32.to_radians(), 0.0);
        assert!(check_perspective(&camera, ASPECT_RATIO).aligned);
    }
}
