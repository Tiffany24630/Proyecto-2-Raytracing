use crate::math::Vec3;

use super::Ray;

pub const MIN_RADIUS: f32 = 5.0;
pub const MAX_RADIUS: f32 = 18.0;
pub const MIN_PITCH: f32 = -10.0_f32.to_radians();
pub const MAX_PITCH: f32 = 80.0_f32.to_radians();

pub struct Camera {
    target: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    vertical_fov_degrees: f32,
    aspect_ratio: f32,
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn orbital(
        target: Vec3,
        radius: f32,
        yaw: f32,
        pitch: f32,
        vertical_fov_degrees: f32,
        aspect_ratio: f32,
    ) -> Self {
        let mut camera = Self {
            target,
            radius: radius.clamp(MIN_RADIUS, MAX_RADIUS),
            yaw,
            pitch: pitch.clamp(MIN_PITCH, MAX_PITCH),
            vertical_fov_degrees,
            aspect_ratio,
            origin: Vec3::default(),
            lower_left_corner: Vec3::default(),
            horizontal: Vec3::default(),
            vertical: Vec3::default(),
        };
        camera.update_view();
        camera
    }

    pub fn orbit(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw = (self.yaw + yaw_delta).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + pitch_delta).clamp(MIN_PITCH, MAX_PITCH);
        self.update_view();
    }

    pub fn zoom(&mut self, radius_delta: f32) {
        self.radius = (self.radius + radius_delta).clamp(MIN_RADIUS, MAX_RADIUS);
        self.update_view();
    }

    pub fn ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin,
        )
    }

    pub fn position(&self) -> Vec3 {
        self.origin
    }

    pub fn target(&self) -> Vec3 {
        self.target
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    fn update_view(&mut self) {
        let horizontal_radius = self.radius * self.pitch.cos();
        self.origin = self.target
            + Vec3::new(
                horizontal_radius * self.yaw.sin(),
                self.radius * self.pitch.sin(),
                horizontal_radius * self.yaw.cos(),
            );

        let theta = self.vertical_fov_degrees.to_radians();
        let viewport_height = 2.0 * (theta * 0.5).tan();
        let viewport_width = self.aspect_ratio * viewport_height;
        let backward = (self.origin - self.target).normalized();
        let right = Vec3::new(0.0, 1.0, 0.0).cross(backward).normalized();
        let up = backward.cross(right);

        self.horizontal = right * viewport_width;
        self.vertical = up * viewport_height;
        self.lower_left_corner =
            self.origin - self.horizontal * 0.5 - self.vertical * 0.5 - backward;
    }
}

#[cfg(test)]
mod tests {
    use super::{Camera, MAX_PITCH, MAX_RADIUS, MIN_PITCH, MIN_RADIUS};
    use crate::math::Vec3;

    fn camera() -> Camera {
        Camera::orbital(Vec3::new(0.0, 0.0, 0.0), 10.0, 0.0, 0.25, 45.0, 16.0 / 9.0)
    }

    #[test]
    fn orbital_position_stays_at_selected_radius() {
        let camera = camera();
        let distance = (camera.position() - camera.target()).length();
        assert!((distance - camera.radius()).abs() < 1e-5);
    }

    #[test]
    fn orbit_changes_position_and_clamps_pitch() {
        let mut camera = camera();
        let initial_position = camera.position();
        camera.orbit(0.5, std::f32::consts::PI);

        assert_ne!(camera.position(), initial_position);
        assert_eq!(camera.pitch(), MAX_PITCH);

        camera.orbit(0.0, -std::f32::consts::TAU);
        assert_eq!(camera.pitch(), MIN_PITCH);
    }

    #[test]
    fn zoom_clamps_radius() {
        let mut camera = camera();
        camera.zoom(-100.0);
        assert_eq!(camera.radius(), MIN_RADIUS);

        camera.zoom(100.0);
        assert_eq!(camera.radius(), MAX_RADIUS);
    }
}
