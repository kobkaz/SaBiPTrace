use crate::material::*;
use crate::pdf::*;

#[derive(Clone, Debug)]
pub struct Lambert(pub RGB);

impl MaterialImpl for Lambert {
    fn sample_win<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        let sgn: f32 = if wout_local[2] > 0.0 { 1.0 } else { -1.0 };
        let bsdf = self.0 * std::f32::consts::FRAC_1_PI;
        let next_dir = CosUnitHemisphere {
            normal: sgn * V3::z(),
            xvec: V3::x(),
        };
        let next_dir = next_dir.sample(rng);
        PdfSample {
            value: (next_dir.value, bsdf, false),
            pdf: next_dir.pdf,
        }
    }

    fn sample_win_pdf(&self, wout_local: &V3, win_local: &V3) -> f32 {
        let sgn: f32 = if wout_local[2] > 0.0 { 1.0 } else { -1.0 };
        let next_dir = CosUnitHemisphere {
            normal: sgn * V3::z(),
            xvec: V3::x(),
        };
        next_dir.pdf(win_local)
    }

    fn bsdf(&self, win_local: &V3, wout_local: &V3, specular_component: bool) -> RGB {
        if specular_component {
            RGB::all(0.0)
        } else {
            if win_local[2] * wout_local[2] > 0.0 {
                self.0 * std::f32::consts::FRAC_1_PI
            } else {
                RGB::all(0.0)
            }
        }
    }

    fn all_specular(&self) -> bool {
        false
    }
}
