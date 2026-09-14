use crate::{
    geometry::{Cube, Object},
    materials::Material,
    math::{Vec3, rotate_y},
    raytracing::Light,
};

pub const MEMORY_CORE_CENTER: Vec3 = Vec3::new(0.0, 1.25, -1.25);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MemoryCorePose {
    pub offset: Vec3,
    pub yaw: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryCoreState {
    Stable,
    Fragmented,
    Restored,
}

impl MemoryCoreState {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Stable => "STABLE",
            Self::Fragmented => "FRAGMENTED",
            Self::Restored => "RESTORED",
        }
    }

    pub const fn light(self) -> Light {
        match self {
            Self::Stable => Light::new(Vec3::new(-5.5, 9.5, 5.5), Vec3::new(1.0, 0.94, 0.82), 1.55),
            Self::Fragmented => {
                Light::new(Vec3::new(-3.5, 8.0, 3.0), Vec3::new(0.74, 0.52, 0.90), 1.28)
            }
            Self::Restored => {
                Light::new(Vec3::new(-4.0, 9.8, 4.5), Vec3::new(0.76, 0.94, 1.0), 1.78)
            }
        }
    }
}

pub(super) fn add_memory_core(
    objects: &mut Vec<Box<dyn Object>>,
    state: MemoryCoreState,
    crystal: Material,
    ink: Material,
    pose: MemoryCorePose,
    selected: bool,
) {
    match state {
        MemoryCoreState::Stable => add_stable_core(objects, crystal, pose),
        MemoryCoreState::Fragmented => add_fragmented_core(objects, crystal, ink, pose),
        MemoryCoreState::Restored => add_restored_core(objects, crystal, pose),
    }
    if selected {
        add_selection_markers(objects, ink, pose);
    }
}

fn add_stable_core(objects: &mut Vec<Box<dyn Object>>, crystal: Material, pose: MemoryCorePose) {
    add_rotated_cube(
        objects,
        "memory core",
        transformed_center(Vec3::default(), pose),
        Vec3::new(0.82, 1.02, 0.92),
        pose.yaw,
        crystal,
    );

    let mote = particle_material(crystal);
    for offset in [
        Vec3::new(-0.58, 0.52, 0.05),
        Vec3::new(0.58, -0.36, 0.08),
        Vec3::new(-0.50, -0.48, -0.05),
        Vec3::new(0.48, 0.46, -0.08),
    ] {
        add_cube(
            objects,
            "memory particle",
            transformed_center(offset, pose),
            Vec3::new(0.13, 0.22, 0.13),
            mote,
        );
    }
}

fn add_fragmented_core(
    objects: &mut Vec<Box<dyn Object>>,
    crystal: Material,
    mut ink: Material,
    pose: MemoryCorePose,
) {
    for (index, offset, size) in [
        (
            0,
            Vec3::new(-0.40, 0.27, -0.03),
            Vec3::new(0.52, 0.64, 0.48),
        ),
        (1, Vec3::new(0.39, -0.03, 0.13), Vec3::new(0.46, 0.55, 0.44)),
        (
            2,
            Vec3::new(0.02, -0.48, -0.22),
            Vec3::new(0.58, 0.35, 0.52),
        ),
    ] {
        add_rotated_cube(
            objects,
            if index == 0 {
                "memory core"
            } else {
                "memory core fragment"
            },
            transformed_center(offset, pose),
            size,
            pose.yaw,
            crystal,
        );
    }

    ink.emission = Vec3::new(0.22, 0.015, 0.30);
    for (offset, size) in [
        (Vec3::new(-0.08, 0.13, 0.05), Vec3::new(0.10, 0.52, 0.10)),
        (Vec3::new(0.23, -0.29, -0.06), Vec3::new(0.09, 0.40, 0.09)),
        (Vec3::new(-0.30, -0.22, -0.16), Vec3::new(0.08, 0.32, 0.08)),
    ] {
        add_rotated_cube(
            objects,
            "memory fracture energy",
            transformed_center(offset, pose),
            size,
            pose.yaw,
            ink,
        );
    }
}

fn add_restored_core(
    objects: &mut Vec<Box<dyn Object>>,
    mut crystal: Material,
    pose: MemoryCorePose,
) {
    crystal.emission = Vec3::new(0.08, 0.22, 0.32);
    crystal.reflectivity = 0.24;
    add_rotated_cube(
        objects,
        "memory core",
        transformed_center(Vec3::default(), pose),
        Vec3::new(0.98, 1.16, 1.08),
        pose.yaw,
        crystal,
    );

    let halo = particle_material(crystal);
    for offset in [
        Vec3::new(-0.78, 0.00, 0.0),
        Vec3::new(0.78, 0.00, 0.0),
        Vec3::new(0.00, -0.78, 0.0),
        Vec3::new(0.00, 0.78, 0.0),
        Vec3::new(-0.55, -0.55, 0.06),
        Vec3::new(0.55, -0.55, 0.06),
        Vec3::new(-0.55, 0.55, -0.06),
        Vec3::new(0.55, 0.55, -0.06),
    ] {
        add_cube(
            objects,
            "restored memory halo",
            transformed_center(offset, pose),
            Vec3::new(0.12, 0.18, 0.12),
            halo,
        );
    }
}

fn add_selection_markers(
    objects: &mut Vec<Box<dyn Object>>,
    mut ink: Material,
    pose: MemoryCorePose,
) {
    ink.emission = Vec3::new(0.10, 0.32, 0.36);
    for offset in [
        Vec3::new(-0.72, -0.72, 0.0),
        Vec3::new(-0.72, 0.72, 0.0),
        Vec3::new(0.72, -0.72, 0.0),
        Vec3::new(0.72, 0.72, 0.0),
    ] {
        add_cube(
            objects,
            "eye selection marker",
            transformed_center(offset, pose),
            Vec3::new(0.10, 0.34, 0.10),
            ink,
        );
    }
}

fn transformed_center(relative: Vec3, pose: MemoryCorePose) -> Vec3 {
    MEMORY_CORE_CENTER + pose.offset + rotate_y(relative, pose.yaw)
}

fn particle_material(mut crystal: Material) -> Material {
    crystal.reflectivity = 0.05;
    crystal.transparency = 0.30;
    crystal.texture_weight = 0.35;
    crystal
}

fn add_cube(
    objects: &mut Vec<Box<dyn Object>>,
    name: &'static str,
    center: Vec3,
    size: Vec3,
    material: Material,
) {
    objects.push(Box::new(Cube::from_center(name, center, size, material)));
}

fn add_rotated_cube(
    objects: &mut Vec<Box<dyn Object>>,
    name: &'static str,
    center: Vec3,
    size: Vec3,
    yaw: f32,
    material: Material,
) {
    objects.push(Box::new(Cube::from_center_rotated(
        name, center, size, yaw, material,
    )));
}

#[cfg(test)]
mod tests {
    use super::MemoryCoreState;

    #[test]
    fn memory_core_exposes_three_distinct_states() {
        assert_eq!(
            [
                MemoryCoreState::Stable,
                MemoryCoreState::Fragmented,
                MemoryCoreState::Restored,
            ]
            .len(),
            3
        );
        assert_ne!(
            MemoryCoreState::Stable.light().color,
            MemoryCoreState::Fragmented.light().color
        );
        assert_ne!(
            MemoryCoreState::Fragmented.light().color,
            MemoryCoreState::Restored.light().color
        );
    }
}
