use crate::*;
pub trait Accumulator {
    fn accum(&mut self, color: &(RGB, usize));
    fn merge(&mut self, another: &Self);
    fn is_finite(&self) -> bool;
    fn reset(&mut self);
    fn scale(&mut self, a: f32);
}

impl Accumulator for RGB {
    fn accum(&mut self, color: &(RGB, usize)) {
        *self += color.0
    }

    fn merge(&mut self, another: &Self) {
        *self += *another
    }

    fn is_finite(&self) -> bool {
        self.is_finite()
    }

    fn reset(&mut self) {
        *self = RGB::all(0.0);
    }

    fn scale(&mut self, a: f32) {
        *self *= a;
    }
}

impl Accumulator for Vec<RGB> {
    fn accum(&mut self, (color, len): &(RGB, usize)) {
        if *len < self.len() {
            self[*len] += *color;
        }
    }

    fn merge(&mut self, another: &Self) {
        let l = self.len().min(another.len());
        for i in 0..l {
            self[i] += another[i]
        }
    }

    fn is_finite(&self) -> bool {
        self.iter().all(RGB::is_finite)
    }

    fn reset(&mut self) {
        for v in self {
            v.reset()
        }
    }

    fn scale(&mut self, a: f32) {
        for v in self {
            v.scale(a)
        }
    }
}

impl<U, V> Accumulator for (U, V)
where
    U: Accumulator,
    V: Accumulator,
{
    fn accum(&mut self, color: &(RGB, usize)) {
        self.0.accum(color);
        self.1.accum(color);
    }

    fn merge(&mut self, another: &Self) {
        self.0.merge(&another.0);
        self.1.merge(&another.1);
    }

    fn is_finite(&self) -> bool {
        self.0.is_finite() && self.1.is_finite()
    }

    fn reset(&mut self) {
        self.0.reset();
        self.1.reset();
    }

    fn scale(&mut self, a: f32) {
        self.0.scale(a);
        self.1.scale(a);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pixel<T> {
    pub accum: T,
    pub samples: usize,
}

impl Pixel<RGB> {
    pub fn average(&self) -> RGB {
        self.accum / (self.samples as f32)
    }
}

impl<T: Accumulator> Accumulator for Pixel<T> {
    fn accum(&mut self, color: &(RGB, usize)) {
        self.accum.accum(color);
        self.samples += 1;
    }
    fn merge(&mut self, another: &Self) {
        self.accum.merge(&another.accum);
        self.samples += another.samples;
    }
    fn is_finite(&self) -> bool {
        self.accum.is_finite()
    }
    fn reset(&mut self) {
        self.accum.reset();
        self.samples = 0;
    }

    fn scale(&mut self, a: f32) {
        self.accum.scale(a)
    }
}
