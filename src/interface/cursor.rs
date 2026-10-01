use std::sync::OnceLock;

struct CursorTexture { width: usize, height: usize, pixels: Vec<[u8; 4]> }

fn texture() -> &'static Option<CursorTexture> {
    static CURSOR: OnceLock<Option<CursorTexture>> = OnceLock::new();
    CURSOR.get_or_init(|| {
        let image = image::open("assets/textures/cursor_nihilita.png").ok()?.to_rgba8();
        let (width, height) = image.dimensions();
        Some(CursorTexture { width: width as usize, height: height as usize,
            pixels: image.pixels().map(|pixel| pixel.0).collect() })
    })
}

pub fn draw(
    buffer: &mut [u32], width: usize, height: usize, position: (f32, f32),
    hovered: bool, pressed: bool, final_scene: bool, idle: f32,
) {
    if let Some(texture) = texture() {
        draw_texture(buffer, width, height, position, texture, hovered, pressed, final_scene, idle);
        return;
    }
    let gold = if final_scene { 0x8eeeff } else { 0xe8bd58 };
    let eye_color = if hovered && !final_scene { 0x000000 } else { gold };
    let red = if final_scene { 0x388ac4 } else { 0xa52e13 };
    let pulse = if idle > 5.0 { ((idle - 5.0) * 2.0).sin() } else { 0.0 };
    let shift = if pressed { 1.0 } else { 0.0 };
    let origin = (position.0 + shift, position.1 + shift);
    let shapes: &[(&[(f32, f32)], u32)] = &[
        (&[(0.,0.),(10.,6.),(13.,19.),(8.,29.),(17.,40.),(9.,36.),(3.,25.),(6.,14.)], 0x30251f),
        (&[(1.,1.),(9.,7.),(11.,18.),(6.,28.),(14.,37.),(7.,33.),(3.,24.),(7.,13.)], 0xf6ebdd),
        (&[(1.,3.),(-2.,14.),(-8.,18.),(-12.,13.),(-10.,23.),(-2.,26.),(5.,18.)], red),
        (&[(-12.,13.),(-11.,8.),(-9.,16.),(-6.,18.),(-8.,20.)], gold),
        (&[(0.,0.),(3.,2.),(7.,10.),(3.,8.)], gold),
        (&[(8.,13.),(8.,23.),(2.,28.),(3.,21.)], 0xffffff),
        (&[(-7.,27.),(9.,24.),(10.,27.),(-6.,30.)], gold),
        (&[(9.,28.),(18.,34.),(24.,43.),(11.,36.),(4.,31.)], 0xe5d7c8),
        (&[(7.,33.),(17.,36.),(12.,45.),(4.,40.)], red),
        (&[(3.,30.),(4.,40.),(10.,48.),(-1.,41.),(-2.,35.)], 0xf6ebdd),
    ];
    for y in -3..52 {
        for x in -16..28 {
            let px = origin.0 as i32 + x;
            let py = origin.1 as i32 + y;
            if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 { continue; }
            let point = (x as f32, y as f32);
            let mut color = None;
            for &(polygon, fill) in shapes {
                if contains(polygon, point) { color = Some(fill); }
            }
            let eye = ((point.0 - 4.0).powi(2) + (point.1 - 20.0).powi(2)).sqrt();
            if eye < 2.4 { color = Some(if pressed { 0xffffff } else { eye_color }); }
            if (hovered || idle > 5.0) && (eye - (4.0 + pulse * 0.6)).abs() < 0.6 {
                color = Some(eye_color);
            }
            if pressed && (eye - 7.0).abs() < 0.6 { color = Some(0xffffff); }
            if idle > 5.0 {
                let sparkle_y = 8.0 + pulse * 4.0;
                if ((point.0 - 17.0).abs() < 0.8 && (point.1 - sparkle_y).abs() < 3.0)
                    || ((point.0 - 17.0).abs() < 3.0 && (point.1 - sparkle_y).abs() < 0.8) {
                    color = Some(gold);
                }
            }
            if let Some(color) = color { buffer[py as usize * width + px as usize] = color; }
        }
    }
}

