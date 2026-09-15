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
