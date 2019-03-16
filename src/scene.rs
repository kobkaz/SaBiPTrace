use crate::*;

use rand::prelude::*;

enum EnvMap {
    Sphere(image::Image),
    Rect(image::Image),
}

pub struct Scene {
    bvh: object::BVH,
    lights: Vec<usize>,
    envmap: Option<EnvMap>,
}

impl Scene {
    pub fn new(objects: Vec<object::SimpleObject>) -> Self {
        let bvh = object::BVH::new(objects);
        let objects = bvh.objects();
        let lights = (0..objects.len())
            .filter(|i| objects[*i].emission.is_some())
            .collect();
        Scene {
            bvh,
            lights,
            envmap: None,
        }
    }

    pub fn set_sphere_envmap(mut self, envmap: image::Image) -> Self {
        self.envmap = Some(EnvMap::Sphere(envmap));
        self
    }

    pub fn set_rect_envmap(mut self, envmap: image::Image) -> Self {
        self.envmap = Some(EnvMap::Rect(envmap));
        self
    }

    pub fn envmap_dir(&self, dir: &V3) -> RGB {
        use EnvMap::*;
        let dir = dir.normalize();
        match &self.envmap {
            Some(Sphere(image)) => {
                let r = std::f32::consts::FRAC_1_PI * dir[2].acos()
                    / (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
                let u = dir[0] * r;
                let v = dir[1] * r;
                *image.at_uv(u, v)
            }
            Some(Rect(image)) => {
                let v = dir[1];
                let u = dir[0].atan2(-dir[2]) * std::f32::consts::FRAC_1_PI;
                *image.at_uv(u, v)
            }
            _ => RGB::all(0.0),
        }
    }

    pub fn sample_light<R: ?Sized>(&self, rng: &mut R) -> Option<pdf::PdfSample<(P3, V3, RGB)>>
    where
        R: Rng,
    {
        use pdf::*;

        self.lights.choose_pdf(rng).map(|ix| {
            ix.and_then(|ix| {
                let obj = &self.bvh.objects()[*ix];
                let e = obj.emission.unwrap();
                obj.shape.sample_surface(rng).map(|(p, n)| (p, n, e))
            })
        })
    }

    pub fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<object::ObjectHit> {
        self.bvh.test_hit(ray, tnear, tfar)
    }

    pub fn visible(&self, x: P3, y: P3) -> bool {
        let r = y - x;
        let dist = r.norm();
        let ray = Ray::new(x, r / dist);
        self.test_hit(&ray, 1e-3, dist - 1e-3).is_none()
    }
}
