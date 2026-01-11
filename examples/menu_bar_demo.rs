//! Menu bar demonstration with editor integration.
//!
//! This example demonstrates the Praxis editor menu bar with:
//! - File menu (New, Open, Save, Save As, Exit)
//! - Edit menu (Undo, Redo, Copy, Paste, Duplicate)
//! - Entity menu (Create Empty, Create Primitives, Delete)
//! - View menu (Toggle Panels)
//! - Help menu (About, Documentation)
//! - Standard keyboard shortcuts
//!
//! # Keyboard Shortcuts
//!
//! ## File Menu
//! - **Ctrl+N**: New Scene
//! - **Ctrl+O**: Open Scene
//! - **Ctrl+S**: Save Scene
//! - **Ctrl+Shift+S**: Save Scene As
//! - **Alt+F4**: Exit (platform default)
//!
//! ## Edit Menu
//! - **Ctrl+Z**: Undo
//! - **Ctrl+Y**: Redo
//! - **Ctrl+C**: Copy
//! - **Ctrl+V**: Paste
//! - **Ctrl+D**: Duplicate
//!
//! ## Entity Menu
//! - **Delete**: Delete Entity
//!
//! ## Other
//! - **Escape**: Exit

#[cfg(feature = "editor")]
use praxis_ecs::World;
#[cfg(feature = "editor")]
use praxis_editor::{EditorState, LogBuffer, UndoRedoSystem};
#[cfg(feature = "editor")]
use praxis_graphics::RenderContext;
#[cfg(feature = "editor")]
use praxis_gui::EguiIntegration;
#[cfg(feature = "editor")]
use praxis_input::InputState;
#[cfg(feature = "editor")]
use praxis_utils::{error, info, Result};
#[cfg(feature = "editor")]
use std::sync::Arc;
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
const WINDOW_WIDTH: u32 = 1920;
#[cfg(feature = "editor")]
const WINDOW_HEIGHT: u32 = 1080;

/// Main application state.
#[cfg(feature = "editor")]
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    editor_state: Option<EditorState>,
    render_context: Option<RenderContext>,
    egui_integration: Option<EguiIntegration>,
}

#[cfg(feature = "editor")]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Application resumed, initializing Menu Bar Demo...");

        // Create window
        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Menu Bar Demo")
                .with_resizable(true),
        ) {
            Ok(w) => Arc::new(w),
            Err(e) => {
                error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Create render context
        let render_context = match pollster::block_on(RenderContext::new(window.clone())) {
            Ok(ctx) => ctx,
            Err(e) => {
                error!("Failed to create render context: {}", e);
                event_loop.exit();
                return;
            }
        };

        // Create egui integration
        let egui_integration = EguiIntegration::new(
            event_loop,
            render_context.surface(),
            render_context.queue(),
            render_context.swapchain_format(),
        );

        // Create editor state with console integration
        let log_buffer = LogBuffer::new();
        let editor_state = EditorState::with_log_buffer(log_buffer);

        // Create ECS world with resources
        let mut world = World::new();
        world.insert_resource(InputState::default());
        world.insert_resource(UndoRedoSystem::new());

        info!("=== Menu Bar Demo ===");
        info!("This demo shows the Praxis editor menu bar with:");
        info!("  • File menu (New, Open, Save, Save As, Exit)");
        info!("  • Edit menu (Undo, Redo, Copy, Paste, Duplicate)");
        info!("  • Entity menu (Create Empty, Create Primitives, Delete)");
        info!("  • View menu (Toggle Panels)");
        info!("  • Help menu (About, Documentation)");
        info!("");
        info!("Keyboard Shortcuts:");
        info!("  File: Ctrl+N/O/S, Ctrl+Shift+S");
        info!("  Edit: Ctrl+Z/Y/C/V/D");
        info!("  Entity: Delete");
        info!("  Other: ESC to exit");

        self.window = Some(window.clone());
        self.world = Some(world);
        self.editor_state = Some(editor_state);
        self.render_context = Some(render_context);
        self.egui_integration = Some(egui_integration);

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let window = match self.window.as_ref() {
            Some(window) => window,
            None => return,
        };

        // Let egui handle the event first
        if let Some(egui_integration) = &mut self.egui_integration {
            if egui_integration.handle_event(window, &event) {
                window.request_redraw();
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting...");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.configure_surface(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // Update input state
                if let Some(world) = &mut self.world {
                    {
                        let input_state = world.get_resource_mut::<InputState>().unwrap();
                        input_state.update();
                    }

                    // Render editor UI with menu bar
                    if let (Some(editor_state), Some(egui_integration), Some(render_context)) = (
                        &mut self.editor_state,
                        &mut self.egui_integration,
                        &mut self.render_context,
                    ) {
                        egui_integration.begin_frame(window);

                        let ctx = egui_integration.context();
                        
                        // Render the full editor UI with menu bar
                        editor_state.ui(ctx, None, Some(world), None, Some(render_context));

                        let (_full_output, _clipped_primitives) = egui_integration.end_frame(window);

                        // Render a simple frame (just clear screen)
                        if let Err(e) = render_context.render(&praxis_graphics::RenderCommands {
                            view: praxis_math::Mat4::IDENTITY,
                            proj: praxis_math::Mat4::IDENTITY,
                            draw_commands: &[],
                            lighting: None,
                        }) {
                            error!("Render error: {}", e);
                        }
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
                    let input_state = world.get_resource_mut::<InputState>().unwrap();
                    praxis_input::winit_integration::process_window_event(
                        input_state,
                        &WindowEvent::KeyboardInput {
                            device_id: winit::event::DeviceId::dummy(),
                            event: event.clone(),
                            is_synthetic: false,
                        },
                    );
                }
                window.request_redraw();
            }
            _ => {
                if let Some(world) = &mut self.world {
                    let input_state = world.get_resource_mut::<InputState>().unwrap();
                    praxis_input::winit_integration::process_window_event(input_state, &event);
                }
                window.request_redraw();
            }
        }
    }
}

#[cfg(all(feature = "editor", not(feature = "headless")))]
fn main() -> Result<()> {
    // Initialize engine systems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_gui::init()?;

    info!("Starting Menu Bar Demo");

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
    println!("menu_bar_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}

#[cfg(all(not(feature = "editor"), not(feature = "headless")))]
fn main() {
    eprintln!("This example requires the 'editor' feature to be enabled.");
    eprintln!("Run with: cargo run --example menu_bar_demo --features editor");
    std::process::exit(1);
}
