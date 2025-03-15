//! Vector type implementations.

use std::ops::{Index, Mul, Neg};

/// A three dimensional vector.
#[derive(Copy, Clone, Debug)]
pub struct Vec3(pub [f32; 3]);

/// A four dimensional vector.
#[derive(Copy, Clone, Debug)]
pub struct Vec4(pub [f32; 4]);

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3([x, y, z])
    }
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4([x, y, z, w])
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        &self.0[i]
    }
}

impl Index<usize> for Vec4 {
    type Output = f32;

    fn index(&self, i: usize) -> &f32 {
        &self.0[i]
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Vec3 {
        Vec3::new(self[0] * rhs, self[1] * rhs, self[2] * rhs)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Vec4 {
        Vec4::new(self[0] * rhs, self[1] * rhs, self[2] * rhs, self[3] * rhs)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        Vec3::new(-self[0], -self[1], -self[2])
    }
}

impl Neg for Vec4 {
    type Output = Vec4;

    fn neg(self) -> Vec4 {
        Vec4::new(-self[0], -self[1], -self[2], -self[3])
    }
}
