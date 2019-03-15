use crate::material::*;
use crate::*;
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct Lambert(pub RGB);

impl MaterialImpl for Lambert {
    fn sample_win<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        let sgn: f32 = if wout_local[2] > 0.0 { 1.0 } else { -1.0 };
        let bsdf = self.0 * std::f32::consts::FRAC_1_PI;
        let next_dir = pdf::CosUnitHemisphere {
            normal: sgn * V3::z(),
            xvec: V3::x(),
        };
        let next_dir = next_dir.sample(rng);
        pdf::PdfSample {
            value: (next_dir.value, bsdf, false),
            pdf: next_dir.pdf,
        }
    }

    fn bsdf(&self, win_local: &V3, wout_local: &V3) -> RGB {
        if win_local[2] * wout_local[2] > 0.0 {
            self.0 * std::f32::consts::FRAC_1_PI
        } else {
            RGB::all(0.0)
        }
    }

    fn all_specular(&self) -> bool {
        false
    }
}
