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
use math::mat::*;

const VIEWPORT_WIDTH: f32 = 1.0;
const VIEWPORT_HEIGHT: f32 = 1.0;
const D: f32 = 1.0;

#[derive(Clone, Copy, PartialEq)]
enum Switch {
    Off,
    On,
}

impl Switch {
    fn toggle(self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::On,
        }
    }
}

struct Camera {
    translation: Vec4,
    rotation: Vec3,
}

#[derive(Clone, Copy)]
struct Fragment {
    x: f32,
    y: f32,
    depth: f32,
    r: f32,
    g: f32,
    b: f32,
}

struct Model {
    vertices: Vec<Vec4>,
    colors: Vec<ColorF32>,
    triangles: Vec<ModelTriangle>,
    normals: Vec<Vec4>,
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
    indices_color: [usize; 3],
}

struct Instance {
    model_index: usize,
    translation: Vec4,
    scaling: Vec3,
    rotation: Vec3,
}

impl Fragment {
    fn slope_by_y(self, to: Self) -> Self {
        let y_delta = to.y - self.y;
        Self {
            x: (to.x - self.x) / y_delta,
            y: 1.0,
            depth: (to.depth - self.depth) / y_delta,
            r: (to.r - self.r) / y_delta,
            g: (to.g - self.g) / y_delta,
            b: (to.b - self.b) / y_delta,
        }
    }

    fn slope_by_x(self, to: Self) -> Self {
        let x_delta = to.x - self.x;
        Self {
            x: 1.0,
            y: (to.y - self.y) / x_delta,
            depth: (to.depth - self.depth) / x_delta,
            r: (to.r - self.r) / x_delta,
            g: (to.g - self.g) / x_delta,
            b: (to.b - self.b) / x_delta,
        }
    }
}

impl std::ops::AddAssign for Fragment {
   fn add_assign(&mut self, rhs: Self) {
       self.x += rhs.x;
       self.y += rhs.y;
       self.depth += rhs.depth;
       self.r += rhs.r;
       self.g += rhs.g;
       self.b += rhs.b;
   } 
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
    fn new(vertices: Vec<Vec4>, colors: Vec<ColorF32>, triangles: Vec<ModelTriangle>) -> Self {
        let mut normals = Vec::new();
        for triangle in triangles.iter() {
            let v1 = vertices[triangle.vertices[1]] - vertices[triangle.vertices[0]];
            let v2 = vertices[triangle.vertices[2]] - vertices[triangle.vertices[0]];
            let normal = v1.cross(v2).normalize();
            normals.push(normal);
        }
        Model {
            vertices,
            colors,
            triangles,
            normals,
        }
    }
}

