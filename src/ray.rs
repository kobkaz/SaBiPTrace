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
