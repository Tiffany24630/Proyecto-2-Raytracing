use crate::{
    geometry::{Cube, Object},
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
    build_temple_view(memory_state, core_pose, core_selected, puzzle, None)
}

pub fn build_temple_view(
    memory_state: MemoryCoreState,
    core_pose: MemoryCorePose,
    core_selected: bool,
    puzzle: PuzzleLayout,
    exterior: Option<bool>,
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
    if exterior != Some(false) { add_exterior(&mut objects); }

    add_cube(
        &mut objects,
        "temple floor",
        Vec3::new(0.0, -0.80, -1.50),
        Vec3::new(22.0, 0.35, 23.0),
        stone,
    );
    if exterior == Some(true) {
        add_stairs(&mut objects, stone);
        add_walls(&mut objects, stone);
        add_roof(&mut objects, stone, wood);
        add_portal(&mut objects, stone, metal, crystal);
        return Scene { objects, light: memory_state.light(),
            camera_target: Vec3::new(0.0, 1.35, 8.65) };
    }
    add_floor_inlays(&mut objects, metal, crystal);
    add_stairs(&mut objects, stone);
    add_walls(&mut objects, stone);
    add_columns(&mut objects, stone, metal);
    add_roof(&mut objects, stone, wood);
    add_platforms(&mut objects, stone, wood);
    add_celestial_apse(&mut objects, stone, metal, crystal);
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
    for (x, z) in [(-3.4, -2.0), (3.4, -2.0), (0.0, 2.2)] {
        add_cube(
            objects,
            "exhibition platform",
            Vec3::new(x, -0.43, z),
            Vec3::new(2.88, 0.58, 2.66),
            stone,
        );
        add_cube(
            objects,
            "exhibition table",
            Vec3::new(x, 0.02, z),
            Vec3::new(2.08, 0.20, 1.70),
            wood,
        );
    }
}

fn add_celestial_apse(
    objects: &mut Vec<Box<dyn Object>>,
    stone: Material,
    metal: Material,
    mut crystal: Material,
) {
    for x in [-2.65, 2.65] {
        add_cube(
            objects,
            "central apse pillar",
            Vec3::new(x, 1.78, -4.85),
            Vec3::new(0.58, 5.05, 0.72),
            stone,
        );
    }
    add_cube(
        objects,
        "central apse crown",
        Vec3::new(0.0, 4.30, -4.85),
        Vec3::new(5.90, 0.48, 0.74),
        metal,
    );

    crystal.transparency = 0.34;
    crystal.reflectivity = 0.12;
    crystal.emission = Vec3::new(0.05, 0.16, 0.24);
    add_cube(
        objects,
        "celestial apse crest",
        Vec3::new(0.0, 4.74, -4.80),
        Vec3::new(0.72, 0.62, 0.30),
        crystal,
    );
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
    mut stone: Material,
    mut metal: Material,
    mut crystal: Material,
) {
    stone.albedo = Vec3::new(0.55, 0.53, 0.46);
    stone.texture_weight = 0.18;
    stone.emission = Vec3::new(0.035, 0.032, 0.025);
    metal.albedo = Vec3::new(0.60, 0.53, 0.33);
    metal.texture_weight = 0.12;
    for x in [-1.65, 1.65] {
        add_cube(objects, "portal pillar", Vec3::new(x, 1.45, 8.65),
            Vec3::new(0.48, 4.0, 0.65), stone);
        add_cube(objects, "portal carved jamb", Vec3::new(x * 0.87, 1.45, 9.01),
            Vec3::new(0.12, 3.95, 0.12), stone);
        add_cube(objects, "portal foot", Vec3::new(x, -0.30, 8.65),
            Vec3::new(0.70, 0.48, 0.90), metal);
        add_cube(objects, "portal capital", Vec3::new(x, 3.36, 8.65),
            Vec3::new(0.82, 0.42, 0.90), stone);
    }
    for step in 0..18 {
        let t = step as f32 / 18.0;
        let half_width = 1.89 * (1.0 - t).powf(0.72);
        let y = 3.46 + t * 1.34;
        for side in [-1.0, 1.0] {
            add_cube(objects, "portal pointed arch", Vec3::new(side * (half_width - 0.18), y, 8.65),
                Vec3::new(0.40, 0.105, 0.65), stone);
        }
        add_cube(objects, "portal arch inset", Vec3::new(0.0, y, 8.65),
            Vec3::new((half_width * 2.0 - 0.65).max(0.06), 0.10, 0.18), {
                let mut dark = stone;
                dark.albedo = Vec3::new(0.095, 0.078, 0.11);
                dark.texture_weight = 0.0;
                dark
            });
    }
    add_cube(objects, "portal threshold", Vec3::new(0.0, -0.51, 8.65),
        Vec3::new(4.2, 0.24, 1.1), stone);
    crystal.transparency = 0.0;
    crystal.reflectivity = 0.03;
    crystal.specular = 0.12;
    crystal.emission = Vec3::new(0.025, 0.020, 0.025);
    crystal.texture_weight = 1.0;
    crystal.texture_scale = 1.0;
    add_cube(objects, "portal membrane", Vec3::new(0.0, 1.46, 8.65),
        Vec3::new(2.86, 3.96, 0.18), crystal);
    add_cube(objects, "portal central seam", Vec3::new(0.0, 1.46, 8.50),
        Vec3::new(0.075, 3.82, 0.08), metal);
    let mut sigil = crystal;
    sigil.albedo = Vec3::new(0.12, 0.90, 1.0);
    sigil.emission = Vec3::new(0.10, 0.52, 0.70);
    sigil.texture_weight = 0.0;
    for (x, y, width) in [(0.0, 1.75, 0.88), (-0.38, 1.42, 0.44), (0.38, 1.42, 0.44)] {
        add_cube(objects, "portal cyan sigil", Vec3::new(x, y, 8.48),
            Vec3::new(width, 0.08, 0.08), sigil);
    }
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
    let mut aperture = crystal;
    aperture.transparency = 0.38;
    aperture.reflectivity = 0.12;
    aperture.emission = Vec3::new(0.05, 0.16, 0.23);
    aperture.texture_weight = 0.44;
    add_cube(
        objects,
        "core energy aperture",
        Vec3::new(0.0, 0.25, -1.25),
        Vec3::new(1.48, 0.10, 1.48),
        aperture,
    );
    add_memory_core(objects, core.state, crystal, ink, core.pose, core.selected);

    for x in [-1.34, 1.34] {
        for z in [-2.30, -0.20] {
            add_cube(
                objects,
                "memory cage",
                Vec3::new(x, 1.35, z),
                Vec3::new(0.12, 3.55, 0.12),
                metal,
            );
        }
    }
    add_cube(
        objects,
        "memory cage crown",
        Vec3::new(0.0, 3.15, -1.25),
        Vec3::new(2.92, 0.16, 2.26),
        metal,
    );
    let mut preserved_echo = crystal;
    preserved_echo.reflectivity = 0.08;
    preserved_echo.transparency = 0.48;
    preserved_echo.emission = Vec3::new(0.03, 0.10, 0.16);
    add_cube(
        objects,
        "preserved echo",
        Vec3::new(0.0, 1.60, -4.76),
        Vec3::new(0.42, 1.25, 0.12),
        preserved_echo,
    );
}
