use crate::camera::Camera;
use crate::image::*;
use crate::manager::*;
use crate::scene::Scene;
use crate::*;

use log::*;
use rand::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
pub enum Integrator {
    PathTrace,
    PathrTraceWithNee,
    BidirectionalPathTrace,
}

pub trait Accumulator<T> {
    fn accum(&mut self, color: T);
    fn merge(&mut self, another: &Self);
    fn is_finite(&self) -> bool;
}

impl Accumulator<(RGB, usize)> for RGB {
    fn accum(&mut self, color: (RGB, usize)) {
        *self += color.0
    }
    fn merge(&mut self, another: &Self) {
        *self += *another
    }
    fn is_finite(&self) -> bool {
        self.is_finite()
    }
}

impl Accumulator<(RGB, usize)> for Vec<RGB> {
    fn accum(&mut self, (color, len): (RGB, usize)) {
        if len < self.len() {
            self[len] += color;
        }
    }

    fn merge(&mut self, another: &Self) {
        let l = self.len().min(another.len());
        for i in 0..l {
            self[i] += another[i]
        }
    }

    fn is_finite(&self) -> bool {
        self.iter().all(RGB::is_finite)
    }
}

#[derive(Clone)]
pub struct FilmConfig<T> {
    pub film_arc: Arc<Mutex<Film<Pixel<T>>>>,
    pub accum_init: T,
}

#[derive(Clone, Copy)]
pub struct RenderConfig {
    pub integrator: Integrator,
    pub nthread: usize,
    pub spp: usize,
    pub cycle_spp: usize,
}

pub struct Renderer;

