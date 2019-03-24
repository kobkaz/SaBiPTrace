use crate::*;
use log::*;
use rand::prelude::*;

pub struct ReverseSampleResult {
    pub u: f32,
    pub v: f32,
    pub lens_point: P3,
    pub measure_conv: f32,
}

pub trait Camera {
    fn film_width(&self) -> f32;
    fn sample_ray<R: Rng + ?Sized>(&self, u: f32, v: f32, rng: &mut R) -> Ray;
    fn sample_film_uv<R: Rng + ?Sized>(&self, p: &P3, rng: &mut R) -> Option<ReverseSampleResult>;
}

#[derive(Clone)]
pub struct PinHole {
    lc: LocalCoord,
    film_width: f32,
    hole_radius: Option<f32>,
}

impl PinHole {
    pub fn new(
        origin: P3,
        view_at: P3,
        view_up: V3,
        fov_degree: f32,
        hole_radius: Option<f32>,
    ) -> Self {
        let lc = LocalCoord::new_zy(&origin, &(origin - view_at), &view_up);
        let fov_rad = fov_degree * std::f32::consts::PI / 180.0;
        let half_tan = (fov_rad / 2.0).tan();
        PinHole {
            lc,
            film_width: 2.0 * half_tan,
            hole_radius,
        }
    }
}

impl Camera for PinHole {
    fn film_width(&self) -> f32 {
        self.film_width
    }

    fn sample_ray<R: Rng + ?Sized>(&self, u: f32, v: f32, rng: &mut R) -> Ray {
        use rand::distributions::Uniform;
        let origin = if let Some(radius) = self.hole_radius {
            let theta = Uniform::new(-std::f32::consts::PI, std::f32::consts::PI).sample(rng);
            let r = radius * Uniform::new(0.0f32, 1.0).sample(rng).sqrt();
            P3::new(r * theta.cos(), r * theta.sin(), 0.0)
        } else {
            P3::origin()
        };

        self.lc.l2w()
            * Ray::new(
                origin,
                V3::new(self.film_width * u, self.film_width * v, -1.0).normalize(),
            )
    }
    fn sample_film_uv<R: Rng + ?Sized>(&self, p: &P3, rng: &mut R) -> Option<ReverseSampleResult> {
        use rand::distributions::Uniform;
        let p_local = self.lc.w2l() * p;
        let hole_point_local = if let Some(radius) = self.hole_radius {
            let theta = Uniform::new(-std::f32::consts::PI, std::f32::consts::PI).sample(rng);
            let r = radius * Uniform::new(0.0f32, 1.0).sample(rng).sqrt();
            P3::new(r * theta.cos(), r * theta.sin(), 0.0)
        } else {
            P3::origin()
        };

        if p_local[2] >= 0.0 {
            None
        } else {
            let film_point_local =
                hole_point_local + (p_local - hole_point_local) / p_local[2].abs();
            let film_to_hole = hole_point_local - film_point_local;
            let sq_dist_fh = film_to_hole.norm_squared();
            let sq_dist_hp = (p_local - hole_point_local).norm_squared();
            let cos_theta = film_to_hole[2].abs() / sq_dist_fh.sqrt();

            Some(ReverseSampleResult {
                u: film_point_local[0] / self.film_width,
                v: film_point_local[1] / self.film_width,
                lens_point: self.lc.l2w() * hole_point_local,
                measure_conv: sq_dist_fh / sq_dist_hp / cos_theta,
            })
        }
    }
}

#[derive(Clone)]
pub struct ThinLens {
    lc: LocalCoord,
    radius: f32,
    focal_length: f32,
    film_distance: f32,
    film_width: f32,
}

impl ThinLens {
    pub fn new(
        origin: P3,
        view_at: P3,
        view_up: V3,
        radius: f32,
        focal_length: f32,
        film_distance: f32,
        film_width: f32,
    ) -> Self {
        assert!(focal_length < film_distance);
        assert!(0.0 < focal_length);
        let lc = LocalCoord::new_zy(&origin, &(origin - view_at), &view_up);
        let lens = ThinLens {
            lc,
            radius,
            focal_length,
            film_distance,
            film_width,
        };
        lens
    }
    pub fn new_with_focus_distance(
        origin: P3,
        view_at: P3,
        view_up: V3,
        radius: f32,
        f_value: f32,
        focus_distance: f32,
        fov_degree: f32,
    ) -> Self {
        assert!(radius > 0.0);
        assert!(f_value > 0.0);
        assert!(focus_distance > 0.0);
        assert!(fov_degree > 0.0);
        let focal_length = 2.0 * f_value * radius;
        let film_distance = 1.0 / (1.0 / focal_length - 1.0 / focus_distance);

        let fov_rad = fov_degree * std::f32::consts::PI / 180.0;
        let half_tan = (fov_rad / 2.0).tan();
        Self::new(
            origin,
            view_at,
            view_up,
            radius,
            focal_length,
            film_distance,
            2.0 * half_tan * film_distance,
        )
    }
}

impl Camera for ThinLens {
    fn film_width(&self) -> f32 {
        self.film_width
    }
    fn sample_ray<R: Rng + ?Sized>(&self, u: f32, v: f32, rng: &mut R) -> Ray {
        use rand::distributions::Uniform;
        let a = self.film_distance;
        let f = self.focal_length;
        let lens_point = {
            let theta = Uniform::new(-std::f32::consts::PI, std::f32::consts::PI).sample(rng);
            let r = self.radius * Uniform::new(0.0f32, 1.0).sample(rng).sqrt();
            P3::new(r * theta.cos(), r * theta.sin(), 0.0)
        };

        let ray_to = f / (a - f) * P3::new(u * self.film_width, v * self.film_width, -a);
        self.lc.l2w() * Ray::from_to(&lens_point, &ray_to)
    }
    fn sample_film_uv<R: Rng + ?Sized>(
        &self,
        _p: &P3,
        _rng: &mut R,
    ) -> Option<ReverseSampleResult> {
        unimplemented!();
    }
}
