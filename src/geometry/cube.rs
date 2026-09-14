use crate::{
    materials::Material,
    math::{Vec3, rotate_y},
    raytracing::{HitRecord, Ray, Uv},
};

use super::Object;

pub struct Cube {
    name: &'static str,
    center: Vec3,
    half_size: Vec3,
    yaw: f32,
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
        Self {
            name,
            center,
            half_size: size * 0.5,
            yaw,
            material,
        }
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

    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let local_ray = Ray::new(
            rotate_y(ray.origin - self.center, -self.yaw),
            rotate_y(ray.direction, -self.yaw),
        );
        let mut near = f32::NEG_INFINITY;
        let mut far = f32::INFINITY;

        for axis in 0..3 {
            let origin = local_ray.origin.component(axis);
            let direction = local_ray.direction.component(axis);
            let minimum = -self.half_size.component(axis);
            let maximum = self.half_size.component(axis);

            if direction.abs() < 1e-8 {
                if origin < minimum || origin > maximum {
                    return None;
                }
                continue;
            }

            let inverse_direction = 1.0 / direction;
            let mut first = (minimum - origin) * inverse_direction;
            let mut second = (maximum - origin) * inverse_direction;
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
        let local_point = local_ray.at(t);
        let local_normal = self.local_outward_normal(local_point);
        let outward_normal = rotate_y(local_normal, self.yaw);
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

#[cfg(test)]
mod tests {
    use super::{Cube, Object};
    use crate::{
        materials::stone,
        math::Vec3,
        raytracing::{Ray, Uv},
    };

    #[test]
    fn ray_hits_front_face_of_cube() {
        let cube = Cube::from_center(
            "test cube",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            stone(),
        );
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = cube.hit(&ray, 0.001, f32::INFINITY).unwrap();

        assert!((hit.t - 4.0).abs() < 1e-6);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.uv, Uv::new(0.5, 0.5));
        assert!(hit.front_face);
    }

    #[test]
    fn ray_misses_cube() {
        let cube = Cube::from_center(
            "test cube",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            stone(),
        );
        let ray = Ray::new(Vec3::new(0.0, 3.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(cube.hit(&ray, 0.001, f32::INFINITY).is_none());
    }

    #[test]
    fn ray_inside_cube_hits_exit_face() {
        let cube = Cube::from_center(
            "test cube",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 2.0, 2.0),
            stone(),
        );
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        let hit = cube.hit(&ray, 0.001, f32::INFINITY).unwrap();

        assert!((hit.t - 1.0).abs() < 1e-6);
        assert!(!hit.front_face);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn ray_intersects_a_rotated_non_uniform_cube() {
        let cube = Cube::from_center_rotated(
            "rotated cube",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 4.0),
            std::f32::consts::FRAC_PI_2,
            stone(),
        );
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = cube.hit(&ray, 0.001, f32::INFINITY).unwrap();

        assert!((hit.t - 1.0).abs() < 1e-5);
        assert!((hit.normal.x - 1.0).abs() < 1e-5);
    }
}
