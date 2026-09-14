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
}

impl Material {
    pub fn validate(self) {
        assert!((0.0..=1.0).contains(&self.specular));
        assert!((0.0..=1.0).contains(&self.reflectivity));
        assert!((0.0..=1.0).contains(&self.transparency));
        assert!(self.refractive_index >= 1.0);
        assert!(self.texture_scale > 0.0);
    }
}

#[cfg(test)]
mod tests {
    use crate::materials::{MaterialKind, crystal, ink, metal, stone, wood};

    #[test]
    fn exactly_five_material_presets_are_valid() {
        let materials = [stone(), wood(), metal(), crystal(), ink()];
        let expected_kinds = [
            MaterialKind::Stone,
            MaterialKind::Wood,
            MaterialKind::Metal,
            MaterialKind::Crystal,
            MaterialKind::Ink,
        ];

        for material in materials {
            material.validate();
        }
        assert_eq!(materials.map(|material| material.kind), expected_kinds);
        assert!(metal().reflectivity > stone().reflectivity);
        assert!(crystal().transparency > metal().transparency);
        assert!((crystal().refractive_index - 1.5).abs() < f32::EPSILON);
        assert!(ink().emission.length_squared() > 0.0);
    }
}
