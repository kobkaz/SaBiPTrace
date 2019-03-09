use crate::*;
use rand::prelude::*;
use rgb::RGB;

#[derive(Clone)]
pub struct Hit {
    pub dist: f32,
    pub pos: P3,
    pub gnorm: V3,
    pub gx: V3,
}

impl Hit {
    pub fn g(&self, x: &P3, n: &V3) -> f32 {
        let r = x - self.pos;
        let sq_dist = r.norm_squared();

        (r.dot(n) * r.dot(&self.gnorm)).abs() / (sq_dist * sq_dist)
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub center: P3,
    pub radius: f32,
}

impl Sphere {
    fn make_hit(&self, ray: &ray::Ray, dist: f32) -> Hit {
        let pos = ray.origin + ray.dir * dist;
        let gnorm = (pos - self.center).normalize();
        let gx_approx = if gnorm[0].abs() < 0.5 {
            V3::new(1.0, 0.0, 0.0)
        } else {
            V3::new(0.0, 1.0, 0.0)
        };
        let gx = (gx_approx - gx_approx.dot(&gnorm) * gnorm).normalize();
        Hit {
            dist,
            pos,
            gnorm,
            gx,
        }
    }

    pub fn test_hit(&self, ray: &ray::Ray, tnear: f32, tfar: f32) -> Option<Hit> {
        if tnear > tfar {
            return None;
        }
        let rel_c = self.center - ray.origin;
        let d_oh = ray.dir.dot(&rel_c);
        let rel_h = ray.dir * d_oh;
        let d_ch_sq = (rel_c - rel_h).norm_squared();
        if d_ch_sq > self.radius * self.radius {
            None
        } else {
            let l = (self.radius * self.radius - d_ch_sq).sqrt();
            let tmin = d_oh - l;
            let tmax = d_oh + l;
            if tnear < tmin && tmin < tfar {
                Some(self.make_hit(ray, tmin))
            } else if tnear < tmax && tmax < tfar {
                Some(self.make_hit(ray, tmax))
            } else {
                None
            }
        }
    }

    pub fn sample_surface<R: Rng + Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)> {
        use rand::distributions::Uniform;
        let u1 = Uniform::<f32>::new(-1.0, 1.0);
        let upi = Uniform::<f32>::new(-std::f32::consts::PI, std::f32::consts::PI);
        let z = u1.sample(rng);
        let theta = upi.sample(rng);
        let r = (1.0 - z * z).sqrt();
        let x = r * theta.cos();
        let y = r * theta.sin();
        let n = V3::new(x, y, z);
        pdf::PdfSample {
            value: (self.center + self.radius * n, n),
            pdf: std::f32::consts::FRAC_1_PI / 4.0 / self.radius / self.radius,
        }
    }
    /*
    fn aabb(&self) -> AABB {
        AABB {
            mins: self.center - V3::new(1.0, 1.0, 1.0) * self.radius,
            maxs: self.center + V3::new(1.0, 1.0, 1.0) * self.radius,
        }
    }
    */
}
