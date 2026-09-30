use crate::math::Vec3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialKind {
    Stone,
    Wood,
    Metal,
    Crystal,
    Ink,
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub kind: MaterialKind,
    pub albedo: Vec3,
    pub specular: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
    pub emission: Vec3,
    pub texture_scale: f32,
    pub texture_weight: f32,
}

impl Material {
    pub fn validate(self) {
        assert!((0.0..=1.0).contains(&self.specular));
        assert!((0.0..=1.0).contains(&self.reflectivity));
        assert!((0.0..=1.0).contains(&self.transparency));
        assert!(self.refractive_index >= 1.0);
        assert!(self.texture_scale > 0.0);
        assert!((0.0..=1.0).contains(&self.texture_weight));
    }
}
