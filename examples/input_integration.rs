//! Input integration example with winit.
//!
//! This example demonstrates how to integrate the input system with a winit
//! event loop and ECS world.

use praxis_ecs::World;
use praxis_input::{Action, InputMap, InputState, MouseButton};
use praxis_utils::Result;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey};
use winit::window::{Window, WindowId};

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("input_integration example requires graphics support and cannot run in headless mode");
    Ok(())
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    world: Option<World>,
}

impl App {
    fn setup_input_bindings(world: &mut World) {
        let mut input_map = InputMap::default();

        input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
        input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
        input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
        input_map.bind_key(&Action::new("right"), KeyCode::KeyD);

        input_map.bind_key(&Action::new("jump"), KeyCode::Space);

        input_map.bind_mouse_button(&Action::new("fire"), MouseButton::Left);
        input_map.bind_mouse_button(&Action::new("alt_fire"), MouseButton::Right);

        input_map.bind_key(&Action::new("reload"), KeyCode::KeyR);
        input_map.bind_key(&Action::new("use"), KeyCode::KeyE);

        world.insert_resource(input_map);
    }

    fn process_input(world: &mut World) {
        let input_state = world.get_resource::<InputState>().unwrap();
        let input_map = world.get_resource::<InputMap>().unwrap();

        let actions = [
            "forward", "backward", "left", "right", "jump", "fire", "alt_fire", "reload", "use",
        ];

        for action_name in &actions {
            let action = Action::new(*action_name);

            if input_map.is_action_just_pressed(&action, input_state) {
                println!("Action '{action_name}' just pressed!");
            }

            if input_map.is_action_just_released(&action, input_state) {
                println!("Action '{action_name}' released");
            }
        }

        let mouse_pos = input_state.mouse_position();
        let mouse_delta = input_state.mouse_delta();

        if mouse_delta.0.abs() > 0.1 || mouse_delta.1.abs() > 0.1 {
            println!(
                "Mouse at ({:.1}, {:.1}), delta: ({:.1}, {:.1})",
                mouse_pos.0, mouse_pos.1, mouse_delta.0, mouse_delta.1
            );
        }

        let scroll = input_state.scroll_delta();
        if scroll.0.abs() > 0.01 || scroll.1.abs() > 0.01 {
            println!("Mouse wheel: ({:.2}, {:.2})", scroll.0, scroll.1);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(PhysicalSize::new(1280, 720))
                    .with_title("Praxis Input Integration Demo")
                    .with_resizable(true),
            )
            .expect("Failed to create window");

        let mut world = World::new();
        world.insert_resource(InputState::default());
        Self::setup_input_bindings(&mut world);

        println!("=== Praxis Input Integration Demo ===");
        println!("Try pressing keys (W/A/S/D, Space, R, E)");
        println!("Try clicking mouse buttons (Left/Right)");
        println!("Try moving the mouse and scrolling");
        println!("Press ESC to exit\n");

        self.window = Some(window);
        self.world = Some(world);

        // Request initial redraw to start the event loop
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let world = match self.world.as_mut() {
            Some(world) => world,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("\nExiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    {
                        let input_state = world.get_resource_mut::<InputState>().unwrap();
                        input_state.update();
                    }

                    Self::process_input(world);

                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                println!("\nEscape pressed, exiting...");
                event_loop.exit();
            }
            _ => {
                let input_state = world.get_resource_mut::<InputState>().unwrap();
                praxis_input::winit_integration::process_window_event(input_state, &event);
            }
        }
    }
}
