use std::{fs, io, path::Path};

use crate::{geometry::Object, materials::TextureSet, math::Vec3};

use super::{
    Camera, Light, MAX_DEPTH, Ray, any_hit, closest_hit,
    lighting::shade,
    reflection::reflect,
    refraction::{refract, schlick_reflectance},
};

const MIN_RAY_WEIGHT: f32 = 0.05;
const MAX_RENDER_WORKERS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct SkyGradient {
    pub horizon: Vec3,
    pub zenith: Vec3,
    pub star_color: Vec3,
    pub star_intensity: f32,
}

impl SkyGradient {
    pub const fn new(horizon: Vec3, zenith: Vec3) -> Self {
        Self {
            horizon,
            zenith,
            star_color: Vec3::new(0.0, 0.0, 0.0),
            star_intensity: 0.0,
        }
    }

    pub const fn with_stars(mut self, color: Vec3, intensity: f32) -> Self {
        self.star_color = color;
        self.star_intensity = intensity;
        self
    }
}

#[derive(Clone, Copy)]
struct TraceContext {
    sky: SkyGradient,
    depth: u32,
    ray_weight: f32,
}

pub struct Renderer {
    width: usize,
    height: usize,
    background: Vec3,
    worker_count: usize,
    max_depth: u32,
}

impl Renderer {
    pub fn new(width: usize, height: usize, background: Vec3) -> Self {
        let worker_count = std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(MAX_RENDER_WORKERS)
            .min(height);
        Self {
            width,
            height,
            background,
            worker_count,
            max_depth: MAX_DEPTH,
        }
    }

    pub const fn with_max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = if max_depth < MAX_DEPTH {
            max_depth
        } else {
            MAX_DEPTH
        };
        self
    }

    pub fn render(
        &self,
        camera: &Camera,
        objects: &[Box<dyn Object>],
        light: &Light,
        textures: &TextureSet,
    ) -> Vec<Vec3> {
        self.render_with_sky(
            camera,
            objects,
            light,
            textures,
            SkyGradient::new(self.background, Vec3::new(0.035, 0.07, 0.20))
                .with_stars(Vec3::new(0.68, 0.88, 1.0), 1.08),
        )
    }

    pub fn render_with_sky(
        &self,
        camera: &Camera,
        objects: &[Box<dyn Object>],
        light: &Light,
        textures: &TextureSet,
        sky: SkyGradient,
    ) -> Vec<Vec3> {
        let rows_per_worker = self.height.div_ceil(self.worker_count);

        std::thread::scope(|scope| {
            let mut workers = Vec::with_capacity(self.worker_count);
            for first_row in (0..self.height).step_by(rows_per_worker) {
                let last_row = (first_row + rows_per_worker).min(self.height);
                workers.push(scope.spawn(move || {
                    let mut rows = Vec::with_capacity((last_row - first_row) * self.width);
                    for y in first_row..last_row {
                        let v = 1.0 - (y as f32 + 0.5) / self.height as f32;
                        for x in 0..self.width {
                            let u = (x as f32 + 0.5) / self.width as f32;
                            let ray = camera.ray(u, v);
                            let linear_color = self.trace_ray(
                                &ray,
                                objects,
                                light,
                                textures,
                                TraceContext {
                                    sky,
                                    depth: 0,
                                    ray_weight: 1.0,
                                },
                            );
                            rows.push(tone_map(linear_color));
                        }
                    }
                    rows
                }));
            }

            let mut pixels = Vec::with_capacity(self.width * self.height);
            for worker in workers {
                pixels.extend(worker.join().expect("render worker panicked"));
            }
            pixels
        })
    }

    fn trace_ray(
        &self,
        ray: &Ray,
        objects: &[Box<dyn Object>],
        light: &Light,
        textures: &TextureSet,
        context: TraceContext,
    ) -> Vec3 {
        let Some(hit) = closest_hit(ray, objects, 0.001, f32::INFINITY) else {
            let sky_factor = 0.5 * (ray.direction.y + 1.0);
            let gradient =
                context.sky.horizon * (1.0 - sky_factor) + context.sky.zenith * sky_factor;
            let stars = procedural_star(ray.direction) * context.sky.star_intensity;
            return gradient + context.sky.star_color * stars;
        };

        let to_light = light.position - hit.point;
        let light_distance = to_light.length();
        let light_direction = to_light / light_distance;
        let shadow_origin = hit.point + hit.normal * 0.001;
        let shadow_ray = Ray::new(shadow_origin, light_direction);
        let in_shadow = any_hit(&shadow_ray, objects, 0.001, light_distance - 0.001);

        let texture_color = textures.sample(hit.material, hit.uv, hit.object_name);
        let texture_weight = hit.material.texture_weight;
        let surface_albedo =
            texture_color * texture_weight + hit.material.albedo * (1.0 - texture_weight);

        let local_color = shade(
            hit.material,
            surface_albedo,
            hit.normal,
            light_direction,
            -ray.direction,
            light,
            in_shadow,
        );

        if context.depth >= self.max_depth {
            return local_color;
        }

        let transparency = hit.material.transparency;
        let opacity = 1.0 - transparency;
        let mut reflection_weight = opacity * hit.material.reflectivity;
        let local_weight = opacity * (1.0 - hit.material.reflectivity);
        let mut refraction_weight = 0.0;
        let mut refracted_color = Vec3::default();

        if transparency > 0.0 {
            let (first_ior, second_ior) = if hit.front_face {
                (1.0, hit.material.refractive_index)
            } else {
                (hit.material.refractive_index, 1.0)
            };
            let eta_ratio = first_ior / second_ior;
            let cos_theta = (-ray.direction).dot(hit.normal).min(1.0);

            if let Some(refracted_direction) = refract(ray.direction, hit.normal, eta_ratio) {
                let fresnel = schlick_reflectance(cos_theta, first_ior, second_ior);
                reflection_weight += transparency * fresnel;
                refraction_weight = transparency * (1.0 - fresnel);

                let refracted_origin = hit.point - hit.normal * 0.001;
                let refracted_ray = Ray::new(refracted_origin, refracted_direction);
                if context.ray_weight * refraction_weight > MIN_RAY_WEIGHT {
                    refracted_color = self.trace_ray(
                        &refracted_ray,
                        objects,
                        light,
                        textures,
                        TraceContext {
                            depth: context.depth + 1,
                            ray_weight: context.ray_weight * refraction_weight,
                            ..context
                        },
                    );
                }
            } else {
                reflection_weight += transparency;
            }
        }

        let reflected_color = if context.ray_weight * reflection_weight > MIN_RAY_WEIGHT {
            let reflected_direction = reflect(ray.direction, hit.normal).normalized();
            let reflected_origin = hit.point + hit.normal * 0.001;
            let reflected_ray = Ray::new(reflected_origin, reflected_direction);
            self.trace_ray(
                &reflected_ray,
                objects,
                light,
                textures,
                TraceContext {
                    depth: context.depth + 1,
                    ray_weight: context.ray_weight * reflection_weight,
                    ..context
                },
            )
        } else {
            Vec3::default()
        };

        local_color * local_weight
            + reflected_color * reflection_weight
            + refracted_color * refraction_weight
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
                let [red, green, blue] = color_to_rgb8(pixels[y * self.width + x]);
                output.extend_from_slice(&[blue, green, red]);
            }
            output.extend(std::iter::repeat_n(0, row_padding));
        }

        fs::write(path, output)
    }

    pub fn to_u32_buffer(&self, pixels: &[Vec3]) -> Vec<u32> {
        assert_eq!(pixels.len(), self.width * self.height);
        pixels
            .iter()
            .map(|color| {
                let [red, green, blue] = color_to_rgb8(*color);
                u32::from(red) << 16 | u32::from(green) << 8 | u32::from(blue)
            })
            .collect()
    }
}

