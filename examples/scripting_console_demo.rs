//! Advanced console demo showcasing scripting integration with ECS introspection.
//!
//! This example demonstrates the full power of the Praxis console with Lua scripting:
//! - Interactive REPL with automatic expression evaluation
//! - Full ECS World access via console commands
//! - Entity queries, inspection, and runtime modification
//! - Live entity spawning and despawning
//! - Transform manipulation from the console
//!
//! Controls:
//! - ~ or F1: Toggle console visibility
//! - Up/Down: Navigate command history
//! - Tab: Cycle through autocomplete suggestions
//! - Enter: Execute command or Lua code
//! - Escape: Close autocomplete or exit application
//!
//! Try these commands:
//! ```lua
//! -- Basic Lua expressions
//! 2 + 2
//! math.sqrt(16)
//!
//! -- Entity introspection
//! console.list_entities()
//! console.entity_count()
//!
//! -- Find and inspect entities
//! local id = console.find_entity("Player")
//! console.inspect(id)
//!
//! -- Modify transforms
//! console.set_transform(id, 10, 5, 0)
//! console.get_transform(id)
//!
//! -- Spawn and despawn entities
//! console.spawn("DynamicEntity")
//! console.list_entities()
//!
//! -- Query by component
//! console.query_with_transform()
//! console.query_with_name()
//! ```

#[cfg(all(feature = "scripting", not(feature = "headless")))]
use parking_lot::RwLock;
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_ecs::{GlobalTransform, Name, Transform, World};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_graphics::{RenderCommands, RenderContext};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_gui::{ConsolePanel, EguiIntegration};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_scripting::{ScriptingConfig, ScriptingContext};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_utils::{info, Result};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use std::sync::Arc;
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use winit::application::ApplicationHandler;
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use winit::dpi::PhysicalSize;
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use winit::event::{ElementState, KeyEvent, WindowEvent};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use winit::keyboard::{Key, NamedKey, PhysicalKey};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use winit::window::{Window, WindowId};

#[cfg(all(feature = "scripting", not(feature = "headless")))]
const WINDOW_WIDTH: u32 = 1280;
#[cfg(all(feature = "scripting", not(feature = "headless")))]
const WINDOW_HEIGHT: u32 = 720;

#[cfg(all(feature = "scripting", not(feature = "headless")))]
#[derive(Default)]
struct ScriptingConsoleDemo {
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    egui_integration: Option<EguiIntegration>,
    console: Option<ConsolePanel>,
    world: Option<World>,
    frame_count: u64,
}

