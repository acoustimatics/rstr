use std::ops::Add;

pub struct Vec4([f32; 4]);

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self([x, y, z, w])
    }

    pub fn new_point(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z, 1.0)
    }

    pub fn x(&self) -> f32 {
        self.0[0]
    }

    pub fn y(&self) -> f32 {
        self.0[1]
    }

    pub fn z(&self) -> f32 {
        self.0[2]
    }

    pub fn w(&self) -> f32 {
        self.0[3]
    }
}

impl Add for &Vec4 {
    type Output = Vec4;

    fn add(self, other: &Vec4) -> Vec4 {
        Vec4::new(
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        )
    }
}
