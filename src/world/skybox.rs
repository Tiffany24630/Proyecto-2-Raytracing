use crate::{math::Vec3, raytracing::SkyGradient};

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
}

const fn normal_skybox() -> SkyGradient {
    SkyGradient::new(Vec3::new(0.10, 0.12, 0.18), Vec3::new(0.30, 0.39, 0.58))
        .with_stars(Vec3::new(0.72, 0.86, 1.0), 0.85)
}

const fn melanta_skybox() -> SkyGradient {
    SkyGradient::new(Vec3::new(0.16, 0.005, 0.012), Vec3::new(0.58, 0.025, 0.035))
        .with_stars(Vec3::new(1.0, 0.14, 0.08), 0.62)
}

fn lerp(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * t
}

fn lerp_vec3(start: Vec3, end: Vec3, t: f32) -> Vec3 {
    start + (end - start) * t
}

#[cfg(test)]
mod tests {
    use super::temple_skybox;

    #[test]
    fn skybox_blends_normal_transition_and_melanta_states() {
        let normal = temple_skybox(0.0);
        let transition = temple_skybox(0.5);
        let melanta = temple_skybox(1.0);

        assert!(normal.zenith.z > normal.zenith.x);
        assert!(melanta.zenith.x > melanta.zenith.z * 8.0);
        assert!(transition.zenith.x > normal.zenith.x);
        assert!(transition.zenith.x < melanta.zenith.x);
        assert!(normal.star_intensity > 0.0);
        assert!(melanta.star_intensity > 0.0);
    }
}
