//! Selection system demonstration.
//!
//! This example demonstrates the SelectionSystem with:
//! - Click-to-select entities with raycast picking
//! - Multi-entity selection with keyboard modifiers
//! - Marquee (box) selection by dragging
//! - Keyboard shortcuts (Ctrl+A, Ctrl+D)
//! - Visual feedback for selected entities
//!
//! # Controls
//!
//! - **Left Click**: Select entity (replace selection)
//! - **Shift+Left Click**: Add entity to selection
//! - **Ctrl+Left Click**: Remove entity from selection
//! - **Alt+Left Click**: Toggle entity selection
//! - **Left Click + Drag**: Marquee selection (box select)
//! - **Ctrl+A**: Select all entities
//! - **Ctrl+D**: Deselect all entities
//! - **Escape**: Exit
//!
//! Selected entities are highlighted with a yellow color.

mod common;

use common::{create_demo_scene, PrimitiveShape};
use praxis_core::{Engine, EngineBuilder};
use praxis_ecs::{
    Camera, CameraMatrices, Commands, Entity, GlobalTransform, IntoSystemConfigs,
    PerspectiveProjection, Query, Res, ResMut, Resource, Schedule, Transform, With,
};
use praxis_editor::{
    handle_selection_input_system, update_selection_system, Selectable, Selected, SelectionMode,
    SelectionSystem,
};
use praxis_graphics::{MaterialProperties, RenderContext};
use praxis_input::{InputState, MouseButton};
use praxis_math::{Vec2, Vec3};
use praxis_utils::{info, FrameTimer, Result};
use praxis_window::State;
use std::sync::Arc;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;

/// Resource tracking viewport information for selection.
#[derive(Resource, Debug, Clone)]
struct ViewportInfo {
    /// Viewport size in pixels.
    size: Vec2,
    /// Whether mouse is over the viewport.
    is_hovered: bool,
}

impl Default for ViewportInfo {
    fn default() -> Self {
        Self {
            size: Vec2::new(1920.0, 1080.0),
            is_hovered: true,
        }
    }
}

/// System that handles mouse input for selection.
fn handle_mouse_selection_system(
    mut selection: ResMut<SelectionSystem>,
    input: Res<InputState>,
    viewport: Res<ViewportInfo>,
    camera_query: Query<(&Transform, &CameraMatrices), With<Camera>>,
    selectable_query: Query<(Entity, &GlobalTransform), With<Selectable>>,
) {
    if !viewport.is_hovered || !selection.is_input_enabled() {
        return;
    }

    let (camera_transform, camera_matrices) = match camera_query.iter().next() {
        Some(camera) => camera,
        None => return,
    };

    let mouse_pos = Vec2::new(
        input.mouse_position().0 as f32,
        input.mouse_position().1 as f32,
    );

    // Determine selection mode from modifiers
    let shift =
        input.is_key_pressed(KeyCode::ShiftLeft) || input.is_key_pressed(KeyCode::ShiftRight);
    let ctrl =
        input.is_key_pressed(KeyCode::ControlLeft) || input.is_key_pressed(KeyCode::ControlRight);
    let alt = input.is_key_pressed(KeyCode::AltLeft) || input.is_key_pressed(KeyCode::AltRight);

    let selection_mode = if shift {
        SelectionMode::Add
    } else if ctrl {
        SelectionMode::Remove
    } else if alt {
        SelectionMode::Toggle
    } else {
        SelectionMode::Replace
    };

    // Handle left mouse button for click selection
    if input.is_mouse_button_just_pressed(MouseButton::Left) {
        // Start potential marquee selection
        selection.start_marquee(mouse_pos);
    }

    // Update marquee while dragging
    if selection.is_marquee_active() && input.is_mouse_button_pressed(MouseButton::Left) {
        selection.update_marquee(mouse_pos);
    }

    // End selection on mouse release
    if input.is_mouse_button_just_released(MouseButton::Left) {
        if let Some((rect_min, rect_max)) = selection.end_marquee() {
            // Check if this was a click (small movement) or a drag (marquee)
            let drag_distance = (rect_max - rect_min).length();

            if drag_distance < 5.0 {
                // Click selection - raycast pick
                if let Some(entity) = selection.raycast_pick(
                    mouse_pos,
                    viewport.size,
                    camera_transform,
                    camera_matrices,
                    &selectable_query,
                ) {
                    selection.select_entity(entity, selection_mode);
                } else if selection_mode == SelectionMode::Replace {
                    // Clicked empty space with no modifiers - clear selection
                    selection.clear();
                }
            } else {
                // Marquee selection
                let entities = selection.marquee_pick(
                    rect_min,
                    rect_max,
                    viewport.size,
                    camera_matrices,
                    &selectable_query,
                );

                if !entities.is_empty() {
                    selection.select_entities(entities, selection_mode);
                }
            }
        }
    }

    // Cancel marquee on right click
    if input.is_mouse_button_just_pressed(MouseButton::Right) {
        selection.cancel_marquee();
    }
}

