#![allow(dead_code)]
#![allow(unused_variables)]

mod math;

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, RenderTarget};
use std::error::Error;
use std::f32::consts::PI;

use math::transform::*;
use math::vec::*;

const CANVAS_WIDTH: u32 = 640;
const CANVAS_HEIGHT: u32 = 640;
const VIEWPORT_WIDTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 1.0;
const D: f32 = 1.0;

struct Camera {
    translation: Vec4,
    rotation: Vec3,
}

#[derive(Debug)]
struct Interpolation {
    i_end: i32,
    i: i32,
    d: f32,
    a: f32,
}

struct Instance {
    model_index: usize,
    translation: Vec4,
    scaling: Vec3,
    rotation: Vec3,
}

struct Model {
    vertices: Vec<Vec4>,
    triangles: Vec<Triangle>,
}

type RenderResult = Result<(), Box<dyn Error>>;

struct Triangle {
    vertices: [usize; 3],
    color: Color,
}

#[derive(Copy, Clone)]
struct Vertex {
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

    fn new_by_y(p0: Vertex, p1: Vertex) -> Self {
        Interpolation::new(p0.y, p0.x, p1.y, p1.x)
    }
}

impl Instance {
    fn new(model_index: usize) -> Self {
        Instance {
            model_index,
            translation: vec4!(0.0),
            scaling: vec3!(1.0),
            rotation: vec3!(0.0),
        }
    }
}

impl Model {
    fn new(vertices: Vec<Vec4>, triangles: Vec<Triangle>) -> Self {
        Model {
            vertices,
            triangles,
        }
    }
}

impl Triangle {
    fn new(vertices: [usize; 3], color: Color) -> Self {
        Triangle { vertices, color }
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
    Point::new(
        (CANVAS_WIDTH as i32) / 2 + p.x,
        (CANVAS_HEIGHT as i32) / 2 - p.y,
    )
}

fn put_pixel<T>(canvas: &mut Canvas<T>, p: Point) -> RenderResult
where
    T: RenderTarget,
{
    let p = plane_to_canvas(p);
    canvas.draw_point(p)?;

    Ok(())
}

fn draw_line<T>(canvas: &mut Canvas<T>, p0: Point, p1: Point) -> RenderResult
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

fn draw_wireframe_triangle<T>(
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

fn render_instance<T>(
    canvas: &mut Canvas<T>,
    model: &Model,
    instance: &Instance,
    camera: &Camera,
) -> RenderResult
where
    T: RenderTarget,
{
    let f = {
        let m_projection = {
            let p = perspective_projection(D);
            let m =
                viewport_to_canvas(CANVAS_WIDTH, CANVAS_HEIGHT, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
            m * p
        };
        let m_camera = {
            let c_t = translation(-camera.translation);
            let c_rx = rotation_x(-camera.rotation[0]);
            let c_ry = rotation_y(-camera.rotation[1]);
            let c_rz = rotation_z(-camera.rotation[2]);
            c_rx * c_ry * c_rz * c_t
        };
        let m_model = {
            let i_t = translation(instance.translation);
            let i_s = scaling(instance.scaling);
            let i_r = rotation(instance.rotation);
            i_t * i_r * i_s
        };
        m_projection * m_camera * m_model
    };

    let mut projected = vec![];

    for &v in model.vertices.iter() {
        let v = f * v;

        let x = v[0] / v[2];
        let y = v[1] / v[2];
        let p = Point::new(x as i32, y as i32);

        projected.push(p);
    }
    for triangle in model.triangles.iter() {
        render_triangle(canvas, &projected, triangle)?;
    }

    Ok(())
}

fn render_triangle<T>(canvas: &mut Canvas<T>, points: &[Point], triangle: &Triangle) -> RenderResult
where
    T: RenderTarget,
{
    canvas.set_draw_color(triangle.color);
    draw_wireframe_triangle(
        canvas,
        points[triangle.vertices[0]],
        points[triangle.vertices[1]],
        points[triangle.vertices[2]],
    )
}

fn render_scene<T>(
    canvas: &mut Canvas<T>,
    models: &[Model],
    instances: &[Instance],
    camera: &Camera,
) -> RenderResult
where
    T: RenderTarget,
{
    for instance in instances.iter() {
        let model = &models[instance.model_index];
        render_instance(canvas, model, instance, camera)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;

    let window = video_subsystem
        .window("rstr", CANVAS_WIDTH, CANVAS_HEIGHT)
        .build()?;

    let mut canvas = window.into_canvas().build()?;

    let vertices = vec![
        vec4!(1.0, 1.0, 1.0, 1.0),
        vec4!(-1.0, 1.0, 1.0, 1.0),
        vec4!(-1.0, -1.0, 1.0, 1.0),
        vec4!(1.0, -1.0, 1.0, 1.0),
        vec4!(1.0, 1.0, -1.0, 1.0),
        vec4!(-1.0, 1.0, -1.0, 1.0),
        vec4!(-1.0, -1.0, -1.0, 1.0),
        vec4!(1.0, -1.0, -1.0, 1.0),
    ];

    let triangles = vec![
        Triangle::new([0, 1, 2], Color::RED),
        Triangle::new([0, 2, 3], Color::RED),
        Triangle::new([4, 0, 3], Color::GREEN),
        Triangle::new([4, 3, 7], Color::GREEN),
        Triangle::new([5, 4, 7], Color::BLUE),
        Triangle::new([5, 7, 6], Color::BLUE),
        Triangle::new([1, 5, 6], Color::YELLOW),
        Triangle::new([1, 6, 2], Color::YELLOW),
        Triangle::new([4, 5, 1], Color::MAGENTA),
        Triangle::new([4, 1, 0], Color::MAGENTA),
        Triangle::new([2, 6, 7], Color::CYAN),
        Triangle::new([2, 7, 3], Color::CYAN),
    ];

    let models = vec![Model::new(vertices, triangles)];

    let mut instances = vec![
        Instance::new(0),
        Instance::new(0),
        Instance::new(0),
        Instance::new(0),
    ];

    let mut camera = Camera {
        translation: vec4!(-1.0, 0.0, 0.0, 0.0),
        rotation: vec3!(0.0),
    };

    let mut t0 = 0.0;

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

        let tau = 2.0 * PI;
        let t14 = t0 + 1.0 * tau / 4.0;
        let t24 = t0 + 2.0 * tau / 4.0;
        let t34 = t0 + 3.0 * tau / 4.0;

        let rad = 2.0;

        camera.rotation = vec3!(0.0, t0.sin()/4.0, 0.0);

        instances[0].translation = vec4!(rad * t0.cos(), rad * t0.sin(), 7.0, 0.0);
        instances[1].translation = vec4!(rad * t14.cos(), rad * t14.sin(), 7.0, 0.0);
        instances[2].translation = vec4!(rad * t24.cos(), rad * t24.sin(), 7.0, 0.0);
        instances[3].translation = vec4!(rad * t34.cos(), rad * t34.sin(), 7.0, 0.0);

        instances[1].scaling = vec3!(t0.sin().abs(), t14.sin().abs(), t24.sin().abs());
        instances[2].scaling = vec3!(t0.sin().abs(), t14.sin().abs(), t24.sin().abs());

        instances[0].rotation = vec3!(t0, t14, t24);
        instances[2].rotation = vec3!(t0, t14, t24);

        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        canvas.clear();
        render_scene(&mut canvas, &models, &instances, &camera)?;
        canvas.present();

        t0 += 0.001;
    }

    Ok(())
}
