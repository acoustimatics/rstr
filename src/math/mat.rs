use crate::math::vec::*;

#[derive(Debug)]
pub struct Mat3 {
    columns: [[f32; 3]; 3],
}

#[derive(Debug)]
pub struct Mat3x4 {
    columns: [[f32; 3]; 4],
}

#[derive(Debug)]
pub struct Mat4 {
    columns: [[f32; 4]; 4],
}

impl Mat3 {
    pub fn new(c0: [f32; 3], c1: [f32; 3], c2: [f32; 3]) -> Self {
        let columns = [c0, c1, c2];
        Self { columns }
    }

    pub fn mul_mat3x4(&self, m1: &Mat3x4) -> Mat3x4 {
        let a = &self.columns;
        let b = &m1.columns;
        let mut m = [[0.0; 3]; 4];
        for c in 0..4 {
            for r in 0..3 {
                m[c][r] = a[0][r] * b[c][0] + a[1][r] * b[c][1] + a[2][r] * b[c][2];
            }
        }

        Mat3x4 { columns: m }
    }

    pub fn mul_vec3(&self, v: Vec3) -> Vec3 {
        let m = &self.columns;
        let x = m[0][0] * v.x + m[1][0] * v.y + m[2][0] * v.z;
        let y = m[0][1] * v.x + m[1][1] * v.y + m[2][1] * v.z;
        let z = m[0][2] * v.x + m[1][2] * v.y + m[2][2] * v.z;
        Vec3::new(x, y, z)
    }
}

impl Mat3x4 {
    pub fn new(c0: [f32; 3], c1: [f32; 3], c2: [f32; 3], c3: [f32; 3]) -> Self {
        let columns = [c0, c1, c2, c3];
        Self { columns }
    }

    pub fn mul_mat4(&self, m1: &Mat4) -> Mat3x4 {
        let a = &self.columns;
        let b = &m1.columns;
        let mut m = [[0.0; 3]; 4];
        for a_row in 0..3 {
            for b_col in 0..4 {
                for i in 0..4 {
                    m[b_col][a_row] += a[i][a_row] * b[b_col][i];
                }
            }
        }
        Mat3x4 { columns: m }
    }

    pub fn mul_vec4(&self, v: Vec4) -> Vec3 {
        let a = &self.columns;
        let x = a[0][0] * v.x + a[1][0] * v.y + a[2][0] * v.z + a[3][0] * v.w;
        let y = a[0][1] * v.x + a[1][1] * v.y + a[2][1] * v.z + a[3][1] * v.w;
        let z = a[0][2] * v.x + a[1][2] * v.y + a[2][2] * v.z + a[3][2] * v.w;
        Vec3::new(x, y, z)
    }
}

impl Mat4 {
    pub fn new(c0: [f32; 4], c1: [f32; 4], c2: [f32; 4], c3: [f32; 4]) -> Self {
        let columns = [c0, c1, c2, c3];
        Self { columns }
    }

    pub fn mul(&self, m1: &Mat4) -> Mat4 {
        let a = &self.columns;
        let b = &m1.columns;
        let mut m = [[0.0; 4]; 4];
        for c in 0..4 {
            for r in 0..4 {
                m[c][r] = a[0][r] * b[c][0] + a[1][r] * b[c][1] + a[2][r] * b[c][2];
            }
        }
        Mat4 { columns: m }
    }

    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        let a = &self.columns;
        let x = a[0][0] * v.x + a[1][0] * v.y + a[2][0] * v.z + a[3][0] * v.w;
        let y = a[0][1] * v.x + a[1][1] * v.y + a[2][1] * v.z + a[3][1] * v.w;
        let z = a[0][2] * v.x + a[1][2] * v.y + a[2][2] * v.z + a[3][2] * v.w;
        let w = a[0][3] * v.x + a[1][3] * v.y + a[2][3] * v.z + a[3][3] * v.w;
        Vec4::new(x, y, z, w)
    }
}
