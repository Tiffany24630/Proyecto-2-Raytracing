use crate::{geometry::Object, math::Vec3, raytracing::Light};

pub struct Scene {
    pub objects: Vec<Box<dyn Object>>,
    pub light: Light,
    pub camera_target: Vec3,
}
