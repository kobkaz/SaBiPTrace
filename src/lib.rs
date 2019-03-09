use nalgebra::{Point3, Vector3};

pub type P3 = Point3<f32>;
pub type V3 = Vector3<f32>;

pub mod image;
pub mod ray {
    use crate::*;
    #[derive(Clone, Debug)]
    pub struct Ray {
        pub origin: P3,
        pub dir: V3,
    }

    impl Ray {
        pub fn new(origin: P3, dir: V3) -> Self {
            Ray { origin, dir }
        }
    }
}
pub mod material;
pub mod object;
pub mod pdf;
pub mod rgb;
pub mod shape;
