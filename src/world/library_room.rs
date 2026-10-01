use crate::{
    geometry::{Cube, Object, Plane},
    materials::{Material, MaterialKind, crystal, ink, metal, stone, wood},
    math::Vec3,
    raytracing::Light,
};

use super::Scene;

/// Paleta de tinte para los lomos de los libros: cada entrada es
/// (color, kind_base, texture_weight). `kind_base` decide quÃ© textura de
/// fondo se muestrea (madera, tinta/cuero u observa oscura de piedra para
/// "papel"); `texture_weight` bajo dejar ver mÃ¡s el tinte que la textura, asÃ­
/// cada estante se lee como una fila de libros de colores distintos en vez de
/// un bloque sÃ³lido.
const SPINE_PALETTE: [(Vec3, MaterialKind, f32); 8] = [
    (Vec3::new(0.55, 0.10, 0.09), MaterialKind::Ink, 0.40),
    (Vec3::new(0.12, 0.32, 0.22), MaterialKind::Ink, 0.40),
    (Vec3::new(0.16, 0.24, 0.52), MaterialKind::Wood, 0.35),
    (Vec3::new(0.62, 0.46, 0.14), MaterialKind::Wood, 0.35),
    (Vec3::new(0.38, 0.14, 0.42), MaterialKind::Ink, 0.40),
    (Vec3::new(0.58, 0.56, 0.50), MaterialKind::Stone, 0.30),
    (Vec3::new(0.20, 0.20, 0.24), MaterialKind::Wood, 0.30),
    (Vec3::new(0.70, 0.30, 0.10), MaterialKind::Ink, 0.35),
];

pub fn build_library_room(collected_pages: [bool; 3], corruption: f32) -> Scene {
    let corruption = corruption.clamp(0.0, 1.0);
    let wood = wood();
    let metal = metal();
    let ink = ink();
    let mut floor = stone();
    floor.albedo = Vec3::new(0.20, 0.27, 0.34);
    floor.texture_weight = 0.58;
    let mut paper = stone();
    paper.albedo = Vec3::new(0.86, 0.73, 0.49);
    paper.texture_scale = 0.40;
    paper.texture_weight = 0.28;
    let mut memory = crystal();
    memory.albedo = Vec3::new(0.18, 0.64, 0.92);
    memory.emission = Vec3::new(0.05, 0.20, 0.34);
    let mut rug = wood;
    rug.albedo = Vec3::new(0.42, 0.10, 0.12);
    rug.specular = 0.05;
    rug.texture_weight = 0.30;
    let mut lamp_glass = crystal();
    lamp_glass.transparency = 0.55;
    lamp_glass.reflectivity = 0.10;
    lamp_glass.emission = Vec3::new(0.62, 0.46, 0.20);
    lamp_glass.texture_weight = 0.25;

    let mut objects: Vec<Box<dyn Object>> = vec![Box::new(Plane::new(
        "library floor",
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        floor,
    ))];

    add_cube(
        &mut objects,
        "library rug",
        Vec3::new(0.0, -0.99, -0.55),
        Vec3::new(2.6, 0.02, 1.7),
        rug,
    );

    for x in [-4.3, 4.3] {
        add_bookcase(&mut objects, x, wood, ink, paper);
    }

    add_reading_table(&mut objects, wood, metal, paper, lamp_glass);

    add_cube(
        &mut objects,
        "library giant book pedestal",
        Vec3::new(0.0, -0.58, -3.25),
        Vec3::new(3.3, 0.72, 2.25),
        metal,
    );
    for x in [-0.82, 0.82] {
        add_cube(
            &mut objects,
            "library giant book",
            Vec3::new(x, 0.08, -3.25),
            Vec3::new(1.56, 0.18, 2.05),
            paper,
        );
    }

    for (index, (position, size)) in [
        (Vec3::new(-2.55, 0.35, -0.65), Vec3::new(0.78, 1.08, 0.035)),
        (Vec3::new(2.45, 1.48, -1.85), Vec3::new(0.78, 1.08, 0.035)),
        (Vec3::new(0.0, 2.32, -4.05), Vec3::new(0.78, 1.08, 0.035)),
    ]
    .into_iter()
    .enumerate()
    {
        if !collected_pages[index] {
            let name = match index {
                0 => "library page A",
                1 => "library page B",
                _ => "library page C",
            };
            let mut page = paper;
            page.albedo = Vec3::new(0.98, 0.94, 0.82);
            page.texture_scale = 1.0;
            page.texture_weight = 0.92;
            page.reflectivity = 0.0;
            page.specular = 0.02;
            page.emission = Vec3::new(0.12, 0.10, 0.07);
            add_cube(&mut objects, name, position, size, page);
        } else {
            add_cube(
                &mut objects,
                "library restored page",
                Vec3::new(-0.52 + index as f32 * 0.52, 0.22, -3.25),
                Vec3::new(0.42, 0.05, 0.72),
                memory,
            );
        }
    }

    if corruption >= 0.35 {
        for (index, x) in [-3.1, -1.55, 0.0, 1.55, 3.1].into_iter().enumerate() {
            if corruption >= 0.7 || index % 2 == 0 {
                add_cube(
                    &mut objects,
                    "library melanta echo",
                    Vec3::new(x, 0.4 + (index % 2) as f32 * 1.25, -5.15),
                    Vec3::new(0.12, 1.15, 0.12),
                    ink,
                );
            }
        }
    }

    let calm_color = Vec3::new(0.54, 0.78, 1.0);
    let danger_color = Vec3::new(1.0, 0.08, 0.06);
    Scene {
        objects,
        light: Light::new(
            Vec3::new(-2.8, 6.8, 2.4),
            calm_color + (danger_color - calm_color) * corruption,
            1.05 + corruption * 0.10,
        ),
        camera_target: Vec3::new(0.0, 1.1, -2.35),
    }
}

