use std::{fs, io, path::Path};

use rayon::prelude::*;

use crate::{geometry::Object, materials::TextureSet, math::Vec3};

use super::{
    Camera, HitRecord, Light, MAX_DEPTH, Ray, any_hit, closest_hit,
    lighting::{shade, shade_light_only},
    reflection::reflect,
    refraction::{refract, schlick_reflectance},
};

const MIN_RAY_WEIGHT: f32 = 0.05;

// Antialiasing adaptativo: solo los pÃ­xeles cuyo color difiere del de algÃºn
// vecino mÃ¡s que este umbral (bordes de geometrÃ­a, brillos especulares) se
// vuelven a muestrear con 4 rayos extra. El resto de la imagen no paga nada,
// asÃ­ que el costo total es una fracciÃ³n del de supermuestrear todo.
const EDGE_THRESHOLD: f32 = 0.12;
const EDGE_SAMPLE_OFFSETS: [(f32, f32); 4] =
    [(0.125, 0.375), (0.375, -0.125), (-0.125, -0.375), (-0.375, 0.125)];

// La luz de acento se atenÃºa con la distancia para que se sienta local (un
// resplandor cerca del Memory Core) en vez de una segunda luz global.
const ACCENT_ATTENUATION: f32 = 0.10;

#[derive(Clone, Copy, Debug)]
pub struct SkyGradient {
    pub horizon: Vec3,
    pub zenith: Vec3,
    pub star_color: Vec3,
    pub star_intensity: f32,
    /// CuÃ¡nto del panorama con textura del skybox se muestra (1.0) frente al
    /// degradado plano de `horizon`/`zenith` (0.0). Permite pasar de un cielo
    /// texturizado (estado normal) a un degradado totalmente plano (por
    /// ejemplo, el cielo corrupto de Melanta) sin necesitar una segunda
    /// textura ni tocar el resto del pipeline de trazado.
    pub texture_weight: f32,
    /// Color al que se tiÃ±e el panorama cuando `panorama_tint_amount > 0`. El
    /// panorama se recolorea a partir de su luminancia, de modo que conserva
    /// las nubes y estrellas pero cambia de paleta (cielo corrupto de Melanta).
    pub panorama_tint: Vec3,
    pub panorama_tint_amount: f32,
}

impl SkyGradient {
    pub const fn new(horizon: Vec3, zenith: Vec3) -> Self {
        Self {
            horizon,
            zenith,
            star_color: Vec3::new(0.0, 0.0, 0.0),
            star_intensity: 0.0,
            texture_weight: 1.0,
            panorama_tint: Vec3::new(1.0, 1.0, 1.0),
            panorama_tint_amount: 0.0,
        }
    }

    pub const fn with_stars(mut self, color: Vec3, intensity: f32) -> Self {
        self.star_color = color;
        self.star_intensity = intensity;
        self
    }

    pub const fn with_texture_weight(mut self, texture_weight: f32) -> Self {
        self.texture_weight = texture_weight;
        self
    }

    pub const fn with_panorama_tint(mut self, tint: Vec3, amount: f32) -> Self {
        self.panorama_tint = tint;
        self.panorama_tint_amount = amount;
        self
    }
}

#[derive(Clone, Copy)]
struct TraceContext {
    sky: SkyGradient,
    accent: Option<Light>,
    depth: u32,
    ray_weight: f32,
}

#[derive(Clone, Copy)]
struct FrameInputs<'a> {
    camera: &'a Camera,
    objects: &'a [Box<dyn Object>],
    light: &'a Light,
    accent: Option<Light>,
    textures: &'a TextureSet,
    sky: SkyGradient,
}

pub struct Renderer {
    width: usize,
    height: usize,
    background: Vec3,
    max_depth: u32,
    edge_antialiasing: bool,
}

impl Renderer {
    pub const fn new(width: usize, height: usize, background: Vec3) -> Self {
        Self {
            width,
            height,
            background,
            max_depth: MAX_DEPTH,
            edge_antialiasing: false,
        }
    }

    /// Activa el antialiasing adaptativo de bordes (ver `EDGE_THRESHOLD`).
    /// Se pensÃ³ para el render de reposo; la previsualizaciÃ³n lo deja apagado.
    pub const fn with_edge_antialiasing(mut self, enabled: bool) -> Self {
        self.edge_antialiasing = enabled;
        self
    }

    pub const fn with_max_depth(mut self, max_depth: u32) -> Self {
        self.max_depth = if max_depth < MAX_DEPTH {
            max_depth
        } else {
            MAX_DEPTH
        };
        self
    }

