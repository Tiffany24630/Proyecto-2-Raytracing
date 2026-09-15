mod animation;
mod game;
mod geometry;
mod interaction;
mod interface;
mod materials;
mod math;
mod raytracing;
mod world;

use std::error::Error;
use std::time::Instant;

use animation::{MemoryTimeline, SceneTransition};
use game::{
    AnimationState, ControlMode, GameEvent, GameState, NarrativeController, SceneState, SkyState,
};
use interaction::{
    CAMERA_TOLERANCE, EyeOfGod, POSITION_TOLERANCE, PerspectiveStatus, Puzzle, ROTATION_TOLERANCE,
    check_perspective,
};
use interface::{UiState, draw_interface, draw_narrative, draw_world_label};
use materials::TextureSet;
use math::Vec3;
use minifb::{Key, KeyRepeat, MouseMode, Scale, Window, WindowOptions};
use raytracing::{Camera, Renderer};
use world::{
    PortalView, PuzzleLayout, PuzzlePieceId, Scene, add_melanta_event, build_temple,
    build_temple_interactive, melanta_light, temple_skybox,
};

const IMAGE_WIDTH: usize = 480;
const IMAGE_HEIGHT: usize = 270;
const ASPECT_RATIO: f32 = IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32;
const ORBIT_STEP: f32 = 7.5_f32.to_radians();
const OBJECT_ROTATION_STEP: f32 = 15.0_f32.to_radians();
const OBJECT_MOVE_STEP: f32 = 0.18;
const ZOOM_STEP: f32 = 0.8;
const SKY_TRANSITION_SECONDS: f32 = 3.0;
const PORTAL_TRANSITION_SECONDS: f32 = 2.4;
const UI_TARGET_FPS: usize = 30;

fn main() -> Result<(), Box<dyn Error>> {
    let temple = build_temple();
    let named_groups = temple
        .objects
        .iter()
        .map(|object| object.name())
        .collect::<std::collections::HashSet<_>>()
        .len();
    println!(
        "Temple scene: {} objects across {named_groups} named groups; focus: {:?}",
        temple.objects.len(),
        temple.camera_target
    );
    println!(
        "Puzzle tolerances: position {:.2}, rotation {:.1} degrees, camera {:.1} degrees",
        POSITION_TOLERANCE,
        ROTATION_TOLERANCE.to_degrees(),
        CAMERA_TOLERANCE.to_degrees()
    );
    let camera = PortalView::Exterior.camera(ASPECT_RATIO);
    let renderer = Renderer::new(IMAGE_WIDTH, IMAGE_HEIGHT, Vec3::new(0.10, 0.12, 0.18));

    let textures = TextureSet::load_from_directory("assets/textures")?;

    if std::env::args().any(|argument| argument == "--render-once") {
        render_checkpoint_views(&renderer, &textures)?;
    } else {
        run_interactive(camera, &renderer, &textures)?;
    }

    Ok(())
}

fn render_checkpoint_views(
    renderer: &Renderer,
    textures: &TextureSet,
) -> Result<(), Box<dyn Error>> {
    let exterior = PortalView::Exterior.camera(ASPECT_RATIO);
    let interior = PortalView::Interior.camera(ASPECT_RATIO);
    let mut interior_side = interior;
    interior_side.orbit(std::f32::consts::FRAC_PI_2, 0.0);
    let mut interior_back = interior;
    interior_back.orbit(std::f32::consts::PI, 0.0);
    for (name, state, progress, camera) in [
        ("exterior", SceneState::Exterior, 0.0, exterior),
        (
            "entering",
            SceneState::Entering,
            0.5,
            Camera::interpolate(&exterior, &interior, 0.5),
        ),
        ("temple", SceneState::Temple, 1.0, interior),
        ("temple_side", SceneState::Temple, 1.0, interior_side),
        ("temple_back", SceneState::Temple, 1.0, interior_back),
        ("puzzle", SceneState::Puzzle, 1.0, interior),
        ("memory_restored", SceneState::MemoryRestored, 1.0, interior),
        ("melanta", SceneState::Melanta, 1.0, interior),
        ("final", SceneState::Final, 1.0, interior),
    ] {
        let transition = SceneTransition::at(PORTAL_TRANSITION_SECONDS, progress);
        render_checkpoint(name, state, &camera, renderer, textures, transition)?;
    }
    Ok(())
}

