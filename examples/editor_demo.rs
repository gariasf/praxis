//! Editor demonstration with EditorState integration.
//!
//! This example demonstrates the Praxis editor with:
//! - Window setup
//! - EditorState initialization with all panels
//! - Event loop integration  
//! - Console logging integration
//! - Menu bar and toolbar
//!
//! # Controls
//!
//! ## File Menu
//! - **Ctrl+N**: New Scene
//! - **Ctrl+O**: Open Scene
//! - **Ctrl+S**: Save Scene
//!
//! ## Edit Menu
//! - **Ctrl+Z**: Undo
//! - **Ctrl+Y**: Redo
//!
//! ## Other
//! - **Escape**: Exit

use praxis_ecs::World;
use praxis_editor::{init_with_console, EditorState, LogBuffer, UndoRedoSystem};
use praxis_gui::EguiContext;
use praxis_input::InputState;
use praxis_utils::{error, info, Result};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 1920;
const WINDOW_HEIGHT: u32 = 1080;

/// Main application state.
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    editor_state: Option<EditorState>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            world: None,
            editor_state: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing...");

        // Create window
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis Editor Demo")
                .with_resizable(true),
        ) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Create editor state with console integration
        let log_buffer = LogBuffer::new();
        let editor_state = EditorState::with_log_buffer(log_buffer);

        // Create ECS world with resources
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(UndoRedoSystem::new());
        world.insert_resource(EguiContext::default());

        info!("=== Praxis Editor Demo ===");
        info!("This demo shows EditorState integration with:");
        info!("  • Scene View panel");
        info!("  • Hierarchy panel");
        info!("  • Inspector panel");
        info!("  • Console panel with log capture");
        info!("  • Assets panel");
        info!("  • Menu bar (File/Edit/Entity/View/Help)");
        info!("  • Toolbar with gizmo controls and playback");
        info!("  • Undo/Redo system");
        info!("");
        info!("Controls:");
        info!("  Ctrl+Z/Y - Undo/Redo");
        info!("  ESC - Exit");

        self.window = Some(window.clone());
        self.world = Some(world);
        self.editor_state = Some(editor_state);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Update input state
                if let Some(world) = &mut self.world {
                    {
                        let mut input_state = world.get_resource_mut::<InputState>().unwrap();
                        input_state.update();
                    }

                    // Render editor UI
                    if let Some(editor_state) = &mut self.editor_state {
                        let ctx = world.get_resource::<EguiContext>().unwrap().context().clone();
                        // Note: We pass None for undo_system and world to avoid borrowing issues
                        // In a full implementation, these would be properly integrated
                        editor_state.ui(&ctx, None, None);
                    }
                }

                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(KeyCode::Escape)
                    && event.state.is_pressed()
                {
                    info!("Escape pressed, exiting...");
                    event_loop.exit();
                }

                if let Some(world) = &mut self.world {
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
            }
            _ => {
                if let Some(world) = &mut self.world {
                    let mut input_state = world.get_resource_mut::<InputState>().unwrap();
                    praxis_input::winit_integration::process_window_event(&mut input_state, &event);
                }
            }
        }

        window.request_redraw();
    }
}

fn main() -> Result<()> {
    // Initialize engine systems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_gui::init()?;

    // Initialize editor with console logging
    let log_buffer = LogBuffer::new();
    init_with_console(log_buffer)?;

    info!("Starting Praxis Editor Demo");

    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}
