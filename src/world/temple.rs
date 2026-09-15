use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, crystal, ink, metal, stone, wood},
    math::Vec3,
};

use super::Scene;
use super::exhibitions::add_exhibitions;
use super::exterior::add_exterior;
use super::memory_core::{MemoryCorePose, MemoryCoreState, add_memory_core};
use super::puzzle_pieces::{PuzzleLayout, add_puzzle_pieces};

#[derive(Clone, Copy)]
struct CorePlacement {
    state: MemoryCoreState,
    pose: MemoryCorePose,
    selected: bool,
}

pub fn build_temple() -> Scene {
    build_temple_with_memory(MemoryCoreState::Stable)
}

pub fn build_temple_with_memory(memory_state: MemoryCoreState) -> Scene {
    build_temple_with_core(memory_state, MemoryCorePose::default(), false)
}

pub fn build_temple_with_core(
    memory_state: MemoryCoreState,
    core_pose: MemoryCorePose,
    core_selected: bool,
) -> Scene {
    build_temple_interactive(
        memory_state,
        core_pose,
        core_selected,
        PuzzleLayout::initial(),
    )
}

pub fn build_temple_interactive(
    memory_state: MemoryCoreState,
    core_pose: MemoryCorePose,
    core_selected: bool,
    puzzle: PuzzleLayout,
) -> Scene {
    let stone = stone();
    let wood = wood();
    let metal = metal();
    let crystal = crystal();
    let ink = ink();
    for material in [stone, wood, metal, crystal, ink] {
        material.validate();
    }

    let mut objects: Vec<Box<dyn Object>> = Vec::new();
    objects.push(Box::new(Plane::new(
        "exterior floor",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        stone,
    )));
    add_exterior(&mut objects);

    add_cube(
        &mut objects,
        "temple floor",
        Vec3::new(0.0, -0.80, -1.50),
        Vec3::new(22.0, 0.35, 23.0),
        stone,
    );
    add_floor_inlays(&mut objects, metal, crystal);
    add_stairs(&mut objects, stone);
    add_walls(&mut objects, stone);
    add_columns(&mut objects, stone, metal);
    add_roof(&mut objects, stone, wood);
    add_platforms(&mut objects, stone, wood);
    add_exhibitions(&mut objects);
    add_portal(&mut objects, stone, metal, crystal);
    add_puzzle_pieces(&mut objects, puzzle);
    add_memory_pedestal(
        &mut objects,
        stone,
        metal,
        crystal,
        ink,
        CorePlacement {
            state: memory_state,
            pose: core_pose,
            selected: core_selected,
        },
    );

    Scene {
        objects,
        light: memory_state.light(),
        camera_target: Vec3::new(0.0, 1.0, -0.25),
    }
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

fn add_stairs(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for step in 0..7 {
        add_cube(
            objects,
            "temple stair",
            Vec3::new(0.0, -0.98 + step as f32 * 0.055, 10.75 - step as f32 * 0.34),
            Vec3::new(4.8, 0.14, 0.52),
            stone,
        );
    }
}

fn add_walls(objects: &mut Vec<Box<dyn Object>>, stone: Material) {
    for x in [-10.82, 10.82] {
        for z in [-9.3, -3.5, 2.3, 7.2] {
            add_cube(
                objects,
                "side wall",
                Vec3::new(x, 1.85, z),
                Vec3::new(0.36, 5.25, 4.3),
                stone,
            );
        }
    }
    for x in [-7.8, -2.6, 2.6, 7.8] {
        add_cube(
            objects,
            "rear wall",
            Vec3::new(x, 1.85, -12.82),
            Vec3::new(4.6, 5.25, 0.36),
            stone,
        );
    }
}

fn add_columns(objects: &mut Vec<Box<dyn Object>>, stone: Material, metal: Material) {
    for x in [-8.3, 8.3] {
        for z in [-10.2, -6.0, -1.8, 2.4, 6.6] {
            add_cube(
                objects,
                "column base",
                Vec3::new(x, -0.44, z),
                Vec3::new(1.45, 0.68, 1.45),
                stone,
            );
            add_cube(
                objects,
                "column shaft",
                Vec3::new(x, 2.18, z),
                Vec3::new(0.72, 4.65, 0.72),
                stone,
            );
            add_cube(
                objects,
                "column golden collar",
                Vec3::new(x, 0.22, z),
                Vec3::new(0.98, 0.24, 0.98),
                metal,
            );
            add_cube(
                objects,
                "column capital",
                Vec3::new(x, 4.66, z),
                Vec3::new(1.55, 0.44, 1.55),
                metal,
            );
        }
    }
}

fn add_roof(objects: &mut Vec<Box<dyn Object>>, stone: Material, wood: Material) {
    for z in [-11.0, -6.6, -2.2, 2.2, 6.6] {
        add_cube(
            objects,
            "roof beam",
            Vec3::new(0.0, 5.25, z),
            Vec3::new(21.5, 0.36, 0.52),
            stone,
        );
    }
    for x in [-8.4, 0.0, 8.4] {
        add_cube(
            objects,
            "roof rafter",
            Vec3::new(x, 5.45, -2.2),
            Vec3::new(0.42, 0.26, 18.6),
            wood,
        );
    }
}

fn add_platforms(objects: &mut Vec<Box<dyn Object>>, stone: Material, wood: Material) {
    for (x, z) in [(-6.2, -4.1), (6.2, -4.1), (-6.2, 3.8)] {
        add_cube(
            objects,
            "exhibition platform",
            Vec3::new(x, -0.43, z),
            Vec3::new(4.5, 0.58, 4.15),
            stone,
        );
        add_cube(
            objects,
            "exhibition table",
            Vec3::new(x, 0.02, z),
            Vec3::new(3.25, 0.32, 2.65),
            wood,
        );
    }
}

fn add_floor_inlays(objects: &mut Vec<Box<dyn Object>>, metal: Material, mut crystal: Material) {
    crystal.transparency = 0.42;
    crystal.reflectivity = 0.08;
    crystal.emission = Vec3::new(0.035, 0.10, 0.16);
    for z in [-10.8, -8.0, -5.2, -2.4, 0.4, 3.2, 6.0] {
        add_cube(
            objects,
            "celestial floor seal",
            Vec3::new(0.0, -0.605, z),
            Vec3::new(4.8, 0.04, 1.55),
            crystal,
        );
    }
    for x in [-2.65, 2.65] {
        add_cube(
            objects,
            "golden aisle inlay",
            Vec3::new(x, -0.57, -1.9),
            Vec3::new(0.12, 0.05, 19.5),
            metal,
        );
    }
}

fn add_portal(
    objects: &mut Vec<Box<dyn Object>>,
    stone: Material,
    metal: Material,
    crystal: Material,
) {
    for x in [-1.9, 1.9] {
        add_cube(
            objects,
            "portal pillar",
            Vec3::new(x, 1.55, 8.65),
            Vec3::new(0.56, 4.75, 0.78),
            stone,
        );
    }
    add_cube(
        objects,
        "portal lintel",
        Vec3::new(0.0, 3.93, 8.65),
        Vec3::new(4.72, 0.52, 0.78),
        metal,
    );
    add_cube(
        objects,
        "portal membrane",
        Vec3::new(0.0, 1.54, 8.61),
        Vec3::new(3.55, 4.18, 0.18),
        crystal,
    );
}

fn add_memory_pedestal(
    objects: &mut Vec<Box<dyn Object>>,
    stone: Material,
    metal: Material,
    crystal: Material,
    ink: Material,
    core: CorePlacement,
) {
    add_cube(
        objects,
        "pedestal base",
        Vec3::new(0.0, -0.38, -1.25),
        Vec3::new(3.4, 0.65, 3.4),
        stone,
    );
    add_cube(
        objects,
        "pedestal tier",
        Vec3::new(0.0, 0.02, -1.25),
        Vec3::new(2.35, 0.34, 2.35),
        metal,
    );
    add_memory_core(objects, core.state, crystal, ink, core.pose, core.selected);

    for x in [-1.10, 1.10] {
        for z in [-2.35, -0.15] {
            add_cube(
                objects,
                "memory cage",
                Vec3::new(x, 1.35, z),
                Vec3::new(0.14, 3.65, 0.14),
                metal,
            );
        }
    }
    add_cube(
        objects,
        "memory cage crown",
        Vec3::new(0.0, 3.18, -1.25),
        Vec3::new(2.42, 0.18, 2.42),
        metal,
    );
    let mut preserved_echo = crystal;
    preserved_echo.reflectivity = 0.08;
    preserved_echo.transparency = 0.48;
    preserved_echo.emission = Vec3::new(0.03, 0.10, 0.16);
    add_cube(
        objects,
        "preserved echo",
        Vec3::new(0.0, 0.55, -5.05),
        Vec3::new(0.72, 1.58, 0.72),
        preserved_echo,
    );
}

#[cfg(test)]
mod tests {
    use super::{build_temple, build_temple_with_memory};
    use crate::world::MemoryCoreState;

    #[test]
    fn temple_contains_every_required_architectural_group() {
        let temple = build_temple();
        let names: Vec<_> = temple.objects.iter().map(|object| object.name()).collect();

        for required in [
            "temple floor",
            "column shaft",
            "side wall",
            "exhibition platform",
            "temple stair",
            "pedestal base",
            "roof beam",
            "portal membrane",
            "memory core",
            "puzzle piece A",
            "puzzle piece B",
            "puzzle piece C",
            "exterior path",
            "exterior ruin pillar",
            "exterior rock",
            "wind fragment",
            "luyang painting",
            "luyang pavilion eave",
            "mahavaipulya book",
            "mahavaipulya floating page",
            "desert pavilion arch",
            "desert pyramid tier",
        ] {
            assert!(names.contains(&required), "missing '{required}'");
        }
        assert!((125..=170).contains(&temple.objects.len()));
    }

    #[test]
    fn every_memory_state_builds_its_distinct_geometry() {
        for (state, required) in [
            (MemoryCoreState::Stable, "memory particle"),
            (MemoryCoreState::Fragmented, "memory core fragment"),
            (MemoryCoreState::Restored, "restored memory halo"),
        ] {
            let temple = build_temple_with_memory(state);
            assert!(
                temple
                    .objects
                    .iter()
                    .any(|object| object.name() == required),
                "{} state is missing '{required}'",
                state.label()
            );
        }
    }
}