impl Renderer {
    pub fn render<T: Send + Clone + Accumulator<(RGB, usize)> + 'static>(
        &self,
        scene: Arc<Scene>,
        camera: &Camera,
        film_config: FilmConfig<T>,
        config: RenderConfig,
        on_cycle_complete: Box<dyn FnMut(usize, usize) + Send>,
    ) {
        use std::thread;
        let mut threads = vec![];
        let film = film_config.film_arc;
        let manager = Manager::new(
            film.lock().unwrap().w() as usize,
            config.spp,
            config.cycle_spp,
            config.nthread,
            on_cycle_complete,
        );
        let manager = Arc::new(Mutex::new(manager));
        for i in 0..config.nthread {
            let film = film.clone();
            let camera = camera.clone();
            let scene = scene.clone();
            let manager = manager.clone();
            let accum_init = film_config.accum_init.clone();
            let thread = thread::spawn(move || {
                Self::render_thread(&scene, camera, film, accum_init, config.integrator, i, manager)
            });
            threads.push(thread);
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn render_thread<T: Clone + Accumulator<(RGB, usize)>>(
        scene: &Scene,
        camera: Camera,
        film: Arc<Mutex<Film<Pixel<T>>>>,
        accum_init: T,
        integrator: Integrator,
        thread_id: usize,
        manager: Arc<Mutex<Manager>>,
    ) {
        use rand::distributions::Uniform;
        let mut rng = SmallRng::from_entropy();
        let (film_w, film_h) = {
            let film = film.lock().unwrap();
            (film.w(), film.h())
        };
        let px_size = camera.width() / film_w as f32;

        loop {
            let rx = manager.lock().unwrap().next(thread_id);
            let task = match rx.recv() {
                Ok(Some(task)) => task,
                Ok(None) => {
                    break;
                }
                Err(_) => {
                    continue;
                }
            };

            let xi = task.chunk as u32;
            let spp = task.amount;
            let mut col = vec![accum_init.clone(); film_h as usize];
            for yi in 0..film_h {
                let mut accum = accum_init.clone();
                for _i in 0..spp {
                    let du = {
                        let x = xi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dx = x - film_w as f32 / 2.0;
                        dx * px_size
                    };
                    let dv = {
                        let y = yi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dy = film_h as f32 / 2.0 - y;
                        dy * px_size
                    };
                    let ray = camera.ray_to(du, dv);
                    match integrator {
                        Integrator::PathTrace => 
                            Self::radiance_pt(false, scene, &ray, &mut accum, &mut rng),
                        Integrator::PathrTraceWithNee => 
                            Self::radiance_pt(true, scene, &ray, &mut accum, &mut rng),
                        Integrator::BidirectionalPathTrace => 
                            Self::radiance_bdpt(scene, &ray, &mut accum, &mut rng),
                    };

                }
                if accum.is_finite() {
                    col[yi as usize] = accum;
                } else {
                    warn!("radiance is not finite");
                }
            }

            let mut film = film.lock().unwrap();
            for yi in 0..film_h {
                let pixel = film.at_mut(xi, yi);
                pixel.accum.merge(&col[yi as usize]);
                pixel.samples += spp;
            }
        }
    }

    fn radiance_pt<R: ?Sized>(enable_nee: bool, scene: &Scene, ray: &Ray, 
                              radiance_accum: &mut impl Accumulator<(RGB, usize)>, rng: &mut R)
    where
        R: Rng,
    {
        let mut ray = ray.clone();
        let mut throughput = RGB::all(1.0);
        let mut prev_specular = true;

        const DEPTH_MAX: usize = 100;
        for depth in 0..DEPTH_MAX {
            let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX / 2.0);
            if let Some(hit) = hit {
                let hit_lc = hit.geom.lc();
                let wout_local = hit_lc.w2l() * -ray.dir;

                if prev_specular || !enable_nee {
                    if let Some(emission) = hit.emission {
                        radiance_accum.accum((throughput * emission, depth));
                    }
                }

                if enable_nee {
                    if let Some(light_sample) = scene.sample_light(rng) {
                        let (light_point, light_normal, light_emission) = light_sample.value;
                        if scene.visible(&light_point, &hit.geom.pos) {
                            let g = hit.geom.g(&light_point, &light_normal);
                            let light_dir = (light_point - hit.geom.pos).normalize();
                            let bsdf = hit.material.bsdf(&(hit_lc.w2l() * light_dir), &wout_local);
                            let nee_contrib =
                                throughput * light_emission * bsdf * g / light_sample.pdf;
                            if !nee_contrib.is_finite() {
                                warn!("nee_radiance is not finite {:?}", nee_contrib);
                                warn!("> throughput {:?}", throughput);
                                warn!("> light_emission {:?}", light_emission);
                                warn!("> bsdf {:?}", bsdf);
                                warn!("> g {:?}", g);
                                warn!("> light_sample.pdf {:?}", light_sample.pdf);
                            } else {
                                radiance_accum.accum((nee_contrib, depth + 1));
                            }
                        }
                    }
                }

                let next = hit.material.sample_win_cos(&wout_local, rng);
                let win_local = next.value.0;
                let bsdf_cos = next.value.1;
                prev_specular = next.value.2;
                throughput *= bsdf_cos;
                throughput /= next.pdf;

                let cont = pdf::RandomBool {
                    chance: (throughput.max() * 0.8).min(1.0).max(0.1),
                };

                let cont = cont.sample(rng);
                if !cont.value {
                    break;
                }
                throughput /= cont.pdf;

                if !throughput.is_finite() {
                    warn!("throughput is not finite {:?}", throughput);
                    warn!("> wout_local {:?}", wout_local);
                    warn!("> hit.geom {:?}", hit.geom);
                    warn!("> hit.material {:?}", hit.material);
                    warn!("> next {:?}", next);
                    warn!("> bsdf_cos {:?}", bsdf_cos);
                    warn!("> next.pdf {:?}", next.pdf);
                    warn!("> cont.pdf {:?}", cont.pdf);
                    break;
                }

                ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
            } else {
                radiance_accum.accum((scene.envmap_dir(&ray.dir) * throughput, depth));
                break;
            }
        }
    }

    fn bdpt_gen_eye<R: ?Sized>(
        scene: &Scene,
        ray: &Ray,
        max_depth: usize,
        rng: &mut R,
    ) -> Vec<(object::ObjectHit, RGB, V3)>
    where
        R: Rng,
    {
        let mut vs = vec![];
        let mut throughput = RGB::all(1.0);
        let mut ray = ray.clone();
        for _depth in 0..max_depth {
            let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX / 2.0);
            if hit.is_none() {
                break;
            }
            let hit = hit.unwrap();
            let hit_lc = hit.geom.lc();
            let wout_local = hit_lc.w2l() * -ray.dir;
            vs.push((hit.clone(), throughput, wout_local));

            let next = hit.material.sample_win_cos(&wout_local, rng);
            let win_local = next.value.0;
            let bsdf_cos = next.value.1;
            throughput *= bsdf_cos / next.pdf;

            let cont = pdf::RandomBool {
                chance: (throughput.max() * 0.8).min(1.0).max(0.1),
            };

            let cont = cont.sample(rng);
            if !cont.value {
                break;
            }
            throughput /= cont.pdf;
            ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
        }
        vs
    }

    fn bdpt_gen_light<R: ?Sized>(
        scene: &Scene,
        light_sample: &pdf::PdfSample<(P3, V3, RGB)>,
        max_depth: usize,
        rng: &mut R,
    ) -> Vec<(object::ObjectHit, RGB, V3)>
    where
        R: Rng,
    {
        use rand::distributions::Uniform;
        let mut vs = vec![];
        let (light_point, light_normal, light_emission) = light_sample.value;
        let mut throughput = light_emission / light_sample.pdf;
        let initial_outdir = pdf::CosUnitHemisphere::from_normal(&light_normal)
            .sample(rng)
            .and_then(|v| {
                pdf::RandomBool { chance: 0.5 }
                    .sample(rng)
                    .map(|b| if b { -v } else { v })
            });
        throughput /= initial_outdir.pdf;

        let mut ray = Ray::new(light_point, initial_outdir.value);
        for _depth in 0..max_depth {
            let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX / 2.0);
            if hit.is_none() {
                break;
            }
            let hit = hit.unwrap();
            let hit_lc = hit.geom.lc();
            let win_local = hit_lc.w2l() * -ray.dir;

            vs.push((hit.clone(), throughput, win_local));
            let next = hit.material.sample_win_cos(&win_local, rng);
            let wout_local = next.value.0;
            let bsdf_cos = next.value.1;
            throughput *= bsdf_cos / next.pdf;

            let cont = pdf::RandomBool {
                chance: (throughput.max() * 0.8).min(1.0).max(0.1),
            };

            let cont = cont.sample(rng);
            if !cont.value {
                break;
            }
            throughput /= cont.pdf;
            ray = hit_lc.l2w() * Ray::new(P3::origin(), wout_local);
        }
        vs
    }

    fn radiance_bdpt<R: ?Sized>(
        scene: &Scene, ray: &Ray, radiance_accum: &mut impl Accumulator<(RGB, usize)>, rng: &mut R)
    where
        R: Rng,
    {
        const LE_MAX: usize = 10;
        const LL_MAX: usize = 10;
        let eye_vs = Self::bdpt_gen_eye(scene, ray, LE_MAX, rng);
        let len_e = eye_vs.len();

        let light_sample = scene.sample_light(rng);
        if light_sample.is_none() {
            return;
        }
        let light_sample = light_sample.unwrap();

        let light_vs = Self::bdpt_gen_light(scene, &light_sample, LL_MAX, rng);
        let len_l = light_vs.len();

        for len in 2..=len_e + len_l + 4 {
            let t_min = len - len.min(LE_MAX + 2);
            let t_max = (len - 2).min(LL_MAX + 2);
            assert!(t_min <= t_max);
            let mut accum_len = RGB::all(0.0);
            let mut weight_sum = 0.0;
            for t in t_min..=t_max {
                let weight = 1.0;
                weight_sum += weight;

                let s = len - t;
                assert!(s >= 2);
                let e_i = s - 2;
                if e_i >= len_e {
                    continue;
                }
                let v_eye = &eye_vs[e_i];

                let contrib = if t == 0 {
                    if let Some(emission) = v_eye.0.emission {
                        emission * v_eye.1
                    } else {
                        RGB::all(0.0)
                    }
                } else if t == 1 {
                    let (hit, throughput, wout_local) = v_eye;
                    let hit_lc = hit.geom.lc();
                    let (light_point, light_normal, light_emission) = light_sample.value;
                    if !scene.visible(&light_point, &hit.geom.pos) {
                        RGB::all(0.0)
                    } else {
                        let g = hit.geom.g(&light_point, &light_normal);
                        let light_dir = (light_point - hit.geom.pos).normalize();
                        let bsdf = hit.material.bsdf(&(hit_lc.w2l() * light_dir), &wout_local);
                        light_emission * throughput * bsdf * g / light_sample.pdf
                    }
                } else {
                    let l_i = t - 2;
                    if l_i >= len_l {
                        continue;
                    }
                    let v_light = &light_vs[l_i];
                    let (e_hit, e_throughput, e_wout_local) = v_eye;
                    let (l_hit, l_throughput, l_win_local) = v_light;
                    if !scene.visible(&e_hit.geom.pos, &l_hit.geom.pos) {
                        RGB::all(0.0)
                    } else {
                        let e_to_l = (l_hit.geom.pos - e_hit.geom.pos).normalize();
                        let e_win_local = e_hit.geom.lc().w2l() * e_to_l;
                        let l_wout_local = l_hit.geom.lc().w2l() * -e_to_l;
                        let g = e_hit.geom.g(&l_hit.geom.pos, &l_hit.geom.gnorm);
                        let l_bsdf = l_hit.material.bsdf(&l_win_local, &l_wout_local);
                        let e_bsdf = e_hit.material.bsdf(&e_win_local, &e_wout_local);
                        *l_throughput * l_bsdf * g * e_bsdf * e_throughput
                    }
                };

                accum_len += contrib * weight;
            }

            radiance_accum.accum((accum_len / weight_sum, len - 2));
        }
    }
}