    pub fn render_with_accent(
        &self,
        camera: &Camera,
        objects: &[Box<dyn Object>],
        light: &Light,
        accent: Option<Light>,
        textures: &TextureSet,
    ) -> Vec<Vec3> {
        self.render_with_lights(
            camera,
            objects,
            light,
            accent,
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
        self.render_with_lights(camera, objects, light, None, textures, sky)
    }

    pub fn render_with_lights(
        &self,
        camera: &Camera,
        objects: &[Box<dyn Object>],
        light: &Light,
        accent: Option<Light>,
        textures: &TextureSet,
        sky: SkyGradient,
    ) -> Vec<Vec3> {
        let frame = FrameInputs {
            camera,
            objects,
            light,
            accent,
            textures,
            sky,
        };
        let width = self.width;
        let base = self.parallel_rows(|y| {
            (0..width)
                .map(|x| self.sample_pixel(&frame, x as f32 + 0.5, y as f32 + 0.5))
                .collect()
        });
        if !self.edge_antialiasing {
            return base;
        }

        let base_ref = &base;
        self.parallel_rows(|y| {
            (0..width)
                .map(|x| self.refine_edge_pixel(&frame, base_ref, x, y))
                .collect()
        })
    }

    /// Reparte las filas entre el pool de hilos de `rayon` y concatena el
    /// resultado en orden. `per_row` calcula una fila completa de pÃ­xeles.
    ///
    /// Antes esto usaba `std::thread::scope`, creando y uniendo hilos nuevos
    /// del sistema operativo en cada llamada â€” un costo fijo que se pagaba en
    /// cada fotograma, incluso para la previsualizaciÃ³n barata que se recalcula
    /// en cada cuadro mientras la cÃ¡mara se mueve. El pool global de `rayon` se
    /// crea una sola vez (perezosamente, con tantos hilos como nÃºcleos lÃ³gicos
    /// detecte el sistema) y se reutiliza en cada llamada: cada fila es una
    /// tarea que un hilo libre roba de la cola de otro (`work stealing`), lo
    /// que ademÃ¡s balancea mejor la carga que repartir filas en bloques fijos
    /// cuando el costo por fila varÃ­a mucho (reflejos, refracciones, sombras).
    fn parallel_rows<F>(&self, per_row: F) -> Vec<Vec3>
    where
        F: Fn(usize) -> Vec<Vec3> + Sync + Send,
    {
        (0..self.height).into_par_iter().flat_map(per_row).collect()
    }

    /// Color final (ya con tone mapping) de un punto de la pantalla dado en
    /// coordenadas de pÃ­xel, donde `(x + 0.5, y + 0.5)` es el centro del pÃ­xel.
    fn sample_pixel(&self, frame: &FrameInputs<'_>, x: f32, y: f32) -> Vec3 {
        let u = x / self.width as f32;
        let v = 1.0 - y / self.height as f32;
        let ray = frame.camera.ray(u, v);
        let linear_color = self.trace_ray(
            &ray,
            frame.objects,
            frame.light,
            frame.textures,
            TraceContext {
                sky: frame.sky,
                accent: frame.accent,
                depth: 0,
                ray_weight: 1.0,
            },
        );
        tone_map(linear_color)
    }

    fn refine_edge_pixel(
        &self,
        frame: &FrameInputs<'_>,
        base: &[Vec3],
        x: usize,
        y: usize,
    ) -> Vec3 {
        let index = y * self.width + x;
        let center = base[index];
        let neighbors = [
            (x > 0).then(|| index - 1),
            (x + 1 < self.width).then(|| index + 1),
            (y > 0).then(|| index - self.width),
            (y + 1 < self.height).then(|| index + self.width),
        ];
        let is_edge = neighbors
            .into_iter()
            .flatten()
            .any(|neighbor| color_distance(center, base[neighbor]) > EDGE_THRESHOLD);
        if !is_edge {
            return center;
        }

        let mut sum = center;
        for (dx, dy) in EDGE_SAMPLE_OFFSETS {
            sum = sum + self.sample_pixel(frame, x as f32 + 0.5 + dx, y as f32 + 0.5 + dy);
        }
        sum / 5.0
    }

    /// Aporte de la luz de acento en un punto de impacto: difuso + especular
    /// con atenuaciÃ³n por distancia. La sombra solo se comprueba cuando el
    /// render admite rayos secundarios (`max_depth > 0`); la previsualizaciÃ³n
    /// se salta ese rayo para mantener la cÃ¡mara fluida.
    fn accent_contribution(
        &self,
        accent: &Light,
        ray: &Ray,
        hit: &HitRecord,
        surface_albedo: Vec3,
        objects: &[Box<dyn Object>],
    ) -> Vec3 {
        let to_light = accent.position - hit.point;
        let distance = to_light.length();
        if distance < 1e-4 {
            return Vec3::default();
        }
        let direction = to_light / distance;
        if hit.normal.dot(direction) <= 0.0 {
            return Vec3::default();
        }
        if self.max_depth > 0 {
            let shadow_ray = Ray::new(hit.point + hit.normal * 0.001, direction);
            if any_hit(&shadow_ray, objects, 0.001, distance - 0.001) {
                return Vec3::default();
            }
        }

        let attenuated = Light::new(
            accent.position,
            accent.color,
            accent.intensity / (1.0 + ACCENT_ATTENUATION * distance * distance),
        );
        shade_light_only(
            hit.material,
            surface_albedo,
            hit.normal,
            direction,
            -ray.direction,
            &attenuated,
        )
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
            let flat_gradient =
                context.sky.horizon * (1.0 - sky_factor) + context.sky.zenith * sky_factor;
            let sky_color = if context.sky.texture_weight > 0.0 {
                let mut panorama = textures.sample_skybox(ray.direction);
                if context.sky.panorama_tint_amount > 0.0 {
                    let amount = context.sky.panorama_tint_amount;
                    let luminance = panorama.dot(Vec3::new(0.299, 0.587, 0.114));
                    panorama = panorama * (1.0 - amount)
                        + context.sky.panorama_tint * luminance * amount;
                }
                panorama * context.sky.texture_weight
                    + flat_gradient * (1.0 - context.sky.texture_weight)
            } else {
                flat_gradient
            };
            let stars = procedural_star(ray.direction) * context.sky.star_intensity;
            return sky_color + context.sky.star_color * stars;
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

        let local_color = match context.accent {
            Some(accent) => {
                local_color
                    + self.accent_contribution(&accent, ray, &hit, surface_albedo, objects)
            }
            None => local_color,
        };

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

fn color_distance(left: Vec3, right: Vec3) -> f32 {
    let difference = left - right;
    difference
        .x
        .abs()
        .max(difference.y.abs())
        .max(difference.z.abs())
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
