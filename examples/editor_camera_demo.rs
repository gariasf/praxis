//! Editor camera controller demonstration.
//!
//! This example demonstrates the orbit camera controller for the editor with:
//! - Orbit rotation (Alt+LMB)
//! - Pan movement (Alt+MMB)
//! - Zoom (scroll wheel)
//! - Focus on selection (F key)
//! - Smooth interpolated movement

#[cfg(feature = "editor")]
use praxis_ecs::{GlobalTransform, PerspectiveCameraBundle, Schedule, Transform, World};
#[cfg(feature = "editor")]
use praxis_editor::{
    update_editor_camera_system, EditorCamera, EditorCameraController, Selectable, SelectionSystem,
};
#[cfg(feature = "editor")]
use praxis_input::InputState;
#[cfg(feature = "editor")]
use praxis_math::Vec3;
#[cfg(feature = "editor")]
use praxis_utils::Result;
#[cfg(feature = "editor")]
use winit::application::ApplicationHandler;
#[cfg(feature = "editor")]
use winit::dpi::PhysicalSize;
#[cfg(feature = "editor")]
use winit::event::WindowEvent;
#[cfg(feature = "editor")]
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(feature = "editor")]
use winit::keyboard::KeyCode;
#[cfg(feature = "editor")]
use winit::window::{Window, WindowId};

#[cfg(feature = "editor")]
const WINDOW_WIDTH: u32 = 1280;
#[cfg(feature = "editor")]
const WINDOW_HEIGHT: u32 = 720;

#[cfg(feature = "editor")]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_input::init()?;
    praxis_ecs::init()?;
    praxis_editor::init()?;

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "editor")]
#[derive(Default)]
struct App {
    window: Option<Window>,
    world: Option<World>,
    schedule: Option<Schedule>,
}

#[cfg(feature = "editor")]
impl App {
    fn setup_world() -> (World, Schedule) {
        let mut world = World::new();

        world.insert_resource(InputState::default());
        world.insert_resource(EditorCameraController::new());
        world.insert_resource(SelectionSystem::new());

        // Create editor camera
        let camera_entity = world.spawn((
            PerspectiveCameraBundle::new(
                Vec3::new(0.0, 5.0, 10.0),
                70.0_f32.to_radians(),
                WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
            ),
            EditorCamera,
        ));

        println!("Created editor camera entity: {:?}", camera_entity);

        // Create some selectable objects in the scene
        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
            Selectable,
        ));

        world.spawn((
            Transform::from_xyz(5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Selectable,
        ));

        world.spawn((
            Transform::from_xyz(-5.0, 0.0, 0.0),
            GlobalTransform::default(),
            Selectable,
        ));

        world.spawn((
            Transform::from_xyz(0.0, 0.0, 5.0),
            GlobalTransform::default(),
            Selectable,
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(praxis_ecs::systems::update_perspective_cameras);
        schedule.add_systems(update_editor_camera_system);

        (world, schedule)
    }
}

#[cfg(feature = "editor")]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                    .with_title("Praxis Editor Camera Demo")
                    .with_resizable(true),
            )
            .expect("Failed to create window");

        let (world, schedule) = Self::setup_world();

        println!("=== Praxis Editor Camera Controller Demo ===");
        println!("Controls:");
        println!("  Alt+LMB - Orbit camera around target");
        println!("  Alt+MMB - Pan camera view");
        println!("  Scroll Wheel - Zoom in/out");
        println!("  F - Focus on selection (when entities selected)");
        println!("  1, 2, 3, 4 - Select different objects");
        println!("  ESC - Exit");
        println!("\nThe camera smoothly interpolates to desired positions.");
        println!();

        self.window = Some(window);
        self.world = Some(world);
        self.schedule = Some(schedule);
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
                {
                    let input_state = world.get_resource_mut::<InputState>().unwrap();
                    input_state.update();
                }

                // Handle number keys to select objects for testing focus
                {
                    // Collect selectables and check input state first
                    let mut selectable_entities = Vec::new();
                    {
                        let mut query = world.inner_mut().query_filtered::<bevy_ecs::entity::Entity, bevy_ecs::query::With<Selectable>>();
                        for entity in query.iter(world.inner()) {
                            selectable_entities.push(entity);
                        }
                    }

                    let pressed_1 = world
                        .get_resource::<InputState>()
                        .unwrap()
                        .is_key_just_pressed(KeyCode::Digit1);
                    let pressed_2 = world
                        .get_resource::<InputState>()
                        .unwrap()
                        .is_key_just_pressed(KeyCode::Digit2);
                    let pressed_3 = world
                        .get_resource::<InputState>()
                        .unwrap()
                        .is_key_just_pressed(KeyCode::Digit3);
                    let pressed_4 = world
                        .get_resource::<InputState>()
                        .unwrap()
                        .is_key_just_pressed(KeyCode::Digit4);

                    let selection = world.get_resource_mut::<SelectionSystem>().unwrap();

                    if pressed_1 && !selectable_entities.is_empty() {
                        selection.select_entity(
                            selectable_entities[0],
                            praxis_editor::SelectionMode::Replace,
                        );
                        println!("Selected entity 1");
                    }
                    if pressed_2 && selectable_entities.len() > 1 {
                        selection.select_entity(
                            selectable_entities[1],
                            praxis_editor::SelectionMode::Replace,
                        );
                        println!("Selected entity 2");
                    }
                    if pressed_3 && selectable_entities.len() > 2 {
                        selection.select_entity(
                            selectable_entities[2],
                            praxis_editor::SelectionMode::Replace,
                        );
                        println!("Selected entity 3");
                    }
                    if pressed_4 && selectable_entities.len() > 3 {
                        selection.select_entity(
                            selectable_entities[3],
                            praxis_editor::SelectionMode::Replace,
                        );
                        println!("Selected entity 4");
                    }
                }

                if let Some(schedule) = &mut self.schedule {
                    schedule.run(world.inner_mut());
                }

                // Display camera info
                {
                    let controller = world.get_resource::<EditorCameraController>().unwrap();
                    let (yaw, pitch) = controller.angles();
                    println!(
                        "Camera - Target: {:?}, Distance: {:.2}, Yaw: {:.2}°, Pitch: {:.2}°",
                        controller.target(),
                        controller.distance(),
                        yaw.to_degrees(),
                        pitch.to_degrees()
                    );
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(KeyCode::Escape)
                    && event.state.is_pressed()
                {
                    println!("\nExiting...");
                    event_loop.exit();
                }

                let mut input_state = world.get_resource_mut::<InputState>().unwrap();
                praxis_input::winit_integration::process_window_event(
                    &mut input_state,
                    &WindowEvent::KeyboardInput {
                        device_id: winit::event::DeviceId::dummy(),
                        event: event.clone(),
                        is_synthetic: false,
                    },
                );
            }
            _ => {
                let mut input_state = world.get_resource_mut::<InputState>().unwrap();
                praxis_input::winit_integration::process_window_event(&mut input_state, &event);
            }
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

#[cfg(not(feature = "editor"))]
fn main() {
    eprintln!("This example requires the 'editor' feature to be enabled.");
    eprintln!("Run with: cargo run --example editor_camera_demo --features editor");
    std::process::exit(1);
}
