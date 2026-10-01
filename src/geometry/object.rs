use crate::{
    math::Vec3,
    raytracing::{HitRecord, Ray},
};

pub trait Object: Sync {
    fn name(&self) -> &'static str;
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;

    fn bounds(&self) -> Option<(Vec3, Vec3)> {
        None
    }
}
