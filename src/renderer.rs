use crate::camera::Camera;
use crate::image::Image;
use crate::scene::Scene;
use crate::*;

use rand::prelude::*;
use std::sync::{Arc, Mutex};

struct Manager {
    max: usize,
    next: usize,
    begin: std::time::Instant,
}

impl Manager {
    fn new(max: usize) -> Self {
        Manager {
            max,
            next: 0,
            begin: std::time::Instant::now(),
        }
    }

    fn next(&mut self, id: Option<usize>) -> Option<usize> {
        if self.next >= self.max {
            None
        } else {
            let n = self.next;
            if let Some(id) = id {
                let t = std::time::Instant::now().duration_since(self.begin);
                let millis = {
                    let secs = t.as_secs();
                    secs * 1000 + t.subsec_millis() as u64
                };
                let progress = n as f32 / self.max as f32;
                let eta = ((millis as f32) * (1.0 - progress) / progress) / 1000.0;
                println!(
                    "{} took {} / {} ({}) elapsed {} eta {}",
                    id,
                    n,
                    self.max,
                    progress * 100.0,
                    millis,
                    eta as usize
                );
            }

            self.next += 1;
            Some(n)
        }
    }
}

pub struct Renderer;
impl Renderer {
    pub fn render(
        &self,
        scene: Arc<Scene>,
        camera: &Camera,
        image: Arc<Mutex<Image>>,
        nthread: usize,
    ) {
        use std::thread;
        let mut threads = vec![];
        let manager = Manager::new(image.lock().unwrap().w() as usize);
        let manager = Arc::new(Mutex::new(manager));
        //let scene = Arc::new(scene);
        for i in 0..nthread {
            let image = image.clone();
            let camera = camera.clone();
            let scene = scene.clone();
            let manager = manager.clone();
            let thread = thread::spawn(move || {
                Self::render_thread(&scene, camera, image, i, nthread, manager)
            });
            threads.push(thread);
        }
        for thread in threads {
            thread.join().unwrap();
        }
    }

    fn render_thread(
        scene: &Scene,
        camera: Camera,
        image: Arc<Mutex<Image>>,
        thread_id: usize,
        nthread: usize,
        manager: Arc<Mutex<Manager>>,
    ) {
        use rand::distributions::Uniform;
        let mut rng = SmallRng::from_entropy();
        let (image_w, image_h) = {
            let image = image.lock().unwrap();
            (image.w(), image.h())
        };
        let px_size = camera.width() / image_w as f32;

        let opt_thid = if thread_id == 0 { Some(0) } else { None };

        loop {
            let xi = {
                if let Some(xi) = manager.lock().unwrap().next(opt_thid) {
                    xi as u32
                } else {
                    break;
                }
            };
            for yi in 0..image_h {
                let mut accum = RGB::all(0.0);
                let n = 50;
                for _i in 0..n {
                    let du = {
                        let x = xi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dx = x - image_w as f32 / 2.0;
                        dx * px_size
                    };
                    let dv = {
                        let y = yi as f32 + Uniform::new(0.0, 1.0).sample(&mut rng);
                        let dy = image_h as f32 / 2.0 - y;
                        dy * px_size
                    };
                    let ray = camera.ray_to(du, dv);
                    const USE_NEE: bool = true;
                    accum = accum + Self::radiance(USE_NEE, scene, &ray, &mut rng);
                }
                let mut image = image.lock().unwrap();
                *image.at_mut(xi, yi) = accum / n as f32;
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
                let cont = pdf::RandomBool {
                    chance: (throughput.max() * 0.8).min(1.0),
                };
                let cont = cont.sample(rng);
                if !cont.value {
                    break;
                }
                throughput /= cont.pdf;

                prev_specular = hit.material.is_specular();
                let next = hit.material.sample_win(&wout_local, rng);
                let win_local = next.value.0;
                let bsdf = next.value.1;
                let cos = win_local[2].abs();
                throughput *= bsdf * cos;
                throughput /= next.pdf;
                ray = hit_lc.l2w() * Ray::new(P3::origin(), win_local);
            } else {
                break;
            }
        }
        return radiance;
    }
}
