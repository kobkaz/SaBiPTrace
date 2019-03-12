use crate::*;

use rand::prelude::*;

pub struct Scene {
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

    pub fn sample_light<R: ?Sized>(&self, rng: &mut R) -> Option<pdf::PdfSample<(P3, V3, RGB)>>
    where
        R: Rng,
    {
        use pdf::*;
        use rand::seq::SliceRandom;

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
