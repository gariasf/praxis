//! Console panel demonstration with command history, Lua REPL, and custom commands.
//!
//! This example demonstrates the Praxis console panel with:
//! - Command history navigation (up/down arrows)
//! - Lua REPL integration for executing Lua code
//! - Custom debug command registration
//! - Autocomplete for commands (Tab key)
//! - Log filtering by level and text search
//! - Auto-scroll toggle
//!
//! Controls:
//! - ~ or F1: Toggle console visibility
//! - Up/Down: Navigate command history
//! - Tab: Cycle through autocomplete suggestions
//! - Enter: Execute command or Lua code
//! - Escape: Exit (when console is closed)

#[cfg(all(feature = "scripting", not(feature = "headless")))]
use parking_lot::RwLock;
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_ecs::{GlobalTransform, Name, Transform, World};
#[cfg(all(feature = "scripting", not(feature = "headless")))]
use praxis_graphics::RenderContext;
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
struct ConsoleDemo {
    window: Option<Arc<Window>>,
    render_context: Option<RenderContext>,
    egui_integration: Option<EguiIntegration>,
    console: Option<ConsolePanel>,
    world: Option<World>,
}

#[cfg(all(feature = "scripting", not(feature = "headless")))]
impl ApplicationHandler for ConsoleDemo {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing Console Demo...");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Console Demo (Press ~ or F1 to toggle)")
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
        setup_test_entities(&mut world);

        let mut console = ConsolePanel::new();
        console.show();
        console.log_info("Console Demo initialized. Press ~ or F1 to toggle visibility.");
        console.log_info("Type 'help' for available commands, or enter Lua code directly.");

        let scripting_config = ScriptingConfig::default();
        let scripting_context = match ScriptingContext::new(scripting_config) {
            Ok(ctx) => Arc::new(RwLock::new(ctx)),
            Err(e) => {
                eprintln!("Failed to create scripting context: {}", e);
                event_loop.exit();
                return;
            }
        };

        #[cfg(feature = "scripting")]
        console.set_lua_context(scripting_context.clone());
        register_custom_commands(&console);

        info!("=== Console Demo ===");
        info!("Controls:");
        info!("  ~ or F1: Toggle console");
        info!("  Up/Down: Command history");
        info!("  Tab: Autocomplete");
        info!("  Enter: Execute");
        info!("  ESC: Exit (when console is closed)");
        info!("");
        info!("Try these commands:");
        info!("  help");
        info!("  echo Hello World");
        info!("  list_entities");
        info!("");
        info!("Or Lua code:");
        info!("  2 + 2");
        info!("  math.sqrt(16)");
        info!("  print('Hello from Lua')");

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
                // Update world reference for console commands
                #[cfg(feature = "scripting")]
                if let (Some(console), Some(world)) = (&mut self.console, &mut self.world) {
                    console.set_world(world);
                }

                // Render frame
                if let (Some(egui_integration), Some(console), Some(render_context)) = (
                    &mut self.egui_integration,
                    &mut self.console,
                    &mut self.render_context,
                ) {
                    egui_integration.begin_frame(&window);

                    let ctx = egui_integration.context();

                    // Render console panel
                    console.render(ctx);

                    let (_full_output, _clipped_primitives) = egui_integration.end_frame(&window);

                    // Render clear frame (no 3D content)
                    if let Err(e) = render_context.render(&praxis_graphics::RenderCommands {
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
fn setup_test_entities(world: &mut World) {
    for i in 0..5 {
        world.spawn((
            Name::new(format!("TestEntity_{}", i)),
            Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
    }
}

#[cfg(all(feature = "scripting", not(feature = "headless")))]
fn register_custom_commands(console: &ConsolePanel) {
    let registry = console.command_registry();
    let mut registry = registry.write();

    registry.register(
        "list_entities",
        "List all entities in the world",
        "list_entities",
        |_args| Ok("Use console.list_entities() in Lua for ECS queries".to_string()),
    );

    registry.register(
        "spawn_entity",
        "Spawn a new entity with a name",
        "spawn_entity <name>",
        |args| {
            if args.is_empty() {
                return Err("Usage: spawn_entity <name>".to_string());
            }
            let name = args.join(" ");
            Ok(format!("Created entity: {}", name))
        },
    );

    registry.register("fps", "Display current FPS", "fps", |_args| {
        Ok("FPS: 60.0".to_string())
    });

    registry.register("mem", "Display memory usage", "mem", |_args| {
        Ok("Memory: 142 MB / 16 GB".to_string())
    });

    registry.register("version", "Display engine version", "version", |_args| {
        Ok("Praxis Engine v0.1.0".to_string())
    });

    registry.register("time", "Display current time", "time", |_args| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(format!("Unix timestamp: {}", now))
    });
}

#[cfg(all(feature = "scripting", not(feature = "headless")))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_gui::init()?;
    praxis_ecs::init()?;

    info!("Starting Console Demo");

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ConsoleDemo::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("console_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}

#[cfg(all(not(feature = "scripting"), not(feature = "headless")))]
fn main() {
    eprintln!("This example requires the 'scripting' feature to be enabled.");
    eprintln!("Run with: cargo run --example console_demo --features scripting");
    std::process::exit(1);
}
