use camera::Camera;
use renderer::Renderer;
use sabiptrace::*;
use scene::Scene;

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

#[allow(dead_code)]
fn make_plane_scene() -> Scene {
    use material::Material::*;
    use shape::Sphere;
    const R: f32 = 10000.0;
    const L: f32 = 50.0;
    let mut objects = vec![];

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, -L - R, 0.0),
            radius: R,
        },
        material: Lambert(RGB::new(1.0, 0.6, 0.6)),
        emission: None,
    });

    objects.push(object::SimpleObject {
        shape: Sphere {
            center: P3::new(0.0, 0.0, 0.0),
            radius: R,
        },
        material: Lambert(RGB::all(0.0)),
        emission: Some(RGB::new(0.2, 0.2, 0.8)),
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
        let view_up = V3::new(0.1, 1.0, 0.0);
        let fov_degree = 45.0;
        Camera::new(origin, view_at, view_up, fov_degree)
    };

    let renderer = Renderer;
    let scene = make_box();
    renderer.render(&scene, &view, &mut image);
    image.write_exr("output/output.exr");
}
