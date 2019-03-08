extern crate sabiptrace;
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
    objects: object::ObjectList,
}

struct Renderer;
impl Renderer {
    pub fn render(&self, scene: &Scene, view: &View, image: &mut image::Image) {
        let px_size = view.width / image.w() as f32;

        for xi in 0..image.w() {
            let du = {
                let x = xi as f32 + 0.5;
                let dx = x - image.w() as f32 / 2.0;
                dx * px_size
            };
            for yi in 0..image.h() {
                let dv = {
                    let y = yi as f32 + 0.5;
                    let dy = image.h() as f32 / 2.0 - y;
                    dy * px_size
                };
                let ray = view.ray_to(du, dv);
                *image.at_mut(xi, yi) = self.l(scene, &ray);
            }
        }
    }

    fn l(&self, scene: &Scene, ray: &Ray) -> RGB {
        use material::Material::*;
        let hit = scene.objects.test_hit(ray, 0.0, std::f32::MAX);
        if let Some(hit) = hit {
            match hit.material {
                Lambert(color) => color,
            }
        } else {
            RGB::new(0.0, 0.0, 0.0)
        }
    }
}

fn make_scene() -> Scene {
    use material::Material::*;
    use shape::Sphere;
    let mut scene = Scene {
        objects: object::ObjectList::new(),
    };

    scene.objects.objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(00.0, 0.0, 0.0),
            radius: 10.0,
        },
        material: Lambert(RGB::new(1.0, 0.0, 0.0)),
    });
    scene.objects.objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(20.0, 0.0, 0.0),
            radius: 10.0,
        },
        material: Lambert(RGB::new(0.0, 1.0, 0.0)),
    });
    scene.objects.objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(40.0, 10.0, 0.0),
            radius: 10.0,
        },
        material: Lambert(RGB::new(0.0, 0.0, 1.0)),
    });
    scene.objects.objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(60.0, -10.0, 0.0),
            radius: 10.0,
        },
        material: Lambert(RGB::new(1.0, 1.0, 1.0)),
    });
    scene
}

fn main() {
    let mut image = {
        let s = 100;
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
