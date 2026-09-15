use crate::math::Vec3;

use super::Ray;

pub const MIN_RADIUS: f32 = 6.0;
pub const MAX_RADIUS: f32 = 9.5;
pub const MIN_PITCH: f32 = -10.0_f32.to_radians();
pub const MAX_PITCH: f32 = 65.0_f32.to_radians();

#[derive(Clone, Copy, Debug)]
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

    pub fn interpolate(start: &Self, end: &Self, progress: f32) -> Self {
        let t = progress.clamp(0.0, 1.0);
        Self::orbital(
            start.target + (end.target - start.target) * t,
            start.radius + (end.radius - start.radius) * t,
            interpolate_angle(start.yaw, end.yaw, t),
            start.pitch + (end.pitch - start.pitch) * t,
            start.vertical_fov_degrees
                + (end.vertical_fov_degrees - start.vertical_fov_degrees) * t,
            start.aspect_ratio + (end.aspect_ratio - start.aspect_ratio) * t,
        )
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

    pub fn forward(&self) -> Vec3 {
        (self.target - self.origin).normalized()
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

    pub fn project_to_screen(
        &self,
        point: Vec3,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        let relative = point - self.origin;
        let forward = self.forward();
        let depth = relative.dot(forward);
        if depth <= 0.01 {
            return None;
        }

        let backward = -forward;
        let right = Vec3::new(0.0, 1.0, 0.0).cross(backward).normalized();
        let up = backward.cross(right);
        let half_height = (self.vertical_fov_degrees.to_radians() * 0.5).tan() * depth;
        let half_width = half_height * self.aspect_ratio;
        let normalized_x = relative.dot(right) / half_width;
        let normalized_y = relative.dot(up) / half_height;
        if normalized_x.abs() > 1.0 || normalized_y.abs() > 1.0 {
            return None;
        }

        let screen_x = ((normalized_x * 0.5 + 0.5) * width as f32) as usize;
        let screen_y = ((0.5 - normalized_y * 0.5) * height as f32) as usize;
        Some((screen_x.min(width - 1), screen_y.min(height - 1)))
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

fn interpolate_angle(start: f32, end: f32, t: f32) -> f32 {
    let difference = (end - start + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    start + difference * t
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

    #[test]
    fn interpolation_reaches_both_camera_poses() {
        let start = camera();
        let end = Camera::orbital(Vec3::new(1.0, 2.0, -3.0), 5.0, 0.8, 0.4, 50.0, 16.0 / 9.0);
        let first = Camera::interpolate(&start, &end, 0.0);
        let last = Camera::interpolate(&start, &end, 1.0);

        assert!((first.position() - start.position()).length() < 1e-5);
        assert!((last.position() - end.position()).length() < 1e-5);
        assert_eq!(last.target(), end.target());
    }

    #[test]
    fn target_projects_to_the_center_of_the_screen() {
        let camera = camera();
        let projected = camera.project_to_screen(camera.target(), 480, 270).unwrap();
        assert!((projected.0 as i32 - 240).abs() <= 1);
        assert!((projected.1 as i32 - 135).abs() <= 1);
    }
}
