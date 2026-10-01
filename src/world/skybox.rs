use crate::{math::Vec3, raytracing::SkyGradient};

// Multiplica la luminancia del panorama: un cielo azulado tenue se vuelve un
// rojo intenso, y las estrellas brillantes casi blancas quedan al rojo vivo.
const MELANTA_PANORAMA_TINT: Vec3 = Vec3::new(2.6, 0.30, 0.22);

pub fn temple_skybox(corruption: f32) -> SkyGradient {
    let t = corruption.clamp(0.0, 1.0);
    let normal = normal_skybox();
    let melanta = melanta_skybox();
    SkyGradient::new(
        lerp_vec3(normal.horizon, melanta.horizon, t),
        lerp_vec3(normal.zenith, melanta.zenith, t),
    )
    .with_stars(
        lerp_vec3(normal.star_color, melanta.star_color, t),
        lerp(normal.star_intensity, melanta.star_intensity, t),
    )
    // El panorama con textura es el cielo normal del templo. Con la corrupciÃ³n
    // de Melanta no desaparece: se tiÃ±e de rojo (conservando nubes y estrellas)
    // y se mezcla un poco con el degradado plano rojizo para dar resplandor.
    .with_texture_weight(1.0 - 0.35 * t)
    .with_panorama_tint(MELANTA_PANORAMA_TINT, t)
}

const fn normal_skybox() -> SkyGradient {
    SkyGradient::new(Vec3::new(0.20, 0.31, 0.54), Vec3::new(0.035, 0.07, 0.20))
        .with_stars(Vec3::new(0.68, 0.88, 1.0), 1.08)
}

const fn melanta_skybox() -> SkyGradient {
    SkyGradient::new(Vec3::new(0.22, 0.006, 0.014), Vec3::new(0.68, 0.018, 0.030))
        .with_stars(Vec3::new(1.0, 0.20, 0.10), 0.72)
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn lerp_vec3(start: Vec3, end: Vec3, t: f32) -> Vec3 {
    start + (end - start) * t
}
