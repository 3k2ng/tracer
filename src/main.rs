mod geometry;

use std::{f32::consts::PI, num::NonZeroU32, rc::Rc, sync::Arc, time::SystemTime};

use geometry::{
    gamma, Color, Dielectric, Hit, Interval, Lambertian, Light, Material, Metal, Object, Point,
    Ray, Sphere, Vector,
};
use rand::Rng;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use softbuffer::{Buffer, Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

struct Scene {
    camera_position: Point,
    camera_direction: Vector,
    camera_up: Vector,
    camera_fov: f32,
    max_samples: u32,
    depth: u32,
    objects: Vec<Object>,
}

impl Scene {
    fn trace(&self, ray: &Ray, interval: &Interval, depth: u32) -> Color {
        if depth == 0 {
            Vector::ZERO
        } else {
            let mut hit: Option<Hit> = None;
            let mut material: Option<Arc<dyn Material>> = None;
            for object in self.objects.iter() {
                let t_min = if let Some(Hit {
                    t,
                    normal: _,
                    is_front: _,
                }) = hit
                {
                    t
                } else {
                    interval.max
                };
                match object.shape.hit(ray, &Interval::new(interval.min, t_min)) {
                    None => (),
                    Some(h) => {
                        if h.t < t_min {
                            hit = Some(h);
                            material = Some(object.material.clone());
                        }
                    }
                }
            }
            if let Some(h) = hit {
                match material.unwrap().on_hit(ray, &h) {
                    geometry::OnHit::None => Vector::ZERO,
                    geometry::OnHit::Scatter {
                        attenuation,
                        scattered,
                    } => self.trace(&scattered, interval, depth - 1) * attenuation,
                    geometry::OnHit::Emitted { color } => color,
                }
            } else {
                // let a = 0.5 * (ray.direction.y + 1.0);
                // (1.0 - a) * Vector::new(1.0, 1.0, 1.0) + a * Vector::new(0.5, 0.7, 1.0)
                Color::ZERO
            }
        }
    }
    fn render(&self, buffer: &mut Buffer<Rc<Window>, Rc<Window>>, width: u32, height: u32) {
        let start_time = SystemTime::now();
        let camera_right = self.camera_direction.cross(self.camera_up).normalize();
        let camera_up = camera_right.cross(self.camera_direction).normalize();
        let l = width as f32 / (self.camera_fov / 2.).tan();
        let contribution = 1.0 / (self.max_samples as f32);
        buffer
            .par_iter_mut()
            .zip(0..width * height)
            .for_each(|(pixel, index)| {
                let mut rng = rand::thread_rng();
                let y = (index / width) as f32 - height as f32 / 2.;
                let x = (index % width) as f32 - width as f32 / 2.;
                let mut vec_pixel = Color::ZERO;
                for _ in 0..self.max_samples {
                    vec_pixel = vec_pixel
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
                                self.depth,
                            );
                }
                *pixel = gamma(vec_pixel);
            });
        let end_time = SystemTime::now();
        println!(
            "{}s",
            end_time.duration_since(start_time).unwrap().as_secs_f64()
        );
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
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(800, 450))
                        .with_resizable(false),
                )
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
        _window_id: winit::window::WindowId,
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
    let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.8)));
    let material_center = Arc::new(Light::new(Color::new(5.0, 8.0, 10.0)));
    let material_left = Arc::new(Dielectric::new(1.5));
    let material_bubble = Arc::new(Dielectric::new(1. / 1.5));
    let material_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));
    let mut app = App {
        window: None,
        context: None,
        surface: None,
        scene: Scene {
            camera_position: Point::ZERO,
            camera_direction: Vector::new(0., 0., -1.).normalize(),
            camera_up: Vector::new(0., 1., 0.),
            camera_fov: 3. * PI / 4.,
            objects: vec![
                Object {
                    shape: Box::new(Sphere {
                        center: Point::new(0.0, -100.5, -1.0),
                        radius: 100.0,
                    }),
                    material: material_ground.clone(),
                },
                Object {
                    shape: Box::new(Sphere {
                        center: Point::new(0.0, 0.5, -1.2),
                        radius: 0.5,
                    }),
                    material: material_center.clone(),
                },
                Object {
                    shape: Box::new(Sphere {
                        center: Point::new(-1.0, 0.0, -1.0),
                        radius: 0.5,
                    }),
                    material: material_left.clone(),
                },
                Object {
                    shape: Box::new(Sphere {
                        center: Point::new(-1.0, 0.0, -1.0),
                        radius: 0.4,
                    }),
                    material: material_bubble.clone(),
                },
                Object {
                    shape: Box::new(Sphere {
                        center: Point::new(1.0, 0.0, -1.0),
                        radius: 0.5,
                    }),
                    material: material_right.clone(),
                },
            ],
            max_samples: 256,
            depth: 32,
        },
    };
    let _ = event_loop.run_app(&mut app);
}
