use crate::*;
use rand::prelude::*;

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

    pub fn lc(&self) -> LocalCoord {
        LocalCoord::new_zx(&self.pos, &self.gnorm, &self.gx)
    }
}

#[derive(Clone, Debug)]
pub struct AABB {
    pub mins: P3,
    pub maxs: P3,
}

impl AABB {
    pub fn new(a: &P3, b: &P3) -> Self {
        let mut mins = *a;
        let mut maxs = *b;
        for i in 0..3 {
            if mins[i] > maxs[i] {
                std::mem::swap(&mut mins[i], &mut maxs[i])
            }
        }
        AABB { mins, maxs }
    }

    pub fn merge(&self, another: &Self) -> Self {
        let mut mins = P3::origin();
        let mut maxs = P3::origin();
        for i in 0..3 {
            mins[i] = self.mins[i].min(another.mins[i]);
            maxs[i] = self.maxs[i].max(another.maxs[i]);
        }
        AABB { mins, maxs }
    }

    pub fn center(&self) -> P3 {
        self.mins + self.diag() / 2.0
    }

    pub fn diag(&self) -> V3 {
        self.maxs - self.mins
    }

    pub fn ray_intersect(&self, ray: &Ray, mut tnear: f32, mut tfar: f32) -> Option<(f32, f32)> {
        let mut axis = [0, 1, 2];
        axis.sort_by(|i, j| ray.dir[*j].abs().partial_cmp(&ray.dir[*i].abs()).unwrap());
        for i in axis.iter() {
            let i = *i;
            let origin = ray.origin[i];
            let dir = ray.dir[i];
            let clip_near = origin + dir * tnear;
            let clip_far = origin + dir * tfar;
            let clip_min = clip_near.min(clip_far);
            let clip_max = clip_near.max(clip_far);
            let min = self.mins[i];
            let max = self.maxs[i];
            if clip_max < min || max < clip_min {
                return None;
            } else if min <= clip_min && clip_max <= max {
                continue;
            } else {
                let clip_min = min.max(clip_min);
                let clip_max = max.min(clip_max);
                let t1 = (clip_min - origin) / dir;
                let t2 = (clip_max - origin) / dir;
                tnear = t1.min(t2);
                tfar = t1.max(t2);
            }
        }
        Some((tnear, tfar))
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub center: P3,
    pub radius: f32,
}

impl Sphere {
    fn make_hit(&self, ray: &Ray, dist: f32) -> Hit {
        let v = (ray.origin + ray.dir * dist - self.center).normalize();
        let pos = self.center + v * self.radius;
        let gnorm = v;
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

    pub fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit> {
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

    pub fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
    where
        R: Rng,
    {
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

    pub fn aabb(&self) -> AABB {
        AABB {
            mins: self.center - V3::new(1.0, 1.0, 1.0) * self.radius,
            maxs: self.center + V3::new(1.0, 1.0, 1.0) * self.radius,
        }
    }
}
