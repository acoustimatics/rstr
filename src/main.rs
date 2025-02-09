use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, RenderTarget};
use std::error::Error;

const CANVAS_WIDTH: u32 = 640;
const CANVAS_HEIGHT: u32 = 640;
const VIEWPORT_WIDTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 1.0;
const D: f32 = 1.0;

#[derive(Debug)]
struct Interpolation {
    i_end: i32,
    i: i32,
    d: f32,
    a: f32,
}

#[derive(Copy, Clone)]
struct Vertex {
    x: i32,
    y: i32,
    h: f32,
}

struct Vector {
    x: f32,
    y: f32,
    z: f32,
}

impl Interpolation {
    fn new(i0: i32, d0: i32, i1: i32, d1: i32) -> Self {
        let i_end = i1;
        let i = i0;
        let d = d0 as f32;
        let a = ((d1 - d0) as f32) / ((i1 - i0) as f32);
        Interpolation { i_end, i, d, a }
    }

    fn new_by_y(p0: Vertex, p1: Vertex) -> Self {
        Interpolation::new(p0.y, p0.x, p1.y, p1.x)
    }
}

impl Vertex {
    fn new(x: i32, y: i32, h: f32) -> Self {
        Vertex { x, y, h }
    }

    fn to_point(self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl Vector {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vector { x, y, z }
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

fn scale_color(c: Color, h: f32) -> Color {
    let r = h * (c.r as f32);
    let g = h * (c.g as f32);
    let b = h * (c.b as f32);
    Color::RGB(r as u8, g as u8, b as u8)
}

fn plane_to_canvas(p: Point) -> Point {
    Point::new((CANVAS_WIDTH as i32) / 2 + p.x, (CANVAS_HEIGHT as i32) / 2 - p.y)
}

fn viewport_to_plane(v: Vector) -> Point {
    let x = v.x * (CANVAS_WIDTH as f32) / VIEWPORT_WIDTH;
    let y = v.y * (CANVAS_HEIGHT as f32) / VIEWPORT_HEIGHT;
    Point::new(x as i32, y as i32)
}

fn project(v: Vector) -> Vector {
    let x = v.x * D / v.z;
    let y = v.y * D / v.z;
    let z = D;
    Vector::new(x, y, z)
}

fn put_pixel<T>(canvas: &mut Canvas<T>, p: Point) -> Result<(), Box<dyn Error>>
where
    T: RenderTarget,
{
    let p = plane_to_canvas(p);
    canvas.draw_point(p)?;

    Ok(())
}

fn draw_line<T>(canvas: &mut Canvas<T>, p0: Point, p1: Point) -> Result<(), Box<dyn Error>>
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

fn draw_horizontal_line<T>(
    canvas: &mut Canvas<T>,
    x0: i32,
    x1: i32,
    y: i32,
    h0: f32,
    h1: f32,
    color: Color,
) -> Result<(), Box<dyn Error>>
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

fn draw_filled_triangle<T>(
    canvas: &mut Canvas<T>,
    p0: Vertex,
    p1: Vertex,
    p2: Vertex,
    color: Color,
) -> Result<(), Box<dyn Error>>
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
    let h_slope_short = dbg!((p1.h - p0.h) / ((p1.y - p0.y) as f32));

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

fn draw_wireframe_triangle<T>(
    canvas: &mut Canvas<T>,
    p0: Point,
    p1: Point,
    p2: Point,
) -> Result<(), Box<dyn Error>>
where
    T: RenderTarget,
{
    draw_line(canvas, p0, p1)?;
    draw_line(canvas, p1, p2)?;
    draw_line(canvas, p2, p0)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;

    let window = video_subsystem.window("rstr", CANVAS_WIDTH, CANVAS_HEIGHT).build()?;

    let mut canvas = window.into_canvas().build()?;

    canvas.set_draw_color(Color::RGB(248, 248, 255));
    canvas.clear();

    /*
    let c = Color::RGB(0, 139, 139);
    let p0 = Vertex::new(-200, -250, 1.0);
    let p1 = Vertex::new(200, 50, 0.5);
    let p2 = Vertex::new(20, 250, 0.25);
    draw_filled_triangle(&mut canvas, p0, p1, p2, c)?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    draw_wireframe_triangle(&mut canvas, p0.to_point(), p1.to_point(), p2.to_point())?;
    canvas.present();
    */

    // The four "front" vertices.
    let v0 = Vector::new(-2.0, -0.5, 5.0);
    let v1 = Vector::new(-2.0,  0.5, 5.0);
    let v2 = Vector::new(-1.0,  0.5, 5.0);
    let v3 = Vector::new(-1.0, -0.5, 5.0);

    // The four "back" vertices.
    let v4 = Vector::new(-2.0, -0.5, 6.0);
    let v5 = Vector::new(-2.0,  0.5, 6.0);
    let v6 = Vector::new(-1.0,  0.5, 6.0);
    let v7 = Vector::new(-1.0, -0.5, 6.0);

    // Project the vertices and convert them to plane space.
    let p0 = viewport_to_plane(project(v0));
    let p1 = viewport_to_plane(project(v1));
    let p2 = viewport_to_plane(project(v2));
    let p3 = viewport_to_plane(project(v3));
    let p4 = viewport_to_plane(project(v4));
    let p5 = viewport_to_plane(project(v5));
    let p6 = viewport_to_plane(project(v6));
    let p7 = viewport_to_plane(project(v7));

    // The front face.
    canvas.set_draw_color(Color::RGB(0, 0, 255));
    draw_line(&mut canvas, p0, p1)?;
    draw_line(&mut canvas, p1, p2)?;
    draw_line(&mut canvas, p2, p3)?;
    draw_line(&mut canvas, p3, p0)?;

    // The back face.
    canvas.set_draw_color(Color::RGB(255, 0, 0));
    draw_line(&mut canvas, p4, p5)?;
    draw_line(&mut canvas, p5, p6)?;
    draw_line(&mut canvas, p6, p7)?;
    draw_line(&mut canvas, p7, p4)?;

    // The front face.
    canvas.set_draw_color(Color::RGB(0, 255, 0));
    draw_line(&mut canvas, p0, p4)?;
    draw_line(&mut canvas, p1, p5)?;
    draw_line(&mut canvas, p2, p6)?;
    draw_line(&mut canvas, p3, p7)?;

    canvas.present();

    let mut event_pump = sdl.event_pump()?;
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'main_loop;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