fn render_checkpoint(
    name: &str,
    state: SceneState,
    camera: &Camera,
    renderer: &Renderer,
    textures: &TextureSet,
    transition: SceneTransition,
) -> Result<(), Box<dyn Error>> {
    let game = GameState::from_scene(state);
    let timeline = MemoryTimeline::at(
        if matches!(
            state,
            SceneState::MemoryRestored | SceneState::Melanta | SceneState::Final
        ) {
            1.0
        } else {
            0.5
        },
    );
    let sampled_layout = timeline.sample().puzzle;
    let puzzle = Puzzle::from_layout(if state == SceneState::Puzzle {
        PuzzleLayout {
            selected: Some(PuzzlePieceId::A),
            ..sampled_layout
        }
    } else {
        sampled_layout
    });
    let mut eye = EyeOfGod::default();
    if state == SceneState::Puzzle {
        eye.toggle();
    }
    let scene = build_game_scene(game, puzzle, timeline);
    let sky_corruption = if state == SceneState::Melanta {
        1.0
    } else {
        0.0
    };
    let mut pixels = render_scene(renderer, camera, &scene, textures, sky_corruption);
    draw_scene_labels(&mut pixels, state, camera);
    draw_game_interface(
        &mut pixels,
        game,
        eye,
        puzzle,
        timeline,
        sky_corruption,
        camera,
    );
    draw_narrative(
        &mut pixels,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        NarrativeController::new(state).message(),
    );
    draw_state_indicator(
        &mut pixels,
        state,
        PerspectiveStatus::from(puzzle, check_perspective(camera, ASPECT_RATIO)),
    );
    apply_fade(&mut pixels, transition.fade_opacity());
    let output_path = format!("output/showcase_{name}.bmp");
    renderer.write_bmp(&output_path, &pixels)?;
    let sky: SkyState = state.sky();
    let animation: AnimationState = state.animation();
    println!(
        "{} / {} at {:.0}% with {} objects, fade {:.0}%, {:?} sky, {:?} animation, events {:?}; {output_path}",
        state.label(),
        state.file_name(),
        transition.progress() * 100.0,
        scene.objects.len(),
        transition.fade_opacity() * 100.0,
        sky,
        animation,
        game.available_events(),
    );
    Ok(())
}

