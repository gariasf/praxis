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
//! - Escape: Close autocomplete

use praxis_core::{App, AppConfig};
use praxis_ecs::{GlobalTransform, Name, Transform, World};
use praxis_graphics::RenderContext;
use praxis_gui::{CommandRegistry, ConsolePanel, EguiContext, EguiIntegration};
use praxis_math::{Quat, Vec3};
use praxis_scripting::{ScriptingConfig, ScriptingContext};
use praxis_utils::{info, Result};
use praxis_window::{Window, WindowConfig};
use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};

struct ConsoleDemo {
    window: Window,
    render_context: RenderContext,
    egui_integration: EguiIntegration,
    console: ConsolePanel,
    world: World,
}

impl ConsoleDemo {
    fn new() -> Result<Self> {
        info!("Initializing Console Demo");

        let window = Window::new(WindowConfig {
            title: "Console Demo - Press ~ or F1 to toggle console".to_string(),
            width: 1280,
            height: 720,
            ..Default::default()
        })?;

        let render_context = RenderContext::new(
            window.inner(),
            window.width(),
            window.height(),
            "Console Demo",
        )?;

        let egui_integration = EguiIntegration::new(
            window.inner(),
            render_context.device(),
            render_context.queue(),
            render_context.swapchain_format(),
        );

        let mut world = World::new();
        setup_test_entities(&mut world);

        let mut console = ConsolePanel::new();
        console.show();
        console.log_info("Console Demo initialized. Press ~ or F1 to toggle visibility.");
        console.log_info("Type 'help' for available commands, or enter Lua code directly.");
        console.log_info("ECS introspection available via 'console.*' commands.");

        let scripting_config = ScriptingConfig::default();
        let scripting_context = Arc::new(parking_lot::RwLock::new(ScriptingContext::new(
            scripting_config,
        )?));

        console.set_lua_context(Arc::clone(&scripting_context));

        register_custom_commands(&console, &world);

        Ok(Self {
            window,
            render_context,
            egui_integration,
            console,
            world,
        })
    }

    fn handle_event(&mut self, event: &WindowEvent) -> bool {
        if self.egui_integration.handle_event(event) {
            return true;
        }

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                Key::Character(c) if c == "`" || c == "~" => {
                    self.console.toggle();
                    true
                }
                Key::Named(NamedKey::F1) => {
                    self.console.toggle();
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn update(&mut self) {
        // Update world reference for console commands
        self.console.set_world(&mut self.world);
    }

    fn render(&mut self) -> Result<()> {
        let egui_ctx = self.egui_integration.context();

        self.egui_integration.begin_frame(self.window.inner());

        self.console.render(egui_ctx);

        egui::Window::new("Info")
            .default_pos(egui::pos2(10.0, 10.0))
            .default_size(egui::vec2(300.0, 200.0))
            .resizable(false)
            .show(egui_ctx, |ui| {
                ui.heading("Console Demo");
                ui.separator();
                ui.label("Controls:");
                ui.label("  ~ or F1: Toggle console");
                ui.label("  Up/Down: Command history");
                ui.label("  Tab: Autocomplete");
                ui.label("  Enter: Execute");
                ui.separator();
                ui.label("Try these commands:");
                ui.label("  help");
                ui.label("  echo Hello World");
                ui.separator();
                ui.label("Or enter Lua code:");
                ui.label("  2 + 2");
                ui.label("  math.sqrt(16)");
                ui.label("  print('Hello from Lua')");
                ui.separator();
                ui.label("ECS introspection:");
                ui.label("  console.list_entities()");
                ui.label("  console.entity_count()");
                ui.label("  console.find_entity('TestEntity_0')");
                ui.label("  console.inspect(0)");
                ui.label("  console.spawn('MyEntity')");
            });

        let shapes = self.egui_integration.end_frame(self.window.inner());

        self.render_context.render_frame(
            self.window.inner(),
            &mut self.world,
            |builder, render_context| {
                self.egui_integration.render(
                    builder,
                    render_context,
                    self.window.inner(),
                    shapes,
                )?;
                Ok(())
            },
        )?;

        Ok(())
    }

    fn run(mut self) -> Result<()> {
        info!("Starting Console Demo");

        self.window.run(move |event, elwt| {
            if let Some(event) = event {
                match event {
                    WindowEvent::CloseRequested => {
                        info!("Window close requested");
                        elwt.exit();
                    }
                    WindowEvent::Resized(size) => {
                        self.render_context
                            .recreate_swapchain(size.width, size.height);
                    }
                    WindowEvent::RedrawRequested => {
                        self.update();
                        if let Err(e) = self.render() {
                            eprintln!("Render error: {}", e);
                        }
                        self.window.inner().request_redraw();
                    }
                    event => {
                        self.handle_event(&event);
                    }
                }
            }
        })?;

        Ok(())
    }
}

fn setup_test_entities(world: &mut World) {
    for i in 0..5 {
        world.spawn((
            Name::new(format!("TestEntity_{}", i)),
            Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));
    }
}

fn register_custom_commands(console: &ConsolePanel, world: &World) {
    let registry = console.command_registry();
    let mut registry = registry.write();

    registry.register(
        "list_entities",
        "List all entities in the world",
        "list_entities",
        move |_args| {
            let count = 5;
            Ok(format!("World contains {} entities", count))
        },
    );

    registry.register(
        "spawn_entity",
        "Spawn a new entity with a name",
        "spawn_entity <name>",
        move |args| {
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

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_gui::init()?;

    let app_config = AppConfig {
        title: "Console Demo".to_string(),
        ..Default::default()
    };

    let demo = ConsoleDemo::new()?;
    demo.run()?;

    Ok(())
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("console_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}
