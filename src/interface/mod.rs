pub mod cursor;

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
    pub desert_lives: u8,
    pub desert_watches: u8,
    pub desert_status: &'static str,
    pub library_pages: u32,
    pub library_seconds: u8,
    pub library_status: &'static str,
    pub academy_selected: &'static str,
    pub academy_status: &'static str,
    pub academy_order: [u8; 3],
    pub hub_unlocked: bool,
    pub exhibitions_completed: u32,
    pub all_exhibitions_completed: bool,
}

pub fn draw_interface(pixels: &mut [Vec3], width: usize, height: usize, state: UiState) {
    draw_panel(pixels, width, height, 0, 0, width, 24, 0.64);
    draw_text(
        pixels,
        width,
        height,
        8,
        7,
        "TEMPLE OF SPACE",
        1,
        Vec3::new(0.72, 0.90, 1.0),
    );
    let state_label = if state.scene == SceneState::Temple && state.hub_unlocked {
        format!("MEMORIES {}/3", state.exhibitions_completed)
    } else {
        state.scene.label().to_string()
    };
    let label_x = width.saturating_sub(state_label.len() * 6 + 8);
    draw_text(
        pixels,
        width,
        height,
        label_x,
        11,
        &state_label,
        1,
        if state.all_exhibitions_completed {
            Vec3::new(0.20, 1.0, 0.78)
        } else {
            Vec3::new(0.95, 0.78, 0.28)
        },
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
        Vec3::new(0.80, 0.86, 0.94),
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

pub fn draw_loading(pixels: &mut [Vec3], width: usize, height: usize, progress: f32) {
    draw_panel(pixels, width, height, 0, 0, width, height, 0.91);
    draw_centered_text(pixels, width, height, 72, "SANTUARIO DE LOS RECUERDOS", 2, Vec3::new(0.72, 0.88, 1.0));
    draw_centered_text(pixels, width, height, 101, "CARGANDO MEMORIA PRESERVADA", 1, Vec3::new(0.86, 0.90, 0.96));
    let center_x = width / 2;
    let center_y = 162_i32;
    let elements = [
        Vec3::new(0.42, 0.86, 1.0), Vec3::new(0.34, 0.70, 0.92),
        Vec3::new(0.34, 0.94, 0.72), Vec3::new(0.92, 0.74, 0.30),
        Vec3::new(0.80, 0.48, 0.94), Vec3::new(1.0, 0.48, 0.34),
        Vec3::new(0.92, 0.88, 0.62),
    ];
    for (index, color) in elements.into_iter().enumerate() {
        let angle = index as f32 * std::f32::consts::TAU / 7.0 - std::f32::consts::FRAC_PI_2;
        let x = center_x as i32 + (angle.cos() * 49.0) as i32;
        let y = center_y + (angle.sin() * 31.0) as i32;
        draw_element_mark(pixels, width, height, x, y, color);
    }
    draw_element_mark(pixels, width, height, center_x as i32, center_y, Vec3::new(0.72, 0.94, 1.0));
    let bar_width = 220;
    let bar_x = width.saturating_sub(bar_width) / 2;
    draw_panel(pixels, width, height, bar_x, 234, bar_width, 7, 0.7);
    fill_rect(pixels, width, height, bar_x + 2, 236,
        ((bar_width - 4) as f32 * progress.clamp(0.0, 1.0)) as usize, 2, Vec3::new(0.40, 0.86, 1.0));
}

fn draw_element_mark(pixels: &mut [Vec3], width: usize, height: usize, x: i32, y: i32, color: Vec3) {
    for offset in -7..=7 {
        set_ui_pixel(pixels, width, height, x + offset, y, color);
        set_ui_pixel(pixels, width, height, x, y + offset, color);
        set_ui_pixel(pixels, width, height, x + offset / 2, y + offset, color);
    }
}

fn set_ui_pixel(pixels: &mut [Vec3], width: usize, height: usize, x: i32, y: i32, color: Vec3) {
    if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
        pixels[y as usize * width + x as usize] = color;
    }
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
            "ENTRANDO EN LA MEMORIA",
            Vec3::new(0.55, 0.88, 1.0),
        ),
        SceneState::Temple if state.eye_active => (
            "OJO DE DIOS ACTIVO",
            "SELECCIONA UNA MINIATURA PARA ENTRAR",
            Vec3::new(0.32, 0.88, 1.0),
        ),
        SceneState::Puzzle => (
            "ALINEA LOS FRAGMENTOS",
            state.interaction_hint,
            Vec3::new(0.32, 0.86, 1.0),
        ),
        SceneState::MemoryRestored => (
            "MEMORIA RESTAURADA",
            "EL ESPACIO RECUERDA",
            Vec3::new(0.22, 0.95, 0.92),
        ),
        SceneState::Final => (
            "MEMORIAS RESTAURADAS",
            "NIHILITA TE ESPERA EN EL TEMPLO",
            Vec3::new(0.38, 0.94, 1.0),
        ),
        SceneState::Melanta => (
            "MEMORIA CORRUPTA",
            "ESPERA A QUE TERMINE LA INTERVENCION",
            Vec3::new(1.0, 0.10, 0.08),
        ),
        SceneState::DesertPavilion => (
            "DESERT PAVILION",
            state.desert_status,
            Vec3::new(1.0, 0.70, 0.24),
        ),
        SceneState::MahavaipulyaChamber => (
            "MAHAVAIPULYA CHAMBER",
            state.library_status,
            Vec3::new(0.38, 0.82, 1.0),
        ),
        SceneState::LuyangAcademy => (
            "LUYANG ACADEMY",
            state.academy_status,
            Vec3::new(1.0, 0.62, 0.28),
        ),
        _ => return,
    };
    draw_centered_text(pixels, width, height, 34, heading, 1, color);
    draw_centered_text(
        pixels,
        width,
        height,
        48,
        detail,
        1,
        Vec3::new(0.86, 0.90, 1.0),
    );
    if state.scene == SceneState::Puzzle {
        draw_puzzle_progress(pixels, width, height, state);
    } else if state.scene == SceneState::DesertPavilion {
        let summary = format!(
            "VIDAS {}/3     APARICIONES {}/3",
            state.desert_lives, state.desert_watches
        );
        draw_centered_text(
            pixels,
            width,
            height,
            70,
            &summary,
            1,
            if state.desert_lives > 1 {
                Vec3::new(1.0, 0.80, 0.30)
            } else {
                Vec3::new(1.0, 0.12, 0.08)
            },
        );
    } else if state.scene == SceneState::MahavaipulyaChamber {
        let summary = format!(
            "TIEMPO {:02}s     PAGINAS {}/3",
            state.library_seconds, state.library_pages
        );
        draw_centered_text(
            pixels,
            width,
            height,
            70,
            &summary,
            1,
            if state.library_seconds > 15 {
                Vec3::new(0.46, 0.88, 1.0)
            } else {
                Vec3::new(1.0, 0.14, 0.08)
            },
        );
    } else if state.scene == SceneState::LuyangAcademy {
        let fragment_label = |fragment| match fragment {
            0 => 'A',
            1 => 'B',
            _ => 'C',
        };
        let summary = format!(
            "ORDEN {}-{}-{}     SELECCION {}",
            fragment_label(state.academy_order[0]),
            fragment_label(state.academy_order[1]),
            fragment_label(state.academy_order[2]),
            state.academy_selected,
        );
        draw_centered_text(
            pixels,
            width,
            height,
            70,
            &summary,
            1,
            Vec3::new(1.0, 0.76, 0.38),
        );
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
            "Click EN LA PUERTA PARA ENTRAR AL SANTUARIO".into(),
            "ARRASTRA CON BOTON DERECHO PARA MIRAR    H AYUDA".into(),
        ),
        SceneState::Entering => (
            "TRANSICION DEL PORTAL EN CURSO".into(),
            "CONTROLES TEMPORALMENTE BLOQUEADOS".into(),
        ),
        SceneState::Temple => {
            let first = if state.eye_active {
                "Click EN UNA MINIATURA PARA VISITAR SU MUNDO"
            } else {
                "PULSA TAB PARA ACTIVAR EL OJO DE DIOS"
            };
            (
                first.into(),
                "E FUERA DE MINIATURAS: VOLVER AL EXTERIOR    H AYUDA".into(),
            )
        }
        SceneState::Puzzle => {
            let eye = if state.eye_active {
                "OJO ACTIVO"
            } else {
                "OJO OFF"
            };
            (
                "Click EN UNA PIEZA; FLECHAS PARA MOVERLA; R PARA GIRAR".into(),
                format!("PGUP/PGDN ALTURA    ESC SOLTAR    H AYUDA    {eye}"),
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
        SceneState::DesertPavilion => (
            "R GIRA EL SELLO; Click LO ACTIVA    B VOLVER    H AYUDA".into(),
            "SI APARECE MELANTA: SUELTA LOS CONTROLES Y ESPERA".into(),
        ),
        SceneState::MahavaipulyaChamber => (
            "Click SOBRE CADA PAGINA PARA RECOGERLA    B VOLVER".into(),
            "REUNE LAS 3 ANTES DE QUE TERMINE EL TIEMPO    H AYUDA".into(),
        ),
        SceneState::LuyangAcademy => (
            "Click ELIGE UNA PIEZA; R CAMBIA SU LUGAR    B VOLVER".into(),
            "AL TERMINAR: Click EN EL BOTON CENTRAL    H AYUDA".into(),
        ),
        SceneState::Final => (
            "EPILOGO COMPLETADO     E  FINALIZAR".into(),
            "RMB / WASD  OBSERVAR     ESC  SALIR".into(),
        ),
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
    let scale = scale.min((width.saturating_sub(16) / (text.chars().count().max(1) * 6)).max(1));
    let text_width = text.chars().count() * 6 * scale;
    let x = width.saturating_sub(text_width) / 2;
    draw_panel(pixels, width, height, x.saturating_sub(5), y.saturating_sub(3),
        text_width + 10, 7 * scale + 6, 0.76);
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
        ';' => [0, 4, 4, 0, 4, 4, 8],
        '.' => [0, 0, 0, 0, 0, 6, 6],
        '(' => [2, 4, 8, 8, 8, 4, 2],
        ')' => [8, 4, 2, 2, 2, 4, 8],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '%' => [17, 2, 4, 4, 8, 16, 17],
        _ => [0; 7],
    }
}