fn run_interactive(
    mut camera: Camera,
    renderer: &Renderer,
    textures: &TextureSet,
) -> Result<(), Box<dyn Error>> {
    let mut game = GameState::default();
    let mut eye = EyeOfGod::default();
    let mut puzzle = Puzzle::default();
    let mut timeline = MemoryTimeline::default();
    let mut portal_transition = SceneTransition::new(PORTAL_TRANSITION_SECONDS);
    let mut narrative = NarrativeController::default();
    let mut portal_start_camera = camera;
    let portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
    let mut sky_corruption = 0.0_f32;
    let mut last_tick = Instant::now();
    let mut scene = build_game_scene(game, puzzle, timeline);
    let initial_title = window_title(game, sky_corruption, &camera);
    let mut window = Window::new(
        &initial_title,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(UI_TARGET_FPS);

    let mut pixels = render_scene(renderer, &camera, &scene, textures, sky_corruption);
    draw_scene_labels(&mut pixels, game.scene(), &camera);
    draw_game_interface(
        &mut pixels,
        game,
        eye,
        puzzle,
        timeline,
        sky_corruption,
        &camera,
    );
    draw_narrative(&mut pixels, IMAGE_WIDTH, IMAGE_HEIGHT, narrative.message());
    draw_state_indicator(
        &mut pixels,
        game.scene(),
        PerspectiveStatus::from(puzzle, check_perspective(&camera, ASPECT_RATIO)),
    );
    let mut buffer = renderer.to_u32_buffer(&pixels);

    while window.is_open() {
        let now = Instant::now();
        let delta_seconds = now.duration_since(last_tick).as_secs_f32();
        last_tick = now;
        let mut changed = false;
        let mut scene_changed = false;

        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            if puzzle.cancel() || eye.cancel() || game.handle(GameEvent::TogglePuzzle) {
                if game.scene() != SceneState::Puzzle && eye.is_active() {
                    eye.toggle();
                }
                changed = true;
                scene_changed = true;
            } else {
                break;
            }
        }
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) && game.handle(GameEvent::TogglePuzzle) {
            if game.scene() == SceneState::Puzzle {
                if !eye.is_active() {
                    eye.toggle();
                }
            } else {
                puzzle.cancel();
                if eye.is_active() {
                    eye.toggle();
                }
            }
            changed = true;
            scene_changed = true;
        }
        if game.scene() == SceneState::Temple && window.is_key_pressed(Key::M, KeyRepeat::No) {
            game = GameState::from_scene(SceneState::Melanta);
            sky_corruption = 0.0;
            changed = true;
            scene_changed = true;
        }
        if window.is_key_pressed(Key::E, KeyRepeat::No) {
            match game.scene() {
                SceneState::Exterior => {
                    if game.handle(GameEvent::UsePortal) {
                        println!("State: {}", game.scene().label());
                        portal_start_camera = camera;
                        portal_transition.restart();
                        changed = true;
                        scene_changed = true;
                    }
                }
                SceneState::Temple => {
                    if game.handle(GameEvent::UsePortal) {
                        camera = PortalView::Exterior.camera(ASPECT_RATIO);
                        changed = true;
                        scene_changed = true;
                    }
                }
                SceneState::Puzzle => {
                    let (u, v) = mouse_uv(&window);
                    let interacted = puzzle.interact_at(&camera, &scene.objects, u, v);
                    if interacted {
                        changed = true;
                        scene_changed = true;
                    }
                }
                _ => {}
            }
        }
        if game.scene() == SceneState::Entering && portal_transition.update(delta_seconds) {
            camera = Camera::interpolate(
                &portal_start_camera,
                &portal_end_camera,
                portal_transition.eased_progress(),
            );
            changed = true;
            if portal_transition.is_finished() && game.handle(GameEvent::TransitionComplete) {
                camera = portal_end_camera;
                scene_changed = true;
            }
        }
        let camera_enabled = game.scene().controls() != ControlMode::Locked;
        if camera_enabled && key_pressed(&window, Key::A) {
            camera.orbit(-ORBIT_STEP, 0.0);
            changed = true;
        }
        if camera_enabled && key_pressed(&window, Key::D) {
            camera.orbit(ORBIT_STEP, 0.0);
            changed = true;
        }
        if camera_enabled && key_pressed(&window, Key::W) {
            camera.orbit(0.0, ORBIT_STEP);
            changed = true;
        }
        if camera_enabled && key_pressed(&window, Key::S) {
            camera.orbit(0.0, -ORBIT_STEP);
            changed = true;
        }
        if game.scene() == SceneState::Puzzle && puzzle.selected().is_some() {
            for (key, delta) in [
                (Key::Left, Vec3::new(-OBJECT_MOVE_STEP, 0.0, 0.0)),
                (Key::Right, Vec3::new(OBJECT_MOVE_STEP, 0.0, 0.0)),
                (Key::Up, Vec3::new(0.0, 0.0, -OBJECT_MOVE_STEP)),
                (Key::Down, Vec3::new(0.0, 0.0, OBJECT_MOVE_STEP)),
                (Key::PageUp, Vec3::new(0.0, OBJECT_MOVE_STEP, 0.0)),
                (Key::PageDown, Vec3::new(0.0, -OBJECT_MOVE_STEP, 0.0)),
            ] {
                let moved = key_pressed(&window, key) && puzzle.move_selected(delta);
                if moved {
                    changed = true;
                    scene_changed = true;
                }
            }
            let rotated =
                key_pressed(&window, Key::R) && puzzle.rotate_selected(OBJECT_ROTATION_STEP);
            if rotated {
                changed = true;
                scene_changed = true;
            }
        } else {
            if camera_enabled && key_pressed(&window, Key::Left) {
                camera.orbit(-ORBIT_STEP, 0.0);
                changed = true;
            }
            if camera_enabled && key_pressed(&window, Key::Right) {
                camera.orbit(ORBIT_STEP, 0.0);
                changed = true;
            }
            if camera_enabled && key_pressed(&window, Key::Up) {
                camera.orbit(0.0, ORBIT_STEP);
                changed = true;
            }
            if camera_enabled && key_pressed(&window, Key::Down) {
                camera.orbit(0.0, -ORBIT_STEP);
                changed = true;
            }
        }
        if camera_enabled
            && (key_pressed(&window, Key::Equal) || key_pressed(&window, Key::NumPadPlus))
        {
            camera.zoom(-ZOOM_STEP);
            changed = true;
        }
        if camera_enabled
            && (key_pressed(&window, Key::Minus) || key_pressed(&window, Key::NumPadMinus))
        {
            camera.zoom(ZOOM_STEP);
            changed = true;
        }

        let perspective = check_perspective(&camera, ASPECT_RATIO);
        if game.scene() == SceneState::Puzzle
            && puzzle.selected().is_none()
            && puzzle.is_solved(perspective.aligned)
            && game.handle(GameEvent::PuzzleSolved)
        {
            if eye.is_active() {
                eye.toggle();
            }
            timeline.restart();
            changed = true;
            scene_changed = true;
        }

        if game.scene() == SceneState::MemoryRestored {
            if window.is_key_pressed(Key::T, KeyRepeat::No) {
                timeline.restart();
                changed = true;
                scene_changed = true;
            }
            if window.is_key_pressed(Key::Space, KeyRepeat::No) {
                timeline.toggle_playback();
                changed = true;
                scene_changed = true;
            }
            if timeline.update(delta_seconds) {
                changed = true;
                scene_changed = true;
            }
            if window.is_key_pressed(Key::M, KeyRepeat::No) && game.handle(GameEvent::BeginMelanta)
            {
                sky_corruption = 0.0;
                changed = true;
                scene_changed = true;
            }
        }

        if game.scene() == SceneState::Melanta && sky_corruption < 1.0 {
            sky_corruption = (sky_corruption + delta_seconds / SKY_TRANSITION_SECONDS).min(1.0);
            changed = true;
        }
        if game.scene() == SceneState::Melanta
            && sky_corruption >= 1.0
            && window.is_key_pressed(Key::F, KeyRepeat::No)
            && game.handle(GameEvent::Finish)
        {
            sky_corruption = 0.0;
            changed = true;
            scene_changed = true;
        }
        if narrative.update(game.scene(), delta_seconds) {
            changed = true;
        }

        if scene_changed {
            scene = build_game_scene(game, puzzle, timeline);
        }

        if changed {
            let mut pixels = render_scene(renderer, &camera, &scene, textures, sky_corruption);
            draw_scene_labels(&mut pixels, game.scene(), &camera);
            draw_game_interface(
                &mut pixels,
                game,
                eye,
                puzzle,
                timeline,
                sky_corruption,
                &camera,
            );
            draw_narrative(&mut pixels, IMAGE_WIDTH, IMAGE_HEIGHT, narrative.message());
            draw_state_indicator(
                &mut pixels,
                game.scene(),
                PerspectiveStatus::from(puzzle, perspective),
            );
            apply_fade(&mut pixels, portal_transition.fade_opacity());
            buffer = renderer.to_u32_buffer(&pixels);
            window.set_title(&window_title(game, sky_corruption, &camera));
        }

        window.update_with_buffer(&buffer, IMAGE_WIDTH, IMAGE_HEIGHT)?;
    }

    Ok(())
}

