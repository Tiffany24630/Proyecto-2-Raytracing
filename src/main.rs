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
use std::time::{Duration, Instant};

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
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions};
use raytracing::{Camera, Renderer};
use world::{
    PortalView, PuzzleLayout, PuzzlePieceId, Scene, add_melanta_event, build_temple,
    build_temple_interactive, melanta_light, melanta_transition_light, temple_skybox,
};

const IMAGE_WIDTH: usize = 576;
const IMAGE_HEIGHT: usize = 324;
const PREVIEW_WIDTH: usize = 352;
const PREVIEW_HEIGHT: usize = 198;
const ASPECT_RATIO: f32 = IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32;
const ORBIT_STEP: f32 = 5.0_f32.to_radians();
const OBJECT_ROTATION_STEP: f32 = 15.0_f32.to_radians();
const OBJECT_MOVE_STEP: f32 = 0.18;
const ZOOM_STEP: f32 = 0.4;
const MOUSE_ORBIT_SENSITIVITY: f32 = 0.004;
const MOUSE_WHEEL_ZOOM_STEP: f32 = 0.35;
const SKY_TRANSITION_SECONDS: f32 = 3.0;
const MELANTA_REVEAL_DELAY_SECONDS: f32 = 2.0;
const PORTAL_TRANSITION_SECONDS: f32 = 2.4;
const UI_TARGET_FPS: usize = 30;
const FULL_QUALITY_DELAY: Duration = Duration::from_millis(60);

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
    let renderer = Renderer::new(IMAGE_WIDTH, IMAGE_HEIGHT, Vec3::new(0.20, 0.31, 0.54));

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
    let mut pixels = render_scene(renderer, camera, &scene, textures, state, sky_corruption);
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
    let preview_renderer =
        Renderer::new(PREVIEW_WIDTH, PREVIEW_HEIGHT, Vec3::new(0.20, 0.31, 0.54)).with_max_depth(1);
    let mut game = GameState::default();
    let mut eye = EyeOfGod::default();
    let mut puzzle = Puzzle::default();
    let mut timeline = MemoryTimeline::default();
    let mut portal_transition = SceneTransition::new(PORTAL_TRANSITION_SECONDS);
    let mut narrative = NarrativeController::default();
    let mut portal_start_camera = camera;
    let portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
    let mut sky_corruption = 0.0_f32;
    let mut memory_restored_hold = 0.0_f32;
    let mut previous_orbit_mouse = None;
    let mut left_mouse_was_down = false;
    let mut refine_at = None;
    let mut last_tick = Instant::now();
    let mut scene = build_game_scene(game, puzzle, timeline);
    let initial_title = window_title(game, sky_corruption, &camera);
    let mut window = Window::new(
        &initial_title,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(UI_TARGET_FPS);

    let mut pixels = render_scene(
        renderer,
        &camera,
        &scene,
        textures,
        game.scene(),
        sky_corruption,
    );
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
        let mut prefer_preview = false;

        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            match handle_escape(&mut game, &mut eye, &mut puzzle) {
                EscapeAction::Redraw => {
                    changed = true;
                    scene_changed = true;
                }
                EscapeAction::Exit => break,
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
        let left_mouse_down = window.get_mouse_down(MouseButton::Left);
        let puzzle_clicked =
            game.scene() == SceneState::Puzzle && left_mouse_down && !left_mouse_was_down;
        left_mouse_was_down = left_mouse_down;
        if window.is_key_pressed(Key::E, KeyRepeat::No) || puzzle_clicked {
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
            prefer_preview = true;
            if portal_transition.is_finished() && game.handle(GameEvent::TransitionComplete) {
                camera = portal_end_camera;
                scene_changed = true;
            }
        }
        let camera_enabled = game.scene().controls() != ControlMode::Locked;
        let mouse_position = window.get_mouse_pos(MouseMode::Clamp);
        if camera_enabled && window.get_mouse_down(MouseButton::Right) {
            if let (Some(previous), Some(current)) = (previous_orbit_mouse, mouse_position) {
                let (yaw_delta, pitch_delta) = mouse_orbit_delta(previous, current);
                if yaw_delta.abs() > f32::EPSILON || pitch_delta.abs() > f32::EPSILON {
                    camera.orbit(yaw_delta, pitch_delta);
                    changed = true;
                    prefer_preview = true;
                }
            }
            previous_orbit_mouse = mouse_position;
        } else {
            previous_orbit_mouse = None;
        }
        if camera_enabled
            && let Some((_, scroll_y)) = window.get_scroll_wheel()
            && scroll_y.abs() > f32::EPSILON
        {
            camera.zoom(-scroll_y * MOUSE_WHEEL_ZOOM_STEP);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::A) {
            camera.orbit(-ORBIT_STEP, 0.0);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::D) {
            camera.orbit(ORBIT_STEP, 0.0);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::W) {
            camera.orbit(0.0, ORBIT_STEP);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::S) {
            camera.orbit(0.0, -ORBIT_STEP);
            changed = true;
            prefer_preview = true;
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
                let moved = key_held(&window, key) && puzzle.move_selected(delta);
                if moved {
                    changed = true;
                    scene_changed = true;
                    prefer_preview = true;
                }
            }
            let rotated = window.is_key_pressed(Key::R, KeyRepeat::No)
                && puzzle.rotate_selected(OBJECT_ROTATION_STEP);
            if rotated {
                changed = true;
                scene_changed = true;
                prefer_preview = true;
            }
        } else {
            if camera_enabled && key_held(&window, Key::Left) {
                camera.orbit(-ORBIT_STEP, 0.0);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && key_held(&window, Key::Right) {
                camera.orbit(ORBIT_STEP, 0.0);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && key_held(&window, Key::Up) {
                camera.orbit(0.0, ORBIT_STEP);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && key_held(&window, Key::Down) {
                camera.orbit(0.0, -ORBIT_STEP);
                changed = true;
                prefer_preview = true;
            }
        }
        if camera_enabled && (key_held(&window, Key::Equal) || key_held(&window, Key::NumPadPlus)) {
            camera.zoom(-ZOOM_STEP);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && (key_held(&window, Key::Minus) || key_held(&window, Key::NumPadMinus))
        {
            camera.zoom(ZOOM_STEP);
            changed = true;
            prefer_preview = true;
        }

        let perspective = check_perspective(&camera, ASPECT_RATIO);
        if try_complete_puzzle(&mut game, puzzle, perspective.aligned) {
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
                prefer_preview = true;
            }
            let force_melanta = window.is_key_pressed(Key::M, KeyRepeat::No);
            if try_begin_melanta(
                &mut game,
                timeline,
                &mut memory_restored_hold,
                delta_seconds,
                force_melanta,
            ) {
                sky_corruption = 0.0;
                changed = true;
                scene_changed = true;
            }
        }

        if game.scene() == SceneState::Melanta && sky_corruption < 1.0 {
            sky_corruption = (sky_corruption + delta_seconds / SKY_TRANSITION_SECONDS).min(1.0);
            changed = true;
            prefer_preview = true;
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

        if !changed && refine_at.is_some_and(|deadline| Instant::now() >= deadline) {
            changed = true;
            refine_at = None;
        }

        if changed {
            let mut pixels = if prefer_preview {
                let preview = render_scene(
                    &preview_renderer,
                    &camera,
                    &scene,
                    textures,
                    game.scene(),
                    sky_corruption,
                );
                refine_at = Some(Instant::now() + FULL_QUALITY_DELAY);
                upscale_bilinear(
                    &preview,
                    PREVIEW_WIDTH,
                    PREVIEW_HEIGHT,
                    IMAGE_WIDTH,
                    IMAGE_HEIGHT,
                )
            } else {
                refine_at = None;
                render_scene(
                    renderer,
                    &camera,
                    &scene,
                    textures,
                    game.scene(),
                    sky_corruption,
                )
            };
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

fn key_held(window: &Window, key: Key) -> bool {
    window.is_key_down(key)
}

fn mouse_orbit_delta(previous: (f32, f32), current: (f32, f32)) -> (f32, f32) {
    (
        -(current.0 - previous.0) * MOUSE_ORBIT_SENSITIVITY,
        -(current.1 - previous.1) * MOUSE_ORBIT_SENSITIVITY,
    )
}

fn upscale_bilinear(
    source: &[Vec3],
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
) -> Vec<Vec3> {
    assert_eq!(source.len(), source_width * source_height);
    let mut target = Vec::with_capacity(target_width * target_height);
    for y in 0..target_height {
        let source_y = ((y as f32 + 0.5) * source_height as f32 / target_height as f32 - 0.5)
            .clamp(0.0, source_height.saturating_sub(1) as f32);
        let y0 = source_y.floor() as usize;
        let y1 = (y0 + 1).min(source_height - 1);
        let ty = source_y.fract();
        for x in 0..target_width {
            let source_x = ((x as f32 + 0.5) * source_width as f32 / target_width as f32 - 0.5)
                .clamp(0.0, source_width.saturating_sub(1) as f32);
            let x0 = source_x.floor() as usize;
            let x1 = (x0 + 1).min(source_width - 1);
            let tx = source_x.fract();
            let top =
                source[y0 * source_width + x0] * (1.0 - tx) + source[y0 * source_width + x1] * tx;
            let bottom =
                source[y1 * source_width + x0] * (1.0 - tx) + source[y1 * source_width + x1] * tx;
            target.push(top * (1.0 - ty) + bottom * ty);
        }
    }
    target
}

fn try_complete_puzzle(game: &mut GameState, puzzle: Puzzle, camera_aligned: bool) -> bool {
    game.scene() == SceneState::Puzzle
        && puzzle.selected().is_none()
        && puzzle.is_solved(camera_aligned)
        && game.handle(GameEvent::PuzzleSolved)
}

fn try_begin_melanta(
    game: &mut GameState,
    timeline: MemoryTimeline,
    restored_hold: &mut f32,
    delta_seconds: f32,
    forced: bool,
) -> bool {
    if game.scene() != SceneState::MemoryRestored {
        *restored_hold = 0.0;
        return false;
    }
    if forced {
        return game.handle(GameEvent::BeginMelanta);
    }
    if timeline.is_playing() || timeline.value() < 1.0 {
        *restored_hold = 0.0;
        return false;
    }

    *restored_hold += delta_seconds.max(0.0);
    *restored_hold >= MELANTA_REVEAL_DELAY_SECONDS && game.handle(GameEvent::BeginMelanta)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EscapeAction {
    Redraw,
    Exit,
}

fn handle_escape(game: &mut GameState, eye: &mut EyeOfGod, puzzle: &mut Puzzle) -> EscapeAction {
    if puzzle.cancel() {
        return EscapeAction::Redraw;
    }
    if game.scene() == SceneState::Puzzle && game.handle(GameEvent::TogglePuzzle) {
        if eye.is_active() {
            eye.toggle();
        }
        EscapeAction::Redraw
    } else {
        EscapeAction::Exit
    }
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
    state: SceneState,
    sky_corruption: f32,
) -> Vec<Vec3> {
    let light = if state == SceneState::Melanta {
        melanta_transition_light(sky_corruption)
    } else {
        scene.light
    };
    let corruption = if state == SceneState::Melanta {
        sky_corruption
    } else {
        0.0
    };
    if state == SceneState::Melanta {
        renderer.render_with_sky(
            camera,
            &scene.objects,
            &light,
            textures,
            temple_skybox(corruption),
        )
    } else {
        renderer.render(camera, &scene.objects, &light, textures)
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
    let interaction_hint = if puzzle.pieces_aligned() && puzzle.selected().is_none() {
        perspective.guidance.label()
    } else {
        puzzle.interaction_hint(perspective.aligned)
    };
    draw_interface(
        pixels,
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        UiState {
            scene: game.scene(),
            eye_active: eye.is_active(),
            selected_fragment: puzzle.selected_label(),
            aligned_pieces: puzzle.aligned_count(),
            piece_statuses: puzzle.piece_statuses(),
            interaction_hint,
            camera_aligned: perspective.aligned,
            camera_progress: perspective.progress,
            camera_stage: perspective.stage.label(),
            timeline: timeline.value(),
            timeline_playing: timeline.is_playing(),
            sky_corruption,
        },
    );
}

fn draw_scene_labels(pixels: &mut [Vec3], state: SceneState, camera: &Camera) {
    if state == SceneState::Exterior {
        draw_world_label(
            pixels,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            camera,
            Vec3::new(0.0, 4.45, 8.65),
            "WINDREST PEAK - MEMORY GATE",
            Vec3::new(0.58, 0.88, 1.0),
        );
        return;
    }
    if state == SceneState::Melanta {
        draw_world_label(
            pixels,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            camera,
            Vec3::new(2.55, 2.95, -3.55),
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
    let center_y = (IMAGE_HEIGHT / 2) as i32;
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

#[cfg(test)]
mod app_tests {
    use super::{
        ASPECT_RATIO, EscapeAction, IMAGE_HEIGHT, IMAGE_WIDTH, MELANTA_REVEAL_DELAY_SECONDS,
        OBJECT_MOVE_STEP, OBJECT_ROTATION_STEP, ORBIT_STEP, PREVIEW_HEIGHT, PREVIEW_WIDTH,
        ZOOM_STEP, handle_escape, mouse_orbit_delta, try_begin_melanta, try_complete_puzzle,
        upscale_bilinear,
    };

    #[test]
    fn presentation_resolution_is_larger_and_keeps_widescreen_aspect() {
        assert_eq!((IMAGE_WIDTH, IMAGE_HEIGHT), (576, 324));
        assert!((ASPECT_RATIO - 16.0 / 9.0).abs() < f32::EPSILON);
    }

    #[test]
    fn right_mouse_drag_orbits_in_both_axes() {
        let (yaw, pitch) = mouse_orbit_delta((100.0, 80.0), (125.0, 60.0));
        assert!(yaw < 0.0);
        assert!(pitch > 0.0);
    }

    #[test]
    fn preview_upscaling_blends_without_losing_dimensions_or_corners() {
        let source = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ];
        let enlarged = upscale_bilinear(&source, 2, 2, 4, 4);

        assert_eq!(enlarged.len(), 16);
        assert_eq!(enlarged[0], source[0]);
        assert_eq!(enlarged[3], source[1]);
        assert_eq!(enlarged[12], source[2]);
        assert_eq!(enlarged[15], source[3]);
        assert_ne!(enlarged[5], source[0]);
    }

    #[test]
    fn movement_preview_uses_at_most_thirty_eight_percent_of_full_resolution() {
        let preview_pixels = PREVIEW_WIDTH * PREVIEW_HEIGHT;
        let full_pixels = IMAGE_WIDTH * IMAGE_HEIGHT;
        assert!(preview_pixels * 100 <= full_pixels * 38);
    }

    use crate::{
        animation::MemoryTimeline,
        game::{GameState, SceneState},
        interaction::{EyeOfGod, Puzzle},
        math::Vec3,
        world::{PortalView, PuzzleLayout, PuzzlePieceId},
    };

    #[test]
    fn escape_never_enters_the_puzzle_from_the_temple() {
        let mut game = GameState::from_scene(SceneState::Temple);
        let mut eye = EyeOfGod::default();
        let mut puzzle = Puzzle::default();

        assert_eq!(
            handle_escape(&mut game, &mut eye, &mut puzzle),
            EscapeAction::Exit
        );
        assert_eq!(game.scene(), SceneState::Temple);
    }

    #[test]
    fn escape_cancels_a_piece_before_leaving_the_puzzle() {
        let mut game = GameState::from_scene(SceneState::Puzzle);
        let mut eye = EyeOfGod::default();
        eye.toggle();
        let mut puzzle = Puzzle::from_layout(PuzzleLayout {
            selected: Some(PuzzlePieceId::A),
            ..PuzzleLayout::initial()
        });

        assert_eq!(
            handle_escape(&mut game, &mut eye, &mut puzzle),
            EscapeAction::Redraw
        );
        assert_eq!(game.scene(), SceneState::Puzzle);
        assert_eq!(puzzle.selected(), None);

        assert_eq!(
            handle_escape(&mut game, &mut eye, &mut puzzle),
            EscapeAction::Redraw
        );
        assert_eq!(game.scene(), SceneState::Temple);
        assert!(!eye.is_active());
    }

    #[test]
    fn documented_controls_complete_the_puzzle_and_restore_memory() {
        let mut game = GameState::from_scene(SceneState::Puzzle);
        let mut puzzle = Puzzle::default();
        let mut camera = PortalView::Interior.camera(ASPECT_RATIO);

        for (piece, movement, rotation_steps) in [
            (
                PuzzlePieceId::A,
                [
                    (4, Vec3::new(OBJECT_MOVE_STEP, 0.0, 0.0)),
                    (3, Vec3::new(0.0, OBJECT_MOVE_STEP, 0.0)),
                    (1, Vec3::new(0.0, 0.0, -OBJECT_MOVE_STEP)),
                ],
                2,
            ),
            (
                PuzzlePieceId::B,
                [
                    (3, Vec3::new(0.0, -OBJECT_MOVE_STEP, 0.0)),
                    (1, Vec3::new(0.0, 0.0, -OBJECT_MOVE_STEP)),
                    (0, Vec3::default()),
                ],
                3,
            ),
            (
                PuzzlePieceId::C,
                [
                    (4, Vec3::new(-OBJECT_MOVE_STEP, 0.0, 0.0)),
                    (2, Vec3::new(0.0, OBJECT_MOVE_STEP, 0.0)),
                    (2, Vec3::new(0.0, 0.0, -OBJECT_MOVE_STEP)),
                ],
                4,
            ),
        ] {
            puzzle = Puzzle::from_layout(PuzzleLayout {
                selected: Some(piece),
                ..puzzle.layout()
            });
            for (steps, delta) in movement {
                for _ in 0..steps {
                    assert!(puzzle.move_selected(delta));
                }
            }
            for _ in 0..rotation_steps {
                assert!(puzzle.rotate_selected(OBJECT_ROTATION_STEP));
            }
            assert!(puzzle.interact_at(&camera, &[], 0.5, 0.5));
        }

        assert!(puzzle.pieces_aligned());
        assert!(!try_complete_puzzle(&mut game, puzzle, false));
        assert_eq!(game.scene(), SceneState::Puzzle);

        for _ in 0..3 {
            camera.orbit(ORBIT_STEP, 0.0);
        }
        for _ in 0..2 {
            camera.zoom(-ZOOM_STEP);
        }
        let camera_aligned = crate::interaction::check_perspective(&camera, ASPECT_RATIO).aligned;
        assert!(try_complete_puzzle(&mut game, puzzle, camera_aligned));
        assert_eq!(game.scene(), SceneState::MemoryRestored);
    }

    #[test]
    fn selected_piece_must_be_confirmed_before_restoration() {
        let mut game = GameState::from_scene(SceneState::Puzzle);
        let puzzle = Puzzle::from_layout(PuzzleLayout {
            selected: Some(PuzzlePieceId::A),
            ..PuzzleLayout::solved()
        });

        assert!(!try_complete_puzzle(&mut game, puzzle, true));
        assert_eq!(game.scene(), SceneState::Puzzle);
    }

    #[test]
    fn melanta_automatically_intervenes_after_the_restored_memory_is_visible() {
        let mut game = GameState::from_scene(SceneState::MemoryRestored);
        let timeline = MemoryTimeline::at(1.0);
        let mut hold = 0.0;

        assert!(!try_begin_melanta(
            &mut game,
            timeline,
            &mut hold,
            MELANTA_REVEAL_DELAY_SECONDS - 0.1,
            false,
        ));
        assert_eq!(game.scene(), SceneState::MemoryRestored);
        assert!(try_begin_melanta(
            &mut game, timeline, &mut hold, 0.1, false,
        ));
        assert_eq!(game.scene(), SceneState::Melanta);
    }

    #[test]
    fn melanta_waits_for_the_timeline_but_can_be_requested_early() {
        let mut game = GameState::from_scene(SceneState::MemoryRestored);
        let mut hold = 1.0;
        assert!(!try_begin_melanta(
            &mut game,
            MemoryTimeline::at(0.8),
            &mut hold,
            10.0,
            false,
        ));
        assert_eq!(hold, 0.0);
        assert!(try_begin_melanta(
            &mut game,
            MemoryTimeline::at(0.8),
            &mut hold,
            0.0,
            true,
        ));
        assert_eq!(game.scene(), SceneState::Melanta);
    }
}
