use camera::Camera;
use material::Material;
use renderer::Renderer;
use sabiptrace::*;
use scene::Scene;

#[allow(dead_code)]
fn make_scene() -> (Camera, Scene) {
    use shape::Sphere;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-50.0, 0.0, 0.0),
            radius: 5.0,
        },
        material: Material::new_lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.0, 10.0, 0.0)),
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 20.0,
        },
        material: Material::new_lambert(RGB::all(1.0)),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(50.0, 0.0, 0.0),
            radius: 5.0,
        },
        material: Material::new_lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.0, 0.0, 10.0)),
    });
    /*
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 10000.0,
        },
        material: Material::new_lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.0, 0.0, 0.05)),
    });
    */
    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.1, 1.0, 0.0);
        let fov_degree = 45.0;
        Camera::new(origin, view_at, view_up, fov_degree)
    };

    (camera, scene)
}

#[allow(dead_code)]
fn make_box() -> (Camera, Scene) {
    use shape::Sphere;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, -L - R),
            radius: R,
        },
        material: Material::new_lambert(RGB::new(0.5, 0.5, 0.5)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(L + R, 0.0, 0.0),
            radius: R,
        },
        material: Material::new_lambert(RGB::new(0.1, 1.0, 0.1)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-L - R, 0.0, 0.0),
            radius: R,
        },
        material: Material::new_lambert(RGB::new(0.2, 0.2, 1.0)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, L + R, 0.0),
            radius: R,
        },
        material: Material::new_lambert(RGB::new(0.8, 0.8, 0.2)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -L - R, 0.0),
            radius: R,
        },
        material: Material::new_lambert(RGB::new(1.0, 0.2, 0.2)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, L, 0.0),
            radius: 10.0,
        },
        material: Material::new_lambert(RGB::all(0.0)),
        emission: Some(RGB::all(50.0)),
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-30.0, -30.0, 10.0),
            radius: 20.0,
        },
        material: Material::new_mirror(RGB::all(1.0)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(30.0, -30.0, -10.0),
            radius: 20.0,
        },
        material: Material::new_lambert(RGB::all(1.0)),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.1, 1.0, 0.0);
        let fov_degree = 45.0;
        Camera::new(origin, view_at, view_up, fov_degree)
    };

    (camera, scene)
}

#[allow(dead_code)]
fn make_plane_scene() -> (Camera, Scene) {
    use shape::Sphere;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    //floor
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -R, 0.0),
            radius: R,
        },
        material: Material::new_lambert(RGB::new(1.0, 0.6, 0.6)),
        emission: None,
    });

    //sky
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: R,
        },
        material: Material::new_lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.7, 0.7, 2.0)),
    });

    use rand::distributions::Uniform;
    use rand::prelude::*;
    let mut rng = SmallRng::from_entropy();
    for _i in 0..30 {
        let radius = {
            let t = Uniform::new(1.5, 3.0).sample(&mut rng);
            t * t * t
        };
        let center = {
            let x = Uniform::new(-200.0, 200.0).sample(&mut rng);
            let z = Uniform::new(-400.0, 100.0).sample(&mut rng);
            let o = P3::new(0.0, -R, 0.0);
            let c_near = P3::new(x, radius, z);
            let v = (c_near - o).normalize() * (R + radius);
            o + v
        };
        let material = {
            let mirror = Uniform::new(0.0, 1.0).sample(&mut rng) < 0.25;
            let color = {
                let r = Uniform::new(0.0, 1.0).sample(&mut rng);
                let g = Uniform::new(0.0, 1.0).sample(&mut rng);
                let b = Uniform::new(0.0, 1.0).sample(&mut rng);
                RGB::new(r, g, b)
            };
            if mirror {
                Material::new_mirror(color)
            } else {
                Material::new_lambert(color)
            }
        };

        objects.push(object::SimpleObject {
            shape: Sphere { center, radius },
            material,
            emission: None,
        });
    }
    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 100.0, 300.0);
        let view_at = P3::new(0.0, 40.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        Camera::new(origin, view_at, view_up, fov_degree)
    };

    (camera, scene)
}

fn main() {
    use std::sync::{Arc, Mutex};
    let image = {
        let s = 50;
        image::Image::new(16 * s, 9 * s)
    };

    let (camera, scene) = make_box();
    let image = Arc::new(Mutex::new(image));
    let scene = Arc::new(scene);
    let cpus = num_cpus::get();
    let renderer = Renderer;
    println!("ncpu {}", cpus);
    renderer.render(scene, &camera, image.clone(), cpus);
    image.lock().unwrap().write_exr("output/output.exr");
}
