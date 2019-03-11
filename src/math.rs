use nalgebra::*;
pub type P3 = Point3<f32>;
pub type V3 = Vector3<f32>;
pub type M3 = Matrix3<f32>;

pub struct LocalCoord {
    l2w: Isometry3<f32>,
    w2l: Isometry3<f32>,
}

impl LocalCoord {
    pub fn new_zx(o: &P3, z: &V3, x_like: &V3) -> Self {
        Self::new_zy(o, z, &z.cross(x_like))
    }

    pub fn new_zy(o: &P3, z: &V3, y_like: &V3) -> Self {
        let tr = Translation3::from(o.coords);
        let rot = UnitQuaternion::face_towards(z, y_like);
        Self::from_iso(Isometry3::from_parts(tr, rot))
    }

    pub fn from_iso(l2w: Isometry3<f32>) -> Self {
        let w2l = l2w.inverse();
        LocalCoord { l2w, w2l }
    }

    //local to world
    pub fn l2w(&self) -> &Isometry3<f32> {
        &self.l2w
    }

    //world to local
    pub fn w2l(&self) -> &Isometry3<f32> {
        &self.w2l
    }
}
