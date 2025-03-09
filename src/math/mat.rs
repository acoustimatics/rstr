//! Matrix type implementations.

use std::ops::Mul;

use crate::math::vec::*;

/// A 3x3 column major matrix.
#[derive(Copy, Clone, Debug)]
pub struct Mat3(pub [[f32; 3]; 3]);

/// A 3x4 column major matrix.
#[derive(Copy, Clone, Debug)]
pub struct Mat3x4(pub [[f32; 3]; 4]);

/// A 4x4 column major matrix.
#[derive(Copy, Clone, Debug)]
pub struct Mat4(pub [[f32; 4]; 4]);

/// Matrix multiplication.
macro_rules! mul {
    ($m1:expr, $m2:expr, $n_rows:expr, $n_cols:expr, $n_inner:expr, $ty:ident) => {{
        let mut m = [[0.0; $n_rows]; $n_cols];
        for r in 0..$n_rows {
            for c in 0..$n_cols {
                for i in 0..$n_inner {
                    m[c][r] += $m1.0[i][r] * $m2.0[c][i];
                }
            }
        }
        $ty(m)
    }};

    ($m:expr, $v: expr, $n_rows:expr, $n_cols:expr, $ty:ident) => {{
        let mut v = [0.0; $n_rows];
        for r in 0..$n_rows {
            for c in 0..$n_cols {
                v[r] += $m.0[c][r] * $v.0[c];
            }
        }
        $ty(v)
    }};
}

impl Mul<Mat3x4> for Mat3 {
    type Output = Mat3x4;

    fn mul(self, rhs: Mat3x4) -> Mat3x4 {
        mul!(self, rhs, 3, 4, 3, Mat3x4)
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        mul!(self, rhs, 3, 3, Vec3)
    }
}

impl Mul<Mat4> for Mat3x4 {
    type Output = Mat3x4;

    fn mul(self, rhs: Mat4) -> Mat3x4 {
        mul!(self, rhs, 3, 4, 4, Mat3x4)
    }
}

impl Mul<Vec4> for Mat3x4 {
    type Output = Vec3;

    fn mul(self, rhs: Vec4) -> Vec3 {
        mul!(self, rhs, 3, 4, Vec3)
    }
}

impl Mul for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        mul!(self, rhs, 4, 4, 4, Mat4)
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        mul!(self, rhs, 4, 4, Vec4)
    }
}
