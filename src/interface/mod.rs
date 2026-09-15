use crate::{
    game::{NarrativeMessage, SceneState},
    interaction::PieceStatus,
    math::Vec3,
    raytracing::Camera,
};

#[derive(Clone, Copy)]
pub struct UiState {
    pub scene: SceneState,
    pub eye_active: bool,
    pub selected_fragment: &'static str,
    pub aligned_pieces: usize,
    pub piece_statuses: [PieceStatus; 3],
    pub interaction_hint: &'static str,
    pub camera_aligned: bool,
    pub camera_progress: f32,
    pub camera_stage: &'static str,
    pub timeline: f32,
    pub timeline_playing: bool,
    pub sky_corruption: f32,
}

pub fn draw_interface(pixels: &mut [Vec3], width: usize, height: usize, state: UiState) {
    draw_panel(pixels, width, height, 0, 0, width, 29, 0.72);
    draw_text(
        pixels,
        width,
        height,
        8,
        7,
        "TEMPLE OF SPACE",
        2,
        Vec3::new(0.72, 0.90, 1.0),
    );
    let state_label = state.scene.label();
    let label_x = width.saturating_sub(state_label.len() * 6 + 8);
    draw_text(
        pixels,
        width,
        height,
        label_x,
        11,
        state_label,
        1,
        Vec3::new(0.95, 0.78, 0.28),
    );

    draw_context_message(pixels, width, height, state);

    draw_panel(
        pixels,
        width,
        height,
        0,
        height.saturating_sub(32),
        width,
        32,
        0.76,
    );
    let (first_line, second_line) = instructions(state);
    draw_text(
        pixels,
        width,
        height,
        8,
        height.saturating_sub(27),
        &first_line,
        1,
        Vec3::new(0.90, 0.94, 1.0),
    );
    draw_text(
        pixels,
        width,
        height,
        8,
        height.saturating_sub(15),
        &second_line,
        1,
        Vec3::new(0.52, 0.76, 0.90),
    );
}

pub fn draw_narrative(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    message: Option<NarrativeMessage>,
) {
    let Some(message) = message else {
        return;
    };
    let panel_width = 350;
    let panel_height = 34;
    let x = width.saturating_sub(panel_width) / 2;
    let y = height.saturating_sub(78);
    draw_panel(pixels, width, height, x, y, panel_width, panel_height, 0.78);
    let primary_color = if message.corrupted {
        Vec3::new(1.0, 0.13, 0.08)
    } else {
        Vec3::new(0.90, 0.78, 0.44)
    };
    draw_centered_text(
        pixels,
        width,
        height,
        y + 6,
        message.first_line,
        1,
        primary_color,
    );
    draw_centered_text(
        pixels,
        width,
        height,
        y + 18,
        message.second_line,
        1,
        Vec3::new(0.78, 0.86, 0.96),
    );
}

pub fn draw_world_label(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    camera: &Camera,
    position: Vec3,
    text: &str,
    color: Vec3,
) {
    let Some((screen_x, screen_y)) = camera.project_to_screen(position, width, height) else {
        return;
    };
    let label_width = text.chars().count() * 6 + 10;
    let x = screen_x
        .saturating_sub(label_width / 2)
        .min(width.saturating_sub(label_width));
    let y = screen_y.clamp(31, height.saturating_sub(47));
    draw_panel(pixels, width, height, x, y, label_width, 13, 0.78);
    draw_text(pixels, width, height, x + 5, y + 3, text, 1, color);
}

fn draw_context_message(pixels: &mut [Vec3], width: usize, height: usize, state: UiState) {
    let (heading, detail, color) = match state.scene {
        SceneState::Entering => (
            "ENTERING",
            "ACCESSING PRESERVED MEMORY",
            Vec3::new(0.55, 0.88, 1.0),
        ),
        SceneState::Puzzle => (
            "MEMORY FRAGMENT",
            state.interaction_hint,
            Vec3::new(0.32, 0.86, 1.0),
        ),
        SceneState::MemoryRestored | SceneState::Final => (
            "MEMORY RESTORED",
            "THE SPACE REMEMBERS",
            Vec3::new(0.22, 0.95, 0.92),
        ),
        SceneState::Melanta => (
            "MEMORY CORRUPTED",
            "MELANTA INTERVENTION",
            Vec3::new(1.0, 0.10, 0.08),
        ),
        _ => return,
    };
    draw_centered_text(pixels, width, height, 38, heading, 2, color);
    draw_centered_text(
        pixels,
        width,
        height,
        57,
        detail,
        1,
        Vec3::new(0.86, 0.90, 1.0),
    );
    if state.scene == SceneState::Puzzle {
        draw_puzzle_progress(pixels, width, height, state);
    }
}

