use crate::*;
use rand::prelude::Rng;

#[derive(Debug, Clone)]
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

    pub fn inside(&self, p: &P3) -> bool {
        for i in 0..3 {
            if !(self.mins[i] <= p[i] && p[i] <= self.maxs[i]) {
                return false;
            }
        }
        true
    }

    pub fn min_distance_from(&self, p: &P3) -> f32 {
        if self.inside(p) {
            return 0.0;
        }
        let to_mins = p - self.mins;
        let d_mins = to_mins[to_mins.iamin()];
        let to_maxs = p - self.maxs;
        let d_maxs = to_maxs[to_maxs.iamin()];
        d_mins.min(d_maxs)
    }

    pub fn max_distance_from(&self, p: &P3) -> f32 {
        self.iter_vertices()
            .map(|q| (p - q).norm())
            .fold(0.0, |x, y| x.max(y))
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

    pub fn include(&self, p: &P3) -> Self {
        self.merge(&AABB::around(p))
    }

    pub fn include_nomargin(&self, p: &P3) -> Self {
        self.merge(&AABB::single_point(p))
    }

    pub fn around(p: &P3) -> Self {
        AABB {
            mins: p - V3::new(0.1, 0.1, 0.1),
            maxs: p + V3::new(0.1, 0.1, 0.1),
        }
    }

    pub fn single_point(p: &P3) -> Self {
        AABB { mins: *p, maxs: *p }
    }

    pub fn center(&self) -> P3 {
        self.mins + self.diag() / 2.0
    }

    pub fn diag(&self) -> V3 {
        self.maxs - self.mins
    }

    pub fn longest_axis(&self) -> usize {
        self.diag().iamax()
    }

    pub fn iter_vertices<'a>(&'a self) -> impl Iterator<Item = P3> + 'a {
        let diag_vertices = [self.mins, self.maxs];
        (0..8).map(move |i| {
            let x = i ^ 2;
            let y = (i >> 1) ^ 1;
            let z = (i >> 2) ^ 1;
            let mut v = P3::origin();
            v[0] = diag_vertices[x][0];
            v[1] = diag_vertices[y][1];
            v[2] = diag_vertices[z][2];
            v
        })
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
                if dir > 0.0 {
                    if clip_near < min {
                        tnear = (min - origin) / dir;
                    }
                    if max < clip_far {
                        tfar = (max - origin) / dir;
                    }
                } else {
                    if max < clip_near {
                        tnear = (max - origin) / dir;
                    }
                    if clip_far < min {
                        tfar = (min - origin) / dir;
                    }
                }
            }
        }
        Some((tnear, tfar))
    }
}

trait ShapeImpl {
    fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit>;
    fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
    where
        R: Rng;
    fn sample_surface_pdf(&self, pos: &P3) -> f32;
    fn aabb(&self) -> AABB;
    fn area(&self) -> f32;
}

