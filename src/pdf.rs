use crate::*;
use rand::prelude::*;
use std::ops::Div;

#[derive(Clone, Debug)]
pub struct PdfSample<T> {
    pub value: T,
    pub pdf: f32,
}

impl<T> PdfSample<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> PdfSample<U> {
        PdfSample {
            value: f(self.value),
            pdf: self.pdf,
        }
    }

    pub fn and_then<U, F: FnOnce(T) -> PdfSample<U>>(self, f: F) -> PdfSample<U> {
        let mut p = f(self.value);
        p.pdf *= self.pdf;
        p
    }
}

impl<T> PdfSample<T>
where
    T: Div<f32>,
{
    pub fn e(self) -> T::Output {
        self.value / self.pdf
    }
}

pub trait SliceRandomPdf: SliceRandom {
    fn choose_pdf<R: ?Sized>(&self, rng: &mut R) -> Option<PdfSample<&Self::Item>>
    where
        R: Rng;
    fn choose_pdf_mut<R: ?Sized>(&mut self, rng: &mut R) -> Option<PdfSample<&mut Self::Item>>
    where
        R: Rng;
}

impl<T> SliceRandomPdf for [T] {
    fn choose_pdf<R: ?Sized>(&self, rng: &mut R) -> Option<PdfSample<&Self::Item>>
    where
        R: Rng,
    {
        self.choose(rng).map(|value| PdfSample {
            value,
            pdf: 1.0 / self.len() as f32,
        })
    }
    fn choose_pdf_mut<R: ?Sized>(&mut self, rng: &mut R) -> Option<PdfSample<&mut Self::Item>>
    where
        R: Rng,
    {
        let len = self.len() as f32;
        self.choose_mut(rng).map(|value| PdfSample {
            value,
            pdf: 1.0 / len,
        })
    }
}

pub struct UniformUnitHemisphere {
    pub normal: V3,
    pub xvec: V3,
}

impl Distribution<PdfSample<V3>> for UniformUnitHemisphere {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PdfSample<V3> {
        use rand::distributions::Uniform;
        let yvec = self.normal.cross(&self.xvec);

        let u01 = Uniform::<f32>::new(0.0, 1.0);
        let upi = Uniform::<f32>::new(-std::f32::consts::PI, std::f32::consts::PI);
        let z = u01.sample(rng);
        let theta = upi.sample(rng);
        let r = (1.0 - z * z).sqrt();
        let x = r * theta.cos();
        let y = r * theta.sin();

        PdfSample {
            value: x * self.xvec + y * yvec + z * self.normal,
            pdf: std::f32::consts::FRAC_1_PI / 2.0,
        }
    }
}

pub struct RandomBool {
    pub chance: f32,
}

impl Distribution<PdfSample<bool>> for RandomBool {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PdfSample<bool> {
        use rand::distributions::Uniform;
        let c = self.chance.min(1.0).max(0.0);
        let x = Uniform::<f32>::new(0.0, 1.0).sample(rng);
        let b = x < c;
        PdfSample {
            value: b,
            pdf: if b { c } else { 1.0 - c },
        }
    }
}