/// System that prints selection change events.
fn print_selection_events_system(mut selection: ResMut<SelectionSystem>) {
    for event in selection.drain_events() {
        info!("Selection event: {:?}", event);
    }
}

/// System that updates materials for selected entities.
fn update_selected_visuals_system(
    mut commands: Commands,
    selected_query: Query<Entity, With<Selected>>,
    not_selected_query: Query<Entity, (With<Selectable>, Without<Selected>)>,
) {
    // Highlight selected entities (yellow)
    for entity in selected_query.iter() {
        commands.entity(entity).insert(
            praxis_ecs::MaterialPropertiesComponent::default()
                .with_base_color([1.0, 1.0, 0.0, 1.0]) // Yellow
                .with_metallic(0.0)
                .with_roughness(0.5),
        );
    }

    // Reset non-selected entities (white)
    for entity in not_selected_query.iter() {
        commands.entity(entity).insert(
            praxis_ecs::MaterialPropertiesComponent::default()
                .with_base_color([1.0, 1.0, 1.0, 1.0]) // White
                .with_metallic(0.0)
                .with_roughness(0.5),
        );
    }
}

fn main() -> Result<()> {
    // Initialize engine systems
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_editor::init()?;

    info!("Starting Selection Demo");
    info!("Controls:");
    info!("  Left Click: Select entity (replace selection)");
    info!("  Shift+Left Click: Add entity to selection");
    info!("  Ctrl+Left Click: Remove entity from selection");
    info!("  Alt+Left Click: Toggle entity selection");
    info!("  Left Click + Drag: Marquee selection (box select)");
    info!("  Ctrl+A: Select all entities");
    info!("  Ctrl+D: Deselect all entities");
    info!("  Escape: Exit");

    // Create event loop and window
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Selection System Demo")
            .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080))
            .build(&event_loop)?,
    );

    // Create ECS world and schedule
    let mut world = praxis_ecs::World::new();

    // Insert resources
    world.insert_resource(InputState::default());
    world.insert_resource(SelectionSystem::new());
    world.insert_resource(ViewportInfo::default());
    world.insert_resource(FrameTimer::new());

    // Setup systems
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            handle_selection_input_system,
            handle_mouse_selection_system,
            update_selection_system,
            print_selection_events_system,
            update_selected_visuals_system,
        )
            .chain(),
    );

    // Create camera
    world.spawn((
        Transform::from_xyz(0.0, 10.0, 20.0),
        Camera::default(),
        PerspectiveProjection::default(),
        CameraMatrices::default(),
    ));

    // Create selectable entities in a grid
    for x in -2..=2 {
        for z in -2..=2 {
            let position = Vec3::new(x as f32 * 3.0, 0.0, z as f32 * 3.0);

            world.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
                Selectable,
                praxis_ecs::MeshHandle::new("cube"),
                praxis_ecs::MaterialPropertiesComponent::default()
                    .with_base_color([1.0, 1.0, 1.0, 1.0])
                    .with_metallic(0.0)
                    .with_roughness(0.5),
            ));
        }
    }

    // Add a ground plane (not selectable)
    world.spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        GlobalTransform::default(),
        praxis_ecs::MeshHandle::new("plane"),
        praxis_ecs::MaterialPropertiesComponent::default()
            .with_base_color([0.5, 0.5, 0.5, 1.0])
            .with_metallic(0.0)
            .with_roughness(0.8),
    ));

    // Add a directional light
    world.spawn(praxis_ecs::DirectionalLight::new(
        Vec3::new(0.5, -1.0, 0.3).normalize(),
        Vec3::new(1.0, 0.95, 0.8),
        1.0,
    ));

    // Initialize lighting data
    world.insert_resource(praxis_ecs::LightingData::default());

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
                WindowEvent::MouseInput {
                    state: button_state,
                    button,
                    ..
                } => {
                    let mut input = world.get_resource_mut::<InputState>().unwrap();
                    input.handle_mouse_button(button.into(), button_state);
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mut input = world.get_resource_mut::<InputState>().unwrap();
                    input.handle_cursor_moved((position.x, position.y));
                }
                WindowEvent::Resized(size) => {
                    state.resize(size);
                    let mut viewport = world.get_resource_mut::<ViewportInfo>().unwrap();
                    viewport.size = Vec2::new(size.width as f32, size.height as f32);
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
