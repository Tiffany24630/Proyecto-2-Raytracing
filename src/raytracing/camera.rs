use crate::math::Vec3;

use super::Ray;

pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn look_at(
        origin: Vec3,
        target: Vec3,
        world_up: Vec3,
        vertical_fov_degrees: f32,
        aspect_ratio: f32,
    ) -> Self {
        let theta = vertical_fov_degrees.to_radians();
        let viewport_height = 2.0 * (theta * 0.5).tan();
        let viewport_width = aspect_ratio * viewport_height;

        let backward = (origin - target).normalized();
        let right = world_up.cross(backward).normalized();
        let up = backward.cross(right);

        let horizontal = right * viewport_width;
        let vertical = up * viewport_height;
        let lower_left_corner = origin - horizontal * 0.5 - vertical * 0.5 - backward;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin,
        )
    }
}
