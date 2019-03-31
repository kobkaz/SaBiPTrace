use crate::accum::*;
use crate::camera::Camera;
use crate::image::*;
use crate::manager::*;
use crate::scene::Scene;
use crate::*;

use log::*;
use rand::prelude::*;
use std::sync::{Arc, Mutex};

pub mod bdpt; //bidirectional path tracing
pub mod lt; //light tracing
pub mod pt; //path tracing

#[derive(Clone, Copy, Debug)]
pub enum IntegratorType {
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
    pub integrator: IntegratorType,
    pub nthread: usize,
}

pub trait Integrator {
    fn integrate<T: Send + Clone + Accumulator + 'static, C: Send + Clone + Camera + 'static>(
        &self,
        scene: Arc<Scene>,
        camera: &C,
        film_config: FilmConfig<T>,
        config: RenderConfig,
        on_cycle_complete: Box<dyn FnMut(usize, usize) -> Option<usize> + Send>,
    );
}

pub trait OnepassIntegrator {
    type Integrator;
    fn clone_integrator(&self) -> Self::Integrator;
    fn render_thread<T: Clone + Accumulator>(
        integrator: &Self::Integrator,
        scene: &Scene,
        camera: impl Camera,
        film: FilmArc<T>,
        accum_init: T,
        thread_id: usize,
        manager: Arc<Mutex<Manager>>,
    );
}

impl<OPI: OnepassIntegrator> Integrator for OPI
where
    OPI::Integrator: Send + 'static,
{
    fn integrate<T: Send + Clone + Accumulator + 'static, C: Send + Clone + Camera + 'static>(
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
            let integrator = self.clone_integrator();
            let thread = thread::spawn(move || {
                Self::render_thread(&integrator, &scene, camera, film, accum_init, i, manager)
            });
            threads.push(thread);
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }
}

struct RayRadianceIntegratorWrapper<T>(T);

pub trait RayRadianceIntegrator {
    fn radiance<R: ?Sized>(
        &self,
        scene: &Scene,
        ray: &Ray,
        radiance_accum: &mut impl Accumulator,
        rng: &mut R,
    ) where
        R: Rng;
}

impl<RRI: RayRadianceIntegrator + Clone> OnepassIntegrator for RayRadianceIntegratorWrapper<RRI> {
    type Integrator = RRI;
    fn clone_integrator(&self) -> RRI {
        self.0.clone()
    }
    fn render_thread<T: Clone + Accumulator>(
        integrator: &RRI,
        scene: &Scene,
        camera: impl Camera,
        film: FilmArc<T>,
        accum_init: T,
        thread_id: usize,
        manager: Arc<Mutex<Manager>>,
    ) {
        let mut rng = SmallRng::from_entropy();

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

            let xi = task.chunk;
            let spp = task.amount;
            let mut samples = vec![];
            for yi in 0..film.h() {
                for _i in 0..spp {
                    let mut radiance = accum_init.clone();
                    let (u, v) = film.sample_uv_in_pixel(xi as i32, yi as i32, &mut rng);
                    let ray = camera.sample_ray(u, v, &mut rng);
                    integrator.radiance(scene, &ray, &mut radiance, &mut rng);
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
}

struct RandomPixelIntegratorWrapper<T>(T);

pub trait RandomPixelIntegrator {
    fn sample<R, C, T>(
        &self,
        scene: &Scene,
        camera: &C,
        film: &mut FilmVec<T>,
        accum_init: &T,
        rng: &mut R,
    ) where
        R: Rng + ?Sized,
        C: Camera + ?Sized,
        T: Clone + Accumulator;
}

impl<RPI: RandomPixelIntegrator + Clone> OnepassIntegrator for RandomPixelIntegratorWrapper<RPI> {
    type Integrator = RPI;

    fn clone_integrator(&self) -> RPI {
        self.0.clone()
    }

    fn render_thread<T: Clone + Accumulator>(
        integrator: &RPI,
        scene: &Scene,
        camera: impl Camera,
        film: FilmArc<T>,
        accum_init: T,
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

            let _xi = task.chunk;
            let spp = task.amount;
            for _i in 0..film.h() * spp {
                integrator.sample(scene, &camera, &mut local_film, &accum_init, &mut rng);
            }
            total_sample += film.h() * spp;
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
        match config.integrator {
            IntegratorType::PathTrace => {
                RayRadianceIntegratorWrapper(pt::PathTracer { enable_nee: false }).integrate(
                    scene,
                    camera,
                    film_config,
                    config,
                    on_cycle_complete,
                );
            }
            IntegratorType::PathTraceWithNee => {
                RayRadianceIntegratorWrapper(pt::PathTracer { enable_nee: true }).integrate(
                    scene,
                    camera,
                    film_config,
                    config,
                    on_cycle_complete,
                );
            }
            IntegratorType::BidirectionalPathTrace => {
                RayRadianceIntegratorWrapper(bdpt::BidirectionalPathTracer).integrate(
                    scene,
                    camera,
                    film_config,
                    config,
                    on_cycle_complete,
                );
            }
            IntegratorType::LightTrace => {
                RandomPixelIntegratorWrapper(lt::LightTracer).integrate(
                    scene,
                    camera,
                    film_config,
                    config,
                    on_cycle_complete,
                );
            }
        }
    }
}
