//! Selection system demonstration with raycast picking.
//!
//! This example demonstrates the editor selection system with:
//! - Click-to-select entities with raycast picking
//! - Multi-entity selection with keyboard modifiers
//! - Visual feedback for selected entities
//! - Keyboard shortcuts (Ctrl+A, Ctrl+D)

#[cfg(feature = "editor")]
use praxis_ecs::{
    BoundingBox, CameraMatrices, GlobalTransform, MeshHandle, PerspectiveCameraBundle, Transform,
    World,
};
#[cfg(feature = "editor")]
use praxis_editor::{Selectable, SelectionMode, SelectionSystem};
#[cfg(feature = "editor")]
use praxis_graphics::RenderContext;
#[cfg(feature = "editor")]
use praxis_gui::EguiIntegration;
#[cfg(feature = "editor")]
use praxis_input::{InputState, MouseButton};
#[cfg(feature = "editor")]
use praxis_math::Vec3;
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
const WINDOW_WIDTH: u32 = 1280;
#[cfg(feature = "editor")]
const WINDOW_HEIGHT: u32 = 720;

#[cfg(all(feature = "editor", not(feature = "headless")))]
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
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    egui_integration: Option<EguiIntegration>,
    camera_entity: Option<bevy_ecs::entity::Entity>,
}

