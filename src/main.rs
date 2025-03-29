#![allow(dead_code)]
#![allow(unused_variables)]

mod gfx;
mod math;

use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Canvas, RenderTarget};
use std::error::Error;
use std::f32::consts::PI;

use gfx::*;
use math::plane::*;
use math::transform::*;
use math::vec::*;
use math::{mat::*, transform};

const VIEWPORT_WIDTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 1.0;
const D: f32 = 1.0;

struct Camera {
    translation: Vec4,
    rotation: Vec3,
}

struct Model {
    vertices: Vec<Vec4>,
    triangles: Vec<ModelTriangle>,
}

struct ClippingPlanes {
    near: Plane,
    left: Plane,
    right: Plane,
    bottom: Plane,
    top: Plane,
}

struct Scene {
    models: Vec<Model>,
    instances: Vec<Instance>,
    camera: Camera,
    clipping_planes: ClippingPlanes,
}

#[derive(Clone)]
struct ModelTriangle {
    vertices: [usize; 3],
    color: Color,
}

struct Instance {
    model_index: usize,
    translation: Vec4,
    scaling: Vec3,
    rotation: Vec3,
}

#[derive(Clone, Copy)]
struct ProjectedPoint {
    x: f32,
    y: f32,
    depth: f32,
}

impl Instance {
    fn new(model_index: usize) -> Self {
        Instance {
            model_index,
            translation: Vec4::new(0.0, 0.0, 0.0, 0.0),
            scaling: Vec3::new(1.0, 1.0, 1.0),
            rotation: Vec3::new(0.0, 0.0, 0.0),
        }
    }
}

impl Model {
    fn new(vertices: Vec<Vec4>, triangles: Vec<ModelTriangle>) -> Self {
        Model {
            vertices,
            triangles,
        }
    }
}

impl ModelTriangle {
    fn new(vertices: [usize; 3], color: Color) -> Self {
        ModelTriangle { vertices, color }
    }
}

fn create_camera_transform(camera: &Camera) -> Mat4 {
    let c_t = translation(-camera.translation);
    let c_rx = rotation_x(-camera.rotation[0]);
    let c_ry = rotation_y(-camera.rotation[1]);
    let c_rz = rotation_z(-camera.rotation[2]);
    c_rx * c_ry * c_rz * c_t
}

fn create_instance_transform(instance: &Instance) -> Mat4 {
    let i_t = translation(instance.translation);
    let i_s = scaling(instance.scaling);
    let i_r = rotation(instance.rotation);
    i_t * i_r * i_s
}

fn projected_to_point(v: Vec3) -> ProjectedPoint {
    let x = v[0] / v[2];
    let y = v[1] / v[2];
    let depth = 1.0 / v[2];
    ProjectedPoint { x, y, depth }
}

fn index_range<T>(v: &Vec<T>) -> core::ops::Range<usize> {
    0..v.len()
}

fn i32_range(x: f32, y: f32) -> core::ops::Range<i32> {
    (x as i32)..(y as i32)
}

fn i32_range_inclusive(x: f32, y: f32) -> core::ops::RangeInclusive<i32> {
    (x as i32)..=(y as i32)
}

fn min_to_max(x: f32, y: f32) -> (f32, f32) {
    if x > y {
        (y, x)
    } else {
        (x, y)
    }
}

fn is_in_canvas(x: i32, y: i32) -> bool {
    x >= 0 && x < CANVAS_WIDTH as i32 && y >= 0 && y < CANVAS_HEIGHT as i32
}

fn x_slope(p: ProjectedPoint, q: ProjectedPoint) -> f32 {
    (q.x - p.x) / (q.y - p.y)
}

#[derive(Clone, Copy)]
enum Draw {
    Depths,
    Pixels,
}

fn draw_line_horizontal<T>(
    canvas: &mut Canvas<T>,
    depth_buffer: &mut [f32],
    x_long: f32,
    x_short: f32,
    d_long: f32,
    d_short: f32,
    y: i32,
    color: Color,
    draw: Draw,
) -> RenderResult
where
    T: RenderTarget,
{
    let (x0, d0, x1, d1) = if x_long > x_short {
        (x_short, d_short, x_long, d_long)
    } else {
        (x_long, d_long, x_short, d_short)
    };
    let d_slope = (d1 - d0) / (x1 - x0);
    let mut d = d0;
    for x in i32_range_inclusive(x0, x1) {
        let p = Point::new(x, y);
        let p = plane_to_canvas(p);
        if is_in_canvas(p.x, p.y) {
            let y = p.y as usize;
            let x = p.x as usize;
            let w = CANVAS_WIDTH as usize;
            let depth_index = y * w + x;
            if d > depth_buffer[depth_index] {
                depth_buffer[depth_index] = d;
                let c = match draw {
                    Draw::Depths => {
                        let c = (255.0 * d) as u8;
                        Color::RGB(c, c, c)
                    }
                    Draw::Pixels => color,
                };
                canvas.set_draw_color(c);
                canvas.draw_point(p)?;
            }
        }
        d += d_slope;
    }
    Ok(())
}

