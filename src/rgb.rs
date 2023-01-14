use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}


impl RGB {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
    pub fn all(x: f32) -> Self {
        Self::new(x, x, x)
    }
    pub fn max(&self) -> f32 {
        self.r.max(self.g).max(self.b)
    }

    pub fn is_finite(&self) -> bool {
        self.r.is_finite() && self.g.is_finite() && self.b.is_finite()
    }
}

impl Default for RGB {
    fn default() -> Self {
        RGB::all(0.0)
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

impl AddAssign for RGB {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
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

impl SubAssign for RGB {
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
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

impl MulAssign for RGB {
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
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

impl MulAssign<f32> for RGB {
    fn mul_assign(&mut self, rhs: f32) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
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

impl DivAssign for RGB {
    fn div_assign(&mut self, rhs: Self) {
        self.r /= rhs.r;
        self.g /= rhs.g;
        self.b /= rhs.b;
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

impl DivAssign<f32> for RGB {
    fn div_assign(&mut self, rhs: f32) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}

