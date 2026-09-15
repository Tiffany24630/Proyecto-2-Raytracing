use crate::{materials::Material, math::Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

impl Light {
    pub const fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
        }
    }
}

pub fn shade(
    material: Material,
    surface_albedo: Vec3,
    normal: Vec3,
    light_direction: Vec3,
    view_direction: Vec3,
    light: &Light,
    in_shadow: bool,
) -> Vec3 {
    let ambient = surface_albedo * Vec3::new(0.075, 0.09, 0.13) + material.emission;
    if in_shadow {
        return ambient;
    }

    let diffuse_strength = normal.dot(light_direction).max(0.0) * light.intensity;
    let diffuse = surface_albedo * light.color * diffuse_strength;

    let half_vector = (light_direction + view_direction).normalized();
    let specular_strength = normal.dot(half_vector).max(0.0).powf(40.0) * material.specular;
    let specular = light.color * specular_strength * light.intensity;

    ambient + diffuse + specular
}
