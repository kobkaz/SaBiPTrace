use crate::*;

use rand::prelude::*;

pub mod materials;

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

    pub fn sample_win<R: ?Sized>(
        &self,
        wout_local: &V3,
        rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB, bool)>
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

    pub fn sample_win_cos<R: ?Sized>(
        &self,
        wout_local: &V3,
        rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB, bool)>
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
            Mix(r, m1, m2) => {
                m1.bsdf(win_local, wout_local) * *r + m2.bsdf(win_local, wout_local) * (1.0 - r)
            }
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
