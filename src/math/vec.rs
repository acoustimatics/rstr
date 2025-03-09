//! Vector type implementations.

use std::ops::{Index, Mul, Neg};

/// A three dimensional vector.
#[derive(Copy, Clone, Debug)]
pub struct Vec3(pub [f32; 3]);

/// A four dimensional vector.
#[derive(Copy, Clone, Debug)]
pub struct Vec4(pub [f32; 4]);

/// Creates a three dimensional vector from three `f32` values.
#[macro_export]
macro_rules! vec3 {
    ($x:expr) => {
        Vec3([$x, $x, $x])
    };

    ($x:expr, $y:expr, $z:expr) => {
        Vec3([$x, $y, $z])
    };
}

/// Creates a four dimensional vector from four `f32` values.
#[macro_export]
macro_rules! vec4 {
    ($x:expr) => {
        Vec4([$x, $x, $x, $x])
    };

    ($x:expr, $y:expr, $z:expr, $w:expr) => {
        Vec4([$x, $y, $z, $w])
    };
}

macro_rules! sq {
    ($x:expr) => {
        $x * $x
    };
}

impl Vec3 {
    fn magnitude(self) -> f32 {
        (sq!(self[0]) + sq!(self[1]) + sq!(self[2])).sqrt()
    }

    fn normalize(self) -> Vec3 {
        self * (1.0 / self.magnitude())
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
        vec3!(self[0] * rhs, self[1] * rhs, self[2] * rhs)
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Vec4 {
        vec4!(self[0] * rhs, self[1] * rhs, self[2] * rhs, self[3] * rhs)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Vec3 {
        vec3!(-self[0], -self[1], -self[2])
    }
}

impl Neg for Vec4 {
    type Output = Vec4;

    fn neg(self) -> Vec4 {
        vec4!(-self[0], -self[1], -self[2], -self[3])
    }
}