fn key_pressed(window: &Window, key: Key) -> bool {
    window.is_key_pressed(key, KeyRepeat::Yes)
}

fn build_game_scene(game: GameState, puzzle: Puzzle, timeline: MemoryTimeline) -> Scene {
    let state = game.scene();
    let mut scene = if state == SceneState::MemoryRestored {
        let sample = timeline.sample();
        let mut scene =
            build_temple_interactive(sample.memory_state, sample.core_pose, false, sample.puzzle);
        scene.light = sample.light;
        scene
    } else {
        build_temple_interactive(
            state.memory_core(),
            world::MemoryCorePose::default(),
            false,
            puzzle.layout(),
        )
    };
    scene
        .objects
        .retain(|object| state.object_visible(object.name()));
    if state == SceneState::Melanta {
        add_melanta_event(&mut scene.objects);
        scene.light = melanta_light();
    }
    scene
}

fn render_scene(
    renderer: &Renderer,
    camera: &Camera,
    scene: &Scene,
    textures: &TextureSet,
    sky_corruption: f32,
) -> Vec<Vec3> {
    if sky_corruption <= 0.0 {
        renderer.render(camera, &scene.objects, &scene.light, textures)
    } else {
        renderer.render_with_sky(
            camera,
            &scene.objects,
            &scene.light,
            textures,
            temple_skybox(sky_corruption),
        )
    }
}

fn apply_fade(pixels: &mut [Vec3], opacity: f32) {
    let visible = 1.0 - opacity.clamp(0.0, 1.0);
    for pixel in pixels {
        *pixel = *pixel * visible;
    }
}

