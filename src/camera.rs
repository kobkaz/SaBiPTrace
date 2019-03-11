use crate::*;

pub struct Camera {
    lc: LocalCoord,
    width: f32,
}

impl Camera {
    pub fn new(origin: P3, view_at: P3, view_up: V3, fov_degree: f32) -> Self {
        let lc = LocalCoord::new_zy(&origin, &(origin - view_at), &view_up);
        let fov_rad = fov_degree * std::f32::consts::PI / 180.0;
        let half_tan = (fov_rad / 2.0).tan();
        Camera {
            lc,
            width: 2.0 * half_tan,
        }
    }

    pub fn ray_to(&self, u: f32, v: f32) -> Ray {
        self.lc.l2w() * Ray::new(P3::origin(), V3::new(u, v, -1.0).normalize())
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}
