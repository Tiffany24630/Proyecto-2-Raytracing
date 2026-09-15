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