fn render_scene<T>(canvas: &mut Canvas<T>, scene: &Scene, draw: Draw) -> RenderResult
where
    T: RenderTarget,
{
    let mut depth_buffer = [0.0; (CANVAS_WIDTH * CANVAS_HEIGHT) as usize];
    let m_projection = {
        let p = perspective_projection(D);
        let m = viewport_to_canvas(CANVAS_WIDTH, CANVAS_HEIGHT, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        m * p
    };

    let camera_transform = create_camera_transform(&scene.camera);

    for instance in scene.instances.iter() {
        let transform = camera_transform * create_instance_transform(&instance);
        let model = &scene.models[instance.model_index];
        for triangle in model.triangles.iter() {
            let triangle_data = [
                model.vertices[triangle.vertices[0]],
                model.vertices[triangle.vertices[1]],
                model.vertices[triangle.vertices[2]],
            ];
            let transformed_triangle_data = [
                transform * triangle_data[0],
                transform * triangle_data[1],
                transform * triangle_data[2],
            ];
            for clipped_triangle in
                clip_triangle(transformed_triangle_data, &scene.clipping_planes.near)
            {
                for clipped_triangle in clip_triangle(clipped_triangle, &scene.clipping_planes.left)
                {
                    for clipped_triangle in
                        clip_triangle(clipped_triangle, &scene.clipping_planes.right)
                    {
                        for clipped_triangle in
                            clip_triangle(clipped_triangle, &scene.clipping_planes.bottom)
                        {
                            for clipped_triangle in
                                clip_triangle(clipped_triangle, &scene.clipping_planes.top)
                            {
                                let p = {
                                    let mut p = [
                                        projected_to_point(m_projection * clipped_triangle[0]),
                                        projected_to_point(m_projection * clipped_triangle[1]),
                                        projected_to_point(m_projection * clipped_triangle[2]),
                                    ];
                                    p.sort_by(|p, q| p.y.total_cmp(&q.y));
                                    p
                                };

                                let x_slope_long = x_slope(p[0], p[2]);
                                let x_slope_short = x_slope(p[0], p[1]);
                                let d_slope_long = (p[2].depth - p[0].depth) / (p[2].y - p[0].y);
                                let d_slope_short = (p[1].depth - p[0].depth) / (p[1].y - p[0].y);
                                let mut x_long = p[0].x;
                                let mut x_short = p[0].x;
                                let mut d_long = p[0].depth;
                                let mut d_short = p[0].depth;
                                for y in i32_range(p[0].y, p[1].y) {
                                    draw_line_horizontal(
                                        canvas,
                                        &mut depth_buffer,
                                        x_long,
                                        x_short,
                                        d_long,
                                        d_short,
                                        y,
                                        triangle.color,
                                        draw,
                                    )?;
                                    x_long += x_slope_long;
                                    x_short += x_slope_short;
                                    d_long += d_slope_long;
                                    d_short += d_slope_short;
                                }

                                let x_slope_short = x_slope(p[1], p[2]);
                                let d_slope_short = (p[2].depth - p[1].depth) / (p[2].y - p[1].y);
                                let mut x_short = p[1].x;
                                let mut d_short = p[1].depth;
                                for y in i32_range(p[1].y, p[2].y) {
                                    draw_line_horizontal(
                                        canvas,
                                        &mut depth_buffer,
                                        x_long,
                                        x_short,
                                        d_long,
                                        d_short,
                                        y,
                                        triangle.color,
                                        draw,
                                    )?;
                                    x_long += x_slope_long;
                                    x_short += x_slope_short;
                                    d_long += d_slope_long;
                                    d_short += d_slope_short;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn clip_triangle(triangle: [Vec4; 3], plane: &Plane) -> Vec<[Vec4; 3]> {
    let mut clipped_triangles = Vec::with_capacity(3);

    let d = [
        plane.signed_distance(triangle[0]),
        plane.signed_distance(triangle[1]),
        plane.signed_distance(triangle[2]),
    ];

    let mut positive = Vec::with_capacity(3);
    let mut negative = Vec::with_capacity(3);

    for i in 0..3 {
        if d[i] > 0.0 {
            positive.push(i);
        } else {
            negative.push(i);
        }
    }

    match positive.len() {
        3 => {
            clipped_triangles.push(triangle);
        }
        2 => {
            let a = triangle[positive[0]];
            let b = triangle[positive[1]];
            let c = triangle[negative[0]];

            let a_prime = plane.intersection(a, c);
            let b_prime = plane.intersection(b, c);

            clipped_triangles.push([a, b, b_prime]);
            clipped_triangles.push([a_prime, b, b_prime]);
        }
        1 => {
            let a = triangle[positive[0]];
            let b = triangle[negative[0]];
            let c = triangle[negative[1]];

            let b_prime = plane.intersection(a, b);
            let c_prime = plane.intersection(a, c);

            clipped_triangles.push([a, b_prime, c_prime]);
        }
        _ => (),
    }

    clipped_triangles
}

fn build_scene() -> Scene {
    let vertices = vec![
        Vec4::new(1.0, 1.0, 1.0, 1.0),
        Vec4::new(-1.0, 1.0, 1.0, 1.0),
        Vec4::new(-1.0, -1.0, 1.0, 1.0),
        Vec4::new(1.0, -1.0, 1.0, 1.0),
        Vec4::new(1.0, 1.0, -1.0, 1.0),
        Vec4::new(-1.0, 1.0, -1.0, 1.0),
        Vec4::new(-1.0, -1.0, -1.0, 1.0),
        Vec4::new(1.0, -1.0, -1.0, 1.0),
    ];

    let triangles = vec![
        ModelTriangle::new([0, 1, 2], Color::RED),
        ModelTriangle::new([0, 2, 3], Color::RED),
        ModelTriangle::new([4, 0, 3], Color::GREEN),
        ModelTriangle::new([4, 3, 7], Color::GREEN),
        ModelTriangle::new([5, 4, 7], Color::BLUE),
        ModelTriangle::new([5, 7, 6], Color::BLUE),
        ModelTriangle::new([1, 5, 6], Color::YELLOW),
        ModelTriangle::new([1, 6, 2], Color::YELLOW),
        ModelTriangle::new([4, 5, 1], Color::MAGENTA),
        ModelTriangle::new([4, 1, 0], Color::MAGENTA),
        ModelTriangle::new([2, 6, 7], Color::CYAN),
        ModelTriangle::new([2, 7, 3], Color::CYAN),
    ];

    let models = vec![Model::new(vertices, triangles)];

    let instances = vec![
        Instance::new(0),
        Instance::new(0),
        Instance::new(0),
        Instance::new(0),
    ];

    let camera = Camera {
        translation: Vec4::new(0.0, 0.0, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.0, 0.0),
    };

    let clipping_planes = {
        let near = Plane::new(Vec4::new(0.0, 0.0, 1.0, 0.0), -D);
        let left = Plane::new(Vec4::new(1.0, 0.0, 1.0, 0.0), 0.0);
        let right = Plane::new(Vec4::new(-1.0, 0.0, 1.0, 0.0), 0.0);
        let bottom = Plane::new(Vec4::new(0.0, 1.0, 1.0, 0.0), 0.0);
        let top = Plane::new(Vec4::new(0.0, -1.0, 1.0, 0.0), 0.0);
        ClippingPlanes {
            near,
            left,
            right,
            bottom,
            top,
        }
    };

    Scene {
        models,
        instances,
        camera,
        clipping_planes,
    }
}

fn update_scene(scene: &mut Scene, t: f32) {
    let tau = 2.0 * PI;
    let t14 = t + 1.0 * tau / 4.0;
    let t24 = t + 2.0 * tau / 4.0;
    let t34 = t + 3.0 * tau / 4.0;

    let rad = 2.0;

    //scene.camera.rotation = Vec3::new(0.0, t * 0.1, 0.0);

    scene.instances[0].translation = Vec4::new(rad * t.cos(), rad * t.sin(), 7.0, 0.0);
    scene.instances[1].translation = Vec4::new(rad * t14.cos(), rad * t14.sin(), 7.0, 0.0);
    scene.instances[2].translation = Vec4::new(rad * t24.cos(), rad * t24.sin(), 7.0, 0.0);
    scene.instances[3].translation = Vec4::new(rad * t34.cos(), rad * t34.sin(), 7.0, 0.0);

    scene.instances[1].scaling = Vec3::new(t.sin().abs(), t14.sin().abs(), t24.sin().abs());
    scene.instances[2].scaling = Vec3::new(t.sin().abs(), t14.sin().abs(), t24.sin().abs());

    scene.instances[0].rotation = Vec3::new(t, t14, t24);
    scene.instances[2].rotation = Vec3::new(t, t14, t24);
}

fn main() -> Result<(), Box<dyn Error>> {
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;

    let window = video_subsystem
        .window("rstr", CANVAS_WIDTH, CANVAS_HEIGHT)
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let mut scene = build_scene();
    let mut t = 0.0;
    let mut event_pump = sdl.event_pump()?;
    let mut draw = Draw::Pixels;
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
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    draw = Draw::Depths;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => {
                    draw = Draw::Pixels;
                }
                _ => {}
            }
        }

        update_scene(&mut scene, t);

        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        canvas.clear();
        render_scene(&mut canvas, &scene, draw)?;
        canvas.present();

        t += 0.001;
    }

    Ok(())
}
