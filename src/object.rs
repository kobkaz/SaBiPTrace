use crate::*;

fn merge_options<T, F: Fn(T, T) -> T>(x: Option<T>, y: Option<T>, merge: F) -> Option<T> {
    if let Some(x) = x {
        if let Some(y) = y {
            Some(merge(x, y))
        } else {
            Some(x)
        }
    } else {
        y
    }
}

pub struct ObjectHit {
    pub hit: shape::Hit,
    pub material: material::Material,
    pub emission: Option<rgb::RGB>,
}

impl ObjectHit {
    fn nearer_option(x: Option<Self>, y: Option<Self>) -> Option<Self> {
        merge_options(x, y, |x, y| if x.hit.dist < y.hit.dist { x } else { y })
    }
}

pub struct SimpleObject {
    pub shape: shape::Sphere,
    pub material: material::Material,
    pub emission: Option<rgb::RGB>,
}

pub struct ObjectList {
    pub objects: Vec<SimpleObject>,
}

impl ObjectList {
    pub fn new() -> Self {
        ObjectList { objects: vec![] }
    }

    pub fn test_hit(&self, ray: &ray::Ray, tnear: f32, mut tfar: f32) -> Option<ObjectHit> {
        let mut hit = None::<ObjectHit>;
        for o in self.objects.iter() {
            tfar = hit.as_ref().map_or(tfar, |h| h.hit.dist);
            let new_hit = o.shape.test_hit(ray, tnear, tfar).map(|h| ObjectHit {
                hit: h,
                material: o.material.clone(),
                emission: o.emission,
            });
            hit = ObjectHit::nearer_option(hit, new_hit);
        }
        hit
    }
    /*
    fn aabb(&self) -> shape::AABB {
        if self.objects.is_empty() {
            panic!("empty objects")
        }
        let mut bb = self.objects[0].aabb();
        for o in self.objects.iter() {
            bb = bb.union(&o.aabb())
        }
        bb
    }
    */
}
