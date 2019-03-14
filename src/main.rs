use camera::Camera;
use material::Material;
use renderer::Renderer;
use sabiptrace::*;
use scene::Scene;

#[allow(dead_code)]
fn make_scene() -> (Camera, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-50.0, 0.0, 0.0),
            radius: 5.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::new(0.0, 10.0, 0.0)),
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(50.0, 0.0, 0.0),
            radius: 5.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::new(0.0, 0.0, 10.0)),
    });
    /*
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 10000.0,
        },
        material: Lambert(RGB::all(0.0)).into(),
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
    use material::materials::*;
    use material::Material;
    use shape::shapes::*;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-2.0 * L, -L, -L),
            P3::new(2.0 * L, -L, -L),
            P3::new(0.0, 4.0 * L, -L),
        ])
        .into(),
        material: Lambert(RGB::new(0.5, 0.5, 0.5)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(L, 2.0 * L, -L),
            P3::new(L, -2.0 * L, -L),
            P3::new(L, 0.0, L * 10.0),
        ])
        .into(),
        material: Lambert(RGB::new(0.0, 1.0, 0.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-L, 2.0 * L, -L),
            P3::new(-L, -2.0 * L, -L),
            P3::new(-L, 0.0, L * 10.0),
        ])
        .into(),
        material: Lambert(RGB::new(0.2, 0.2, 1.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(2.0 * L, L, -L),
            P3::new(-2.0 * L, L, -L),
            P3::new(0.0, L, L * 10.0),
        ])
        .into(),
        material: Lambert(RGB::new(0.8, 0.8, 0.2)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(2.0 * L, -L, -L),
            P3::new(-2.0 * L, -L, -L),
            P3::new(0.0, -L, L * 10.0),
        ])
        .into(),
        material: Lambert(RGB::new(1.0, 0.2, 0.2)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, L, 0.0),
            radius: 10.0,
        }
        .into(),
        material: Mirror(RGB::all(0.0)).into(),
        emission: Some(RGB::all(50.0)),
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-30.0, -30.0, 10.0),
            radius: 20.0,
        }
        .into(),
        material: Mirror(RGB::all(1.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(30.0, -30.0, -10.0),
            radius: 20.0,
        }
        .into(),
        material: Material::mix(
            0.1,
            Mirror(RGB::all(1.0)).into(),
            Lambert(RGB::all(1.0)).into(),
        ),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(10.0, -40.0, 30.0),
            radius: 10.0,
        }
        .into(),
        material: Lambert(RGB::new(0.0, 1.0, 1.0)).into(),
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
    use material::materials::*;
    use shape::shapes::*;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    //floor
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -R, 0.0),
            radius: R,
        }
        .into(),
        material: Lambert(RGB::new(1.0, 0.6, 0.6)).into(),
        emission: None,
    });

    //sky
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: R,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
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
                material::materials::Mirror(color).into()
            } else {
                material::materials::Lambert(color).into()
            }
        };

        objects.push(object::SimpleObject {
            shape: Sphere { center, radius }.into(),
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
    let film = {
        let s = 50;
        image::Film::new(16 * s, 9 * s)
    };

    let (camera, scene) = make_box();
    let film = Arc::new(Mutex::new(film));
    let scene = Arc::new(scene);
    let renderer = Renderer;
    let ncpu = num_cpus::get();
    let spp = 50;
    let cycle_spp = 10;
    println!("ncpu {}", ncpu);
    renderer.render(scene, &camera, film.clone(), ncpu, spp, cycle_spp);
    film.lock()
        .unwrap()
        .to_image()
        .write_exr("output/output.exr");
}
