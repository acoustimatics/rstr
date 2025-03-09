//! Functions that create transformation matrices.

use crate::math::mat::*;
use crate::math::vec::*;

/// Creates a perspective projection matrix.
pub fn perspective_projection(d: f32) -> Mat3x4 {
    Mat3x4([[d, 0.0, 0.0], [0.0, d, 0.0], [0.0, 0.0, 1.0], [0.0; 3]])
}

/// Creates a translation matrix for a given vector.
pub fn translation(t: Vec4) -> Mat4 {
    Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [t[0], t[1], t[2], 1.0],
    ])
}

/// Creates a rotation matrix based on the x, y, and z angles in radians in a
/// given vector. The rotations are applied in x, y, z order.
pub fn rotation(r: Vec3) -> Mat4 {
    let r_x = rotation_x(r[0]);
    let r_y = rotation_y(r[1]);
    let r_z = rotation_z(r[2]);
    r_z * r_y * r_x
}

/// Creates a rotation matrix about the x-axis for given angle in radians.
pub fn rotation_x(r: f32) -> Mat4 {
    let c = r.cos();
    let s = r.sin();
    Mat4([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, c, -s, 0.0],
        [0.0, s, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Creates a rotation matrix about the y-axis for given angle in radians.
pub fn rotation_y(r: f32) -> Mat4 {
    let c = r.cos();
    let s = r.sin();
    Mat4([
        [c, 0.0, s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Creates a rotation matrix about the z-axis for given angle in radians.
pub fn rotation_z(r: f32) -> Mat4 {
    let c = r.cos();
    let s = r.sin();
    Mat4([
        [c, s, 0.0, 0.0],
        [-s, c, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Creates a scaling matrix for the given factors.
pub fn scaling(s: Vec3) -> Mat4 {
    Mat4([
        [s[0], 0.0, 0.0, 0.0],
        [0.0, s[1], 0.0, 0.0],
        [0.0, 0.0, s[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

/// Creates a matrix that converts to viewport space to canvas (pixel display)
/// space.
pub fn viewport_to_canvas(
    canvas_width: u32,
    canvas_height: u32,
    viewport_width: f32,
    viewport_height: f32,
) -> Mat3 {
    let canvas_width = canvas_width as f32;
    let canvas_height = canvas_height as f32;
    Mat3([
        [canvas_width / viewport_width, 0.0, 0.0],
        [0.0, canvas_height / viewport_height, 0.0],
        [0.0, 0.0, 1.0],
    ])
}
