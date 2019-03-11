use rand::prelude::*;
use sabiptrace::{ray::Ray, rgb::RGB, *};

struct View {
    e_u: V3,
    e_v: V3,
    e_w: V3,
    width: f32,
    origin: P3,
}

impl View {
    fn new(origin: P3, view_at: P3, view_up: V3, fov_degree: f32) -> Self {
        let e_w = (origin - view_at).normalize();
        let e_v = (view_up - e_w * (e_w.dot(&view_up))).normalize();
        let e_u = e_v.cross(&e_w);
        let fov_rad = fov_degree * std::f32::consts::PI / 180.0;
        let half_tan = (fov_rad / 2.0).tan();
        View {
            e_u,
            e_v,
            e_w,
            width: 2.0 * half_tan,
            origin,
        }
    }
    fn ray_to(&self, u: f32, v: f32) -> Ray {
        let ray_dir = (self.e_u * u + self.e_v * v - self.e_w).normalize();
        Ray::new(self.origin, ray_dir)
    }
}

struct Scene {
    bvh: object::BVH,
    lights: Vec<usize>,
}

impl Scene {
    pub fn new(objects: Vec<object::SimpleObject>) -> Self {
        let bvh = object::BVH::new(objects);
        let objects = bvh.objects();
        let lights = (0..objects.len())
            .filter(|i| objects[*i].emission.is_some())
            .collect();
        Scene { bvh, lights }
    }

    pub fn sample_light<R: Rng>(&self, rng: &mut R) -> Option<pdf::PdfSample<(P3, V3, RGB)>> {
        use pdf::PdfSample;
        use rand::seq::SliceRandom;
        self.lights.choose(rng).map(|ix| {
            let obj = &self.bvh.objects()[*ix];
            let PdfSample { value: (p, n), pdf } = obj.shape.sample_surface(rng);
            let e = obj.emission.unwrap();
            PdfSample {
                value: (p, n, e),
                pdf: pdf / self.lights.len() as f32,
            }
        })
    }

    pub fn test_hit(&self, ray: &ray::Ray, tnear: f32, tfar: f32) -> Option<object::ObjectHit> {
        self.bvh.test_hit(ray, tnear, tfar)
    }

    pub fn visible(&self, x: P3, y: P3) -> bool {
        let r = y - x;
        let dist = r.norm();
        let ray = Ray::new(x, r / dist);
        self.test_hit(&ray, 1e-3, dist - 1e-3).is_none()
    }
}

struct Renderer;
impl Renderer {
    pub fn render(&self, scene: &Scene, view: &View, image: &mut image::Image) {
        use rand::distributions::Uniform;
        let mut rng = SmallRng::from_entropy();
        let px_size = view.width / image.w() as f32;
        for xi in 0..image.w() {
            for yi in 0..image.h() {
                let mut accum = RGB::all(0.0);
                let n = 10;
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
                    let ray = view.ray_to(du, dv);
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

#[allow(dead_code)]
fn make_scene() -> Scene {
    use material::Material::*;
    use shape::Sphere;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-50.0, 0.0, 0.0),
            radius: 5.0,
        },
        material: Lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.0, 10.0, 0.0)),
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 20.0,
        },
        material: Lambert(RGB::all(1.0)),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(50.0, 0.0, 0.0),
            radius: 5.0,
        },
        material: Lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.0, 0.0, 10.0)),
    });
    /*
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 10000.0,
        },
        material: Lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.0, 0.0, 0.05)),
    });
    */
    Scene::new(objects)
}

#[allow(dead_code)]
fn make_box() -> Scene {
    use material::Material::*;
    use shape::Sphere;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, -L - R),
            radius: R,
        },
        material: Lambert(RGB::new(0.5, 0.5, 0.5)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(L + R, 0.0, 0.0),
            radius: R,
        },
        material: Lambert(RGB::new(0.0, 0.5, 0.0)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-L - R, 0.0, 0.0),
            radius: R,
        },
        material: Lambert(RGB::new(0.0, 0.0, 0.5)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, L + R, 0.0),
            radius: R,
        },
        material: Lambert(RGB::new(0.5, 0.5, 0.0)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -L - R, 0.0),
            radius: R,
        },
        material: Lambert(RGB::new(0.5, 0.0, 0.0)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 10.0,
        },
        material: Lambert(RGB::all(0.0)),
        emission: Some(RGB::all(1e2)),
    });

    Scene::new(objects)
}
fn main() {
    let mut image = {
        let s = 50;
        image::Image::new(16 * s, 9 * s)
    };

    let view = {
        let origin = P3::new(0.0, 0.0, 200.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        View::new(origin, view_at, view_up, fov_degree)
    };

    let renderer = Renderer;
    let scene = make_scene();
    renderer.render(&scene, &view, &mut image);
    image.write_exr("output/output.exr");
}
