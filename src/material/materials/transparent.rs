use crate::material::*;

#[derive(Clone, Debug)]
pub struct Transparent {
    pub color: RGB,
    pub index: f32,
}

impl Transparent {
    fn fresnel_reflection(ix1: f32, cos1: f32, ix2: f32, cos2: f32) -> f32 {
        let p = (ix2 * cos1 - ix1 * cos2) / (ix2 * cos1 + ix1 * cos2);
        let s = (ix1 * cos1 - ix2 * cos2) / (ix1 * cos1 + ix2 * cos2);
        return (p * p + s * s) / 2.0;
    }
}

impl MaterialImpl for Transparent {
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
        rng: &mut R,
    ) -> pdf::PdfSample<(V3, RGB, bool)>
    where
        R: Rng,
    {
        use rand::distributions::*;
        let cos_out = wout_local[2];
        let sin_out = (1.0 - cos_out * cos_out).sqrt();
        let (index_in, index_out) = if cos_out > 0.0 {
            //println!("Light Out trace In");
            (self.index, 1.0)
        } else {
            //println!("Light In trace Out");
            (1.0, self.index)
        };
        let index_ratio = index_out / index_in;
        let sin_in = sin_out * index_ratio;

        if sin_in < 1.0 {
            let cos_in = (1.0 - sin_in * sin_in).sqrt();
            let c_ref = Self::fresnel_reflection(index_in, cos_in.abs(), index_out, cos_out.abs());
            if Uniform::new(0.0, 1.0).sample(rng) < c_ref {
                let win_local = V3::new(-wout_local[0], -wout_local[1], wout_local[2]);
                pdf::PdfSample {
                    value: (win_local.normalize(), self.color * c_ref, true),
                    pdf: c_ref,
                }
            } else {
                let c_trans = 1.0 - c_ref;
                let win_local = V3::new(
                    -wout_local[0] * index_ratio,
                    -wout_local[1] * index_ratio,
                    if cos_out > 0.0 { -cos_in } else { cos_in },
                );
                let norm = (win_local + wout_local).norm();
                pdf::PdfSample {
                    value: (win_local.normalize(), self.color * c_trans, true),
                    pdf: c_trans,
                }
            }
        } else {
            let win_local = V3::new(-wout_local[0], -wout_local[1], wout_local[2]);
            pdf::PdfSample {
                value: (win_local.normalize(), self.color, true),
                pdf: 1.0,
            }
        }
    }

    fn bsdf(&self, _win: &V3, _wout: &V3) -> RGB {
        RGB::all(0.0)
    }

    fn all_specular(&self) -> bool {
        true
    }
}