#[cfg(feature = "editor")]
impl App {
    fn setup_world() -> (World, bevy_ecs::entity::Entity) {
        let mut world = World::new();

        world.insert_resource(InputState::default());
        world.insert_resource(SelectionSystem::new());

        // Create camera
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 5.0, 15.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        info!("Created camera entity: {:?}", camera_entity);

        // Create selectable objects in the scene with bounding boxes
        let positions = [
            Vec3::new(-5.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(-5.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(5.0, 0.0, -5.0),
        ];

        for (i, pos) in positions.iter().enumerate() {
            world.spawn((
                Transform::from_xyz(pos.x, pos.y, pos.z),
                GlobalTransform::default(),
                BoundingBox::from_center_half_extents(Vec3::ZERO, Vec3::ONE),
                MeshHandle::new("cube"),
                Selectable,
            ));
            info!("Created selectable entity {} at {:?}", i, pos);
        }

        (world, camera_entity)
    }
}

#[cfg(feature = "editor")]
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis Selection Demo")
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

        let (world, camera_entity) = Self::setup_world();

        info!("=== Praxis Selection System Demo ===");
        info!("Controls:");
        info!("  LMB - Select entity (replace selection)");
        info!("  Shift+LMB - Add entity to selection");
        info!("  Ctrl+LMB - Remove entity from selection");
        info!("  Alt+LMB - Toggle entity selection");
        info!("  Ctrl+A - Select all");
        info!("  Ctrl+D - Deselect all");
        info!("  ESC - Exit");
        info!("");
        info!("Click on the cubes to select them!");

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.egui_integration = Some(egui_integration);
        self.camera_entity = Some(camera_entity);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let world = match self.world.as_mut() {
            Some(world) => world,
            None => return,
        };

        let window = match self.window.as_ref() {
            Some(w) => w,
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
                info!("\nExiting...");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(render_context) = &mut self.render_context {
                    render_context.configure_surface(size.width, size.height);
                }
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                {
                    let input_state = world.get_resource_mut::<InputState>().unwrap();
                    input_state.update();
                }

                // Handle raycast picking
                {
                    let window_opt = self.window.as_ref();
                    let camera_entity_opt = self.camera_entity;

                    if let (Some(window_ref), Some(camera_entity)) = (window_opt, camera_entity_opt)
                    {
                        let window_size = {
                            let size = window_ref.inner_size();
                            praxis_math::Vec2::new(size.width as f32, size.height as f32)
                        };

                        let (is_lmb_pressed, mouse_pos) = {
                            let input = world.get_resource::<InputState>().unwrap();
                            (
                                input.is_mouse_button_just_pressed(MouseButton::Left),
                                input.mouse_position(),
                            )
                        };

                        if is_lmb_pressed {
                            let mouse_pos_vec2 =
                                praxis_math::Vec2::new(mouse_pos.0 as f32, mouse_pos.1 as f32);

                            let camera_transform =
                                match world.inner().get::<Transform>(camera_entity) {
                                    Some(t) => *t,
                                    None => Transform::default(),
                                };

                            let camera_matrices =
                                match world.inner().get::<CameraMatrices>(camera_entity) {
                                    Some(m) => *m,
                                    None => {
                                        return;
                                    }
                                };

                            let mut selectable_query = world.inner_mut().query_filtered::<(
                                bevy_ecs::entity::Entity,
                                &GlobalTransform,
                            ), bevy_ecs::query::With<
                                Selectable,
                            >>(
                            );
                            let selectables: Vec<_> = selectable_query
                                .iter(world.inner())
                                .map(|(e, t)| (e, *t))
                                .collect();

                            let mut bounds_data = std::collections::HashMap::new();
                            {
                                let mut bounds_query = world.inner_mut().query::<&BoundingBox>();
                                for (entity, _) in &selectables {
                                    if let Ok(bb) = bounds_query.get(world.inner(), *entity) {
                                        bounds_data.insert(*entity, *bb);
                                    }
                                }
                            }

                            let picked = {
                                let mut closest_entity = None;
                                let mut closest_distance = f32::MAX;

                                let ndc_x = (2.0 * mouse_pos_vec2.x) / window_size.x - 1.0;
                                let ndc_y = 1.0 - (2.0 * mouse_pos_vec2.y) / window_size.y;

                                let inv_projection = camera_matrices.projection.inverse();
                                let clip = praxis_math::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
                                let view = inv_projection * clip;
                                let view_dir =
                                    praxis_math::Vec3::new(view.x, view.y, view.z) / view.w;
                                let ray_dir =
                                    (camera_transform.rotation * view_dir.normalize()).normalize();
                                let ray_origin = camera_transform.translation;

                                for (entity, global_transform) in &selectables {
                                    if let Some(bounding_box) = bounds_data.get(entity) {
                                        let world_matrix = global_transform.matrix;
                                        let aabb = praxis_spatial::Aabb::from_min_max(
                                            bounding_box.min,
                                            bounding_box.max,
                                        )
                                        .transform(&world_matrix);

                                        if let Some(distance) = aabb.ray_intersection_distance(
                                            ray_origin,
                                            ray_dir,
                                            closest_distance,
                                        ) {
                                            if distance < closest_distance {
                                                closest_distance = distance;
                                                closest_entity = Some(*entity);
                                            }
                                        }
                                    }
                                }

                                closest_entity
                            };

                            let (ctrl, shift, alt) = {
                                let input = world.get_resource::<InputState>().unwrap();
                                (
                                    input.is_key_pressed(KeyCode::ControlLeft)
                                        || input.is_key_pressed(KeyCode::ControlRight),
                                    input.is_key_pressed(KeyCode::ShiftLeft)
                                        || input.is_key_pressed(KeyCode::ShiftRight),
                                    input.is_key_pressed(KeyCode::AltLeft)
                                        || input.is_key_pressed(KeyCode::AltRight),
                                )
                            };

                            let mode = if shift {
                                SelectionMode::Add
                            } else if ctrl {
                                SelectionMode::Remove
                            } else if alt {
                                SelectionMode::Toggle
                            } else {
                                SelectionMode::Replace
                            };

                            let selection = world.get_resource_mut::<SelectionSystem>().unwrap();

                            if let Some(entity) = picked {
                                selection.select_entity(entity, mode);
                                info!("Selected entity: {:?} (mode: {:?})", entity, mode);
                            } else if mode == SelectionMode::Replace {
                                selection.clear();
                                info!("Cleared selection");
                            }
                        }
                    }
                }

                // Log selection info
                {
                    let selection = world.get_resource::<SelectionSystem>().unwrap();
                    let selected_count = selection.selected_count();
                    if selected_count > 0 {
                        info!("Selected: {} entities", selected_count);
                    }
                }

                // Render a simple frame (just clear screen)
                if let Some(render_context) = &mut self.render_context {
                    if let Err(e) = render_context.render(&praxis_graphics::RenderCommands {
                        view: praxis_math::Mat4::IDENTITY,
                        proj: praxis_math::Mat4::IDENTITY,
                        draw_commands: &[],
                        lighting: None,
                    }) {
                        error!("Render error: {}", e);
                    }
                }

                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(KeyCode::Escape)
                    && event.state.is_pressed()
                {
                    info!("\nExiting...");
                    event_loop.exit();
                }

                let input_state = world.get_resource_mut::<InputState>().unwrap();
                praxis_input::winit_integration::process_window_event(
                    input_state,
                    &WindowEvent::KeyboardInput {
                        device_id: winit::event::DeviceId::dummy(),
                        event: event.clone(),
                        is_synthetic: false,
                    },
                );
                window.request_redraw();
            }
            _ => {
                let input_state = world.get_resource_mut::<InputState>().unwrap();
                praxis_input::winit_integration::process_window_event(input_state, &event);
                window.request_redraw();
            }
        }
    }
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("selection_demo example requires graphics support and cannot run in headless mode");
    Ok(())
}

#[cfg(all(not(feature = "editor"), not(feature = "headless")))]
fn main() {
    eprintln!("This example requires the 'editor' feature to be enabled.");
    eprintln!("Run with: cargo run --example selection_demo --features editor");
    std::process::exit(1);
}
