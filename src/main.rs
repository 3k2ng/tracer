mod geometry;

use std::{f32::consts::PI, num::NonZeroU32, rc::Rc};

use geometry::{Interval, Point, Ray, Renderable, Sphere, Vector};
use rand::Rng;
use softbuffer::{Buffer, Context, Surface};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

fn color(r: f32, g: f32, b: f32) -> u32 {
    let red = (r * 255.) as u32;
    let green = (g * 255.) as u32;
    let blue = (b * 255.) as u32;
    red << 16 | green << 8 | blue
}

type Color = Vector;

fn vec_color(c: Color) -> u32 {
    color(c.x, c.y, c.z)
}

struct Scene {
    camera_position: Point,
    camera_direction: Vector,
    camera_up: Vector,
    camera_fov: f32,
    antialiasing_samples: u32,
    rendering_depth: u32,
    objects: Vec<Box<dyn Renderable>>,
}

impl Scene {
    fn trace(&self, ray: &Ray, interval: &Interval, depth: u32) -> Color {
        if depth == 0 {
            Vector::ZERO
        } else {
            let mut hit = geometry::Hit::Miss;
            let mut t_min = interval.max;
            let mut hit_object: Option<&Box<dyn Renderable>> = None;
            for object in self.objects.iter() {
                match object.hit(ray, &Interval::new(interval.min, t_min)) {
                    geometry::Hit::Miss => (),
                    geometry::Hit::Outside(t) => {
                        if t < t_min {
                            hit = geometry::Hit::Outside(t);
                            t_min = t;
                            hit_object = Some(object);
                        }
                    }
                    geometry::Hit::Inside(t) => {
                        if t < t_min {
                            hit = geometry::Hit::Inside(t);
                            t_min = t;
                            hit_object = Some(object);
                        }
                    }
                }
            }
            if let geometry::Hit::Miss = hit {
                let a = 0.5 * (ray.direction.y + 1.0);
                (1.0 - a) * Vector::new(1.0, 1.0, 1.0) + a * Vector::new(0.5, 0.7, 1.0)
            } else {
                let normal = hit_object.unwrap().normal(ray, hit).unwrap();
                let reflection = normal.random_reflection();
                self.trace(&Ray::new(ray.at(t_min), reflection), interval, depth - 1) * 0.5
            }
        }
    }
    fn render(&self, buffer: &mut Buffer<Rc<Window>, Rc<Window>>, width: u32, height: u32) {
        let camera_right = self.camera_direction.cross(self.camera_up).normalize();
        let camera_up = camera_right.cross(self.camera_direction).normalize();
        let l = width as f32 / (self.camera_fov / 2.).tan();
        let contribution = 1.0 / (self.antialiasing_samples as f32);
        let mut rng = rand::thread_rng();
        for index in 0..(width * height) {
            let y = (index / width) as f32 - height as f32 / 2.;
            let x = (index % width) as f32 - width as f32 / 2.;
            if self.antialiasing_samples == 1 {
                buffer[index as usize] = vec_color(self.trace(
                    &Ray::new(
                        self.camera_position,
                        (x * camera_right - y * camera_up + l * self.camera_direction).normalize(),
                    ),
                    &Interval::RENDER_RANGE,
                    self.rendering_depth,
                ));
            } else {
                let mut pixel = Color::ZERO;
                for _ in 0..self.antialiasing_samples {
                    pixel = pixel
                        + contribution
                            * self.trace(
                                &Ray::new(
                                    self.camera_position,
                                    ((x + rng.gen::<f32>() - 0.5) * camera_right
                                        - (y + rng.gen::<f32>() - 0.5) * camera_up
                                        + l * self.camera_direction)
                                        .normalize(),
                                ),
                                &Interval::RENDER_RANGE,
                                self.rendering_depth,
                            );
                }
                buffer[index as usize] = vec_color(pixel);
            }
        }
    }
}

struct App {
    window: Option<Rc<Window>>,
    context: Option<Context<Rc<Window>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    scene: Scene,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(Rc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        ));
        self.context = Some(Context::new(self.window.as_ref().unwrap().clone()).unwrap());
        self.surface = Some(
            Surface::new(
                self.context.as_ref().unwrap(),
                self.window.as_ref().unwrap().clone(),
            )
            .unwrap(),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Enter),
                        state: ElementState::Pressed,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                let (width, height) = {
                    let size = self.window.as_ref().unwrap().inner_size();
                    (size.width, size.height)
                };
                self.surface
                    .as_mut()
                    .unwrap()
                    .resize(
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    )
                    .unwrap();

                let mut buffer = self.surface.as_mut().unwrap().buffer_mut().unwrap();
                self.scene.render(&mut buffer, width, height);
                buffer.present().unwrap();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        context: None,
        surface: None,
        scene: Scene {
            camera_position: Point::ZERO,
            camera_direction: Vector::new(0., 0., -1.).normalize(),
            camera_up: Vector::new(0., 1., 0.),
            camera_fov: 2. * PI / 3.,
            objects: vec![
                Box::new(Sphere {
                    center: Point::new(0., 0., -1.),
                    radius: 0.5,
                }),
                Box::new(Sphere {
                    center: Point::new(0., -100.5, -1.),
                    radius: 100.,
                }),
            ],
            antialiasing_samples: 256,
            rendering_depth: 32,
        },
    };
    let _ = event_loop.run_app(&mut app);
}