pub mod shapes {
    use super::*;
    use rand::prelude::*;

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
            let gx = pick_orthogonal(&gnorm);
            Hit {
                dist,
                pos,
                gnorm,
                gx,
            }
        }
    }

    impl ShapeImpl for Sphere {
        fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit> {
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

        fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
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

        fn sample_surface_pdf(&self, _pos: &P3) -> f32 {
            1.0 / self.area()
        }

        fn aabb(&self) -> AABB {
            AABB {
                mins: self.center - V3::new(1.0, 1.0, 1.0) * self.radius - V3::new(0.1, 0.1, 0.1),
                maxs: self.center + V3::new(1.0, 1.0, 1.0) * self.radius + V3::new(0.1, 0.1, 0.1),
            }
        }

        fn area(&self) -> f32 {
            4.0 * std::f32::consts::PI * self.radius * self.radius
        }
    }

    #[derive(Clone)]
    pub struct Triangle {
        abc: [V3; 3],
        lc: LocalCoord,
        aabb: AABB,
        area: f32,
    }

    impl Triangle {
        pub fn new(vs: [P3; 3]) -> Self {
            let center = (vs[0].coords + vs[1].coords + vs[2].coords) / 3.0;
            let center = P3 { coords: center };
            let n = {
                let ab = vs[1] - vs[0];
                let ac = vs[2] - vs[0];
                ab.cross(&ac)
            };
            let lc = LocalCoord::new_zx(&center, &n, &(vs[0] - center));
            let aabb = AABB::around(&vs[0]).include(&vs[1]).include(&vs[2]);
            Triangle {
                abc: [
                    lc.w2l() * (vs[0] - center),
                    lc.w2l() * (vs[1] - center),
                    lc.w2l() * (vs[2] - center),
                ],
                lc,
                aabb,
                area: n.norm() / 2.0,
            }
        }

        pub fn normal(&self) -> V3 {
            self.lc.w()
        }
    }

    impl ShapeImpl for Triangle {
        fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit> {
            //dbg!(ray);
            let ray_local = self.lc.w2l() * ray.clone();
            let dir_z = ray_local.dir[2];
            let clip_near = ray_local.origin[2] + dir_z * tnear;
            let clip_far = ray_local.origin[2] + dir_z * tfar;
            let clip_min = clip_near.min(clip_far);
            let clip_max = clip_near.max(clip_far);

            if clip_min <= 0.0 && 0.0 <= clip_max {
                let dist = -ray_local.origin[2] / dir_z;
                let pos_local = ray_local.origin + dist * ray_local.dir;
                let ca = (self.abc[1] - self.abc[0]).cross(&(pos_local.coords - self.abc[0]))[2];
                let cb = (self.abc[2] - self.abc[1]).cross(&(pos_local.coords - self.abc[1]))[2];
                let cc = (self.abc[0] - self.abc[2]).cross(&(pos_local.coords - self.abc[2]))[2];
                if (ca >= 0.0 && cb >= 0.0 && cc >= 0.0) || (ca <= 0.0 && cb <= 0.0 && cc <= 0.0) {
                    Some(Hit {
                        dist,
                        pos: self.lc.l2w() * pos_local,
                        gnorm: self.normal(),
                        gx: self.lc.u(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }

        fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
        where
            R: Rng,
        {
            use rand::distributions::Uniform;
            let mut s = Uniform::<f32>::new(0.0, 1.0).sample(rng);
            let mut t = Uniform::<f32>::new(0.0, 1.0).sample(rng);
            if s + t > 1.0 {
                s = 1.0 - s;
                t = 1.0 - t;
            }
            let p_local =
                P3::origin() + self.abc[0] * s + self.abc[1] * t + self.abc[2] * (1.0 - s - t);
            pdf::PdfSample {
                value: (self.lc.l2w() * p_local, self.normal()),
                pdf: 1.0 / self.area,
            }
        }

        fn sample_surface_pdf(&self, _pos: &P3) -> f32 {
            1.0 / self.area()
        }

        fn aabb(&self) -> AABB {
            self.aabb.clone()
        }

        fn area(&self) -> f32 {
            self.area
        }
    }

    #[derive(Clone)]
    pub struct Parallelogram(Triangle, Triangle);
    impl Parallelogram {
        pub fn new(a: &P3, b: &P3, d: &P3) -> Self {
            let c = d + (b - a);
            let t1 = Triangle::new([*a, *b, *d]);
            let t2 = Triangle::new([*d, *b, c]);
            Parallelogram(t1, t2)
        }

        pub fn new_rectangle(center: &P3, half_edge_1: &V3, half_edge_2: &V3) -> Self {
            let a = center + half_edge_1 + half_edge_2;
            let b = center - half_edge_1 + half_edge_2;
            let d = center + half_edge_1 - half_edge_2;
            Self::new(&a, &b, &d)
        }
    }

    impl ShapeImpl for Parallelogram {
        fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit> {
            self.0
                .test_hit(ray, tnear, tfar)
                .or_else(|| self.1.test_hit(ray, tnear, tfar))
        }

        fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
        where
            R: Rng,
        {
            use crate::pdf::SliceRandomPdf;
            [&self.0, &self.1]
                .choose_pdf(rng)
                .unwrap()
                .and_then(|tri| tri.sample_surface(rng))
        }

        fn aabb(&self) -> AABB {
            self.0.aabb().merge(&self.1.aabb())
        }

        fn sample_surface_pdf(&self, _pos: &P3) -> f32 {
            1.0 / self.area()
        }

        fn area(&self) -> f32 {
            self.0.area * 2.0
        }
    }

    #[derive(Debug, Clone)]
    pub struct AARectangular(pub P3, pub P3);

    impl AARectangular {
        fn center(&self) -> P3 {
            P3 {
                coords: (self.0.coords + self.1.coords) / 2.0,
            }
        }

        fn normal_at(&self, p: P3) -> V3 {
            let p0 = p - self.0;
            let p1 = p - self.1;
            let i0 = p0.iamin();
            let i1 = p1.iamin();
            let i = if p0[i0].abs() < p1[i1].abs() { i0 } else { i1 };
            let mut n = V3::zeros();
            n[i] = 1.0;
            if (self.center() - p).dot(&n) > 0.0 {
                -n
            } else {
                n
            }
        }
    }

    impl ShapeImpl for AARectangular {
        fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit> {
            let test_box = AABB::new(&self.0, &self.1);
            let (near, far) = test_box.ray_intersect(ray, tnear, tfar)?;
            let dist = if tnear < near {
                Some(near)
            } else if far < tfar {
                Some(far)
            } else {
                None
            }?;
            let pos = ray.at(dist);
            let gnorm = self.normal_at(pos);
            let hit = Hit {
                dist,
                pos,
                gnorm,
                gx: pick_orthogonal(&gnorm),
            };
            Some(hit)
        }

        fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
        where
            R: Rng,
        {
            use crate::pdf::SliceRandomPdf;
            use rand::distributions::Uniform;
            [0 as usize, 1, 2, 3, 4, 5]
                .choose_pdf(rng)
                .unwrap()
                .and_then(|s| {
                    let axis = s / 2;
                    let side = s % 2;
                    let (p0, p1) = if side == 0 {
                        (self.0, self.1)
                    } else {
                        (self.1, self.0)
                    };
                    let mut dx = V3::zeros();
                    dx[axis] = p1[axis] - p0[axis];
                    let mut dy = V3::zeros();
                    dy[(axis + 1) % 3] = p1[(axis + 1) % 3] - p0[(axis + 1) % 3];

                    let u = Uniform::new(0.0, 1.0).sample(rng);
                    let v = Uniform::new(0.0, 1.0).sample(rng);
                    let p = p0 + dx * u + dy * v;

                    pdf::PdfSample {
                        value: (p, self.normal_at(p)),
                        pdf: 1.0 / (dx.norm() * dy.norm()),
                    }
                })
        }

        fn sample_surface_pdf(&self, _pos: &P3) -> f32 {
            unimplemented!()
        }

        fn aabb(&self) -> AABB {
            AABB::around(&self.0).include(&self.1)
        }

        fn area(&self) -> f32 {
            let dx = (self.0[0] - self.1[0]).abs();
            let dy = (self.0[1] - self.1[1]).abs();
            let dz = (self.0[2] - self.1[2]).abs();
            2.0 * (dx * dy + dy * dz + dz * dx)
        }
    }
}

pub enum Shape {
    Sphere(shapes::Sphere),
    Triangle(shapes::Triangle),
    Parallelogram(shapes::Parallelogram),
    AARectangular(shapes::AARectangular),
}

impl_wrap_from_many! {Shape, shapes, [Sphere, Triangle, Parallelogram, AARectangular]}

impl Shape {
    pub fn test_hit(&self, ray: &Ray, tnear: f32, tfar: f32) -> Option<Hit> {
        match self {
            Shape::Sphere(s) => s.test_hit(ray, tnear, tfar),
            Shape::Triangle(s) => s.test_hit(ray, tnear, tfar),
            Shape::Parallelogram(s) => s.test_hit(ray, tnear, tfar),
            Shape::AARectangular(s) => s.test_hit(ray, tnear, tfar),
        }
    }

    pub fn sample_surface<R: ?Sized>(&self, rng: &mut R) -> pdf::PdfSample<(P3, V3)>
    where
        R: Rng,
    {
        match self {
            Shape::Sphere(s) => s.sample_surface(rng),
            Shape::Triangle(s) => s.sample_surface(rng),
            Shape::Parallelogram(s) => s.sample_surface(rng),
            Shape::AARectangular(s) => s.sample_surface(rng),
        }
    }

    pub fn sample_surface_pdf(&self, pos: &P3) -> f32 {
        match self {
            Shape::Sphere(s) => s.sample_surface_pdf(pos),
            Shape::Triangle(s) => s.sample_surface_pdf(pos),
            Shape::Parallelogram(s) => s.sample_surface_pdf(pos),
            Shape::AARectangular(s) => s.sample_surface_pdf(pos),
        }
    }

    pub fn aabb(&self) -> AABB {
        match self {
            Shape::Sphere(s) => s.aabb(),
            Shape::Triangle(s) => s.aabb(),
            Shape::Parallelogram(s) => s.aabb(),
            Shape::AARectangular(s) => s.aabb(),
        }
    }

    pub fn area(&self) -> f32 {
        match self {
            Shape::Sphere(s) => s.area(),
            Shape::Triangle(s) => s.area(),
            Shape::Parallelogram(s) => s.area(),
            Shape::AARectangular(s) => s.area(),
        }
    }
}
