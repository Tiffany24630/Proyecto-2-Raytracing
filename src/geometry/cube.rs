use crate::{
    materials::Material,
    math::Vec3,
    raytracing::{HitRecord, Ray, Uv},
};

use super::Object;

pub struct Cube {
    name: &'static str,
    min: Vec3,
    max: Vec3,
    material: Material,
}

impl Cube {
    pub fn from_center(name: &'static str, center: Vec3, size: Vec3, material: Material) -> Self {
        assert!(size.x > 0.0 && size.y > 0.0 && size.z > 0.0);
        let half_size = size * 0.5;
        Self {
            name,
            min: center - half_size,
            max: center + half_size,
            material,
        }
    }

    fn outward_normal(&self, point: Vec3) -> Vec3 {
        let distances = [
            ((point.x - self.min.x).abs(), Vec3::new(-1.0, 0.0, 0.0)),
            ((point.x - self.max.x).abs(), Vec3::new(1.0, 0.0, 0.0)),
            ((point.y - self.min.y).abs(), Vec3::new(0.0, -1.0, 0.0)),
            ((point.y - self.max.y).abs(), Vec3::new(0.0, 1.0, 0.0)),
            ((point.z - self.min.z).abs(), Vec3::new(0.0, 0.0, -1.0)),
            ((point.z - self.max.z).abs(), Vec3::new(0.0, 0.0, 1.0)),
        ];

        distances
            .into_iter()
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .unwrap()
            .1
    }

    fn uv(&self, point: Vec3, normal: Vec3) -> Uv {
        let size = self.max - self.min;
        if normal.x.abs() > 0.5 {
            Uv::new(
                (point.z - self.min.z) / size.z,
                (point.y - self.min.y) / size.y,
            )
        } else if normal.y.abs() > 0.5 {
            Uv::new(
                (point.x - self.min.x) / size.x,
                (point.z - self.min.z) / size.z,
            )
        } else {
            Uv::new(
                (point.x - self.min.x) / size.x,
                (point.y - self.min.y) / size.y,
            )
        }
    }
}

impl Object for Cube {
    fn name(&self) -> &'static str {
        self.name
    }

    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut near = f32::NEG_INFINITY;
        let mut far = f32::INFINITY;

        for axis in 0..3 {
            let origin = ray.origin.component(axis);
            let direction = ray.direction.component(axis);
            let minimum = self.min.component(axis);
            let maximum = self.max.component(axis);

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
        let outward_normal = self.outward_normal(point);
        Some(HitRecord::new(
            ray,
            point,
            outward_normal,
            t,
            self.material,
            self.uv(point, outward_normal),
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
}
