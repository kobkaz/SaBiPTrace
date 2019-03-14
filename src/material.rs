use crate::*;

use rand::prelude::*;

pub mod materials {
    use crate::*;
    use rand::prelude::*;

    pub trait MaterialImpl {
        fn sample_win<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
        where
            R: Rng;

        fn sample_win_cos<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
        where
            R: Rng
        {
            self.sample_win(wout_local, rng).map(|(win_local, bsdf, spec)|
                (win_local, bsdf * win_local[2].abs(), spec)
            )
        }

        fn bsdf(&self, win_local: &V3, wout_local: &V3) -> RGB;

        fn all_specular(&self) -> bool;
    }

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

    #[derive(Clone, Debug)]
    pub struct Mirror(pub RGB);

    impl MaterialImpl for Mirror {
        fn sample_win<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
        where
            R: Rng,
        {
            self.sample_win_cos(wout_local, rng).map(|(win_local, bsdf, spec)|
                (win_local, bsdf / win_local[2].abs(), spec)
            )
        }

        fn sample_win_cos<R: ?Sized>(&self, wout_local: &V3, _rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
        where
            R: Rng,
        {
            let mut dir = *wout_local;
            dir[0] *= -1.0;
            dir[1] *= -1.0;
            pdf::PdfSample {
                value: (dir.normalize(), self.0, true),
                pdf: 1.0,
            }
        }

        fn bsdf(&self, _win: &V3, _wout: &V3) -> RGB {
            RGB::all(0.0)
        }

        fn all_specular(&self) -> bool {
            true
        }
    }

}
#[derive(Clone, Debug)]
pub enum Material {
    Lambert(materials::Lambert),
    Mirror(materials::Mirror),
    Mix(f32, Box<Material>, Box<Material>),
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

    pub fn mix(r: f32, m1: Self, m2: Self) -> Self {
        Mix(r, Box::new(m1), Box::new(m2))
    }

    pub fn sample_win<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        match self {
            Lambert(m) => m.sample_win(wout_local, rng),
            Mirror(m) => m.sample_win(wout_local, rng),
            Mix(r, m1, m2) => {
                //TODO: MIS
                use rand::distributions::Uniform;
                if Uniform::new(0.0, 1.0).sample(rng) < *r {
                    m1.sample_win(wout_local, rng)
                } else {
                    m2.sample_win(wout_local, rng)
                }
            }
        }
    }

    pub fn sample_win_cos<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        match self {
            Lambert(m) => m.sample_win_cos(wout_local, rng),
            Mirror(m) => m.sample_win_cos(wout_local, rng),
            Mix(r, m1, m2) => {
                //TODO: MIS
                use rand::distributions::Uniform;
                if Uniform::new(0.0, 1.0).sample(rng) < *r {
                    m1.sample_win_cos(wout_local, rng)
                } else {
                    m2.sample_win_cos(wout_local, rng)
                }
            }
        }
    }

    pub fn bsdf(&self, win_local: &V3, wout_local: &V3) -> RGB {
        match self {
            Lambert(m) => m.bsdf(win_local, wout_local),
            Mirror(m) => m.bsdf(win_local, wout_local),
            Mix(r, m1, m2) => m1.bsdf(win_local, wout_local) * *r + m2.bsdf(win_local, wout_local) * (1.0 - r),
        }
    }

    pub fn all_specular(&self) -> bool {
        match self {
            Lambert(m) => m.all_specular(),
            Mirror(m) => m.all_specular(),
            Mix(_, m1, m2) => m1.all_specular() && m2.all_specular(),
        }
    }
}
