use crate::*;
use rand::prelude::*;
use rgb::RGB;

#[derive(Clone, Debug)]
pub enum Material {
    Lambert(RGB),
}

use Material::*;

impl Material {
    pub fn sample_win<R: Rng>(
        &self,
        normal: V3,
        xvec: V3,
        _wout: V3,
        rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB)> {
        let Lambert(color) = self;
        let bsdf = *color * (std::f32::consts::FRAC_1_PI / 2.0);
        let next_dir = pdf::UniformUnitHemisphere { normal, xvec };
        let next_dir = next_dir.sample(rng);
        pdf::PdfSample {
            value: (next_dir.value, bsdf),
            pdf: next_dir.pdf,
        }
    }

    pub fn bsdf(&self, normal: &V3, win: &V3, wout: &V3) -> RGB {
        let Lambert(color) = self;
        if normal.dot(&win) * normal.dot(&wout) > 0.0 {
            *color * (std::f32::consts::FRAC_1_PI / 2.0)
        } else {
            RGB::all(0.0)
        }
    }
}
