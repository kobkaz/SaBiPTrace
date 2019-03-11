use crate::*;

pub struct Camera {
    e_u: V3,
    e_v: V3,
    e_w: V3,
    width: f32,
    origin: P3,
}

impl Camera {
    pub fn new(origin: P3, view_at: P3, view_up: V3, fov_degree: f32) -> Self {
        let e_w = (origin - view_at).normalize();
        let e_v = (view_up - e_w * (e_w.dot(&view_up))).normalize();
        let e_u = e_v.cross(&e_w);
        let fov_rad = fov_degree * std::f32::consts::PI / 180.0;
        let half_tan = (fov_rad / 2.0).tan();
        Camera {
            e_u,
            e_v,
            e_w,
            width: 2.0 * half_tan,
            origin,
        }
    }

    pub fn ray_to(&self, u: f32, v: f32) -> Ray {
        let ray_dir = (self.e_u * u + self.e_v * v - self.e_w).normalize();
        Ray::new(self.origin, ray_dir)
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}
