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

    /// Muestrea la textura como un panorama equirectangular a partir de una
    /// direcciÃ³n 3D normalizada: el azimut (`atan2`) recorre las columnas de
    /// forma cÃ­clica (por eso solo el eje horizontal se envuelve con
    /// `rem_euclid`) y la elevaciÃ³n (`asin`) recorre las filas de cenit
    /// (arriba) a nadir (abajo), sin envolver verticalmente. Se usa para el
    /// skybox: en vez de mezclar dos colores planos, cada direcciÃ³n de rayo
    /// que no golpea geometrÃ­a cae en un texel real de un panorama.
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
        let texture = if surface_name.starts_with("exterior grass") {
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

    /// Color del panorama del skybox para una direcciÃ³n de rayo que no
    /// impactÃ³ ninguna geometrÃ­a de la escena.
    pub fn sample_skybox(&self, direction: Vec3) -> Vec3 {
        self.skybox.sample_direction(direction)
    }
}
