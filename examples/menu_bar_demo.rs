//! MenuBar demonstration.
//!
//! This example demonstrates the MenuBar system with:
//! - File menu (New, Open, Save, Save As, Exit)
//! - Edit menu (Undo, Redo, Copy, Paste, Duplicate)
//! - Entity menu (Create Empty, Create Primitives, Delete)
//! - View menu (Toggle Panels)
//! - Help menu (About, Documentation)
//! - Standard keyboard shortcuts
//!
//! # Controls
//!
//! ## File Menu
//! - **Ctrl+N**: New Scene
//! - **Ctrl+O**: Open Scene
//! - **Ctrl+S**: Save Scene
//! - **Ctrl+Shift+S**: Save Scene As
//! - **Alt+F4**: Exit
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
//! ## Help Menu
//! - **F1**: Documentation
//!
//! ## Other
//! - **Escape**: Exit

use praxis_core::{Engine, EngineBuilder};
use praxis_ecs::{Commands, IntoSystemConfigs, Query, Res, ResMut, Resource, Schedule, Transform, With, World};
use praxis_editor::{EditorMode, EditorState, UndoRedoSystem};
use praxis_gui::EguiContext;
use praxis_input::InputState;
use praxis_utils::{info, FrameTimer, Result};
use praxis_window::State;
use std::sync::Arc;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;

/// System that updates the editor UI.
fn editor_ui_system(
    egui_context: Res<EguiContext>,
    mut editor_state: ResMut<EditorState>,
    mut undo_system: ResMut<UndoRedoSystem>,
    world: &mut World,
) {
    let ctx = egui_context.context();
    editor_state.ui(ctx, Some(&mut undo_system), Some(world));
}

fn main() -> Result<()> {
    // Initialize engine systems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_editor::init()?;
    praxis_gui::init()?;

    info!("Starting MenuBar Demo");
    info!("Controls:");
    info!("  File Menu:");
    info!("    Ctrl+N: New Scene");
    info!("    Ctrl+O: Open Scene");
    info!("    Ctrl+S: Save Scene");
    info!("    Ctrl+Shift+S: Save Scene As");
    info!("    Alt+F4: Exit");
    info!("  Edit Menu:");
    info!("    Ctrl+Z: Undo");
    info!("    Ctrl+Y: Redo");
    info!("    Ctrl+C: Copy");
    info!("    Ctrl+V: Paste");
    info!("    Ctrl+D: Duplicate");
    info!("  Entity Menu:");
    info!("    Delete: Delete Entity");
    info!("  Help Menu:");
    info!("    F1: Documentation");
    info!("  Other:");
    info!("    Escape: Exit");

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("MenuBar Demo - Praxis Editor")
            .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
            .build(&event_loop)?,
    );

    // Create ECS world and schedule
    let mut world = praxis_ecs::World::new();
    
    // Insert resources
    world.insert_resource(InputState::default());
    world.insert_resource(EditorState::new());
    world.insert_resource(UndoRedoSystem::new());
    world.insert_resource(FrameTimer::new());
    world.insert_resource(EguiContext::default());

    // Setup systems
    let mut schedule = Schedule::default();
    schedule.add_systems(editor_ui_system);

    // Create state and run event loop
    let mut state = State::new(window.clone()).await?;
    
    event_loop.run(move |event, elwt| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested, exiting");
                    elwt.exit();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.physical_key == winit::keyboard::PhysicalKey::Code(KeyCode::Escape)
                        && event.state == ElementState::Pressed
                    {
                        info!("Escape pressed, exiting");
                        elwt.exit();
                    }
                    
                    // Update input state
                    let mut input = world.get_resource_mut::<InputState>().unwrap();
                    input.handle_keyboard_input(event.physical_key, event.state);
                }
                WindowEvent::MouseInput { state: button_state, button, .. } => {
                    let mut input = world.get_resource_mut::<InputState>().unwrap();
                    input.handle_mouse_button(button.into(), button_state);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mut input = world.get_resource_mut::<InputState>().unwrap();
                    input.handle_cursor_moved((position.x, position.y));
                }
                WindowEvent::Resized(size) => {
                    state.resize(size);
                }
                WindowEvent::RedrawRequested => {
                    // Update frame timer
                    {
                        let mut timer = world.get_resource_mut::<FrameTimer>().unwrap();
                        timer.tick();
                    }

                    // Run systems
                    schedule.run(world.inner_mut());

                    // Update input state for next frame
                    {
                        let mut input = world.get_resource_mut::<InputState>().unwrap();
                        input.update();
                    }

                    // Request next frame
                    window.request_redraw();
                }
                _ => {}
            },
            winit::event::Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
