mod geometry;

use std::{f32::consts::PI, num::NonZeroU32, rc::Rc};

use geometry::{Hittable, Point3, Vec3};
use softbuffer::{Buffer, Context, Surface};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

struct Scene {
    camera_position: Point3,
    camera_direction: Vec3,
    camera_fov: f32,
    objects: Vec<Box<dyn Hittable>>,
}

trait Renderable {
    fn render(&self, buffer: &mut Buffer<Rc<Window>, Rc<Window>>, width: u32, height: u32);
}

impl Renderable for Scene {
    fn render(&self, buffer: &mut Buffer<Rc<Window>, Rc<Window>>, width: u32, height: u32) {
        for index in 0..(width * height) {
            let y = index / width;
            let x = index % width;
            let red = x % 255;
            let green = y % 255;
            let blue = (x * y) % 255;

            buffer[index as usize] = blue | (green << 8) | (red << 16);
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
            camera_position: Point3::ZERO,
            camera_direction: Vec3::new(1., 1., 0.),
            camera_fov: PI / 2.,
            objects: vec![],
        },
    };
    let _ = event_loop.run_app(&mut app);
}
