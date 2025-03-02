use crate::math::mat::*;
use crate::math::vec::*;

pub fn translation(v: Vec4) -> Mat4 {
    let c0 = [1.0, 0.0, 0.0, 0.0];
    let c1 = [0.0, 1.0, 0.0, 0.0];
    let c2 = [0.0, 0.0, 1.0, 0.0];
    let c3 = [v.x, v.y, v.z, 1.0];
    Mat4::new(c0, c1, c2, c3)
}

pub fn perspective_projection(d: f32) -> Mat3x4 {
    let c0 = [d, 0.0, 0.0];
    let c1 = [0.0, d, 0.0];
    let c2 = [0.0, 0.0, 1.0];
    let c3 = [0.0; 3];
    Mat3x4::new(c0, c1, c2, c3)
}

fn viewport_to_canvas(
    canvas_width: u32,
    canvas_height: u32,
    viewport_width: f32,
    viewport_height: f32,
) -> Mat3 {
    let canvas_width = canvas_width as f32;
    let canvas_height = canvas_height as f32;
    let c0 = [canvas_width / viewport_width, 0.0, 0.0];
    let c1 = [0.0, canvas_height / viewport_height, 0.0];
    let c2 = [0.0, 0.0, 1.0];
    Mat3::new(c0, c1, c2)
}