impl ModelTriangle {
    fn new(vertices: [usize; 3], indices_color: [usize; 3]) -> Self {
        ModelTriangle { vertices, indices_color }
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

fn projected_to_point(v: Vec3, color: ColorF32) -> Fragment {
    let x = v[0] / v[2];
    let y = v[1] / v[2];
    let depth = 1.0 / v[2];
    let (r, g, b) = color.rgb();
    Fragment { x, y, depth, r, g, b }
}

fn i32_range(x: f32, y: f32) -> core::ops::Range<i32> {
    (x as i32)..(y as i32)
}

fn i32_range_inclusive(x: f32, y: f32) -> core::ops::RangeInclusive<i32> {
    (x as i32)..=(y as i32)
}

fn is_in_canvas(x: i32, y: i32) -> bool {
    x >= 0 && x < CANVAS_WIDTH as i32 && y >= 0 && y < CANVAS_HEIGHT as i32
}

#[derive(Clone, Copy)]
enum Draw {
    Depths,
    Pixels,
    Wireframe,
}

fn draw_line_horizontal<T>(
    canvas: &mut Canvas<T>,
    depth_buffer: &mut [f32],
    f1: Fragment,
    f2: Fragment,
    y: i32,
    draw: Draw,
) -> RenderResult
where
    T: RenderTarget,
{
    let (f_left, f_right) = if f1.x > f2.x {
        (f2, f1)
    } else {
        (f1, f2)
    };
    let mut f = f_left;
    let f_slope = f_left.slope_by_x(f_right);
    for x in i32_range_inclusive(f_left.x, f_right.x) {
        let p = Point::new(x, y);
        let p = plane_to_canvas(p);
        if is_in_canvas(p.x, p.y) {
            let y = p.y as usize;
            let x = p.x as usize;
            let w = CANVAS_WIDTH as usize;
            let depth_index = y * w + x;
            if f.depth > depth_buffer[depth_index] {
                depth_buffer[depth_index] = f.depth;
                let c = match draw {
                    Draw::Depths => {
                        let c = (255.0 * f.depth) as u8;
                        Color::RGB(c, c, c)
                    }
                    _ => create_color_sdl(f.r, f.g, f.b),
                };
                canvas.set_draw_color(c);
                canvas.draw_point(p)?;
            }
        }
        f += f_slope;
    }
    Ok(())
}

fn render_scene<T>(canvas: &mut Canvas<T>, scene: &Scene, draw: Draw, cull_backfaces: Switch) -> RenderResult
where
    T: RenderTarget,
{
    let mut depth_buffer = vec![0.0; (CANVAS_WIDTH * CANVAS_HEIGHT) as usize];
    let m_projection = {
        let p = perspective_projection(D);
        let m = viewport_to_canvas(CANVAS_WIDTH, CANVAS_HEIGHT, VIEWPORT_WIDTH, VIEWPORT_HEIGHT);
        m * p
    };

    let camera_transform = create_camera_transform(&scene.camera);

    for instance in scene.instances.iter() {
        let transform = camera_transform * create_instance_transform(&instance);
        let model = &scene.models[instance.model_index];
        for (triangle_index, triangle) in model.triangles.iter().enumerate() {
            let triangle_data = [
                model.vertices[triangle.vertices[0]],
                model.vertices[triangle.vertices[1]],
                model.vertices[triangle.vertices[2]],
            ];
            let colors_triangle = [
                model.colors[triangle.indices_color[0]],
                model.colors[triangle.indices_color[1]],
                model.colors[triangle.indices_color[2]],
            ];
            let transformed_triangle_data = [
                transform * triangle_data[0],
                transform * triangle_data[1],
                transform * triangle_data[2],
            ];

            // back-face culling
            if cull_backfaces == Switch::On {
                let normal = model.normals[triangle_index];
                let transformed_normal = transform * normal;
                let view_vector = transformed_triangle_data[0]; // camera always at origin.
                let normal_dot_view = transformed_normal.dot(view_vector);
                if normal_dot_view >= 0.0 {
                    continue;
                }
            }

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
                                let mut p = [
                                    projected_to_point(m_projection * clipped_triangle[0], colors_triangle[0]),
                                    projected_to_point(m_projection * clipped_triangle[1], colors_triangle[1]),
                                    projected_to_point(m_projection * clipped_triangle[2], colors_triangle[2]),
                                ];

                                match draw {
                                    Draw::Wireframe => {
                                        let color = create_color_sdl(p[0].r, p[0].g, p[0].b);
                                        canvas.set_draw_color(color);
                                        let p0 = Point::new(p[0].x as i32, p[0].y as i32);
                                        let p1 = Point::new(p[1].x as i32, p[1].y as i32);
                                        let p2 = Point::new(p[2].x as i32, p[2].y as i32);
                                        draw_wireframe_triangle(canvas, p0, p1, p2)?;
                                        continue;
                                    }
                                    _ => ()
                                }

                                p.sort_by(|p, q| p.y.total_cmp(&q.y));

                                let mut long = p[0];
                                let long_slope = p[0].slope_by_y(p[2]);
                                
                                for i in 0..=1 {
                                    let mut short = p[i];
                                    let short_slope = p[i].slope_by_y(p[i + 1]);
                                    for y in i32_range(p[i].y, p[i + 1].y) {
                                        draw_line_horizontal(
                                            canvas,
                                            &mut depth_buffer,
                                            long,
                                            short,
                                            y,
                                            draw,
                                        )?;
                                        long += long_slope;
                                        short += short_slope;
                                    }
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
        Vec4::new(1.0, 1.0, 1.0, 1.0), // 0 - black(0)
        Vec4::new(-1.0, 1.0, 1.0, 1.0), // 1 - red(0)
        Vec4::new(-1.0, -1.0, 1.0, 1.0), // 2 - yellow(3)
        Vec4::new(1.0, -1.0, 1.0, 1.0), // 3 - green(1)
        Vec4::new(1.0, 1.0, -1.0, 1.0), // 4 - blue(2)
        Vec4::new(-1.0, 1.0, -1.0, 1.0), // 5 - magenta(4)
        Vec4::new(-1.0, -1.0, -1.0, 1.0), // 6 - white (0)
        Vec4::new(1.0, -1.0, -1.0, 1.0), // 7 - cyan(5)
    ];
    
    let colors = vec![
        ColorF32::RED, // 0
        ColorF32::GREEN, // 1
        ColorF32::BLUE, // 2
        ColorF32::YELLOW, // 3
        ColorF32::MAGENTA, // 4
        ColorF32::CYAN, // 5
        ColorF32::WHITE, // 6
        ColorF32::BLACK, // 7
    ];

    let triangles = vec![
        ModelTriangle::new([0, 1, 2], [7, 0, 3]),
        ModelTriangle::new([0, 2, 3], [7, 3, 1]),
        ModelTriangle::new([4, 0, 3], [2, 7, 1]),
        ModelTriangle::new([4, 3, 7], [2, 1, 5]),
        ModelTriangle::new([5, 4, 7], [4, 2, 5]),
        ModelTriangle::new([5, 7, 6], [4, 5, 6]),
        ModelTriangle::new([1, 5, 6], [0, 4, 6]),
        ModelTriangle::new([1, 6, 2], [0, 6, 3]),
        ModelTriangle::new([4, 5, 1], [2, 4, 0]),
        ModelTriangle::new([4, 1, 0], [2, 0, 7]),
        ModelTriangle::new([2, 6, 7], [3, 6, 5]),
        ModelTriangle::new([2, 7, 3], [3, 5, 1]),
    ];

    let models = vec![Model::new(vertices, colors, triangles)];

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
    let mut cull_backfaces = Switch::On;
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
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => {
                    draw = Draw::Wireframe;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    cull_backfaces = cull_backfaces.toggle();
                }
                _ => {}
            }
        }

        update_scene(&mut scene, t);

        canvas.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        canvas.clear();
        render_scene(&mut canvas, &scene, draw, cull_backfaces)?;
        canvas.present();

        t += 0.005;
    }

    Ok(())
}