#[cfg(all(feature = "scripting", not(feature = "headless")))]
impl ApplicationHandler for ScriptingConsoleDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Initializing Scripting Console Demo");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Scripting Console Demo - Press ~ or F1 to toggle console")
                .with_resizable(true),
        ) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                eprintln!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let render_context = match pollster::block_on(RenderContext::new(window.clone())) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("Failed to create render context: {}", e);
                event_loop.exit();
                return;
            }
        };

        let egui_integration = EguiIntegration::new(
            event_loop,
            render_context.surface(),
            render_context.queue(),
            render_context.swapchain_format(),
        );

        let mut world = World::new();
        setup_demo_entities(&mut world);

        let mut console = ConsolePanel::new();
        console.show();
        console.log_success("=== Scripting Console Demo ===");
        console.log_info("Full Lua REPL with ECS introspection enabled!");
        console.log_info("");
        console.log_info("Quick Start:");
        console.log_info("  Type 'help' for built-in commands");
        console.log_info("  Try: console.list_entities()");
        console.log_info("  Try: 2 + 2");
        console.log_info("  Try: math.sqrt(16)");
        console.log_info("");

        let scripting_config = ScriptingConfig::default();
        let scripting_context = match ScriptingContext::new(scripting_config) {
            Ok(ctx) => Arc::new(RwLock::new(ctx)),
            Err(e) => {
                eprintln!("Failed to create scripting context: {}", e);
                event_loop.exit();
                return;
            }
        };

        console.set_lua_context(Arc::clone(&scripting_context));

        self.window = Some(window.clone());
        self.render_context = Some(render_context);
        self.egui_integration = Some(egui_integration);
        self.console = Some(console);
        self.world = Some(world);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            Some(w) => w.clone(),
            None => return,
        };

        // Let egui handle the event first
        if let Some(egui_integration) = &mut self.egui_integration {
            if egui_integration.handle_event(&window, &event) {
                window.request_redraw();
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.configure_surface(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.frame_count += 1;

                // Update world reference for console commands
                if let (Some(console), Some(world)) = (&mut self.console, &mut self.world) {
                    console.set_world(world);
                }

                // Animate entities slightly for visual feedback
                if self.frame_count % 60 == 0 {
                    if let Some(world) = &self.world {
                        let entity_count = world.inner().entities().len();
                        if entity_count > 0 {
                            if let Some(console) = &mut self.console {
                                console.log_debug(format!(
                                    "Frame {}: {} entities active",
                                    self.frame_count, entity_count
                                ));
                            }
                        }
                    }
                }

                // Render frame
                if let (Some(egui_integration), Some(console), Some(render_context), Some(world)) = (
                    &mut self.egui_integration,
                    &mut self.console,
                    &mut self.render_context,
                    &self.world,
                ) {
                    egui_integration.begin_frame(&window);

                    let ctx = egui_integration.context();

                    // Render console panel
                    console.render(ctx);

                    // Render instruction window
                    egui::Window::new("Scripting Console Demo")
                        .default_pos(egui::pos2(10.0, 10.0))
                        .default_size(egui::vec2(400.0, 350.0))
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.heading("Interactive Lua REPL with ECS");
                            ui.separator();

                            ui.label("Controls:");
                            ui.label("  ~ or F1: Toggle console");
                            ui.label("  Up/Down: Command history");
                            ui.label("  Tab: Autocomplete");
                            ui.separator();

                            ui.label("Lua Expressions:");
                            ui.label("  2 + 2");
                            ui.label("  math.sqrt(16)");
                            ui.label("  print('Hello')");
                            ui.separator();

                            ui.label("ECS Introspection:");
                            ui.label("  console.list_entities()");
                            ui.label("  console.entity_count()");
                            ui.label("  console.query_with_name()");
                            ui.label("  console.query_with_transform()");
                            ui.separator();

                            ui.label("Entity Operations:");
                            ui.label("  id = console.find_entity('Player')");
                            ui.label("  console.inspect(id)");
                            ui.label("  console.get_transform(id)");
                            ui.label("  console.set_transform(id, 10, 5, 0)");
                            ui.separator();

                            ui.label("Spawn/Despawn:");
                            ui.label("  console.spawn('NewEntity')");
                            ui.label("  console.despawn(id)");
                            ui.separator();

                            ui.label(format!("Frame: {}", self.frame_count));
                            ui.label(format!("Entities: {}", world.inner().entities().len()));
                        });

                    let (_full_output, _clipped_primitives) = egui_integration.end_frame(&window);

                    // Render clear frame (no 3D content)
                    if let Err(e) = render_context.render(&RenderCommands {
                        view: praxis_math::Mat4::IDENTITY,
                        proj: praxis_math::Mat4::IDENTITY,
                        draw_commands: &[],
                        lighting: None,
                    }) {
                        eprintln!("Render error: {}", e);
                    }
                }

                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        physical_key,
                        ..
                    },
                ..
            } => {
                // Handle console toggle
                let toggle = matches!(key, Key::Character(ref c) if c == "`" || c == "~")
                    || physical_key == PhysicalKey::Code(winit::keyboard::KeyCode::F1);

                if toggle {
                    if let Some(console) = &mut self.console {
                        console.toggle();
                    }
                    window.request_redraw();
                    return;
                }

                // Handle escape key when console is closed
                if matches!(key, Key::Named(NamedKey::Escape)) {
                    if let Some(console) = &self.console {
                        if !console.visible {
                            info!("Escape pressed, exiting...");
                            event_loop.exit();
                        }
                    }
                }

                window.request_redraw();
            }
            _ => {}
        }
    }
}

#[cfg(all(feature = "scripting", not(feature = "headless")))]
fn setup_demo_entities(world: &mut World) {
    // Spawn various entities for demonstration
    world.spawn((
        Name::new("Player"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    world.spawn((
        Name::new("Enemy_1"),
        Transform::from_xyz(5.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    world.spawn((
        Name::new("Enemy_2"),
        Transform::from_xyz(-5.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    world.spawn((
        Name::new("Pickup"),
        Transform::from_xyz(0.0, 2.0, 0.0),
        GlobalTransform::default(),
    ));

    world.spawn((
        Name::new("Camera"),
        Transform::from_xyz(0.0, 5.0, 10.0),
        GlobalTransform::default(),
    ));

    // Spawn some unnamed entities for testing
    world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));
}

#[cfg(all(feature = "scripting", not(feature = "headless")))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_gui::init()?;
    praxis_ecs::init()?;

    info!("Starting Scripting Console Demo");

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ScriptingConsoleDemo::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!(
        "scripting_console_demo example requires graphics support and cannot run in headless mode"
    );
    Ok(())
}

#[cfg(all(not(feature = "scripting"), not(feature = "headless")))]
fn main() {
    eprintln!("This example requires the 'scripting' feature to be enabled.");
    eprintln!("Run with: cargo run --example scripting_console_demo --features scripting");
    std::process::exit(1);
}
