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
}

pub struct TextureSet {
    stone: Texture,
    wood: Texture,
    metal: Texture,
    crystal: Texture,
    ink: Texture,
}

impl TextureSet {
    pub fn load_from_directory(directory: impl AsRef<Path>) -> ImageResult<Self> {
        let directory = directory.as_ref();
        Ok(Self {
            stone: Texture::load(directory.join("stone.png"))?,
            wood: Texture::load(directory.join("wood.png"))?,
            metal: Texture::load(directory.join("metal.png"))?,
            crystal: Texture::load(directory.join("crystal.png"))?,
            ink: Texture::load(directory.join("ink.png"))?,
        })
    }

    pub fn sample(&self, material: Material, uv: Uv) -> Vec3 {
        let texture = match material.kind {
            MaterialKind::Stone => &self.stone,
            MaterialKind::Wood => &self.wood,
            MaterialKind::Metal => &self.metal,
            MaterialKind::Crystal => &self.crystal,
            MaterialKind::Ink => &self.ink,
        };
        texture.sample_bilinear(uv, material.texture_scale)
    }
}

#[cfg(test)]
mod tests {
    use super::Texture;
    use crate::raytracing::Uv;

    #[test]
    fn bilinear_sampling_repeats_uv_coordinates() {
        let texture = Texture {
            width: 2,
            height: 2,
            pixels: vec![[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]],
        };

        let original = texture.sample_bilinear(Uv::new(0.125, 0.375), 1.0);
        let repeated = texture.sample_bilinear(Uv::new(1.125, -0.625), 1.0);
        assert!((original - repeated).length() < 1e-6);
    }
}
