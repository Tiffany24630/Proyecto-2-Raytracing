use crate::{
    materials::Material,
    math::Vec3,
    raytracing::{HitRecord, Ray, Uv},
};

use super::Object;

pub struct Plane {
    name: &'static str,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

impl Plane {
    pub fn new(name: &'static str, point: Vec3, normal: Vec3, material: Material) -> Self {
        Self {
            name,
            point,
            normal: normal.normalized(),
            material,
        }
    }
}

impl Object for Plane {
    fn name(&self) -> &'static str {
        self.name
    }

    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let denominator = self.normal.dot(ray.direction);
        if denominator.abs() < 1e-6 {
            return None;
        }

        let t = (self.point - ray.origin).dot(self.normal) / denominator;
        if !(t_min..=t_max).contains(&t) {
            return None;
        }

        let point = ray.at(t);
        Some(HitRecord::new(
            ray,
            point,
            self.normal,
            t,
            self.material,
            self.uv(point),
            self.name,
        ))
    }
}

impl Plane {
    fn uv(&self, point: Vec3) -> Uv {
        let relative = point - self.point;
        let helper = if self.normal.y.abs() < 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let tangent = helper.cross(self.normal).normalized();
        let bitangent = self.normal.cross(tangent);
        Uv::new(relative.dot(tangent), relative.dot(bitangent))
    }
}

#[cfg(test)]
mod tests {
    use super::{Object, Plane};
    use crate::{materials::stone, math::Vec3, raytracing::Ray};

    #[test]
    fn downward_ray_hits_horizontal_plane() {
        let plane = Plane::new(
            "test plane",
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            stone(),
        );
        let ray = Ray::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = plane.hit(&ray, 0.001, f32::INFINITY).unwrap();

        assert!((hit.t - 2.0).abs() < 1e-6);
        assert_eq!(hit.normal, Vec3::new(0.0, 1.0, 0.0));
    }
}
