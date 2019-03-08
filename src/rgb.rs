use std::ops::{Add, Div, Mul, Sub};
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

use openexr::frame_buffer::PixelStruct;
use openexr::PixelType;
unsafe impl PixelStruct for RGB {
    fn channel_count() -> usize {
        3
    }
    fn channel(i: usize) -> (PixelType, usize) {
        use openexr::*;
        [
            (PixelType::FLOAT, 0),
            (PixelType::FLOAT, 4),
            (PixelType::FLOAT, 8),
        ][i]
    }
}

impl RGB {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
    pub fn all(x: f32) -> Self {
        Self::new(x, x, x)
    }
}

impl<'a> Add<&'a Self> for RGB {
    type Output = Self;
    fn add(self, rhs: &'a Self) -> Self {
        RGB {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl Add for RGB {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        self.add(&rhs)
    }
}

impl<'a> Sub<&'a Self> for RGB {
    type Output = Self;
    fn sub(self, rhs: &'a Self) -> Self {
        RGB {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl Sub for RGB {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self.sub(&rhs)
    }
}

impl<'a> Mul<&'a Self> for RGB {
    type Output = Self;
    fn mul(self, rhs: &'a Self) -> Self {
        RGB {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul for RGB {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.mul(&rhs)
    }
}

impl Mul<f32> for RGB {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        RGB {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl<'a> Div<&'a Self> for RGB {
    type Output = Self;
    fn div(self, rhs: &'a Self) -> Self {
        RGB {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl Div for RGB {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self.div(&rhs)
    }
}

impl Div<f32> for RGB {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        RGB {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}
