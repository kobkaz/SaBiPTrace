use crate::*;
use nalgebra::*;
use std::ops::Mul;

#[derive(Clone, Debug)]
pub struct Ray {
    pub origin: P3,
    pub dir: V3,
}

impl Ray {
    pub fn new(origin: P3, dir: V3) -> Self {
        Ray { origin, dir }
    }

    pub fn from_to(origin: &P3, toward: &P3) -> Self {
        Ray {
            origin: *origin,
            dir: (toward - origin).normalize(),
        }
    }

    pub fn new_from_origin(dir: V3) -> Self {
        Self::new(P3::origin(), dir)
    }

    pub fn at(&self, dist: f32) -> P3 {
        self.origin + dist * self.dir
    }
}

impl Mul<Ray> for &Isometry3<f32> {
    type Output = Ray;
    fn mul(self, ray: Ray) -> Ray {
        let origin = self * ray.origin;
        let dir = self * ray.dir;
        Ray { origin, dir }
    }
}

impl Mul<Ray> for Isometry3<f32> {
    type Output = Ray;
    fn mul(self, ray: Ray) -> Ray {
        (&self) * ray
    }
}
