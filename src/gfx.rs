//! Graphics rendering code.

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, RenderTarget};
use std::error::Error;

/// Width of canvas in pixels.
pub const CANVAS_WIDTH: u32 = 640;

/// Height of canvas in pixels.
pub const CANVAS_HEIGHT: u32 = 640;

/// An RGB color where the channel values are floating point values between
/// 0.0 and 1.0, inclusive.
#[derive(Clone, Copy, Debug)]
pub struct ColorF32 {
    r: f32,
    g: f32,
    b: f32,
}

/// Allows for iterating through a sequence of interpolations between two
/// values.
#[derive(Debug)]
struct Interpolation {
    i_end: i32,
    i: i32,
    d: f32,
    a: f32,
}

/// The result of a rendering function.
pub type RenderResult = Result<(), Box<dyn Error>>;

impl ColorF32 {
    pub const fn new(r: f32, b: f32, g: f32) -> Self {
        Self { r, g, b }
    }
    
    pub fn rgb(self) -> (f32, f32, f32) {
        (self.r, self.g, self.b)
    }
    
    pub const RED: Self = Self::new(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0);
    pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::new(0.0, 1.0, 1.0);
}

impl Interpolation {
    fn new(i0: i32, d0: i32, i1: i32, d1: i32) -> Self {
        let i_end = i1;
        let i = i0;
        let d = d0 as f32;
        let a = ((d1 - d0) as f32) / ((i1 - i0) as f32);
        Interpolation { i_end, i, d, a }
    }

    //  fn new_by_y(p0: Vertex, p1: Vertex) -> Self {
    //      Interpolation::new(p0.y, p0.x, p1.y, p1.x)
    //  }
}

impl Iterator for Interpolation {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i <= self.i_end {
            let result = (self.i, self.d as i32);
            self.i += 1;
            self.d += self.a;
            Some(result)
        } else {
            None
        }
    }
}

pub fn create_color_sdl(r: f32, g: f32, b: f32) -> Color {
   let r = 255.0 * r;
   let g = 255.0 * g;
   let b = 255.0 * b;
   Color::RGB(r as u8, g as u8, b as u8)
}

pub fn draw_line<T>(canvas: &mut Canvas<T>, p0: Point, p1: Point) -> RenderResult
where
    T: RenderTarget,
{
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    if dx.abs() > dy.abs() {
        // Line is oriented horizontally.
        let interpolation = if p0.x > p1.x {
            Interpolation::new(p1.x, p1.y, p0.x, p0.y)
        } else {
            Interpolation::new(p0.x, p0.y, p1.x, p1.y)
        };

        for (x, y) in interpolation {
            put_pixel(canvas, Point::new(x, y))?;
        }
    } else {
        // Line is oriented vertically.
        let interpolation = if p0.y > p1.y {
            Interpolation::new(p1.y, p1.x, p0.y, p0.x)
        } else {
            Interpolation::new(p0.y, p0.x, p1.y, p1.x)
        };

        for (y, x) in interpolation {
            put_pixel(canvas, Point::new(x, y))?;
        }
    }

    Ok(())
}

pub fn draw_wireframe_triangle<T>(
    canvas: &mut Canvas<T>,
    p0: Point,
    p1: Point,
    p2: Point,
) -> RenderResult
where
    T: RenderTarget,
{
    draw_line(canvas, p0, p1)?;
    draw_line(canvas, p1, p2)?;
    draw_line(canvas, p2, p0)?;

    Ok(())
}

/// Converts a point from plane space to canvas space. Plane space is like
/// canvas space, except the origin is in the middle of the canvas.
pub fn plane_to_canvas(p: Point) -> Point {
    Point::new(
        (CANVAS_WIDTH as i32) / 2 + p.x,
        (CANVAS_HEIGHT as i32) / 2 - p.y,
    )
}

/// Draws a point in plane space on the canvas.
pub fn put_pixel<T>(canvas: &mut Canvas<T>, p: Point) -> RenderResult
where
    T: RenderTarget,
{
    let p = plane_to_canvas(p);
    canvas.draw_point(p)?;

    Ok(())
}