fn tone_map(color: Vec3) -> Vec3 {
    let color = color.clamp(0.0, f32::MAX);
    Vec3::new(
        color.x / (1.0 + color.x),
        color.y / (1.0 + color.y),
        color.z / (1.0 + color.z),
    )
}

fn procedural_star(direction: Vec3) -> f32 {
    if direction.y < -0.18 {
        return 0.0;
    }

    let cell_x = (direction.x * 260.0).floor() as i32;
    let cell_y = (direction.y * 260.0).floor() as i32;
    let cell_z = (direction.z * 260.0).floor() as i32;
    let hash = (cell_x.wrapping_mul(73_856_093)
        ^ cell_y.wrapping_mul(19_349_663)
        ^ cell_z.wrapping_mul(83_492_791)) as u32;
    let bucket = hash % 1024;
    if bucket >= 1019 {
        0.55 + (bucket - 1019) as f32 * 0.11
    } else {
        0.0
    }
}

fn color_to_rgb8(color: Vec3) -> [u8; 3] {
    let gamma_corrected =
        Vec3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt()).clamp(0.0, 0.999);
    [
        (256.0 * gamma_corrected.x) as u8,
        (256.0 * gamma_corrected.y) as u8,
        (256.0 * gamma_corrected.z) as u8,
    ]
}

#[cfg(test)]
mod tone_mapping_tests {
    use super::{MAX_DEPTH, MAX_RENDER_WORKERS, Renderer, tone_map};
    use crate::math::Vec3;

    #[test]
    fn tone_mapping_preserves_color_order_without_clipping_highlights() {
        let mapped = tone_map(Vec3::new(0.5, 2.0, 8.0));
        assert!(mapped.x < mapped.y && mapped.y < mapped.z);
        assert!(mapped.x >= 0.0 && mapped.z < 1.0);
    }

    #[test]
    fn renderer_uses_a_bounded_worker_pool() {
        let renderer = Renderer::new(480, 270, Vec3::default());
        assert!((1..=MAX_RENDER_WORKERS).contains(&renderer.worker_count));
        assert_eq!(MAX_RENDER_WORKERS, 8);
        assert_eq!(renderer.max_depth, MAX_DEPTH);
        assert_eq!(
            Renderer::new(480, 270, Vec3::default())
                .with_max_depth(1)
                .max_depth,
            1
        );
        assert_eq!(
            Renderer::new(480, 270, Vec3::default())
                .with_max_depth(0)
                .max_depth,
            0
        );
        assert_eq!(
            Renderer::new(480, 270, Vec3::default())
                .with_max_depth(MAX_DEPTH + 1)
                .max_depth,
            MAX_DEPTH
        );
    }
}
