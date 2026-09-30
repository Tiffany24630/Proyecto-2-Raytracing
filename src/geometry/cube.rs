use crate::{
    materials::Material,
    math::Vec3,
    raytracing::{HitRecord, Ray, Uv},
};

use super::Object;

pub struct Cube {
    name: &'static str,
    center: Vec3,
    half_size: Vec3,
    sin_yaw: f32,
    cos_yaw: f32,
    material: Material,
}

impl Cube {
    pub fn from_center(name: &'static str, center: Vec3, size: Vec3, material: Material) -> Self {
        Self::from_center_rotated(name, center, size, 0.0, material)
    }

    pub fn from_center_rotated(
        name: &'static str,
        center: Vec3,
        size: Vec3,
        yaw: f32,
        material: Material,
    ) -> Self {
        assert!(size.x > 0.0 && size.y > 0.0 && size.z > 0.0);
        let (sin_yaw, cos_yaw) = yaw.sin_cos();
        Self {
            name,
            center,
            half_size: size * 0.5,
            sin_yaw,
            cos_yaw,
            material,
        }
    }

    fn is_axis_aligned(&self) -> bool {
        self.sin_yaw == 0.0 && self.cos_yaw == 1.0
    }

    fn to_local(&self, vector: Vec3) -> Vec3 {
        if self.is_axis_aligned() {
            return vector;
        }
        Vec3::new(
            vector.x * self.cos_yaw - vector.z * self.sin_yaw,
            vector.y,
            vector.x * self.sin_yaw + vector.z * self.cos_yaw,
        )
    }

    fn to_world(&self, vector: Vec3) -> Vec3 {
        if self.is_axis_aligned() {
            return vector;
        }
        Vec3::new(
            vector.x * self.cos_yaw + vector.z * self.sin_yaw,
            vector.y,
            -vector.x * self.sin_yaw + vector.z * self.cos_yaw,
        )
    }

    fn local_outward_normal(&self, point: Vec3) -> Vec3 {
        let distances = [
            (
                (point.x + self.half_size.x).abs(),
                Vec3::new(-1.0, 0.0, 0.0),
            ),
            ((point.x - self.half_size.x).abs(), Vec3::new(1.0, 0.0, 0.0)),
            (
                (point.y + self.half_size.y).abs(),
                Vec3::new(0.0, -1.0, 0.0),
            ),
            ((point.y - self.half_size.y).abs(), Vec3::new(0.0, 1.0, 0.0)),
            (
                (point.z + self.half_size.z).abs(),
                Vec3::new(0.0, 0.0, -1.0),
            ),
            ((point.z - self.half_size.z).abs(), Vec3::new(0.0, 0.0, 1.0)),
        ];

        distances
            .into_iter()
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .unwrap()
            .1
    }

    fn uv(&self, point: Vec3, normal: Vec3) -> Uv {
        let size = self.half_size * 2.0;
        if normal.x.abs() > 0.5 {
            Uv::new(
                (point.z + self.half_size.z) / size.z,
                (point.y + self.half_size.y) / size.y,
            )
        } else if normal.y.abs() > 0.5 {
            Uv::new(
                (point.x + self.half_size.x) / size.x,
                (point.z + self.half_size.z) / size.z,
            )
        } else {
            Uv::new(
                (point.x + self.half_size.x) / size.x,
                (point.y + self.half_size.y) / size.y,
            )
        }
    }
}

impl Object for Cube {
    fn name(&self) -> &'static str {
        self.name
    }

    fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let cos = self.cos_yaw.abs();
        let sin = self.sin_yaw.abs();
        let extent = Vec3::new(
            cos * self.half_size.x + sin * self.half_size.z,
            self.half_size.y,
            sin * self.half_size.x + cos * self.half_size.z,
        );
        Some((self.center - extent, self.center + extent))
    }

    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let origin = self.to_local(ray.origin - self.center);
        let direction = self.to_local(ray.direction);
        let origins = [origin.x, origin.y, origin.z];
        let directions = [direction.x, direction.y, direction.z];
        let halves = [self.half_size.x, self.half_size.y, self.half_size.z];
        let mut near = f32::NEG_INFINITY;
        let mut far = f32::INFINITY;

        for axis in 0..3 {
            let axis_origin = origins[axis];
            let axis_direction = directions[axis];
            let minimum = -halves[axis];
            let maximum = halves[axis];

            if axis_direction.abs() < 1e-8 {
                if axis_origin < minimum || axis_origin > maximum {
                    return None;
                }
                continue;
            }

            let inverse_direction = 1.0 / axis_direction;
            let mut first = (minimum - axis_origin) * inverse_direction;
            let mut second = (maximum - axis_origin) * inverse_direction;
            if inverse_direction < 0.0 {
                std::mem::swap(&mut first, &mut second);
            }

            near = near.max(first);
            far = far.min(second);
            if far < near {
                return None;
            }
        }

        if far < t_min || near > t_max {
            return None;
        }
        let t = if near >= t_min { near } else { far };
        if !(t_min..=t_max).contains(&t) {
            return None;
        }
        let point = ray.at(t);
        let local_point = origin + direction * t;
        let local_normal = self.local_outward_normal(local_point);
        let outward_normal = self.to_world(local_normal);
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            t,
            self.material,
            self.uv(local_point, local_normal),
            self.name,
        ))
    }
}
