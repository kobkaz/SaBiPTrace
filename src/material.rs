use crate::*;

use rand::prelude::*;

pub mod materials {
    use crate::*;
    use rand::prelude::*;

    pub trait MaterialImpl {
        fn sample_win<R: ?Sized>(&self, wout: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB)>
        where
            R: Rng;
        fn bsdf(&self, win: &V3, wout: &V3) -> RGB;

        fn is_specular(&self) -> bool;
    }

    #[derive(Clone, Debug)]
    pub struct Lambert(pub RGB);

    impl MaterialImpl for Lambert {
        fn sample_win<R: ?Sized>(&self, wout: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB)>
        where
            R: Rng,
        {
            let sgn: f32 = if wout[2] > 0.0 { 1.0 } else { -1.0 };
            let bsdf = self.0 * (std::f32::consts::FRAC_1_PI / 2.0);
            let next_dir = pdf::UniformUnitHemisphere {
                normal: sgn * V3::z(),
                xvec: V3::x(),
            };
            let next_dir = next_dir.sample(rng);
            pdf::PdfSample {
                value: (next_dir.value, bsdf),
                pdf: next_dir.pdf,
            }
        }

        fn bsdf(&self, win: &V3, wout: &V3) -> RGB {
            if win[2] * wout[2] > 0.0 {
                self.0 * (std::f32::consts::FRAC_1_PI / 2.0)
            } else {
                RGB::all(0.0)
            }
        }

        fn is_specular(&self) -> bool {
            false
        }
    }

    #[derive(Clone, Debug)]
    pub struct Mirror(pub RGB);

    impl MaterialImpl for Mirror {
        fn sample_win<R: ?Sized>(&self, wout: &V3, _rng: &mut R) -> pdf::PdfSample<(V3, RGB)>
        where
            R: Rng,
        {
            let mut dir = *wout;
            dir[0] *= -1.0;
            dir[1] *= -1.0;
            pdf::PdfSample {
                value: (dir.normalize(), self.0 / dir[2].abs()),
                pdf: 1.0,
            }
        }

        fn bsdf(&self, _win: &V3, _wout: &V3) -> RGB {
            RGB::all(0.0)
        }

        fn is_specular(&self) -> bool {
            true
        }
    }

}
#[derive(Clone, Debug)]
pub enum Material {
    Lambert(materials::Lambert),
    Mirror(materials::Mirror),
}
use materials::MaterialImpl;

impl_wrap_from_many! {Material, materials, [Lambert, Mirror]}

use Material::*;
impl Material {
    pub fn new_lambert(color: RGB) -> Self {
        Lambert(materials::Lambert(color))
    }

    pub fn new_mirror(color: RGB) -> Self {
        Mirror(materials::Mirror(color))
    }

    pub fn sample_win<R: ?Sized>(&self, wout: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB)>
    where
        R: Rng,
    {
        match self {
            Lambert(m) => m.sample_win(wout, rng),
            Mirror(m) => m.sample_win(wout, rng),
        }
    }

    pub fn bsdf(&self, win: &V3, wout: &V3) -> RGB {
        match self {
            Lambert(m) => m.bsdf(win, wout),
            Mirror(m) => m.bsdf(win, wout),
        }
    }

    pub fn is_specular(&self) -> bool {
        match self {
            Lambert(m) => m.is_specular(),
            Mirror(m) => m.is_specular(),
        }
    }
}
