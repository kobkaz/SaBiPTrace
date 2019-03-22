use crate::camera::Camera;
use crate::image::*;
use crate::manager::*;
use crate::scene::Scene;
use crate::*;

use log::*;
use rand::prelude::*;
use std::sync::{Arc, Mutex};

pub mod bdpt;
pub mod pt;

#[derive(Clone, Copy, Debug)]
pub enum Integrator {
    PathTrace,
    PathTraceWithNee,
    BidirectionalPathTrace,
}

pub trait Accumulator<T> {
    fn accum(&mut self, color: &T);
    fn merge(&mut self, another: &Self);
    fn is_finite(&self) -> bool;
}

impl Accumulator<(RGB, usize)> for RGB {
    fn accum(&mut self, color: &(RGB, usize)) {
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
    fn accum(&mut self, (color, len): &(RGB, usize)) {
        if *len < self.len() {
            self[*len] += *color;
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

impl<T, U, V> Accumulator<T> for (U, V)
where
    U: Accumulator<T>,
    V: Accumulator<T>,
{
    fn accum(&mut self, color: &T) {
        self.0.accum(color);
        self.1.accum(color);
    }

    fn merge(&mut self, another: &Self) {
        self.0.merge(&another.0);
        self.1.merge(&another.1);
    }

    fn is_finite(&self) -> bool {
        self.0.is_finite() && self.1.is_finite()
    }
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
    pub fn render<T: Send + Clone + Accumulator<(RGB, usize)> + 'static>(
        &self,
        scene: Arc<Scene>,
        camera: &Camera,
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

    fn render_thread<T: Clone + Accumulator<(RGB, usize)>>(
        scene: &Scene,
        camera: Camera,
        film: FilmArc<T>,
        accum_init: T,
        integrator: Integrator,
        thread_id: usize,
        manager: Arc<Mutex<Manager>>,
    ) {
        use rand::distributions::Uniform;
        let mut rng = SmallRng::from_entropy();
        let px_size = camera.width() / film.w() as f32;

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
            let mut samples = vec![];
            for yi in 0..film.h() {
                for _i in 0..spp {
                    let mut radiance = accum_init.clone();
                    let du = {
                        let x = xi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dx = x - film.w() as f32 / 2.0;
                        dx * px_size
                    };
                    let dv = {
                        let y = yi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dy = film.h() as f32 / 2.0 - y;
                        dy * px_size
                    };
                    let ray = camera.ray_to(du, dv);
                    match integrator {
                        Integrator::PathTrace => {
                            pt::radiance(false, scene, &ray, &mut radiance, &mut rng)
                        }
                        Integrator::PathTraceWithNee => {
                            pt::radiance(true, scene, &ray, &mut radiance, &mut rng)
                        }
                        Integrator::BidirectionalPathTrace => {
                            bdpt::radiance(scene, &ray, &mut radiance, &mut rng)
                        }
                    };
                    if radiance.is_finite() {
                        samples.push((xi, yi, radiance));
                    } else {
                        warn!("radiance is not finite");
                    }
                }
            }

            film.with_lock(|mut film| {
                for (xi, yi, sample) in samples {
                    let pixel = film.at_mut(xi, yi);
                    pixel.accum.merge(&sample);
                    pixel.samples += 1;
                }
            })
            .unwrap();
        }
    }
}
