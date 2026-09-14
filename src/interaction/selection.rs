use crate::{
    geometry::Object,
    raytracing::{Camera, closest_hit},
};

pub(super) fn object_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<&'static str> {
    closest_hit(&camera.ray(u, v), objects, 0.001, f32::INFINITY).map(|hit| hit.object_name)
}

pub(super) fn core_at(
    camera: &Camera,
    objects: &[Box<dyn Object>],
    u: f32,
    v: f32,
) -> Option<&'static str> {
    let name = object_at(camera, objects, u, v)?;
    matches!(name, "memory core" | "memory core fragment").then_some(name)
}
