use crate::camera::Camera;
use crate::image::*;
use crate::scene::Scene;
use crate::*;

use rand::prelude::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Task {
    chunk: usize,
    amount: usize,
}

#[derive(Debug)]
enum ManagerState {
    Halted,
    CycleWait(usize, usize, Vec<(usize, Sender<Option<Task>>)>),
    CycleProgress { cycle: usize, chunk: usize },
}

struct Manager {
    chunks: usize,
    total_amount: usize,
    max_cycle_amount: usize,
    state: ManagerState,
    thread_idle: Vec<bool>,
    on_cycle_complete: Box<dyn FnMut(usize, usize) + Send>,
}

impl Manager {
    pub fn new(
        chunks: usize,
        total_amount: usize,
        max_cycle_amount: usize,
        nthread: usize,
        on_cycle_complete: Box<dyn FnMut(usize, usize) + Send>,
    ) -> Self {
        Manager {
            chunks,
            total_amount,
            max_cycle_amount,
            state: ManagerState::CycleProgress { cycle: 0, chunk: 0 },
            thread_idle: vec![true; nthread],
            on_cycle_complete,
        }
    }

    pub fn next(&mut self, thid: usize) -> Receiver<Option<Task>> {
        use ManagerState::*;

        let (tx, rx) = mpsc::channel();
        self.thread_idle[thid] = true;
        let all_idle = self.all_idle();
        match self.state {
            Halted => {
                let _ = tx.send(None);
                if all_idle {
                    (self.on_cycle_complete)(self.total_amount, self.total_amount)
                }
            }
            CycleWait(prev_cycle, amount, ref mut txs) => {
                let cycle = prev_cycle + 1;
                txs.push((thid, tx));
                if all_idle {
                    (self.on_cycle_complete)(
                        (self.max_cycle_amount * cycle).min(self.total_amount),
                        self.total_amount,
                    );
                    for (i, (thid, tx)) in txs.into_iter().enumerate() {
                        let _ = tx.send(Some(Task { chunk: i, amount }));
                        self.thread_idle[*thid] = false;
                    }
                    self.state = CycleProgress {
                        cycle,
                        chunk: txs.len(),
                    };
                }
            }
            CycleProgress { cycle, chunk } => {
                let amount = self.cycle_amount(cycle);
                if chunk + 1 == self.chunks {
                    if (cycle + 1) * self.max_cycle_amount >= self.total_amount {
                        self.state = Halted;
                    } else {
                        self.state = CycleWait(cycle, self.cycle_amount(cycle + 1), vec![]);
                    }
                } else {
                    self.state = CycleProgress {
                        cycle,
                        chunk: chunk + 1,
                    };
                }
                let _ = tx.send(Some(Task { chunk, amount }));
                self.thread_idle[thid] = false;
            }
        }
        rx
    }

    fn all_idle(&self) -> bool {
        self.thread_idle.iter().all(|b| *b)
    }

    fn cycle_amount(&self, cycle: usize) -> usize {
        let completed = cycle * self.max_cycle_amount;
        if completed >= self.total_amount {
            0
        } else {
            (self.total_amount - completed).min(self.max_cycle_amount)
        }
    }
}

pub struct Renderer;
impl Renderer {
    pub fn render(
        &self,
        scene: Arc<Scene>,
        camera: &Camera,
        film: Arc<Mutex<Film>>,
        nthread: usize,
        spp: usize,
        cycle_spp: usize,
    ) {
        use std::thread;
        let mut threads = vec![];
        let cb = {
            let start = std::time::Instant::now();
            let film = film.clone();
            let mut cycle = 0;
            Box::new(move |completed_samples, total_samples| {
                cycle += 1;
                let elapsed = std::time::Instant::now().duration_since(start);
                let ms = { elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64 };
                let secs = (ms as f64) / 1000.0;
                let progress = completed_samples as f64 / total_samples as f64;
                let eta = secs * (1.0 - progress) / progress;
                let spd = completed_samples as f64 / secs;
                let spd_pc = spd / nthread as f64;
                println!(
                    "completed {} / {} ({:.2} %) elapsed {:.2} sec  ETA {:.2} sec",
                    completed_samples,
                    total_samples,
                    progress * 100.0,
                    secs,
                    eta
                );
                println!("Speed {:.2} spp/sec {:.2} spp/sec/core", spd, spd_pc);
                let film = film.lock().unwrap();
                film.to_image().write_exr(&format!("output/{}.exr", cycle));
            })
        };
        let manager = Manager::new(
            film.lock().unwrap().w() as usize,
            spp,
            cycle_spp,
            nthread,
            cb,
        );
        let manager = Arc::new(Mutex::new(manager));
        for i in 0..nthread {
            let film = film.clone();
            let camera = camera.clone();
            let scene = scene.clone();
            let manager = manager.clone();
            let thread =
                thread::spawn(move || Self::render_thread(&scene, camera, film, i, manager));
            threads.push(thread);
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn render_thread(
        scene: &Scene,
        camera: Camera,
        film: Arc<Mutex<Film>>,
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
            let mut col = vec![RGB::all(0.0); film_h as usize];
            for yi in 0..film_h {
                let mut accum = RGB::all(0.0);
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
                    const USE_NEE: bool = true;
                    accum = accum + Self::radiance(USE_NEE, scene, &ray, &mut rng);
                }
                col[yi as usize] = accum;
            }

            let mut film = film.lock().unwrap();
            for yi in 0..film_h {
                let pixel = film.at_mut(xi, yi);
                pixel.accum += col[yi as usize];
                pixel.samples += spp;
            }
        }
    }

    fn radiance<R: ?Sized>(enable_nee: bool, scene: &Scene, ray: &Ray, rng: &mut R) -> RGB
    where
        R: Rng,
    {
        let mut depth = 0;
        let mut ray = ray.clone();
        let mut radiance = RGB::all(0.0);
        let mut throughput = RGB::all(1.0);
        let mut prev_specular = true;

        const DEPTH_MAX: usize = 100;
        loop {
            depth += 1;
            let hit = scene.test_hit(&ray, 1e-3, std::f32::MAX);
            if let Some(hit) = hit {
                let hit_lc = hit.geom.lc();
                let wout_local = hit_lc.w2l() * -ray.dir;

                if prev_specular || !enable_nee {
                    if let Some(emission) = hit.emission {
                        radiance += throughput * emission;
                    }
                }

                if enable_nee {
                    if let Some(light_sample) = scene.sample_light(rng) {
                        let (light_point, light_normal, light_emission) = light_sample.value;
                        if scene.visible(light_point, hit.geom.pos) {
                            let g = hit.geom.g(&light_point, &light_normal);
                            let light_dir = (light_point - hit.geom.pos).normalize();
                            let bsdf = hit.material.bsdf(&(hit_lc.w2l() * light_dir), &wout_local);
                            radiance += throughput * light_emission * bsdf * g / light_sample.pdf;
                        }
                    }
                }

                if depth >= DEPTH_MAX {
                    break;
                }

                let next = hit.material.sample_win(&wout_local, rng);
                let win_local = next.value.0;
                let bsdf = next.value.1;
                prev_specular = next.value.2;
                let cos = win_local[2].abs();
                throughput *= bsdf * cos;
                throughput /= next.pdf;

                let cont = pdf::RandomBool {
                    chance: (throughput.max() * 0.8).min(1.0),
                };
                let cont = cont.sample(rng);
                if !cont.value {
                    break;
                }
                throughput /= cont.pdf;

                ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
            } else {
                break;
            }
        }
        return radiance;
    }
}