/// Un mÃ³dulo de estanterÃ­a completo en `x`: marco de madera (postes + panel
/// trasero), cuatro estantes y, sobre cada uno, una fila de libros
/// individuales de ancho y color variados generados a partir de un hash
/// determinista (nada de aleatoriedad real, para que la escena sea
/// reproducible entre ejecuciones).
fn add_bookcase(
    objects: &mut Vec<Box<dyn Object>>,
    x: f32,
    wood: Material,
    ink: Material,
    paper: Material,
) {
    let shelf_ys = [-0.55, 0.65, 1.85, 3.05];

    add_cube(
        objects,
        "library shelf frame",
        Vec3::new(x, 1.6, -3.95),
        Vec3::new(2.34, 4.80, 0.14),
        wood,
    );
    for side in [-1.0, 1.0] {
        add_cube(
            objects,
            "library shelf post",
            Vec3::new(x + side * 1.13, 1.6, -3.55),
            Vec3::new(0.14, 4.80, 0.86),
            wood,
        );
    }

    for &y in &shelf_ys {
        add_cube(
            objects,
            "library shelf",
            Vec3::new(x, y, -3.7),
            Vec3::new(2.2, 0.18, 0.72),
            wood,
        );
        add_book_row(objects, x, y + 0.09, ink, paper);
    }
}

/// Genera libros individuales sobre un estante, uno junto a otro, hasta llenar
/// casi todo el ancho disponible. El primer libro de cada fila usa `ink` y el
/// segundo `paper` como base de textura antes del tinte, asÃ­ que ademÃ¡s de la
/// paleta de colores hay variaciÃ³n de material real entre lomos vecinos.
fn add_book_row(
    objects: &mut Vec<Box<dyn Object>>,
    shelf_x: f32,
    base_y: f32,
    ink: Material,
    paper: Material,
) {
    const ROW_WIDTH: f32 = 1.94;
    let mut cursor = shelf_x - ROW_WIDTH * 0.5;
    let mut index = 0_u32;
    while cursor < shelf_x + ROW_WIDTH * 0.5 - 0.08 {
        let seed = (shelf_x * 100.0) as i32 ^ (base_y * 100.0) as i32 ^ index as i32;
        let width = 0.14 + book_variation(seed, 1) * 0.12;
        let height = 0.46 + book_variation(seed, 2) * 0.30;
        let depth = 0.60 + book_variation(seed, 3) * 0.10;
        let lean = (book_variation(seed, 4) - 0.5) * 0.18;

        let (tint, kind, texture_weight) = SPINE_PALETTE[(seed.unsigned_abs() as usize
            + index as usize)
            % SPINE_PALETTE.len()];
        let mut spine = if index % 2 == 0 { ink } else { paper };
        spine.kind = kind;
        spine.albedo = tint;
        spine.texture_weight = texture_weight;
        spine.specular = 0.15 + book_variation(seed, 5) * 0.25;
        spine.reflectivity = 0.0;
        spine.transparency = 0.0;

        add_rotated_cube(
            objects,
            "library book spine",
            Vec3::new(cursor + width * 0.5, base_y + height * 0.5, -3.30),
            Vec3::new(width, height, depth),
            lean,
            spine,
        );

        cursor += width + 0.015;
        index += 1;
    }
}

/// Hash determinista en `[0, 1)`: mismo patrÃ³n que `procedural_star` en el
/// renderer, adaptado para variar el tamaÃ±o y el color de cada libro sin
/// depender de ningÃºn generador aleatorio externo.
fn book_variation(seed: i32, salt: i32) -> f32 {
    let hash = (seed.wrapping_mul(73_856_093) ^ salt.wrapping_mul(19_349_663)) as u32;
    (hash % 10_000) as f32 / 10_000.0
}

/// Mesa de lectura con dos bancas, un libro abierto y una lÃ¡mpara de cristal
/// suspendida encima: da al salÃ³n un punto de interÃ©s en primer plano ademÃ¡s
/// de las estanterÃ­as del fondo.
fn add_reading_table(
    objects: &mut Vec<Box<dyn Object>>,
    wood: Material,
    metal: Material,
    paper: Material,
    lamp_glass: Material,
) {
    let table_top_y = -0.42;
    add_cube(
        objects,
        "library table top",
        Vec3::new(0.0, table_top_y, -0.55),
        Vec3::new(1.7, 0.08, 1.0),
        wood,
    );
    for (side_x, side_z) in [(-0.75, 0.08), (0.75, 0.08), (-0.75, -1.18), (0.75, -1.18)] {
        add_cube(
            objects,
            "library table leg",
            Vec3::new(side_x, -0.71, side_z),
            Vec3::new(0.10, 0.58, 0.10),
            metal,
        );
    }
    for bench_z in [0.45, -1.55] {
        add_cube(
            objects,
            "library bench",
            Vec3::new(0.0, -0.74, bench_z),
            Vec3::new(1.3, 0.10, 0.36),
            wood,
        );
    }
    let mut open_book = paper;
    open_book.texture_weight = 0.22;
    add_cube(
        objects,
        "library open book",
        Vec3::new(0.0, table_top_y + 0.06, -0.55),
        Vec3::new(0.62, 0.04, 0.44),
        open_book,
    );

    add_cube(
        objects,
        "library lamp post",
        Vec3::new(0.62, 0.55, -0.55),
        Vec3::new(0.07, 1.9, 0.07),
        metal,
    );
    add_cube(
        objects,
        "library lamp shade",
        Vec3::new(0.62, 1.42, -0.55),
        Vec3::new(0.34, 0.30, 0.34),
        lamp_glass,
    );
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
