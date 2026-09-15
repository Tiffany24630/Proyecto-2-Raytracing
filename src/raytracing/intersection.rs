use crate::{geometry::Object, materials::Material, math::Vec3};

use super::Ray;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Uv {
    pub u: f32,
    pub v: f32,
}

impl Uv {
    pub const fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub uv: Uv,
    pub t: f32,
    pub front_face: bool,
    pub object_name: &'static str,
}

impl HitRecord {
    pub fn new(
        ray: &Ray,
        point: Vec3,
        outward_normal: Vec3,
        t: f32,
        material: Material,
        uv: Uv,
        object_name: &'static str,
    ) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };

        Self {
            point,
            normal,
            material,
            uv,
            t,
            front_face,
            object_name,
        }
    }
}

pub fn closest_hit(
    ray: &Ray,
    objects: &[Box<dyn Object>],
    t_min: f32,
    t_max: f32,
) -> Option<HitRecord> {
    let mut closest = t_max;
    let mut result = None;

    for object in objects {
        if let Some(hit) = object.hit(ray, t_min, closest) {
            closest = hit.t;
            result = Some(hit);
        }
    }

    result
}

pub fn any_hit(ray: &Ray, objects: &[Box<dyn Object>], t_min: f32, t_max: f32) -> bool {
    objects
        .iter()
        .any(|object| object.hit(ray, t_min, t_max).is_some())
}

#[cfg(test)]
mod tests {
    use super::any_hit;
    use crate::{
        geometry::{Cube, Object},
        materials::stone,
        math::Vec3,
        raytracing::Ray,
    };

    #[test]
    fn shadow_query_reports_only_hits_inside_the_requested_distance() {
        let objects: Vec<Box<dyn Object>> = vec![Box::new(Cube::from_center(
            "blocker",
            Vec3::new(0.0, 0.0, -3.0),
            Vec3::new(1.0, 1.0, 1.0),
            stone(),
        ))];
        let ray = Ray::new(Vec3::default(), Vec3::new(0.0, 0.0, -1.0));

        assert!(any_hit(&ray, &objects, 0.001, 10.0));
        assert!(!any_hit(&ray, &objects, 0.001, 2.0));
    }
}