fn draw_texture(buffer: &mut [u32], width: usize, height: usize, position: (f32, f32),
    texture: &CursorTexture, hovered: bool, pressed: bool, final_scene: bool, idle: f32) {
    const DISPLAY_HEIGHT: usize = 64;
    const ROTATION: f32 = -0.30;

    let state = if final_scene { 3 } else if pressed { 2 } else if hovered { 1 } else { 0 };
    let frame_width = texture.width / 4;
    let display_width = (frame_width * DISPLAY_HEIGHT / texture.height).max(1);
    // The point of the hand is near the upper-right part of the sprite.
    let anchor_x = display_width * 3 / 4;
    let anchor_y = 3usize;
    let bob = if idle > 5.0 { ((idle - 5.0) * 2.0).sin() as i32 } else { 0 };
    draw_afk_effect(buffer, width, height, position, idle, final_scene);

    let cosine = ROTATION.cos();
    let sine = ROTATION.sin();
    for local_y in -10..(DISPLAY_HEIGHT as i32 + 12) {
        for local_x in -(display_width as i32)..(display_width as i32 + 14) {
            // Rotate the art slightly toward the usual upper-left mouse-pointer direction.
            let source_x = anchor_x as f32 + cosine * local_x as f32 + sine * local_y as f32;
            let source_y = anchor_y as f32 - sine * local_x as f32 + cosine * local_y as f32;
            if source_x < 0.0 || source_y < 0.0
                || source_x >= display_width as f32 || source_y >= DISPLAY_HEIGHT as f32 {
                continue;
            }
            let texture_x = source_x as usize * frame_width / display_width;
            let texture_y = source_y as usize * texture.height / DISPLAY_HEIGHT;
            let pixel = texture.pixels[texture_y * texture.width + state * frame_width + texture_x];
            if pixel[3] < 16 { continue; }
            let target_x = position.0 as i32 + local_x;
            let target_y = position.1 as i32 + local_y + bob;
            if target_x >= 0 && target_y >= 0 && target_x < width as i32 && target_y < height as i32 {
                buffer[target_y as usize * width + target_x as usize] =
                    ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | pixel[2] as u32;
            }
        }
    }
}

fn draw_afk_effect(
    buffer: &mut [u32], width: usize, height: usize, position: (f32, f32), idle: f32,
    final_scene: bool,
) {
    if idle <= 5.0 { return; }

    let elapsed = idle - 5.0;
    let radius = 17.0 + (elapsed * 2.4).sin() * 2.0;
    let color = if final_scene { 0x4edfff } else { 0xd8a83d };
    let center_x = position.0 as i32;
    let center_y = position.1 as i32 + 17;

    for y in -25..26 {
        for x in -25..26 {
            let distance = ((x * x + y * y) as f32).sqrt();
            if (distance - radius).abs() < 0.8 || (distance - radius - 6.0).abs() < 0.45 {
                let px = center_x + x;
                let py = center_y + y;
                if px >= 0 && py >= 0 && px < width as i32 && py < height as i32 {
                    buffer[py as usize * width + px as usize] = color;
                }
            }
        }
    }

    for orbit in 0..3 {
        let angle = elapsed * 1.7 + orbit as f32 * 2.094;
        let px = center_x + (angle.cos() * (radius + 5.0)) as i32;
        let py = center_y + (angle.sin() * (radius + 5.0)) as i32;
        for y in -1i32..2 {
            for x in -1i32..2 {
                if x.abs() + y.abs() > 1 { continue; }
                let star_x = px + x;
                let star_y = py + y;
                if star_x >= 0 && star_y >= 0 && star_x < width as i32 && star_y < height as i32 {
                    buffer[star_y as usize * width + star_x as usize] = color;
                }
            }
        }
    }
}

fn contains(polygon: &[(f32, f32)], p: (f32, f32)) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current.1 > p.1) != (previous.1 > p.1)
            && p.0 < (previous.0 - current.0) * (p.1 - current.1)
                / (previous.1 - current.1) + current.0 {
            inside = !inside;
        }
        previous = current;
    }
    inside
}
