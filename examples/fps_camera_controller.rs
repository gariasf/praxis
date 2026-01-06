//! FPS camera controller example.
//!
//! This example demonstrates an FPS-style camera controller that combines
//! the input system with the camera system, featuring:
//! - WASD movement
//! - Mouse look with configurable sensitivity
//! - Vertical look clamping
//! - Sprint mode (Shift)
//! - Mouse cursor locking

#[path = "common.rs"]
mod common;

use common::CameraController;
use praxis_ecs::{
    Camera, PerspectiveCameraBundle, PerspectiveProjection, Query, Schedule, Transform, World,
};
use praxis_input::{Action, InputMap, InputState};
use praxis_math::Vec3;
use praxis_utils::Result;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, KeyCode, NamedKey};
use winit::window::{CursorGrabMode, Window, WindowId};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;

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
    println!("fps_camera_controller example requires graphics support and cannot run in headless mode");
    Ok(())
}

#[derive(Default)]
struct App {
    window: Option<Window>,
    world: Option<World>,
    schedule: Option<Schedule>,
    cursor_locked: bool,
}

impl App {
    fn setup_world() -> (World, Schedule) {
        let mut world = World::new();

        world.insert_resource(InputState::default());
        let controller = CameraController {
            yaw: 0.0,
            ..CameraController::default()
        };
        world.insert_resource(controller);

        let mut input_map = InputMap::default();
        input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
        input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
        input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
        input_map.bind_key(&Action::new("right"), KeyCode::KeyD);
        input_map.bind_key(&Action::new("up"), KeyCode::Space);
        input_map.bind_key(&Action::new("down"), KeyCode::ControlLeft);
        input_map.bind_key(&Action::new("sprint"), KeyCode::ShiftLeft);
        world.insert_resource(input_map);

        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 1.8, 5.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        println!("Created FPS camera entity: {camera_entity:?}");

        let mut schedule = Schedule::default();
        schedule.add_systems(praxis_ecs::systems::update_perspective_cameras);
        schedule.add_systems(fps_camera_movement_system);

        (world, schedule)
    }

    fn lock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(false);
            let _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked));
            self.cursor_locked = true;
        }
    }

    fn unlock_cursor(&mut self) {
        if let Some(window) = &self.window {
            window.set_cursor_visible(true);
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            self.cursor_locked = false;
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
                    .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                    .with_title("Praxis FPS Camera Controller")
                    .with_resizable(true),
            )
            .expect("Failed to create window");

        let (world, schedule) = Self::setup_world();

        println!("=== Praxis FPS Camera Controller ===");
        println!("Controls:");
        println!("  WASD - Move horizontally");
        println!("  Space - Move up");
        println!("  Left Ctrl - Move down");
        println!("  Left Shift - Sprint (hold)");
        println!("  Mouse - Look around");
        println!("  ESC - Toggle cursor lock / Exit (when unlocked)");
        println!("\nCamera will start locked. Press ESC to unlock cursor.");
        println!();

        self.window = Some(window);
        self.world = Some(world);
        self.schedule = Some(schedule);

        self.lock_cursor();
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
            WindowEvent::Focused(focused) => {
                if focused && self.cursor_locked {
                    self.lock_cursor();
                }
            }
            WindowEvent::RedrawRequested => {
                {
                    let input_state = world.get_resource_mut::<InputState>().unwrap();
                    input_state.update();
                }

                if let Some(schedule) = &mut self.schedule {
                    schedule.run(world.inner_mut());
                }

                // Camera matrices would be used for rendering here
                // The camera system runs via the schedule above

                if let Some(window) = &self.window {
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
                if self.cursor_locked {
                    println!("Cursor unlocked. Press ESC again to exit.");
                    self.unlock_cursor();
                } else {
                    println!("\nExiting...");
                    event_loop.exit();
                }
            }
            _ => {
                let input_state = world.get_resource_mut::<InputState>().unwrap();
                praxis_input::winit_integration::process_window_event(input_state, &event);
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if !self.cursor_locked {
            return;
        }

        let world = match self.world.as_mut() {
            Some(world) => world,
            None => return,
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            let controller = world.get_resource_mut::<CameraController>().unwrap();
            controller.update_rotation(delta.0 as f32, delta.1 as f32);
        }
    }
}

fn fps_camera_movement_system(
    input_state: praxis_ecs::Res<InputState>,
    input_map: praxis_ecs::Res<InputMap>,
    controller: praxis_ecs::ResMut<CameraController>,
    mut cameras: Query<(&Camera, &mut Transform), praxis_ecs::With<PerspectiveProjection>>,
) {
    for (camera, mut transform) in cameras.iter_mut() {
        if !camera.is_active {
            continue;
        }

        let mut velocity = Vec3::ZERO;

        if input_map.is_action_pressed(&Action::new("forward"), &input_state) {
            velocity.z -= 1.0;
        }
        if input_map.is_action_pressed(&Action::new("backward"), &input_state) {
            velocity.z += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("left"), &input_state) {
            velocity.x -= 1.0;
        }
        if input_map.is_action_pressed(&Action::new("right"), &input_state) {
            velocity.x += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("up"), &input_state) {
            velocity.y += 1.0;
        }
        if input_map.is_action_pressed(&Action::new("down"), &input_state) {
            velocity.y -= 1.0;
        }

        if velocity.length_squared() > 0.0 {
            velocity = velocity.normalize();
        }

        let mut speed = controller.move_speed;
        if input_map.is_action_pressed(&Action::new("sprint"), &input_state) {
            speed *= controller.sprint_multiplier;
        }

        let dt = 1.0 / 60.0;

        transform.rotation = controller.get_rotation();

        let forward = transform.rotation * Vec3::NEG_Z;
        let right = transform.rotation * Vec3::X;
        let up = Vec3::Y;

        transform.translation += forward * velocity.z * speed * dt;
        transform.translation += right * velocity.x * speed * dt;
        transform.translation += up * velocity.y * speed * dt;
    }
}
