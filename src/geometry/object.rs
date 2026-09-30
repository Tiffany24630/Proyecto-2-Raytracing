use crate::{
    math::Vec3,
    raytracing::{HitRecord, Ray},
};

pub trait Object: Sync {
    fn name(&self) -> &'static str;
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;

    /// Caja envolvente alineada a los ejes en coordenadas de mundo, como
    /// `(mínimo, máximo)`. Los objetos no acotados (planos infinitos) devuelven
    /// `None` y se comprueban siempre; el resto entra en la jerarquía de
    /// volúmenes envolventes (`Bvh`) para descartar rápido los que un rayo
    /// no puede tocar.
    fn bounds(&self) -> Option<(Vec3, Vec3)> {
        None
    }
}
