use crate::math::vec::Vec4;

pub struct Plane {
    normal: Vec4,
    distance: f32,
}

impl Plane {
    pub fn new(front_direction: Vec4, distance: f32) -> Self {
        let normal = front_direction.normalize();
        Plane { normal, distance }
    }

    pub fn intersection(&self, v0: Vec4, v1: Vec4) -> Vec4 {
        let t = (-self.distance - self.normal.dot(v0)) / self.normal.dot(v1 - v0);
        v0 + (v1 - v0) * t
    }

    pub fn signed_distance(&self, v: Vec4) -> f32 {
        self.normal.dot(v) + self.distance
    }
}
