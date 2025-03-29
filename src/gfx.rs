//! Graphics rendering code.

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, RenderTarget};
use std::error::Error;

/// Width of canvas in pixels.
pub const CANVAS_WIDTH: u32 = 640;

/// Height of canvas in pixels.
pub const CANVAS_HEIGHT: u32 = 640;

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

#[derive(Copy, Clone)]
pub struct Vertex {
    x: i32,
    y: i32,
    h: f32,
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

impl Vertex {
    fn new(x: i32, y: i32, h: f32) -> Self {
        Vertex { x, y, h }
    }

    fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }
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

fn draw_filled_triangle<T>(
    canvas: &mut Canvas<T>,
    p0: Vertex,
    p1: Vertex,
    p2: Vertex,
    color: Color,
) -> RenderResult
where
    T: RenderTarget,
{
    // Sort triangles from bottom to top.
    let (p0, p1) = if p1.y < p0.y { (p1, p0) } else { (p0, p1) };
    let (p0, p2) = if p2.y < p0.y { (p2, p0) } else { (p0, p2) };
    let (p1, p2) = if p2.y < p1.y { (p2, p1) } else { (p1, p2) };

    let x_slope_long = ((p2.x - p0.x) as f32) / ((p2.y - p0.y) as f32);
    let x_slope_short = ((p1.x - p0.x) as f32) / ((p1.y - p0.y) as f32);
    let h_slope_long = (p2.h - p0.h) / ((p2.y - p0.y) as f32);
    let h_slope_short = (p1.h - p0.h) / ((p1.y - p0.y) as f32);

    let mut x_long = p0.x as f32;
    let mut x_short = p0.x as f32;
    let mut h_long = p0.h;
    let mut h_short = p0.h;
    for y in p0.y..p1.y {
        draw_horizontal_line(
            canvas,
            x_long as i32,
            x_short as i32,
            y,
            h_long,
            h_short,
            color,
        )?;

        x_long += x_slope_long;
        x_short += x_slope_short;
        h_long += h_slope_long;
        h_short += h_slope_short;
    }

    let x_slope_short = ((p2.x - p1.x) as f32) / ((p2.y - p1.y) as f32);
    let mut x_short = p1.x as f32;
    for y in p1.y..=p2.y {
        draw_horizontal_line(
            canvas,
            x_long as i32,
            x_short as i32,
            y,
            h_long,
            h_short,
            color,
        )?;

        x_long += x_slope_long;
        x_short += x_slope_short;
        h_long += h_slope_long;
    }

    Ok(())
}

fn draw_horizontal_line<T>(
    canvas: &mut Canvas<T>,
    x0: i32,
    x1: i32,
    y: i32,
    h0: f32,
    h1: f32,
    color: Color,
) -> RenderResult
where
    T: RenderTarget,
{
    let (x0, h0, x1, h1) = if x0 > x1 {
        (x1, h1, x0, h0)
    } else {
        (x0, h0, x1, h1)
    };
    let h_slope = (h1 - h0) / ((x1 - x0) as f32);
    let mut h = h0;
    for x in x0..=x1 {
        let c = scale_color(color, h);
        canvas.set_draw_color(c);

        put_pixel(canvas, Point::new(x, y))?;

        h += h_slope;
    }

    Ok(())
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

fn scale_color(c: Color, h: f32) -> Color {
    let r = h * (c.r as f32);
    let g = h * (c.g as f32);
    let b = h * (c.b as f32);
    Color::RGB(r as u8, g as u8, b as u8)
}
