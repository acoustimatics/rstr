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
use math::transform::*;
use math::vec::*;

const VIEWPORT_WIDTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 1.0;
const D: f32 = 1.0;

struct Camera {
    translation: Vec4,
    rotation: Vec3,
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

struct Scene {
    models: Vec<Model>,
    instances: Vec<Instance>,
    camera: Camera,
}

struct Triangle {
    vertices: [usize; 3],
    color: Color,
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

fn render_scene<T>(
    canvas: &mut Canvas<T>,
    scene: &Scene
) -> RenderResult
where
    T: RenderTarget,
{
    let m_projection = {
        let p = perspective_projection(D);
        let m =
            viewport_to_canvas(CANVAS_WIDTH, CANVAS_HEIGHT, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        m * p
    };
    let m_camera = {
        let c_t = translation(-scene.camera.translation);
        let c_rx = rotation_x(-scene.camera.rotation[0]);
        let c_ry = rotation_y(-scene.camera.rotation[1]);
        let c_rz = rotation_z(-scene.camera.rotation[2]);
        c_rx * c_ry * c_rz * c_t
    };

    for instance in scene.instances.iter() {
        let model = &scene.models[instance.model_index];

        let f = {
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
            canvas.set_draw_color(triangle.color);
            draw_wireframe_triangle(
                canvas,
                projected[triangle.vertices[0]],
                projected[triangle.vertices[1]],
                projected[triangle.vertices[2]],
            )?;
        }
    }

    Ok(())
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

    let instances = vec![
        Instance::new(0),
        Instance::new(0),
        Instance::new(0),
        Instance::new(0),
    ];

    let camera = Camera {
        translation: Vec4::new(-1.0, 0.0, 0.0, 0.0),
        rotation: Vec3::new(0.0, 0.0, 0.0),
    };

    Scene { models, instances, camera }
}

fn update_scene(scene: &mut Scene, t: f32) {
    let tau = 2.0 * PI;
    let t14 = t + 1.0 * tau / 4.0;
    let t24 = t + 2.0 * tau / 4.0;
    let t34 = t + 3.0 * tau / 4.0;

    let rad = 2.0;

    scene.camera.rotation = Vec3::new(0.0, t.sin()/4.0, 0.0);

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

        update_scene(&mut scene, t);

        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        canvas.clear();
        render_scene(&mut canvas, &scene)?;
        canvas.present();

        t += 0.001;
    }

    Ok(())
}
