use std::path::Path;

use image::ImageResult;

use crate::{math::Vec3, raytracing::Uv};

use super::{Material, MaterialKind};

pub struct Texture {
    width: usize,
    height: usize,
    pixels: Vec<[u8; 3]>,
}

impl Texture {
    fn generated(width: usize, height: usize, sample: impl Fn(f32, f32) -> Vec3) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let c = sample(x as f32 / width as f32, 1.0 - y as f32 / height as f32);
                pixels.push([(c.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (c.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (c.z.clamp(0.0, 1.0) * 255.0) as u8]);
            }
        }
        Self { width, height, pixels }
    }

    pub fn load(path: impl AsRef<Path>) -> ImageResult<Self> {
        let image = image::open(path)?.to_rgb8();
        let (width, height) = image.dimensions();
        let pixels = image
            .pixels()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect();

        Ok(Self {
            width: width as usize,
            height: height as usize,
            pixels,
        })
    }

    pub fn sample_bilinear(&self, uv: Uv, scale: f32) -> Vec3 {
        let u = (uv.u * scale).rem_euclid(1.0);
        let v = (uv.v * scale).rem_euclid(1.0);
        let x = u * self.width as f32;
        let y = (1.0 - v) * self.height as f32;
        let x0 = x.floor() as usize % self.width;
        let y0 = y.floor() as usize % self.height;
        let x1 = (x0 + 1) % self.width;
        let y1 = (y0 + 1) % self.height;
        let tx = x.fract();
        let ty = y.fract();

        let top = self.pixel(x0, y0) * (1.0 - tx) + self.pixel(x1, y0) * tx;
        let bottom = self.pixel(x0, y1) * (1.0 - tx) + self.pixel(x1, y1) * tx;
        top * (1.0 - ty) + bottom * ty
    }

    fn pixel(&self, x: usize, y: usize) -> Vec3 {
        let pixel = self.pixels[y * self.width + x];
        Vec3::new(
            pixel[0] as f32 / 255.0,
            pixel[1] as f32 / 255.0,
            pixel[2] as f32 / 255.0,
        )
    }

    pub fn sample_direction(&self, direction: Vec3) -> Vec3 {
        let azimuth = direction.z.atan2(direction.x);
        let u = (azimuth / std::f32::consts::TAU + 0.5).rem_euclid(1.0);
        let elevation = direction.y.clamp(-1.0, 1.0).asin();
        let v = (0.5 - elevation / std::f32::consts::PI).clamp(0.0, 1.0);

        let x = u * self.width as f32;
        let y = v * self.height as f32;
        let x0 = x.floor() as usize % self.width;
        let y0 = (y.floor() as usize).min(self.height - 1);
        let x1 = (x0 + 1) % self.width;
        let y1 = (y0 + 1).min(self.height - 1);
        let tx = x.fract();
        let ty = y.fract();

        let top = self.pixel(x0, y0) * (1.0 - tx) + self.pixel(x1, y0) * tx;
        let bottom = self.pixel(x0, y1) * (1.0 - tx) + self.pixel(x1, y1) * tx;
        top * (1.0 - ty) + bottom * ty
    }
}

pub struct TextureSet {
    paper: Texture,
    gate: Texture,
    grass: Texture,
    stone: Texture,
    wood: Texture,
    metal: Texture,
    crystal: Texture,
    ink: Texture,
    skybox: Texture,
}

impl TextureSet {
    pub fn load_from_directory(directory: impl AsRef<Path>) -> ImageResult<Self> {
        let directory = directory.as_ref();
        Ok(Self {
            paper: Texture::load(directory.join("library_page.png")).unwrap_or_else(|_| Texture::generated(256, 384, |u, v| {
                let grain = ((u * 1921.0 + v * 7919.0).sin() * 43758.545).fract().abs();
                let edge = u.min(1.0 - u).min(v.min(1.0 - v));
                let base = Vec3::new(0.94, 0.88, 0.70) * (0.91 + grain * 0.09);
                let line = (v * 24.0).fract();
                let written = u > 0.16 && u < 0.81 - 0.08 * (v * 43.0).sin()
                    && v > 0.18 && v < 0.76 && line < 0.065
                    && (u * 83.0).fract() > 0.13;
                let ornament = ((u - 0.5).abs() + (v - 0.86).abs() * 0.8 - 0.045).abs() < 0.006;
                if written { Vec3::new(0.37, 0.26, 0.16) }
                else if ornament { Vec3::new(0.56, 0.37, 0.14) }
                else if edge < 0.04 { base * 0.78 }
                else { base }
            })),
            gate: Texture::load(directory.join("domain_door.png")).unwrap_or_else(|_| Texture::generated(384, 512, |u, v| {
                let x = (u - 0.5).abs() * 2.0;
                let grain = (u * 1631.0 + v * 3917.0).sin() * 0.015;
                let border = x > 0.93 || u < 0.035 || u > 0.965 || v < 0.025 || v > 0.97;
                let seam = x < 0.018;
                let panel = ((u * 2.0).fract() - 0.5).abs() + (v - 0.48).abs() * 0.82;
                let carved = (panel - 0.31).abs() < 0.014 || (panel - 0.48).abs() < 0.012;
                let triangle = (x * 0.78 + (v - 0.50).abs() * 1.16 - 0.26).abs() < 0.018
                    && v > 0.25 && v < 0.72;
                if triangle { Vec3::new(0.12, 0.88, 1.0) }
                else if border || carved { Vec3::new(0.68 + grain, 0.55 + grain, 0.25 + grain) }
                else if seam { Vec3::new(0.08, 0.38, 0.48) }
                else { Vec3::new(0.20 + grain, 0.31 + grain, 0.40 + grain) }
            })),
            grass: Texture::load(directory.join("grass.png"))?,
            stone: Texture::load(directory.join("stone_celestial.png"))?,
            wood: Texture::load(directory.join("wood.png"))?,
            metal: Texture::load(directory.join("metal.png"))?,
            crystal: Texture::load(directory.join("crystal.png"))?,
            ink: Texture::load(directory.join("ink.png"))?,
            skybox: Texture::load(directory.join("skybox.png"))?,
        })
    }

    pub fn sample(&self, material: Material, uv: Uv, surface_name: &str) -> Vec3 {
        let texture = if surface_name.starts_with("library page ") {
            &self.paper
        } else if surface_name == "portal membrane" {
            &self.gate
        } else if surface_name.starts_with("exterior grass") {
            &self.grass
        } else {
            match material.kind {
                MaterialKind::Stone => &self.stone,
                MaterialKind::Wood => &self.wood,
                MaterialKind::Metal => &self.metal,
                MaterialKind::Crystal => &self.crystal,
                MaterialKind::Ink => &self.ink,
            }
        };
        texture.sample_bilinear(uv, material.texture_scale)
    }

    pub fn sample_skybox(&self, direction: Vec3) -> Vec3 {
        self.skybox.sample_direction(direction)
    }
}
