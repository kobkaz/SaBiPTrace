use crate::*;

use rand::prelude::*;

pub mod materials;

#[derive(Clone, Debug)]
pub enum Material {
    Lambert(materials::Lambert),
    Mirror(materials::Mirror),
    Transparent(materials::Transparent),
    Mix(f32, Box<Material>, Box<Material>),
}
use materials::MaterialImpl;

impl_wrap_from_many! {Material, materials, [Lambert, Mirror, Transparent]}

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
            Transparent(m) => m.sample_win(wout_local, rng),
            Mix(_, m1, m2) => {
                use rand::distributions::Uniform;
                let m = if Uniform::new(0.0, 1.0).sample(rng) < 0.5 {
                    &m1
                } else {
                    &m2
                };
                let pdf::PdfSample {
                    value: (win_local, _, specular),
                    ..
                } = m.sample_win(wout_local, rng);
                pdf::PdfSample {
                    value: (
                        win_local,
                        self.bsdf(wout_local, &win_local, specular),
                        specular,
                    ),
                    pdf: self.sample_win_pdf(wout_local, &win_local, specular),
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
            Transparent(m) => m.sample_win_cos(wout_local, rng),
            Mix(_, m1, m2) => {
                use rand::distributions::Uniform;
                let m = if Uniform::new(0.0, 1.0).sample(rng) < 0.5 {
                    &m1
                } else {
                    &m2
                };
                let pdf::PdfSample {
                    value: (win_local, _, specular),
                    ..
                } = m.sample_win_cos(wout_local, rng);
                pdf::PdfSample {
                    value: (
                        win_local,
                        self.bsdf_cos(&win_local, &wout_local, specular),
                        specular,
                    ),
                    pdf: self.sample_win_pdf(wout_local, &win_local, specular),
                }
            }
        }
    }

    pub fn sample_win_pdf(&self, wout_local: &V3, win_local: &V3, specular_component: bool) -> f32 {
        match self {
            Lambert(m) => m.sample_win_pdf(wout_local, win_local, specular_component),
            Mirror(m) => m.sample_win_pdf(wout_local, win_local, specular_component),
            Transparent(m) => m.sample_win_pdf(wout_local, win_local, specular_component),
            Mix(_r, m1, m2) => {
                (m1.sample_win_pdf(wout_local, win_local, specular_component)
                    + m2.sample_win_pdf(wout_local, win_local, specular_component))
                    / 2.0
            }
        }
    }

    //'specular_component' switches bsdf component to calculate
    //when calcuate for specular components, it assumes that win_local and wout_local are oriented
    //to have delta bsdf
    pub fn bsdf(&self, win_local: &V3, wout_local: &V3, specular_component: bool) -> RGB {
        match self {
            Lambert(m) => m.bsdf(win_local, wout_local, specular_component),
            Mirror(m) => m.bsdf(win_local, wout_local, specular_component),
            Transparent(m) => m.bsdf(win_local, wout_local, specular_component),
            Mix(r, m1, m2) => {
                m1.bsdf(win_local, wout_local, specular_component) * *r
                    + m2.bsdf(win_local, wout_local, specular_component) * (1.0 - r)
            }
        }
    }

    pub fn bsdf_cos(&self, win_local: &V3, wout_local: &V3, specular_component: bool) -> RGB {
        match self {
            Lambert(m) => m.bsdf_cos(win_local, wout_local, specular_component),
            Mirror(m) => m.bsdf_cos(win_local, wout_local, specular_component),
            Transparent(m) => m.bsdf_cos(win_local, wout_local, specular_component),
            Mix(r, m1, m2) => {
                m1.bsdf_cos(win_local, wout_local, specular_component) * *r
                    + m2.bsdf_cos(win_local, wout_local, specular_component) * (1.0 - r)
            }
        }
    }

    pub fn all_specular(&self) -> bool {
        match self {
            Lambert(m) => m.all_specular(),
            Mirror(m) => m.all_specular(),
            Transparent(m) => m.all_specular(),
            Mix(_, m1, m2) => m1.all_specular() && m2.all_specular(),
        }
    }

    pub fn has_specular(&self) -> bool {
        match self {
            Lambert(m) => m.has_specular(),
            Mirror(m) => m.has_specular(),
            Transparent(m) => m.has_specular(),
            Mix(_, m1, m2) => m1.has_specular() || m2.has_specular(),
        }
    }
}
