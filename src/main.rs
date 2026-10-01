mod animation;
mod audio;
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
    AnimationState, ControlMode, ExhibitionId, GameEvent, GameState, NarrativeController,
    SceneState, SkyState, TransitionDestination,
};
use interaction::{
    AcademyChallenge, AcademyPhase, AcademyTarget, CAMERA_TOLERANCE, DesertChallenge, DesertPhase,
    ControllerInput, EyeOfGod, LibraryChallenge, LibraryPhase, POSITION_TOLERANCE, PerspectiveStatus, Puzzle,
    ROTATION_TOLERANCE, academy_target_at, check_perspective, exhibition_at, library_page_at,
};
use interface::{UiState, draw_interface, draw_narrative, draw_world_label};
use materials::TextureSet;
use math::Vec3;
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Scale, ScaleMode, Window, WindowOptions};
use raytracing::{Camera, Light, Renderer};
use world::{
    PortalView, PuzzleLayout, PuzzlePieceId, Scene, add_melanta_event, add_nihilita_epilogue,
    build_academy_room, build_desert_room_with_seal, build_library_room, build_temple,
    build_temple_interactive, melanta_light, melanta_transition_light, temple_skybox,
};

const IMAGE_WIDTH: usize = 576;
const IMAGE_HEIGHT: usize = 324;
const PREVIEW_WIDTH: usize = 480;
const PREVIEW_HEIGHT: usize = 270;
const INTERACTIVE_MAX_DEPTH: u32 = 2;
const ASPECT_RATIO: f32 = IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32;
const CAMERA_ORBIT_SPEED: f32 = 1.05;
const MAX_INPUT_DELTA_SECONDS: f32 = 0.05;
const OBJECT_ROTATION_STEP: f32 = 15.0_f32.to_radians();
const OBJECT_MOVE_STEP: f32 = 0.18;
const ZOOM_STEP: f32 = 0.4;
const MOUSE_ORBIT_SENSITIVITY: f32 = 0.003;
const MAX_MOUSE_ORBIT_DELTA: f32 = 4.0_f32.to_radians();
const MOUSE_WHEEL_ZOOM_STEP: f32 = 0.35;
const CONTROLLER_CURSOR_SPEED: f32 = 430.0;
const SKY_TRANSITION_SECONDS: f32 = 3.0;
const MELANTA_REVEAL_DELAY_SECONDS: f32 = 2.0;
const PORTAL_TRANSITION_SECONDS: f32 = 2.4;
const UI_TARGET_FPS: usize = 60;
const FULL_QUALITY_DELAY: Duration = Duration::from_millis(100);
const CHECKPOINT_AMBIENT_TIME: f32 = 1.2;
const ANIMATION_INTERVAL: Duration = Duration::from_millis(90);
const MAX_ANIMATION_DELTA_SECONDS: f32 = 0.25;