fn draw_puzzle_progress(pixels: &mut [Vec3], width: usize, height: usize, state: UiState) {
    let [piece_a, piece_b, piece_c] = state.piece_statuses;
    let progress = state.camera_progress.clamp(0.0, 1.0);
    let summary = format!(
        "{}/3  A:{} B:{} C:{}  VINCULO:{}  CAMARA {:03.0}% {}",
        state.aligned_pieces,
        piece_a.label(),
        piece_b.label(),
        piece_c.label(),
        state.selected_fragment,
        progress * 100.0,
        state.camera_stage,
    );
    draw_centered_text(
        pixels,
        width,
        height,
        69,
        &summary,
        1,
        if state.camera_aligned {
            Vec3::new(0.20, 1.0, 0.84)
        } else {
            Vec3::new(0.58, 0.82, 1.0)
        },
    );

    let bar_width = width.min(180);
    let bar_x = width.saturating_sub(bar_width) / 2;
    draw_panel(pixels, width, height, bar_x, 80, bar_width, 5, 0.84);
    let fill_width = ((bar_width.saturating_sub(4)) as f32 * progress).round() as usize;
    let color = if state.camera_aligned {
        Vec3::new(0.12, 1.0, 0.78)
    } else if progress >= 0.82 {
        Vec3::new(0.94, 0.78, 0.24)
    } else {
        Vec3::new(0.20, 0.72, 1.0)
    };
    fill_rect(pixels, width, height, bar_x + 2, 82, fill_width, 1, color);
}

fn instructions(state: UiState) -> (String, String) {
    match state.scene {
        SceneState::Exterior => (
            "E  CRUZAR MEMORY GATE     RMB / WASD  CAMARA".into(),
            "RUEDA O + / -  ZOOM     ESC  SALIR".into(),
        ),
        SceneState::Entering => (
            "TRANSICION DEL PORTAL EN CURSO".into(),
            "CONTROLES TEMPORALMENTE BLOQUEADOS".into(),
        ),
        SceneState::Temple => (
            "TAB  ACTIVAR OJO DE DIOS     RMB / WASD  CAMARA".into(),
            "E  VOLVER AL PORTAL     RUEDA O + / -  ZOOM     ESC  SALIR".into(),
        ),
        SceneState::Puzzle => {
            let eye = if state.eye_active {
                "OJO ACTIVO"
            } else {
                "OJO OFF"
            };
            (
                "CLICK/E VINCULAR  ESC CANCELAR  R ROTAR".into(),
                format!("FLECHAS MOVER  PGUP/PGDN ALTURA  RMB/WASD CAMARA  {eye}"),
            )
        }
        SceneState::MemoryRestored => {
            let playback = if state.timeline_playing {
                "ACTIVA"
            } else {
                "PAUSA"
            };
            (
                "INTERVENCION AUTOMATICA     M  ADELANTAR     T  REPETIR".into(),
                format!(
                    "TIMELINE {:03.0}% {playback}     SPACE CONTROL     WASD CAMARA",
                    state.timeline * 100.0,
                ),
            )
        }
        SceneState::Melanta => {
            let action = if state.sky_corruption >= 1.0 {
                "F  RESTAURAR MEMORIA     ESC  SALIR"
            } else {
                "CORRUPCION EN CURSO     CONTROLES BLOQUEADOS"
            };
            (
                format!("CORRUPCION {:03.0}%", state.sky_corruption * 100.0),
                action.into(),
            )
        }
        SceneState::Final => ("MEMORY PRESERVATION COMPLETE".into(), "ESC  SALIR".into()),
    }
}

