#![allow(dead_code)]

use crate::*;
use camera::{Camera, PinHole, ThinLens};
use scene::Scene;

pub fn make_scene() -> (PinHole, Scene) {
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

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.1, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_box() -> (impl Camera + Clone, Scene) {
    use material::materials::*;
    use material::Material;
    use shape::shapes::*;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-4.0 * L, -L, -L),
            P3::new(4.0 * L, -L, -L),
            P3::new(0.0, 4.0 * L, -L),
        ])
        .into(),
        material: Lambert(RGB::new(0.5, 0.5, 0.5)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(L + 20.0, 2.0 * L, -L),
            P3::new(L + 20.0, -2.0 * L, -L),
            P3::new(L + 20.0, 0.0, L * 10.0),
        ])
        .into(),
        material: Lambert(RGB::new(0.0, 0.8, 0.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-L - 20.0, 2.0 * L, -L),
            P3::new(-L - 20.0, -2.0 * L, -L),
            P3::new(-L - 20.0, 0.0, L * 10.0),
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
        material: Lambert(RGB::new(0.8, 0.2, 0.2)).into(),
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
            center: P3::new(20.0, -35.0, 60.0),
            radius: 15.0,
        }
        .into(),
        //material: Mirror(RGB::all(1.0)).into(),
        material: Transparent {
            color: RGB::new(1.0, 1.0, 1.0),
            index: 1.4,
        }
        .into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(40.0, -20.0, -10.0),
            radius: 30.0,
        }
        .into(),
        material: Lambert(RGB::new(0.0, 1.0, 0.5)).into(),
        //material: Material::mix(
        //    0.1,
        //    Mirror(RGB::all(1.0)).into(),
        //    Lambert(RGB::all(1.0)).into(),
        //),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-30.0, -20.0, 60.0),
            radius: 30.0,
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
        PinHole::new(origin, view_at, view_up, fov_degree, None)
        //let radius = 1.0;
        //let f = 10.0;
        //let focus_dist = 250.0;
        //ThinLens::new_with_focus_distance(
        //    origin, view_at, view_up, radius, f, focus_dist, fov_degree,
        //)
    };

    (camera, scene)
}

