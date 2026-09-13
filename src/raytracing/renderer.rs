use std::{fs, io, path::Path};

use crate::{geometry::Object, math::Vec3};

use super::{Camera, Light, Ray, closest_hit, lighting::shade};

pub struct Renderer {
    width: usize,
    height: usize,
    background: Vec3,
}

impl Renderer {
    pub const fn new(width: usize, height: usize, background: Vec3) -> Self {
        Self {
            width,
            height,
            background,
        }
    }

    pub fn render(&self, camera: &Camera, objects: &[Box<dyn Object>], light: &Light) -> Vec<Vec3> {
        let mut pixels = Vec::with_capacity(self.width * self.height);

        for y in 0..self.height {
            let v = 1.0 - (y as f32 + 0.5) / self.height as f32;
            for x in 0..self.width {
                let u = (x as f32 + 0.5) / self.width as f32;
                let ray = camera.ray(u, v);
                pixels.push(self.trace_primary(&ray, objects, light));
            }
        }

        pixels
    }

    fn trace_primary(&self, ray: &Ray, objects: &[Box<dyn Object>], light: &Light) -> Vec3 {
        let Some(hit) = closest_hit(ray, objects, 0.001, f32::INFINITY) else {
            let sky_factor = 0.5 * (ray.direction.y + 1.0);
            return self.background * (1.0 - sky_factor) + Vec3::new(0.30, 0.39, 0.58) * sky_factor;
        };

        let to_light = light.position - hit.point;
        let light_distance = to_light.length();
        let light_direction = to_light / light_distance;
        let shadow_origin = hit.point + hit.normal * 0.001;
        let shadow_ray = Ray::new(shadow_origin, light_direction);
        let in_shadow = closest_hit(&shadow_ray, objects, 0.001, light_distance - 0.001).is_some();

        shade(
            hit.material,
            hit.normal,
            light_direction,
            -ray.direction,
            light,
            in_shadow,
        )
    }

    pub fn write_bmp(&self, path: impl AsRef<Path>, pixels: &[Vec3]) -> io::Result<()> {
        assert_eq!(pixels.len(), self.width * self.height);
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let row_bytes = self.width * 3;
        let row_padding = (4 - row_bytes % 4) % 4;
        let pixel_data_size = (row_bytes + row_padding) * self.height;
        let file_size = 14 + 40 + pixel_data_size;
        let mut output = Vec::with_capacity(file_size);

        output.extend_from_slice(b"BM");
        output.extend_from_slice(&(file_size as u32).to_le_bytes());
        output.extend_from_slice(&[0; 4]);
        output.extend_from_slice(&54_u32.to_le_bytes());
        output.extend_from_slice(&40_u32.to_le_bytes());
        output.extend_from_slice(&(self.width as i32).to_le_bytes());
        output.extend_from_slice(&(self.height as i32).to_le_bytes());
        output.extend_from_slice(&1_u16.to_le_bytes());
        output.extend_from_slice(&24_u16.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        output.extend_from_slice(&2835_i32.to_le_bytes());
        output.extend_from_slice(&2835_i32.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());
        output.extend_from_slice(&0_u32.to_le_bytes());

        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let color = pixels[y * self.width + x];
                let gamma_corrected =
                    Vec3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt()).clamp(0.0, 0.999);
                let red = (256.0 * gamma_corrected.x) as u8;
                let green = (256.0 * gamma_corrected.y) as u8;
                let blue = (256.0 * gamma_corrected.z) as u8;
                output.extend_from_slice(&[blue, green, red]);
            }
            output.extend(std::iter::repeat_n(0, row_padding));
        }

        fs::write(path, output)
    }
}
