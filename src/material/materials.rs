use crate::*;
use rand::prelude::*;

pub trait MaterialImpl {
    fn sample_win<R: ?Sized>(
        &self,
        wout_local: &V3,
        rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng;

    fn sample_win_cos<R: ?Sized>(
        &self,
        wout_local: &V3,
        rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        self.sample_win(wout_local, rng)
            .map(|(win_local, bsdf, spec)| (win_local, bsdf * win_local[2].abs(), spec))
    }

    fn sample_win_pdf(&self, wout_local: &V3, win_local: &V3, specular_component: bool) -> f32;

    fn bsdf(&self, win_local: &V3, wout_local: &V3, specular_component: bool) -> RGB;

    fn bsdf_cos(&self, win_local: &V3, wout_local: &V3, specular_component: bool) -> RGB {
        self.bsdf(win_local, wout_local, specular_component) * win_local[2].abs()
    }

    fn all_specular(&self) -> bool;
    fn has_specular(&self) -> bool;
}

mod lambert;
pub use lambert::*;

mod mirror;
pub use mirror::*;

mod transparent;
pub use transparent::*;
