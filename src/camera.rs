use crate::*;
use log::*;
use rand::prelude::*;

pub trait Camera {
    fn sample_ray<R: Rng + ?Sized>(&self, u: f32, v: f32, rng: &mut R) -> Ray;
}

#[derive(Clone)]
pub struct PinHole {
    lc: LocalCoord,
    width: f32,
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
            width: 2.0 * half_tan,
            hole_radius,
        }
    }
}

impl Camera for PinHole {
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
                V3::new(self.width * u, self.width * v, -1.0).normalize(),
            )
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
}
