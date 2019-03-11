use crate::camera::Camera;
use crate::scene::Scene;
use crate::*;

use rand::prelude::*;

pub struct Renderer;
impl Renderer {
    pub fn render(&self, scene: &Scene, camera: &Camera, image: &mut image::Image) {
        use rand::distributions::Uniform;
        let mut rng = SmallRng::from_entropy();
        let px_size = camera.width() / image.w() as f32;
        for xi in 0..image.w() {
            for yi in 0..image.h() {
                let mut accum = RGB::all(0.0);
                let n = 5;
                for _i in 0..n {
                    let du = {
                        let x = xi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dx = x - image.w() as f32 / 2.0;
                        dx * px_size
                    };
                    let dv = {
                        let y = yi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dy = image.h() as f32 / 2.0 - y;
                        dy * px_size
                    };
                    let ray = camera.ray_to(du, dv);
                    const USE_NEE: bool = false;
                    accum = accum + self.radiance(USE_NEE, scene, &ray, &mut rng);
                }
                *image.at_mut(xi, yi) = accum / n as f32;
            }
        }
    }

    fn radiance<R: Rng + Sized>(
        &self,
        enable_nee: bool,
        scene: &Scene,
        ray: &Ray,
        rng: &mut R,
    ) -> RGB {
        let mut depth = 0;
        let mut ray = ray.clone();
        let mut radiance = RGB::all(0.0);
        let mut throughput = RGB::all(1.0);
        let mut prev_specular = true;

        loop {
            depth += 1;
            let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX);
            if let Some(hit) = hit {
                let hit_gnorm = hit.geom.gnorm;
                let hit_xvec = hit.geom.gx;
                let wout = -ray.dir;

                if prev_specular || !enable_nee {
                    if let Some(emission) = hit.emission {
                        radiance += throughput * emission;
                    }
                }

                if enable_nee {
                    prev_specular = false;
                    if let Some(light_sample) = scene.sample_light(rng) {
                        let (light_point, light_normal, light_emission) = light_sample.value;
                        if scene.visible(light_point, hit.geom.pos) {
                            let g = hit.geom.g(&light_point, &light_normal);
                            let light_dir = (light_point - hit.geom.pos).normalize();
                            let bsdf = hit.material.bsdf(&hit_gnorm, &light_dir, &wout);
                            radiance += throughput * light_emission * bsdf * g / light_sample.pdf;
                        }
                    }
                }

                let cont = pdf::RandomBool {
                    chance: throughput.max(),
                };
                let cont = cont.sample(rng);
                if !cont.value {
                    break;
                }
                throughput /= cont.pdf;

                let next = hit.material.sample_win(hit_gnorm, hit_xvec, wout, rng);
                let win = next.value.0;
                let bsdf = next.value.1;
                throughput *= bsdf * hit_gnorm.dot(&win).abs();
                throughput /= next.pdf;
                ray = Ray::new(hit.geom.pos, win);
            } else {
                break;
            }
        }
        return radiance;
    }
}
