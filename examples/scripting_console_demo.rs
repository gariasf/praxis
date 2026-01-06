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
//! - Escape: Close autocomplete
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

use praxis_core::{App, AppConfig};
use praxis_ecs::{GlobalTransform, Name, Transform, World};
use praxis_graphics::RenderContext;
use praxis_gui::{ConsolePanel, EguiIntegration};
use praxis_math::Vec3;
use praxis_scripting::{ScriptingConfig, ScriptingContext};
use praxis_utils::{info, Result};
use praxis_window::{Window, WindowConfig};
use std::sync::Arc;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{Key, NamedKey};

struct ScriptingConsoleDemo {
    window: Window,
    render_context: RenderContext,
    egui_integration: EguiIntegration,
    console: ConsolePanel,
    world: World,
    frame_count: u64,
}

impl ScriptingConsoleDemo {
    fn new() -> Result<Self> {
        info!("Initializing Scripting Console Demo");

        let window = Window::new(WindowConfig {
            title: "Scripting Console Demo - Press ~ or F1 to toggle console".to_string(),
            width: 1280,
            height: 720,
            ..Default::default()
        })?;

        let render_context = RenderContext::new(
            window.inner(),
            window.width(),
            window.height(),
            "Scripting Console Demo",
        )?;

        let egui_integration = EguiIntegration::new(
            window.inner(),
            render_context.device(),
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
        let scripting_context = Arc::new(parking_lot::RwLock::new(ScriptingContext::new(
            scripting_config,
        )?));

        console.set_lua_context(Arc::clone(&scripting_context));

        Ok(Self {
            window,
            render_context,
            egui_integration,
            console,
            world,
            frame_count: 0,
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
        self.frame_count += 1;

        // Set world reference for console commands
        self.console.set_world(&mut self.world);

        // Animate entities slightly for visual feedback
        if self.frame_count % 60 == 0 {
            let entity_count = self.world.inner().entities().len();
            if entity_count > 0 {
                self.console.log_debug(format!("Frame {}: {} entities active", self.frame_count, entity_count));
            }
        }
    }

    fn render(&mut self) -> Result<()> {
        let egui_ctx = self.egui_integration.context();

        self.egui_integration.begin_frame(self.window.inner());

        self.console.render(egui_ctx);

        egui::Window::new("Scripting Console Demo")
            .default_pos(egui::pos2(10.0, 10.0))
            .default_size(egui::vec2(400.0, 350.0))
            .resizable(false)
            .show(egui_ctx, |ui| {
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
                ui.label(format!("Entities: {}", self.world.inner().entities().len()));
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
        info!("Starting Scripting Console Demo");

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

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_gui::init()?;

    let demo = ScriptingConsoleDemo::new()?;
    demo.run()?;

    Ok(())
}