fn main() -> Result<(), Box<dyn Error>> {
    let audio_config = audio::AudioConfig::discover();
    let _background_music = audio_config.background_music();
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
    let renderer = Renderer::new(IMAGE_WIDTH, IMAGE_HEIGHT, Vec3::new(0.20, 0.31, 0.54))
        .with_edge_antialiasing(true);

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
        ("memory_restored", SceneState::MemoryRestored, 1.0, interior),
        ("melanta", SceneState::Melanta, 1.0, interior),
        (
            "desert_pavilion",
            SceneState::DesertPavilion,
            1.0,
            PortalView::DesertPavilion.camera(ASPECT_RATIO),
        ),
        (
            "mahavaipulya_chamber",
            SceneState::MahavaipulyaChamber,
            1.0,
            PortalView::MahavaipulyaChamber.camera(ASPECT_RATIO),
        ),
        (
            "luyang_academy",
            SceneState::LuyangAcademy,
            1.0,
            PortalView::LuyangAcademy.camera(ASPECT_RATIO),
        ),
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
    let desert_challenge = DesertChallenge::default();
    let library_challenge = LibraryChallenge::default();
    let academy_challenge = AcademyChallenge::default();
    let scene = build_game_scene_at(
        game,
        puzzle,
        timeline,
        desert_challenge,
        library_challenge,
        academy_challenge,
        CHECKPOINT_AMBIENT_TIME,
    );
    let sky_corruption = if state == SceneState::Melanta {
        1.0
    } else {
        0.0
    };
    let mut pixels = render_scene(
        renderer,
        camera,
        &scene,
        textures,
        state,
        sky_corruption,
        (state == SceneState::Temple).then(|| world::ambient_accent_light(CHECKPOINT_AMBIENT_TIME)),
    );
    draw_scene_labels(&mut pixels, state, camera);
    draw_game_interface(
        &mut pixels,
        game,
        eye,
        puzzle,
        timeline,
        sky_corruption,
        camera,
        desert_challenge,
        library_challenge,
        academy_challenge,
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
    apply_white_fade(&mut pixels, transition.white_opacity());
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
    let mut audio = audio::AudioDirector::new();
    audio.play_music_for(SceneState::Exterior);
    let mut audio_scene = SceneState::Exterior;
    let mut controller = ControllerInput::new();
    let interactive_renderer =
        Renderer::new(IMAGE_WIDTH, IMAGE_HEIGHT, Vec3::new(0.20, 0.31, 0.54))
            .with_max_depth(INTERACTIVE_MAX_DEPTH);
    let refine_renderer = Renderer::new(IMAGE_WIDTH, IMAGE_HEIGHT, Vec3::new(0.20, 0.31, 0.54))
        .with_max_depth(INTERACTIVE_MAX_DEPTH)
        .with_edge_antialiasing(true);
    let preview_renderer =
        Renderer::new(PREVIEW_WIDTH, PREVIEW_HEIGHT, Vec3::new(0.20, 0.31, 0.54)).with_max_depth(0);
    let mut game = GameState::default();
    let mut eye = EyeOfGod::default();
    let mut puzzle = Puzzle::default();
    let mut timeline = MemoryTimeline::default();
    let mut desert_challenge = DesertChallenge::default();
    let mut library_challenge = LibraryChallenge::default();
    let mut academy_challenge = AcademyChallenge::default();
    let mut portal_transition = SceneTransition::new(PORTAL_TRANSITION_SECONDS);
    let mut narrative = NarrativeController::default();
    let mut portal_start_camera = camera;
    let mut portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
    let mut transition_scene_swapped = true;
    let mut sky_corruption = 0.0_f32;
    let mut memory_restored_hold = 0.0_f32;
    let mut previous_orbit_mouse = None;
    let mut left_mouse_was_down = false;
    let mut help_visible = false;
    let mut cursor_position = None;
    let mut controller_cursor = (IMAGE_WIDTH as f32 * 0.5, IMAGE_HEIGHT as f32 * 0.5);
    let mut controller_cursor_active = false;
    let mut last_activity = Instant::now();
    let mut refine_at = None;
    let mut animations_enabled = true;
    let mut anim_time = 0.0_f32;
    let mut next_animation_at = Instant::now();
    let mut last_tick = Instant::now();
    let mut scene = build_game_scene(
        game,
        puzzle,
        timeline,
        desert_challenge,
        library_challenge,
        academy_challenge,
    );
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

    let mut cached_scene_pixels = render_scene(
        &refine_renderer,
        &camera,
        &scene,
        textures,
        game.scene(),
        sky_corruption,
        scene_accent_light(game.scene(), anim_time),
    );
    let mut pixels = cached_scene_pixels.clone();
    draw_scene_labels(&mut pixels, game.scene(), &camera);
    draw_game_interface(
        &mut pixels,
        game,
        eye,
        puzzle,
        timeline,
        sky_corruption,
        &camera,
        desert_challenge,
        library_challenge,
        academy_challenge,
    );
    draw_narrative(&mut pixels, IMAGE_WIDTH, IMAGE_HEIGHT, narrative.message());
    draw_state_indicator(
        &mut pixels,
        game.scene(),
        PerspectiveStatus::from(puzzle, check_perspective(&camera, ASPECT_RATIO)),
    );
    let mut buffer = renderer.to_u32_buffer(&pixels);

    let mut display_buffer = buffer.clone();
    while window.is_open() {
        let previous_scene = game.scene();
        let previous_desert_phase = desert_challenge.phase();
        let previous_library_phase = library_challenge.phase();
        let previous_academy_phase = academy_challenge.phase();
        controller.update();
        let now = Instant::now();
        let delta_seconds = now.duration_since(last_tick).as_secs_f32();
        last_tick = now;
        let (controller_cursor_x, controller_cursor_y) = controller.right_stick();
        if controller_cursor_x != 0.0 || controller_cursor_y != 0.0 {
            let cursor_step = CONTROLLER_CURSOR_SPEED * delta_seconds.min(MAX_INPUT_DELTA_SECONDS);
            controller_cursor.0 = (controller_cursor.0 + controller_cursor_x * cursor_step)
                .clamp(0.0, IMAGE_WIDTH as f32 - 1.0);
            controller_cursor.1 = (controller_cursor.1 - controller_cursor_y * cursor_step)
                .clamp(0.0, IMAGE_HEIGHT as f32 - 1.0);
            controller_cursor_active = true;
        }
        let mut changed = false;
        let mut scene_changed = false;
        let mut prefer_preview = false;
        let mut force_full_render = false;

        if window.is_key_pressed(Key::N, KeyRepeat::No) {
            narrative = NarrativeController::new(game.scene());
            changed = true;
        }
        if window.is_key_pressed(Key::H, KeyRepeat::No) {
            help_visible = !help_visible;
            changed = true;
        }
        if window.is_key_pressed(Key::Escape, KeyRepeat::No)
            || (controller.back_pressed() && game.scene() == SceneState::Puzzle) {
            match handle_escape(&mut game, &mut eye, &mut puzzle) {
                EscapeAction::Redraw => {
                    changed = true;
                    scene_changed = true;
                }
                EscapeAction::Exit => break,
            }
        }
        if game.scene() == SceneState::Temple
            && !eye.is_active()
            && (window.is_key_pressed(Key::Tab, KeyRepeat::No) || controller.eye_pressed())
            && game.handle(GameEvent::ActivateEyeOfGod)
        {
            eye.toggle();
            changed = true;
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) || controller.pause_pressed() {
            animations_enabled = !animations_enabled;
            changed = true;
            force_full_render = true;
        }
        if game.scene() == SceneState::Temple && window.is_key_pressed(Key::M, KeyRepeat::No) {
            game = GameState::from_scene(SceneState::Melanta);
            sky_corruption = 0.0;
            changed = true;
            scene_changed = true;
        }
        let left_mouse_down = window.get_mouse_down(MouseButton::Left);
        let controller_confirm = controller.confirm_pressed();
        let scene_clicked = (left_mouse_down && !left_mouse_was_down) || controller_confirm;
        left_mouse_was_down = left_mouse_down;
        let interact_pressed = window.is_key_pressed(Key::E, KeyRepeat::No) || controller_confirm;
        if !help_visible && (interact_pressed || scene_clicked) {
            match game.scene() {
                SceneState::Exterior => {
                    if (interact_pressed || portal_hovered(&window, &camera, &scene))
                        && game.handle(GameEvent::UsePortal) {
                        audio.play_effect(audio::SoundEffect::DoorOpen);
                        println!("State: {}", game.scene().label());
                        portal_start_camera = camera;
                        portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
                        portal_transition.restart();
                        transition_scene_swapped = false;
                        changed = true;
                    }
                }
                SceneState::Temple => {
                    let (u, v) = interaction_uv(&window, controller_cursor_active.then_some(controller_cursor));
                    let exhibition = exhibition_at(&camera, &scene.objects, u, v);
                    if let Some(exhibition) = exhibition {
                        let destination_view = match exhibition {
                            ExhibitionId::DesertPavilion => Some(PortalView::DesertPavilion),
                            ExhibitionId::MahavaipulyaChamber => {
                                Some(PortalView::MahavaipulyaChamber)
                            }
                            ExhibitionId::LuyangAcademy => Some(PortalView::LuyangAcademy),
                        };
                        if let Some(destination_view) = destination_view
                            && game.handle(GameEvent::EnterExhibition(exhibition))
                        {
                            match exhibition {
                                ExhibitionId::DesertPavilion => desert_challenge.restart(),
                                ExhibitionId::MahavaipulyaChamber => library_challenge.restart(),
                                ExhibitionId::LuyangAcademy => academy_challenge.restart(),
                            }
                            portal_start_camera = camera;
                            portal_end_camera = destination_view.camera(ASPECT_RATIO);
                            portal_transition.restart();
                            transition_scene_swapped = false;
                            changed = true;
                        }
                    } else if interact_pressed && game.handle(GameEvent::UsePortal) {
                        camera = PortalView::Exterior.camera(ASPECT_RATIO);
                        changed = true;
                        scene_changed = true;
                    }
                }
                SceneState::Puzzle => {
                    let (u, v) = interaction_uv(&window, controller_cursor_active.then_some(controller_cursor));
                    let interacted = puzzle.interact_at(&camera, &scene.objects, u, v);
                    if interacted {
                        changed = true;
                        scene_changed = true;
                    }
                }
                SceneState::DesertPavilion => {
                    let interaction_changed = match desert_challenge.phase() {
                        DesertPhase::AccessSeal => desert_challenge.confirm_seal(),
                        DesertPhase::Defeated => {
                            desert_challenge.restart();
                            true
                        }
                        DesertPhase::Complete => {
                            game.complete_exhibition(ExhibitionId::DesertPavilion);
                            if game.handle(GameEvent::LeaveExhibition) {
                                portal_start_camera = camera;
                                portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
                                portal_transition.restart();
                                transition_scene_swapped = false;
                            }
                            true
                        }
                        _ => false,
                    };
                    if interaction_changed {
                        changed = true;
                        scene_changed = game.scene() == SceneState::DesertPavilion;
                    }
                }
                SceneState::MahavaipulyaChamber => {
                    let (u, v) = interaction_uv(&window, controller_cursor_active.then_some(controller_cursor));
                    if let Some(page) = library_page_at(&camera, &scene.objects, u, v) {
                        if library_challenge.collect(page) {
                            audio.play_effect(audio::SoundEffect::PageCollect);
                            changed = true;
                            scene_changed = true;
                        }
                    } else {
                        match library_challenge.phase() {
                            LibraryPhase::Complete => {
                                game.complete_exhibition(ExhibitionId::MahavaipulyaChamber);
                                if game.handle(GameEvent::LeaveExhibition) {
                                    portal_start_camera = camera;
                                    portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
                                    portal_transition.restart();
                                    transition_scene_swapped = false;
                                    changed = true;
                                }
                            }
                            LibraryPhase::Defeated => {
                                library_challenge.restart();
                                changed = true;
                                scene_changed = true;
                            }
                            LibraryPhase::Collecting => {}
                        }
                    }
                }
                SceneState::LuyangAcademy => {
                    match academy_challenge.phase() {
                        AcademyPhase::Complete => {
                            game.complete_exhibition(ExhibitionId::LuyangAcademy);
                            if game.handle(GameEvent::LeaveExhibition) {
                                portal_start_camera = camera;
                                portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
                                portal_transition.restart();
                                transition_scene_swapped = false;
                                changed = true;
                            }
                        }
                        AcademyPhase::Defeated => {
                            academy_challenge.restart();
                            changed = true;
                            scene_changed = true;
                        }
                        AcademyPhase::Arranging => {
                            let (u, v) = interaction_uv(&window, controller_cursor_active.then_some(controller_cursor));
                            match academy_target_at(&camera, &scene.objects, u, v) {
                                Some(AcademyTarget::Fragment(fragment)) => {
                                    if academy_challenge.select(fragment) {
                                        changed = true;
                                        scene_changed = true;
                                    }
                                }
                                Some(AcademyTarget::ConfirmButton)
                                    if academy_challenge.confirm() =>
                                {
                                    changed = true;
                                    scene_changed = true;
                                }
                                Some(AcademyTarget::ConfirmButton) => {}
                                None => {}
                            }
                        }
                    }
                }
                SceneState::Final if interact_pressed => break,
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
            if !transition_scene_swapped && portal_transition.progress() >= 0.5 {
                transition_scene_swapped = true;
                scene_changed = true;
            }
            if portal_transition.is_finished() && game.handle(GameEvent::TransitionComplete) {
                camera = portal_end_camera;
                scene_changed = true;
            }
        }
        if matches!(
            game.scene(),
            SceneState::DesertPavilion
                | SceneState::MahavaipulyaChamber
                | SceneState::LuyangAcademy
        ) && (window.is_key_pressed(Key::B, KeyRepeat::No) || controller.back_pressed())
            && game.handle(GameEvent::LeaveExhibition)
        {
            portal_start_camera = camera;
            portal_end_camera = PortalView::Interior.camera(ASPECT_RATIO);
            portal_transition.restart();
            transition_scene_swapped = false;
            changed = true;
        }
        if game.scene() == SceneState::DesertPavilion
            && (window.is_key_pressed(Key::R, KeyRepeat::No) || controller.rotate_pressed())
            && desert_challenge.rotate_seal()
        {
            changed = true;
            scene_changed = true;
        }
        if game.scene() == SceneState::LuyangAcademy
            && (window.is_key_pressed(Key::R, KeyRepeat::No) || controller.rotate_pressed())
            && academy_challenge.move_selected_right()
        {
            audio.play_effect(audio::SoundEffect::PaintingMove);
            changed = true;
            scene_changed = true;
        }
        let camera_pose_before_input = (camera.yaw(), camera.pitch(), camera.radius());
        let orbit_step = camera_orbit_step(delta_seconds);
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
            camera.orbit(-orbit_step, 0.0);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::D) {
            camera.orbit(orbit_step, 0.0);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::W) {
            camera.orbit(0.0, orbit_step);
            changed = true;
            prefer_preview = true;
        }
        if camera_enabled && key_held(&window, Key::S) {
            camera.orbit(0.0, -orbit_step);
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
            for (active, delta) in [
                (controller.dpad_left(), Vec3::new(-OBJECT_MOVE_STEP, 0.0, 0.0)),
                (controller.dpad_right(), Vec3::new(OBJECT_MOVE_STEP, 0.0, 0.0)),
                (controller.dpad_up(), Vec3::new(0.0, 0.0, -OBJECT_MOVE_STEP)),
                (controller.dpad_down(), Vec3::new(0.0, 0.0, OBJECT_MOVE_STEP)),
            ] {
                if active && puzzle.move_selected(delta) {
                    changed = true;
                    scene_changed = true;
                    prefer_preview = true;
                }
            }
            let (stick_x, stick_y) = controller.left_stick();
            if (stick_x != 0.0 || stick_y != 0.0)
                && puzzle.move_selected(Vec3::new(stick_x * OBJECT_MOVE_STEP, 0.0, stick_y * OBJECT_MOVE_STEP))
            {
                changed = true;
                scene_changed = true;
                prefer_preview = true;
            }
            let rotated = (window.is_key_pressed(Key::R, KeyRepeat::No) || controller.rotate_pressed())
                && puzzle.rotate_selected(OBJECT_ROTATION_STEP);
            if rotated {
                changed = true;
                scene_changed = true;
                prefer_preview = true;
            }
        } else {
            if camera_enabled && key_held(&window, Key::Left) {
                camera.orbit(-orbit_step, 0.0);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && key_held(&window, Key::Right) {
                camera.orbit(orbit_step, 0.0);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && key_held(&window, Key::Up) {
                camera.orbit(0.0, orbit_step);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && key_held(&window, Key::Down) {
                camera.orbit(0.0, -orbit_step);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && controller.dpad_left() {
                camera.orbit(-orbit_step, 0.0);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && controller.dpad_right() {
                camera.orbit(orbit_step, 0.0);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && controller.dpad_up() {
                camera.orbit(0.0, orbit_step);
                changed = true;
                prefer_preview = true;
            }
            if camera_enabled && controller.dpad_down() {
                camera.orbit(0.0, -orbit_step);
                changed = true;
                prefer_preview = true;
            }
        }
        if camera_enabled && !(game.scene() == SceneState::Puzzle && puzzle.selected().is_some()) {
            let (stick_x, stick_y) = controller.left_stick();
            if stick_x != 0.0 || stick_y != 0.0 {
                camera.orbit(stick_y * CAMERA_ORBIT_SPEED * delta_seconds, stick_x * CAMERA_ORBIT_SPEED * delta_seconds);
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

        let camera_moved =
            camera_pose_before_input != (camera.yaw(), camera.pitch(), camera.radius());
        let lives_before_update = desert_challenge.lives();
        if game.scene() == SceneState::DesertPavilion
            && desert_challenge.update(delta_seconds, camera_moved)
        {
            if desert_challenge.lives() < lives_before_update {
                audio.play_effect(audio::SoundEffect::DesertLifeLost);
            }
            changed = true;
            scene_changed = true;
        }

        if game.scene() == SceneState::MahavaipulyaChamber {
            let update = library_challenge.update(delta_seconds);
            if update.ui_changed {
                changed = true;
            }
            if update.visual_changed {
                changed = true;
                scene_changed = true;
            }
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

        let melanta_active = match game.scene() {
            SceneState::Melanta => true,
            SceneState::DesertPavilion => desert_challenge.melanta_visible(),
            SceneState::LuyangAcademy => academy_challenge.phase() == AcademyPhase::Defeated,
            SceneState::MahavaipulyaChamber => library_challenge.phase() == LibraryPhase::Defeated,
            _ => false,
        };
        if game.scene() != audio_scene {
            audio.play_music_for(game.scene());
            audio_scene = game.scene();
        }
        audio.set_melanta(melanta_active, game.scene() == SceneState::Melanta);
        if game.scene() == SceneState::Entering && previous_scene != SceneState::Entering {
            audio.play_effect(audio::SoundEffect::Loading);
        }
        let task_completed = match game.scene() {
            SceneState::DesertPavilion => previous_desert_phase != DesertPhase::Complete
                && desert_challenge.phase() == DesertPhase::Complete,
            SceneState::MahavaipulyaChamber => previous_library_phase != LibraryPhase::Complete
                && library_challenge.phase() == LibraryPhase::Complete,
            SceneState::LuyangAcademy => previous_academy_phase != AcademyPhase::Complete
                && academy_challenge.phase() == AcademyPhase::Complete,
            _ => false,
        };
        if task_completed { audio.play_effect(audio::SoundEffect::TaskComplete); }
        let animating = animations_enabled
            && (game.scene() == SceneState::Temple
                || game.scene() == SceneState::Final
                || melanta_active);
        if animating {
            anim_time += delta_seconds.min(MAX_ANIMATION_DELTA_SECONDS);
            if changed || Instant::now() >= next_animation_at {
                changed = true;
                scene_changed = true;
            }
        }

        if scene_changed {
            scene = build_game_scene_at(
                game,
                puzzle,
                timeline,
                desert_challenge,
                library_challenge,
                academy_challenge,
                anim_time,
            );
        }

        let camera_input_active = camera_enabled
            && (window.get_mouse_down(MouseButton::Right)
                || key_held(&window, Key::A)
                || key_held(&window, Key::D)
                || key_held(&window, Key::W)
                || key_held(&window, Key::S)
                || key_held(&window, Key::Left)
                || key_held(&window, Key::Right)
                || key_held(&window, Key::Up)
                || key_held(&window, Key::Down)
                || key_held(&window, Key::Equal)
                || key_held(&window, Key::NumPadPlus)
                || key_held(&window, Key::Minus)
                || key_held(&window, Key::NumPadMinus)
                || controller.dpad_left()
                || controller.dpad_right()
                || controller.dpad_up()
                || controller.dpad_down()
                || controller.left_stick().0 != 0.0
                || controller.left_stick().1 != 0.0);
        if camera_input_active && refine_at.is_some() {
            refine_at = Some(Instant::now() + FULL_QUALITY_DELAY);
        }
        let refinement_due = refine_at.is_some_and(|deadline| Instant::now() >= deadline);
        if should_refine(changed, camera_input_active, refinement_due) {
            changed = true;
            refine_at = None;
            force_full_render = true;
        }

        if changed {
            let raytrace_required =
                needs_raytrace(prefer_preview, scene_changed, force_full_render);
            let render_started = Instant::now();
            let accent = scene_accent_light(game.scene(), anim_time);
            let mut pixels = if raytrace_required {
                let rendered = if prefer_preview {
                    let preview = render_scene(
                        &preview_renderer,
                        &camera,
                        &scene,
                        textures,
                        game.scene(),
                        sky_corruption,
                        accent,
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
                    let renderer = if animating {
                        &interactive_renderer
                    } else {
                        &refine_renderer
                    };
                    render_scene(
                        renderer,
                        &camera,
                        &scene,
                        textures,
                        game.scene(),
                        sky_corruption,
                        accent,
                    )
                };
                cached_scene_pixels = rendered.clone();
                rendered
            } else {
                cached_scene_pixels.clone()
            };
            if raytrace_required && animating {
                next_animation_at =
                    Instant::now() + ANIMATION_INTERVAL.max(render_started.elapsed() * 2);
            }
            draw_scene_labels(&mut pixels, game.scene(), &camera);
            draw_game_interface(
                &mut pixels,
                game,
                eye,
                puzzle,
                timeline,
                sky_corruption,
                &camera,
                desert_challenge,
                library_challenge,
                academy_challenge,
            );
            draw_narrative(&mut pixels, IMAGE_WIDTH, IMAGE_HEIGHT, narrative.message());
            draw_state_indicator(
                &mut pixels,
                game.scene(),
                PerspectiveStatus::from(puzzle, perspective),
            );
            if help_visible {
                interface::draw_help(&mut pixels, IMAGE_WIDTH, IMAGE_HEIGHT);
            }
            if game.scene() == SceneState::Entering {
                interface::draw_loading(&mut pixels, IMAGE_WIDTH, IMAGE_HEIGHT, portal_transition.progress());
            }
            buffer = renderer.to_u32_buffer(&pixels);
            window.set_title(&window_title(game, sky_corruption, &camera));
        }

        let mouse = window.get_mouse_pos(MouseMode::Discard);
        if mouse != cursor_position {
            controller_cursor_active = false;
        }
        if mouse != cursor_position || left_mouse_down || window.get_mouse_down(MouseButton::Right)
            || !window.get_keys().is_empty() || window.get_scroll_wheel().is_some()
            || controller_cursor_x != 0.0 || controller_cursor_y != 0.0 {
            last_activity = Instant::now();
        }
        cursor_position = mouse;
        let active_cursor = if controller_cursor_active { Some(controller_cursor) } else { mouse };
        window.set_cursor_visibility(active_cursor.is_none());
        display_buffer.copy_from_slice(&buffer);
        if let Some(position) = active_cursor {
            let (u, v) = cursor_uv(position);
            let hovered = match game.scene() {
                SceneState::Exterior => portal_hovered_at(&camera, &scene, u, v),
                SceneState::Temple => exhibition_at(&camera, &scene.objects, u, v)
                    .is_some_and(|id| game.exhibitions().can_enter(id)),
                SceneState::MahavaipulyaChamber => library_page_at(&camera, &scene.objects, u, v).is_some()
                    || library_challenge.phase() != LibraryPhase::Collecting,
                SceneState::LuyangAcademy => academy_target_at(&camera, &scene.objects, u, v).is_some()
                    || academy_challenge.phase() != AcademyPhase::Arranging,
                SceneState::DesertPavilion => matches!(desert_challenge.phase(),
                    DesertPhase::AccessSeal | DesertPhase::Complete | DesertPhase::Defeated),
                SceneState::Puzzle => {
                    let ray = camera.ray(u, v);
                    scene.objects.iter().any(|object| object.name().starts_with("puzzle piece")
                        && object.hit(&ray, 0.001, f32::INFINITY).is_some())
                }
                _ => false,
            };
            interface::cursor::draw(&mut display_buffer, IMAGE_WIDTH, IMAGE_HEIGHT, position,
                hovered, left_mouse_down, game.scene() == SceneState::Final,
                last_activity.elapsed().as_secs_f32());
        }
        window.update_with_buffer(&display_buffer, IMAGE_WIDTH, IMAGE_HEIGHT)?;
    }

    Ok(())
}

fn key_held(window: &Window, key: Key) -> bool {
    window.is_key_down(key)
}

fn needs_raytrace(camera_or_preview_changed: bool, scene_changed: bool, refine: bool) -> bool {
    camera_or_preview_changed || scene_changed || refine
}

fn should_refine(changed: bool, camera_input_active: bool, refinement_due: bool) -> bool {
    !changed && !camera_input_active && refinement_due
}

fn mouse_orbit_delta(previous: (f32, f32), current: (f32, f32)) -> (f32, f32) {
    (
        (-(current.0 - previous.0) * MOUSE_ORBIT_SENSITIVITY)
            .clamp(-MAX_MOUSE_ORBIT_DELTA, MAX_MOUSE_ORBIT_DELTA),
        (-(current.1 - previous.1) * MOUSE_ORBIT_SENSITIVITY)
            .clamp(-MAX_MOUSE_ORBIT_DELTA, MAX_MOUSE_ORBIT_DELTA),
    )
}

fn camera_orbit_step(delta_seconds: f32) -> f32 {
    CAMERA_ORBIT_SPEED * delta_seconds.clamp(0.0, MAX_INPUT_DELTA_SECONDS)
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

fn build_game_scene(
    game: GameState,
    puzzle: Puzzle,
    timeline: MemoryTimeline,
    desert_challenge: DesertChallenge,
    library_challenge: LibraryChallenge,
    academy_challenge: AcademyChallenge,
) -> Scene {
    build_game_scene_at(
        game,
        puzzle,
        timeline,
        desert_challenge,
        library_challenge,
        academy_challenge,
        0.0,
    )
}

fn build_game_scene_at(
    game: GameState,
    puzzle: Puzzle,
    timeline: MemoryTimeline,
    desert_challenge: DesertChallenge,
    library_challenge: LibraryChallenge,
    academy_challenge: AcademyChallenge,
    time: f32,
) -> Scene {
    let state = game.scene();
    if state == SceneState::DesertPavilion
        || (state == SceneState::Entering
            && game.transition_destination()
                == TransitionDestination::Exhibition(ExhibitionId::DesertPavilion))
    {
        let mut scene = build_desert_room_with_seal(desert_challenge.seal_yaw());
        if desert_challenge.melanta_visible() {
            add_melanta_event(&mut scene.objects, time);
            scene.light = melanta_light();
        }
        return scene;
    }
    if state == SceneState::LuyangAcademy
        || (state == SceneState::Entering
            && game.transition_destination()
                == TransitionDestination::Exhibition(ExhibitionId::LuyangAcademy))
    {
        let mut scene = build_academy_room(academy_challenge.order(), academy_challenge.selected());
        if academy_challenge.phase() == AcademyPhase::Defeated {
            add_melanta_event(&mut scene.objects, time);
            scene.light = melanta_light();
        }
        return scene;
    }
    if state == SceneState::MahavaipulyaChamber
        || (state == SceneState::Entering
            && game.transition_destination()
                == TransitionDestination::Exhibition(ExhibitionId::MahavaipulyaChamber))
    {
        let collected = [
            library_challenge.page_collected(0),
            library_challenge.page_collected(1),
            library_challenge.page_collected(2),
        ];
        let mut scene = build_library_room(collected, library_challenge.corruption());
        if library_challenge.phase() == LibraryPhase::Defeated {
            add_melanta_event(&mut scene.objects, time);
        }
        return scene;
    }
    let visual_state = if state == SceneState::Entering
        && game.transition_destination() == TransitionDestination::Temple
    {
        if game.exhibitions().all_completed() {
            SceneState::Final
        } else {
            SceneState::Temple
        }
    } else {
        state
    };
    let mut scene = if visual_state == SceneState::MemoryRestored {
        let sample = timeline.sample();
        let mut scene =
            world::build_temple_view(sample.memory_state, sample.core_pose, false, sample.puzzle, Some(false));
        scene.light = sample.light;
        scene
    } else {
        let core_pose = if visual_state == SceneState::Temple {
            world::ambient_core_pose(time)
        } else {
            world::MemoryCorePose::default()
        };
        world::build_temple_view(
            visual_state.memory_core(),
            core_pose,
            false,
            puzzle.layout(),
            Some(visual_state == SceneState::Exterior),
        )
    };
    scene
        .objects
        .retain(|object| visual_state.object_visible(object.name()));
    if visual_state == SceneState::Temple {
        world::add_ambient_shards(&mut scene.objects, time);
    }
    if visual_state == SceneState::Melanta {
        add_melanta_event(&mut scene.objects, time);
        scene.light = melanta_light();
    } else if visual_state == SceneState::Final {
        add_nihilita_epilogue(&mut scene.objects, time);
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
    accent: Option<Light>,
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
        renderer.render_with_accent(camera, &scene.objects, &light, accent, textures)
    }
}

fn scene_accent_light(state: SceneState, time: f32) -> Option<Light> {
    (state == SceneState::Temple).then(|| world::ambient_accent_light(time))
}

fn apply_white_fade(pixels: &mut [Vec3], opacity: f32) {
    let opacity = opacity.clamp(0.0, 1.0);
    for pixel in pixels {
        *pixel = *pixel * (1.0 - opacity) + Vec3::new(1.0, 1.0, 1.0) * opacity;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_game_interface(
    pixels: &mut [Vec3],
    game: GameState,
    eye: EyeOfGod,
    puzzle: Puzzle,
    timeline: MemoryTimeline,
    sky_corruption: f32,
    camera: &Camera,
    desert_challenge: DesertChallenge,
    library_challenge: LibraryChallenge,
    academy_challenge: AcademyChallenge,
) {
    let perspective = check_perspective(camera, ASPECT_RATIO);
    let exhibition_progress = game.exhibitions();
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
            desert_lives: desert_challenge.lives(),
            desert_watches: desert_challenge.survived_watches(),
            desert_status: desert_challenge.status(),
            library_pages: library_challenge.collected_count(),
            library_seconds: library_challenge.remaining_display(),
            library_status: library_challenge.status(),
            academy_selected: academy_challenge.selected_label(),
            academy_status: academy_challenge.status(),
            academy_order: academy_challenge.order(),
            hub_unlocked: exhibition_progress.hub_unlocked(),
            exhibitions_completed: exhibition_progress.completed_count(),
            all_exhibitions_completed: exhibition_progress.all_completed(),
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
    if state == SceneState::Final {
        draw_world_label(
            pixels,
            IMAGE_WIDTH,
            IMAGE_HEIGHT,
            camera,
            Vec3::new(2.65, 2.95, -2.85),
            "NIHILITA - GUARDIANA DE LA MEMORIA",
            Vec3::new(0.42, 0.94, 1.0),
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
    if matches!(
        state,
        SceneState::DesertPavilion
            | SceneState::MahavaipulyaChamber
            | SceneState::LuyangAcademy
    ) {
        return;
    }
    if state != SceneState::Temple {
        return;
    }
    for (position, text, color) in [
        (
            Vec3::new(-3.4, 2.40, -2.0),
            "ARTE - Click",
            Vec3::new(1.0, 0.62, 0.30),
        ),
        (
            Vec3::new(3.4, 2.25, -2.0),
            "BIBLIOTECA - Click",
            Vec3::new(0.48, 0.88, 1.0),
        ),
        (
            Vec3::new(0.0, 1.95, 2.2),
            "DESIERTO - Click",
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

fn portal_hovered(window: &Window, camera: &Camera, scene: &Scene) -> bool {
    let (u, v) = mouse_uv(window);
    portal_hovered_at(camera, scene, u, v)
}

fn portal_hovered_at(camera: &Camera, scene: &Scene, u: f32, v: f32) -> bool {
    let ray = camera.ray(u, v);
    scene.objects.iter()
        .filter_map(|object| object.hit(&ray, 0.001, f32::INFINITY))
        .min_by(|a, b| a.t.total_cmp(&b.t))
        .is_some_and(|hit| hit.object_name.starts_with("portal "))
}

fn mouse_uv(window: &Window) -> (f32, f32) {
    window
        .get_mouse_pos(MouseMode::Clamp)
        .map(cursor_uv)
        .unwrap_or((0.5, 0.5))
}

fn cursor_uv((x, y): (f32, f32)) -> (f32, f32) {
    (x / IMAGE_WIDTH as f32, 1.0 - y / IMAGE_HEIGHT as f32)
}

fn interaction_uv(window: &Window, controller_cursor: Option<(f32, f32)>) -> (f32, f32) {
    controller_cursor.map(cursor_uv).unwrap_or_else(|| mouse_uv(window))
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