fn draw_centered_text(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    y: usize,
    text: &str,
    scale: usize,
    color: Vec3,
) {
    let text_width = text.chars().count() * 6 * scale;
    let x = width.saturating_sub(text_width) / 2;
    draw_text(pixels, width, height, x, y, text, scale, color);
}

#[allow(clippy::too_many_arguments)]
fn draw_panel(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    panel_width: usize,
    panel_height: usize,
    opacity: f32,
) {
    let panel_color = Vec3::new(0.008, 0.012, 0.025);
    for row in y..(y + panel_height).min(height) {
        for column in x..(x + panel_width).min(width) {
            let index = row * width + column;
            pixels[index] = pixels[index] * (1.0 - opacity) + panel_color * opacity;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_rect(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    rect_width: usize,
    rect_height: usize,
    color: Vec3,
) {
    for row in y..(y + rect_height).min(height) {
        for column in x..(x + rect_width).min(width) {
            pixels[row * width + column] = color;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_text(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    text: &str,
    scale: usize,
    color: Vec3,
) {
    let mut cursor = x;
    for character in text.chars() {
        draw_glyph(
            pixels,
            width,
            height,
            cursor,
            y,
            glyph(character),
            scale,
            color,
        );
        cursor += 6 * scale;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_glyph(
    pixels: &mut [Vec3],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    rows: [u8; 7],
    scale: usize,
    color: Vec3,
) {
    for (row, bits) in rows.into_iter().enumerate() {
        for column in 0..5 {
            if bits & (1 << (4 - column)) == 0 {
                continue;
            }
            for offset_y in 0..scale {
                for offset_x in 0..scale {
                    let pixel_x = x + column * scale + offset_x;
                    let pixel_y = y + row * scale + offset_y;
                    if pixel_x < width && pixel_y < height {
                        pixels[pixel_y * width + pixel_x] = color;
                    }
                }
            }
        }
    }
}

fn glyph(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '%' => [17, 2, 4, 4, 8, 16, 17],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::{PieceStatus, UiState, draw_interface, instructions};
    use crate::{game::SceneState, math::Vec3};

    #[test]
    fn overlay_changes_pixels_without_changing_buffer_size() {
        let mut pixels = vec![Vec3::new(0.5, 0.5, 0.5); 160 * 90];
        draw_interface(
            &mut pixels,
            160,
            90,
            UiState {
                scene: SceneState::Puzzle,
                eye_active: true,
                selected_fragment: "--",
                aligned_pieces: 0,
                piece_statuses: [PieceStatus::Fragmented; 3],
                interaction_hint: "VINCULA UN FRAGMENTO CON E",
                camera_aligned: false,
                camera_progress: 0.5,
                camera_stage: "ACERCANDOTE",
                timeline: 0.0,
                timeline_playing: false,
                sky_corruption: 0.0,
            },
        );
        assert_eq!(pixels.len(), 160 * 90);
        assert!(
            pixels
                .iter()
                .any(|pixel| *pixel != Vec3::new(0.5, 0.5, 0.5))
        );
    }

    #[test]
    fn instructions_fit_the_window_and_explain_automatic_melanta() {
        let base = UiState {
            scene: SceneState::Exterior,
            eye_active: true,
            selected_fragment: "--",
            aligned_pieces: 0,
            piece_statuses: [PieceStatus::Fragmented; 3],
            interaction_hint: "VINCULA UN FRAGMENTO CON E",
            camera_aligned: false,
            camera_progress: 0.5,
            camera_stage: "ACERCANDOTE",
            timeline: 1.0,
            timeline_playing: false,
            sky_corruption: 0.0,
        };

        for scene in [
            SceneState::Exterior,
            SceneState::Entering,
            SceneState::Temple,
            SceneState::Puzzle,
            SceneState::MemoryRestored,
            SceneState::Melanta,
            SceneState::Final,
        ] {
            let (first, second) = instructions(UiState { scene, ..base });
            assert!(first.chars().count() <= 77, "first line clips in {scene:?}");
            assert!(
                second.chars().count() <= 77,
                "second line clips in {scene:?}"
            );
        }

        let (first, _) = instructions(UiState {
            scene: SceneState::MemoryRestored,
            ..base
        });
        assert!(first.contains("INTERVENCION AUTOMATICA"));
        assert!(first.contains("M  ADELANTAR"));
    }
}
