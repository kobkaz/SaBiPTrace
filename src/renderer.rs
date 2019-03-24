use crate::accum::*;
use crate::camera::Camera;
use crate::image::*;
use crate::manager::*;
use crate::scene::Scene;
use crate::*;

use log::*;
use rand::prelude::*;
use std::sync::{Arc, Mutex};

pub mod bdpt;
pub mod lt;
pub mod pt;

#[derive(Clone, Copy, Debug)]
pub enum Integrator {
    PathTrace,
    PathTraceWithNee,
    BidirectionalPathTrace,
    LightTrace,
}

#[derive(Clone)]
pub struct FilmConfig<T> {
    pub film_arc: FilmArc<T>,
    pub accum_init: T,
}

#[derive(Clone, Copy)]
pub struct RenderConfig {
    pub integrator: Integrator,
    pub nthread: usize,
}

pub struct Renderer;

impl Renderer {
    pub fn render<T: Send + Clone + Accumulator + 'static, C: Send + Clone + Camera + 'static>(
        &self,
        scene: Arc<Scene>,
        camera: &C,
        film_config: FilmConfig<T>,
        config: RenderConfig,
        on_cycle_complete: Box<dyn FnMut(usize, usize) -> Option<usize> + Send>,
    ) {
        use std::thread;
        let mut threads = vec![];
        let film = film_config.film_arc;
        let manager = Manager::new(film.w() as usize, config.nthread, on_cycle_complete);
        let manager = Arc::new(Mutex::new(manager));
        for i in 0..config.nthread {
            let film = film.clone();
            let camera = camera.clone();
            let scene = scene.clone();
            let manager = manager.clone();
            let accum_init = film_config.accum_init.clone();
            let thread = thread::spawn(move || {
                Self::render_thread(
                    &scene,
                    camera,
                    film,
                    accum_init,
                    config.integrator,
                    i,
                    manager,
                )
            });
            threads.push(thread);
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn render_thread<T: Clone + Accumulator>(
        scene: &Scene,
        camera: impl Camera,
        film: FilmArc<T>,
        accum_init: T,
        integrator: Integrator,
        thread_id: usize,
        manager: Arc<Mutex<Manager>>,
    ) {
        let mut rng = SmallRng::from_entropy();

        //for LightTrace
        let mut local_film = FilmVec::new(film.w(), film.h(), accum_init.clone());
        let mut total_sample = 0;

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

            match integrator {
                Integrator::PathTrace => Self::pixl_wise_render(
                    task,
                    scene,
                    &camera,
                    &film,
                    &accum_init,
                    &mut rng,
                    &mut |scene, ray, radiance, rng| pt::radiance(false, scene, ray, radiance, rng),
                ),
                Integrator::PathTraceWithNee => Self::pixl_wise_render(
                    task,
                    scene,
                    &camera,
                    &film,
                    &accum_init,
                    &mut rng,
                    &mut |scene, ray, radiance, rng| pt::radiance(true, scene, ray, radiance, rng),
                ),
                Integrator::BidirectionalPathTrace => Self::pixl_wise_render(
                    task,
                    scene,
                    &camera,
                    &film,
                    &accum_init,
                    &mut rng,
                    &mut bdpt::radiance,
                ),
                Integrator::LightTrace => {
                    let _xi = task.chunk;
                    let spp = task.amount;
                    for _i in 0..film.h() * spp {
                        lt::sample(scene, &camera, &mut local_film, &accum_init, &mut rng);
                    }
                    total_sample += film.h() * spp;
                }
            }
        }
        film.with_lock(|mut film| {
            for xi in 0..film.w() {
                for yi in 0..film.h() {
                    let pixel = film.at_mut(xi, yi);
                    let local = &mut local_film.at_mut(xi, yi).accum;
                    pixel.accum.merge(local);
                    pixel.samples += total_sample
                }
            }
        })
        .unwrap();
    }

    fn pixl_wise_render<T, R, F>(
        task: Task,
        scene: &Scene,
        camera: &impl Camera,
        film: &FilmArc<T>,
        accum_init: &T,
        rng: &mut R,
        f: &mut F,
    ) where
        T: Clone + Accumulator,
        R: Rng + ?Sized,
        F: FnMut(&Scene, &Ray, &mut T, &mut R),
    {
        let xi = task.chunk;
        let spp = task.amount;
        let mut samples = vec![];
        for yi in 0..film.h() {
            for _i in 0..spp {
                let mut radiance = accum_init.clone();
                let (u, v) = film.sample_uv_in_pixel(xi as i32, yi as i32, rng);
                let ray = camera.sample_ray(u, v, rng);
                f(scene, &ray, &mut radiance, rng);
                if radiance.is_finite() {
                    samples.push((xi, yi, radiance));
                } else {
                    warn!("radiance is not finite");
                }
            }
        }

        film.with_lock(|mut film| {
            for (xi, yi, sample) in samples {
                if xi < film.w() && yi < film.h() {
                    let pixel = film.at_mut(xi, yi);
                    pixel.accum.merge(&sample);
                    pixel.samples += 1;
                }
            }
        })
        .unwrap();
    }
}