pub fn draw_help(pixels: &mut [Vec3], width: usize, height: usize) {
    let lines = [
        "CONTROLES - H CIERRA ESTA AYUDA",
        "MUEVE EL RATON HASTA EL OBJETO Y HAZ Click IZQUIERDO",
        "EL OJO DEL CURSOR BRILLA SOBRE ZONAS INTERACTIVAS",
        "MANTEN BOTON DERECHO Y ARRASTRA PARA MIRAR",
        "W A S D TAMBIEN GIRAN LA CAMARA",
        "RUEDA DEL RATON O + / - ACERCAN Y ALEJAN",
        "TAB ACTIVA EL OJO DE DIOS EN EL SANTUARIO",
        "PUZZLE: Click ELIGE; FLECHAS MUEVEN; R GIRA",
        "PGUP / PGDN SUBEN Y BAJAN; ESC SUELTA LA PIEZA",
        "EN LAS SALAS: B REGRESA AL SANTUARIO",
        "E CONSERVA SU ACCION COMO ATAJO DE TECLADO",
        "P PAUSA ANIMACIONES; ESC SALE SI NO HAY SELECCION",
        "MEMORIA RESTAURADA: ESPACIO PAUSA; T REPITE; M AVANZA",
        "TRAS MELANTA: F RESTAURA; EN EL FINAL: E TERMINA",
        "MANDO XBOX: A Click; B VOLVER; Y OJO; X ROTAR; START PAUSA",
        "STICK DERECHO MUEVE LA CAMARA; DPAD Y STICK IZQUIERDO MUEVEN PIEZAS",
        "N REPITE LOS DIALOGOS DE ESTA ESCENA",
        "ESTA AYUDA NO PAUSA EL TIEMPO DE LOS RETOS",
    ];
    let panel_height = lines.len() * 14 + 16;
    let y = height.saturating_sub(panel_height) / 2;
    draw_panel(pixels, width, height, 12, y, width.saturating_sub(24), panel_height, 0.97);
    for (i, line) in lines.iter().enumerate() {
        draw_text(pixels, width, height, 23, y + 9 + i * 14, line, 1,
            if i == 0 { Vec3::new(1.0, 0.85, 0.48) } else { Vec3::new(0.92, 0.94, 0.98) });
    }
}