pub fn make_plane_scene() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    //floor
    objects.push(object::SimpleObject {
        shape: Parallelogram::new(
            &P3::new(1e4, 0.0, 1e4),
            &P3::new(-1e4, 0.0, 1e4),
            &P3::new(1e4, 0.0, -1e4),
        )
        .into(),
        material: Lambert(RGB::new(1.0, 0.6, 0.6)).into(),
        emission: None,
    });

    use rand::distributions::Uniform;
    use rand::prelude::*;
    let mut rng = SmallRng::from_entropy();
    let mut spheres: Vec<Sphere> = vec![];
    'generate_spheres: for _i in 0..200 {
        'generate: for _try in 0..10 {
            let radius = {
                let t = Uniform::new(3.0, 6.0).sample(&mut rng);
                t * t
            };
            let center = {
                let x = Uniform::new(-600.0, 600.0).sample(&mut rng);
                let z = Uniform::new(-800.0, 200.0).sample(&mut rng);
                P3::new(x, radius, z)
            };

            for s in spheres.iter() {
                if (s.center - center).norm() < s.radius + radius {
                    continue 'generate;
                }
            }
            spheres.push(Sphere { center, radius });
            continue 'generate_spheres;
        }
        //fail
        break;
    }

    for sphere in spheres {
        let (material, emission) = {
            let material_type = Uniform::new(0.0, 1.0).sample(&mut rng);
            let color = {
                let r = Uniform::new(0.0, 1.0).sample(&mut rng);
                let g = Uniform::new(0.0, 1.0).sample(&mut rng);
                let b = Uniform::new(0.0, 1.0).sample(&mut rng);
                RGB::new(r, g, b)
            };
            let emission = None;
            let m = if material_type < 0.3 {
                material::materials::Mirror(color).into()
            } else if material_type < 0.6 {
                let index = Uniform::new(1.0, 2.0).sample(&mut rng);
                material::materials::Transparent { color, index }.into()
            } else {
                material::materials::Lambert(color).into()
            };
            (m, emission)
        };

        objects.push(object::SimpleObject {
            shape: sphere.into(),
            material,
            emission,
        });
    }
    let envmap = image::Image::read_exr16("envmap_rect.exr").unwrap();
    let scene = Scene::new(objects).set_rect_envmap(envmap);

    let camera = {
        let origin = P3::new(0.0, 100.0, 300.0);
        let view_at = P3::new(0.0, 40.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 100.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_parallel() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-50.0, 1e3, -1e3),
            P3::new(-50.0, -1e3, -1e3),
            P3::new(-50.0, 0.0, 1e3),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(50.0, 1e3, -1e3),
            P3::new(50.0, -1e3, -1e3),
            P3::new(50.0, 0.0, 1e3),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 5.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(10.0)),
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_parallel_and_mirror() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-50.0, 1e3, -1e3),
            P3::new(-50.0, -1e3, -1e3),
            P3::new(-50.0, 0.0, 1e3),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(50.0, 1e3, -1e3),
            P3::new(50.0, -1e3, -1e3),
            P3::new(50.0, 0.0, 1e3),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-50.0, -50.0, 50.0),
            P3::new(50.0, -50.0, 50.0),
            P3::new(0.0, -50.0, -200.0),
        ])
        .into(),
        material: Mirror(RGB::all(1.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 5.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(10.0)),
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_fiveballs_mirror() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 5.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(10.0)),
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(50.0, 0.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Mirror(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-50.0, 0.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 50.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Mirror(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -50.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_fiveballs_lambert() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 5.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(10.0)),
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(50.0, 0.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-50.0, 0.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 50.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -50.0, 0.0),
            radius: 20.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_black_shell() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    //objects.push(object::SimpleObject {
    //    shape: Sphere {
    //        center: P3::new(0.0, 0.0, 0.0),
    //        radius: 1000.0,
    //    }
    //    .into(),
    //    material: Lambert(RGB::all(0.0)).into(),
    //    emission: None,
    //});

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-50.0, 0.0, 0.0),
            radius: 45.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(50.0, 0.0, 0.0),
            radius: 45.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(1.0)),
    });

    //for i in 0..100 {
    //    objects.push(object::SimpleObject {
    //        shape: Sphere {
    //            center: P3::new(i as f32, 0.0, -2e3),
    //            radius: 0.1,
    //        }
    //        .into(),
    //        material: Lambert(RGB::all(0.0)).into(),
    //        emission: Some(RGB::all(1000.0)),
    //    });
    //}

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_ball_triangle() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(100.0)),
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-30.00, -30.0, -10.0),
            P3::new(30.00, -30.0, -10.0),
            P3::new(0.00, 30.0, -10.0),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_ball_triangle_2() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(100.0)),
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(0.0, 50.0, -50.0),
            P3::new(0.0, -50.0, -50.0),
            P3::new(-50.0, 0.0, 50.0),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(0.0, 50.0, -50.0),
            P3::new(0.0, -50.0, -50.0),
            P3::new(50.0, 0.0, 50.0),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_ball_triangle_mirror() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(100.0)),
    });

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(10.0, 10.0, -50.0),
            P3::new(50.0, 10.0, 50.0),
            P3::new(10.0, 50.0, 50.0),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-10.0, 10.0, -50.0),
            P3::new(-50.0, 10.0, 50.0),
            P3::new(-10.0, 50.0, 50.0),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(10.0, -10.0, -50.0),
            P3::new(50.0, -10.0, 50.0),
            P3::new(10.0, -50.0, 50.0),
        ])
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });
    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-10.0, -10.0, -50.0),
            P3::new(-50.0, -10.0, 50.0),
            P3::new(-10.0, -50.0, 50.0),
        ])
        .into(),
        material: Mirror(RGB::all(1.0)).into(),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_emissive_triangle() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Triangle::new([
            P3::new(-10.0, -10.0, 0.0),
            P3::new(10.0, -10.0, 0.0),
            P3::new(0.0, 10.0, 0.0),
        ])
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(0.5)),
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 20.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}

pub fn make_debug() -> (PinHole, Scene) {
    use material::materials::*;
    use shape::shapes::*;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(40.0, 0.0, 0.0),
            radius: 30.0,
        }
        .into(),
        material: Lambert(RGB::all(0.0)).into(),
        emission: Some(RGB::all(1.0)),
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(-40.0, 0.0, 0.0),
            radius: 30.0,
        }
        .into(),
        material: Lambert(RGB::all(1.0)).into(),
        emission: None,
    });

    let scene = Scene::new(objects);

    let camera = {
        let origin = P3::new(0.0, 0.0, 300.0);
        let view_at = P3::new(0.0, 0.0, 0.0);
        let view_up = V3::new(0.0, 1.0, 0.0);
        let fov_degree = 45.0;
        PinHole::new(origin, view_at, view_up, fov_degree, None)
    };

    (camera, scene)
}
