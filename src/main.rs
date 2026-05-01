use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::resources::{Camera, Input};
use crate::state::State;

mod assets;
mod components;
mod render;
mod resources;
mod state;
mod systems;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_inner_size(winit::dpi::PhysicalSize::new(2560, 1440));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .ok();
        window.set_cursor_visible(false);

        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();

                match state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if key_state.is_pressed() {
                    state.world.resource_mut::<Input>().pressed.insert(code);
                } else {
                    state.world.resource_mut::<Input>().pressed.remove(&code);
                }
                if code == KeyCode::Escape && key_state.is_pressed() {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(state) = &mut self.state
            && let winit::event::DeviceEvent::MouseMotion { delta } = event
        {
            let mut camera = state.world.resource_mut::<Camera>();
            camera.yaw += delta.0 as f32 * camera.sensitivity;
            camera.pitch -= delta.1 as f32 * camera.sensitivity;
            camera.pitch = camera.pitch.clamp(-1.5, 1.5); // ~86 degrees
        }
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn main() {
    tracing_subscriber::fmt::init();

    let _app = run();
}