fn draw_game_interface(
    pixels: &mut [Vec3],
    game: GameState,
    eye: EyeOfGod,
    puzzle: Puzzle,
    timeline: MemoryTimeline,
    sky_corruption: f32,
    camera: &Camera,
) {
    let perspective = check_perspective(camera, ASPECT_RATIO);
    draw_interface(
        pixels,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        UiState {
            scene: game.scene(),
            eye_active: eye.is_active(),
            object_selected: puzzle.selected().is_some(),
            aligned_pieces: puzzle.aligned_count(),
            interaction_hint: puzzle.interaction_hint(perspective.aligned),
            pieces_aligned: puzzle.pieces_aligned(),
            camera_aligned: perspective.aligned,
            timeline: timeline.value(),
            timeline_playing: timeline.is_playing(),
            sky_corruption,
        },
    );
}

fn draw_scene_labels(pixels: &mut [Vec3], state: SceneState, camera: &Camera) {
    if state == SceneState::Melanta {
        draw_world_label(
            pixels,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            camera,
            Vec3::new(3.55, 2.45, -2.75),
            "MELANTA - WATCHER",
            Vec3::new(1.0, 0.16, 0.08),
        );
        return;
    }
    if state != SceneState::Temple {
        return;
    }
    for (position, text, color) in [
        (
            Vec3::new(-6.2, 3.48, -4.1),
            "LUYANG - ARTE",
            Vec3::new(1.0, 0.62, 0.30),
        ),
        (
            Vec3::new(6.2, 3.32, -4.1),
            "MAHAVAIPULYA - TEXTOS",
            Vec3::new(0.48, 0.88, 1.0),
        ),
        (
            Vec3::new(-6.2, 3.36, 3.8),
            "DESERT PAVILION",
            Vec3::new(1.0, 0.78, 0.30),
        ),
        (
            Vec3::new(0.0, 3.58, -1.25),
            "MEMORY CORE",
            Vec3::new(0.64, 0.94, 1.0),
        ),
    ] {
        draw_world_label(
            pixels,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            camera,
            position,
            text,
            color,
        );
    }
}

fn draw_perspective_indicator(pixels: &mut [Vec3], status: PerspectiveStatus) {
    let (color, radius) = match status {
        PerspectiveStatus::Searching => (Vec3::new(0.42, 0.08, 0.48), 5),
        PerspectiveStatus::CameraAligned => (Vec3::new(0.10, 0.95, 0.92), 8),
        PerspectiveStatus::Solved => (Vec3::new(1.0, 0.82, 0.22), 11),
    };
    let center_x = (IMAGE_WIDTH / 2) as i32;
    let center_y = 76_i32;
    for delta in -radius..=radius {
        set_indicator_pixel(pixels, center_x + delta, center_y, color);
        set_indicator_pixel(pixels, center_x, center_y + delta, color);
    }
}

fn draw_state_indicator(pixels: &mut [Vec3], state: SceneState, status: PerspectiveStatus) {
    if matches!(state, SceneState::Puzzle | SceneState::MemoryRestored) {
        draw_perspective_indicator(pixels, status);
    }
}

fn set_indicator_pixel(pixels: &mut [Vec3], x: i32, y: i32, color: Vec3) {
    if x >= 0 && y >= 0 && x < IMAGE_WIDTH as i32 && y < IMAGE_HEIGHT as i32 {
        pixels[y as usize * IMAGE_WIDTH + x as usize] = color;
    }
}

fn mouse_uv(window: &Window) -> (f32, f32) {
    window
        .get_mouse_pos(MouseMode::Clamp)
        .map(|(x, y)| (x / IMAGE_WIDTH as f32, 1.0 - y / IMAGE_HEIGHT as f32))
        .unwrap_or((0.5, 0.5))
}

fn window_title(game: GameState, sky_corruption: f32, camera: &Camera) -> String {
    let state = game.scene();
    if state == SceneState::Melanta {
        format!(
            "Temple of Space - {} - Corruption {:.0}% | yaw {:.0} pitch {:.0} r {:.1} | Core {}",
            state.label(),
            sky_corruption * 100.0,
            camera.yaw().to_degrees(),
            camera.pitch().to_degrees(),
            camera.radius(),
            state.memory_core().label(),
        )
    } else {
        format!(
            "Temple of Space - {} | yaw {:.0} pitch {:.0} r {:.1} | Core {}",
            state.label(),
            camera.yaw().to_degrees(),
            camera.pitch().to_degrees(),
            camera.radius(),
            state.memory_core().label(),
        )
    }
}
