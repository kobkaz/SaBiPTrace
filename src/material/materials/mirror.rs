use crate::material::*;

#[derive(Clone, Debug)]
pub struct Mirror(pub RGB);

impl MaterialImpl for Mirror {
    fn sample_win<R: ?Sized>(&self, wout_local: &V3, rng: &mut R) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        self.sample_win_cos(wout_local, rng)
            .map(|(win_local, bsdf, spec)| (win_local, bsdf / win_local[2].abs(), spec))
    }

    fn sample_win_cos<R: ?Sized>(
        &self,
        wout_local: &V3,
        _rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB, bool)>
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
