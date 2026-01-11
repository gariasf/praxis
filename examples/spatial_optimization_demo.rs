//! Spatial Optimization Demo
//!
//! This example demonstrates the spatial optimization systems including:
//! - Frustum culling visualization
//! - Octree spatial partitioning bounds
//! - BVH queries
//! - LOD (Level of Detail) transitions
//! - Visual debugging with line rendering
//!
//! Controls:
//! - W/A/S/D: Move camera
//! - Mouse: Look around
//! - Space/Shift: Move camera up/down
//! - F: Toggle frustum culling visualization
//! - O: Toggle octree visualization
//! - L: Toggle LOD level display
//! - ESC: Exit

use praxis_ecs::{BoundingBox, LodComponent, PerspectiveCameraBundle, Transform, World};
use praxis_graphics::{
    create_bounding_box, create_grid, GridConfig, LineBatch, LineRenderer, RenderContext,
};
use praxis_input::InputState;
use praxis_math::{Mat4, Quat, Vec3};
use praxis_spatial::{
    Aabb, Bvh, FrustumCuller, LodGroup, LodLevel, Octree, SpatialLodManager, VisibilitySystem,
};
use praxis_utils::{info, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    },
    swapchain::{self, SwapchainPresentInfo},
    sync::{self, GpuFuture},
    Validated, VulkanError,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

// Camera movement parameters
const CAMERA_SPEED: f32 = 10.0;
const MOUSE_SENSITIVITY: f32 = 0.002;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
    line_renderer: Option<LineRenderer>,
    camera_entity: Option<praxis_ecs::Entity>,
    octree: Option<Octree>,
    bvh: Option<Bvh>,
    visibility_system: Option<VisibilitySystem>,
    entities_with_bounds: Vec<(praxis_ecs::Entity, Aabb)>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,

    // Visualization toggles
    show_frustum: bool,
    show_octree: bool,
    show_lod: bool,

    // Camera control
    camera_yaw: f32,
    camera_pitch: f32,
    input_state: InputState,
    cursor_locked: bool,
    delta_time: f32,
}

impl App {
    async fn setup_scene(
        window: Arc<Window>,
    ) -> Result<(
        World,
        RenderContext,
        LineRenderer,
        praxis_ecs::Entity,
        Octree,
        Bvh,
        VisibilitySystem,
        Vec<(praxis_ecs::Entity, Aabb)>,
        Box<dyn GpuFuture>,
    )> {
        info!("Initializing Spatial Optimization Demo");

        // Create the render context
        let render_context = RenderContext::new(window.clone()).await?;

        // Initialize line renderer with the same render pass
        let line_renderer = LineRenderer::new(
            render_context.device.clone(),
            render_context.render_pass().clone(),
            render_context.memory_allocator().clone(),
            [WINDOW_WIDTH, WINDOW_HEIGHT],
        )?;

        // Create octree and other spatial structures
        let mut octree = Octree::new(Vec3::ZERO, 1000.0, 8);
        let mut bvh = Bvh::new();
        let mut lod_manager = SpatialLodManager::new();
        let mut visibility_system = VisibilitySystem::with_max_distance(500.0);

        // Configure LOD groups
        setup_lod_groups(&mut lod_manager);

        // Create the world
        let mut world = World::new();

        // Spawn a grid of objects
        let grid_size = 20;
        let spacing = 10.0;
        let mut entities_with_bounds = Vec::new();

        for x in 0..grid_size {
            for z in 0..grid_size {
                let position = Vec3::new(
                    (x as f32 - grid_size as f32 / 2.0) * spacing,
                    0.0,
                    (z as f32 - grid_size as f32 / 2.0) * spacing,
                );

                // Alternate between different object types
                let object_type = ((x + z) % 3) as usize;
                let (lod_group, bounds_size) = match object_type {
                    0 => ("tree", 3.0),
                    1 => ("rock", 2.0),
                    _ => ("bush", 1.5),
                };

                let bounds =
                    BoundingBox::from_center_half_extents(position, Vec3::splat(bounds_size));

                let entity = world.spawn((
                    Transform::from_translation(position),
                    bounds,
                    LodComponent::new(lod_group),
                ));

                // Insert into octree
                let aabb = Aabb::from_min_max(bounds.min, bounds.max);
                octree.insert(entity, aabb);
                entities_with_bounds.push((entity, aabb));

                // Assign to LOD manager
                visibility_system
                    .lod_manager_mut()
                    .assign_entity(entity, lod_group);
            }
        }

        info!("Spawned {} entities in octree", entities_with_bounds.len());

        // Build BVH
        bvh.build(entities_with_bounds.clone());
        info!("Built BVH with {} entities", bvh.entity_count());

        // Create camera
        let camera_entity = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 10.0, 50.0),
            70.0_f32.to_radians(),
            WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32,
        ));

        // Initialize frame synchronization
        let previous_frame_end = sync::now(render_context.device.clone()).boxed();

        info!("Scene setup complete");

        Ok((
            world,
            render_context,
            line_renderer,
            camera_entity,
            octree,
            bvh,
            visibility_system,
            entities_with_bounds,
            previous_frame_end,
        ))
    }

    fn update_camera(&mut self, delta_time: f32) {
        if let (Some(world), Some(camera_entity)) = (&mut self.world, self.camera_entity) {
            let inner = world.inner_mut();

            if let Some(mut transform) = inner.get_mut::<Transform>(camera_entity) {
                // Mouse look
                if self.cursor_locked {
                    let rotation = Quat::from_euler(
                        glam::EulerRot::YXZ,
                        self.camera_yaw,
                        self.camera_pitch,
                        0.0,
                    );
                    transform.rotation = rotation;
                }

                // Calculate forward, right, and up vectors
                let forward = transform.rotation * Vec3::NEG_Z;
                let right = transform.rotation * Vec3::X;
                let up = Vec3::Y;

                // Movement
                let mut velocity = Vec3::ZERO;

                if self.input_state.is_key_pressed(KeyCode::KeyW) {
                    velocity += forward;
                }
                if self.input_state.is_key_pressed(KeyCode::KeyS) {
                    velocity -= forward;
                }
                if self.input_state.is_key_pressed(KeyCode::KeyD) {
                    velocity += right;
                }
                if self.input_state.is_key_pressed(KeyCode::KeyA) {
                    velocity -= right;
                }
                if self.input_state.is_key_pressed(KeyCode::Space) {
                    velocity += up;
                }
                if self.input_state.is_key_pressed(KeyCode::ShiftLeft) {
                    velocity -= up;
                }

                if velocity.length_squared() > 0.0 {
                    velocity = velocity.normalize() * CAMERA_SPEED * delta_time;
                    transform.translation += velocity;
                }
            }
        }
    }

    fn update_camera_matrices(&mut self) {
        if let (Some(world), Some(camera_entity)) = (&mut self.world, self.camera_entity) {
            let inner = world.inner_mut();

            if let (Some(transform), Some(projection)) = (
                inner.get::<Transform>(camera_entity),
                inner.get::<praxis_ecs::PerspectiveProjection>(camera_entity),
            ) {
                let view = Mat4::look_at_rh(
                    transform.translation,
                    transform.translation + (transform.rotation * Vec3::NEG_Z),
                    Vec3::Y,
                );

                let proj = projection.compute_matrix();

                if let Some(mut matrices) =
                    inner.get_mut::<praxis_ecs::CameraMatrices>(camera_entity)
                {
                    matrices.update(view, proj);
                }
            }
        }
    }

    fn create_debug_visualization(&self) -> LineBatch {
        let mut batch = LineBatch::with_capacity(2000);

        // Add ground grid
        let grid_config = GridConfig {
            size: 200.0,
            divisions: 20,
            line_color: Vec3::new(0.2, 0.2, 0.2),
            axis_color: Vec3::new(0.4, 0.4, 0.4),
            height: -2.0,
        };
        let grid_batch = create_grid(&grid_config);
        batch.add_lines(grid_batch.to_vertices().chunks_exact(2).map(|chunk| {
            praxis_graphics::Line::new(
                Vec3::from(chunk[0].position),
                Vec3::from(chunk[1].position),
                Vec3::from(chunk[0].color),
            )
        }));

        if let (Some(world), Some(camera_entity)) = (&self.world, self.camera_entity) {
            let inner = world.inner();

            // Get camera view-projection matrix for frustum culling
            if let Some(matrices) = inner.get::<praxis_ecs::CameraMatrices>(camera_entity) {
                let view_proj = matrices.projection * matrices.view;
                let mut frustum_culler = FrustumCuller::new();
                frustum_culler.update(view_proj);

                let camera_pos = if let Some(transform) = inner.get::<Transform>(camera_entity) {
                    transform.translation
                } else {
                    Vec3::ZERO
                };

                // Count statistics
                let mut visible_count = 0;
                let mut culled_count = 0;
                let mut lod_counts = [0; 4];

                // Visualize entity bounds and LOD levels
                for (entity, aabb) in &self.entities_with_bounds {
                    let is_visible = frustum_culler.is_visible(aabb);

                    if is_visible {
                        visible_count += 1;
                    } else {
                        culled_count += 1;
                    }

                    // Show octree bounds in white (if enabled)
                    if self.show_octree {
                        let color = Vec3::new(0.3, 0.3, 0.3);
                        let center = aabb.center();
                        let size = aabb.half_extents();
                        let bbox_batch = create_bounding_box(center, size, color);
                        batch.add_lines(bbox_batch.to_vertices().chunks_exact(2).map(|chunk| {
                            praxis_graphics::Line::new(
                                Vec3::from(chunk[0].position),
                                Vec3::from(chunk[1].position),
                                Vec3::from(chunk[0].color),
                            )
                        }));
                    }

                    // Show frustum culling results (if enabled)
                    if self.show_frustum {
                        let color = if is_visible {
                            Vec3::new(0.0, 1.0, 0.0) // Green for visible
                        } else {
                            Vec3::new(1.0, 0.0, 0.0) // Red for culled
                        };
                        let center = aabb.center();
                        let size = aabb.half_extents();
                        let bbox_batch = create_bounding_box(center, size, color);
                        batch.add_lines(bbox_batch.to_vertices().chunks_exact(2).map(|chunk| {
                            praxis_graphics::Line::new(
                                Vec3::from(chunk[0].position),
                                Vec3::from(chunk[1].position),
                                Vec3::from(chunk[0].color),
                            )
                        }));
                    }

                    // Show LOD transitions (if enabled)
                    if self.show_lod && is_visible {
                        // Calculate distance from camera
                        let center = aabb.center();
                        let distance = (center - camera_pos).length();

                        // Determine LOD level based on distance
                        let (lod_color, lod_level) = if distance < 30.0 {
                            (Vec3::new(0.0, 1.0, 1.0), 0) // Cyan - LOD 0 (high detail)
                        } else if distance < 60.0 {
                            (Vec3::new(0.0, 0.5, 1.0), 1) // Blue - LOD 1 (medium)
                        } else if distance < 120.0 {
                            (Vec3::new(1.0, 1.0, 0.0), 2) // Yellow - LOD 2 (low)
                        } else {
                            (Vec3::new(1.0, 0.5, 0.0), 3) // Orange - LOD 3 (billboard)
                        };

                        lod_counts[lod_level] += 1;

                        let size = aabb.half_extents();
                        let bbox_batch = create_bounding_box(center, size, lod_color);
                        batch.add_lines(bbox_batch.to_vertices().chunks_exact(2).map(|chunk| {
                            praxis_graphics::Line::new(
                                Vec3::from(chunk[0].position),
                                Vec3::from(chunk[1].position),
                                Vec3::from(chunk[0].color),
                            )
                        }));
                    }
                }

                // Visualize frustum planes (if enabled)
                if self.show_frustum {
                    // Draw frustum view direction
                    let forward = if let Some(transform) = inner.get::<Transform>(camera_entity) {
                        transform.rotation * Vec3::NEG_Z
                    } else {
                        Vec3::NEG_Z
                    };

                    let far_center = camera_pos + forward * 100.0;

                    // Draw line from camera forward
                    batch.add(camera_pos, far_center, Vec3::new(1.0, 1.0, 0.0));
                }

                // Visualize octree root bounds (if enabled)
                if self.show_octree {
                    if let Some(octree) = &self.octree {
                        let root_bounds = octree.bounds();
                        let center = root_bounds.center();
                        let size = root_bounds.half_extents();
                        let octree_color = Vec3::new(0.5, 0.5, 1.0); // Light blue for octree root
                        let bbox_batch = create_bounding_box(center, size, octree_color);
                        batch.add_lines(bbox_batch.to_vertices().chunks_exact(2).map(|chunk| {
                            praxis_graphics::Line::new(
                                Vec3::from(chunk[0].position),
                                Vec3::from(chunk[1].position),
                                Vec3::from(chunk[0].color),
                            )
                        }));
                    }
                }
            }
        }

        batch
    }

    fn render_scene(&mut self) -> Result<()> {
        let world = self.world.as_ref().unwrap();
        let render_context = self.render_context.as_mut().unwrap();
        let line_renderer = self.line_renderer.as_mut().unwrap();
        let camera_entity = self.camera_entity.unwrap();

        // Get camera matrices and position
        let (camera_matrices, camera_position) = {
            let inner = world.inner();
            let matrices = inner
                .get::<praxis_ecs::CameraMatrices>(camera_entity)
                .unwrap();
            let transform = inner.get::<Transform>(camera_entity).unwrap();
            (*matrices, transform.translation)
        };

        // Clean up previous frame
        let mut previous_frame_end = self.previous_frame_end.take().unwrap();
        previous_frame_end.cleanup_finished();

        // Acquire next image
        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(render_context.swapchain().clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    return Ok(());
                }
                Err(e) => panic!("Failed to acquire next image: {e}"),
            };

        if suboptimal {
            // Swapchain is suboptimal, should recreate but we'll continue for now
        }

        // Update line renderer matrices
        line_renderer.update_view_projection(
            camera_matrices.view,
            camera_matrices.projection,
            camera_position,
        )?;

        // Create debug visualization
        let debug_batch = self.create_debug_visualization();

        // Build command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            render_context.command_buffer_allocator().clone(),
            render_context.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.1, 0.15, 0.2, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(
                        render_context.framebuffer(image_index as usize).clone(),
                    )
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )?
            .set_viewport(0, [render_context.viewport().clone()].into_iter().collect())?;

        // Render debug lines
        line_renderer.render(&mut builder, &debug_batch)?;

        builder.end_render_pass(SubpassEndInfo::default())?;

        let command_buffer = builder.build()?;

        // Submit and present
        let future = previous_frame_end
            .join(acquire_future)
            .then_execute(render_context.graphics_queue.clone(), command_buffer)?
            .then_swapchain_present(
                render_context.present_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    render_context.swapchain().clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.previous_frame_end = Some(sync::now(render_context.device.clone()).boxed());
            }
            Err(e) => {
                eprintln!("Failed to flush future: {e}");
                self.previous_frame_end = Some(sync::now(render_context.device.clone()).boxed());
            }
        }

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Creating window");

        let window = match event_loop.create_window(
            Window::default_attributes()
                .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
                .with_title("Praxis - Spatial Optimization Demo")
                .with_resizable(false),
        ) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("Failed to create window: {e}");
                event_loop.exit();
                return;
            }
        };

        let (
            world,
            render_context,
            line_renderer,
            camera_entity,
            octree,
            bvh,
            visibility_system,
            entities_with_bounds,
            previous_frame_end,
        ) = match pollster::block_on(Self::setup_scene(window.clone())) {
            Ok(result) => result,
            Err(e) => {
                eprintln!("Failed to setup scene: {e}");
                event_loop.exit();
                return;
            }
        };

        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
        self.line_renderer = Some(line_renderer);
        self.camera_entity = Some(camera_entity);
        self.octree = Some(octree);
        self.bvh = Some(bvh);
        self.visibility_system = Some(visibility_system);
        self.entities_with_bounds = entities_with_bounds;
        self.previous_frame_end = Some(previous_frame_end);

        self.show_frustum = true;
        self.show_octree = false;
        self.show_lod = false;
        self.cursor_locked = false;
        self.camera_yaw = 0.0;
        self.camera_pitch = 0.0;

        self.update_camera_matrices();

        println!("\n╔═══════════════════════════════════════════╗");
        println!("║    SPATIAL OPTIMIZATION DEMO             ║");
        println!("╚═══════════════════════════════════════════╝");
        println!("\nVisualization Controls:");
        println!("  F - Toggle frustum culling (Green=Visible, Red=Culled)");
        println!("  O - Toggle octree bounds (White wireframes)");
        println!("  L - Toggle LOD levels (Cyan/Blue/Yellow/Orange)");
        println!("\nCamera Controls:");
        println!("  W/A/S/D - Move");
        println!("  Space/Shift - Up/Down");
        println!("  Mouse - Look around (click to lock cursor)");
        println!("  ESC - Exit\n");
        println!("Initial state: Frustum culling ON\n");

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            if self.cursor_locked {
                self.camera_yaw -= delta.0 as f32 * MOUSE_SENSITIVITY;
                self.camera_pitch -= delta.1 as f32 * MOUSE_SENSITIVITY;

                // Clamp pitch to prevent gimbal lock
                self.camera_pitch = self.camera_pitch.clamp(-1.5, 1.5);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Update input state
                if event.state.is_pressed() {
                    if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                        self.input_state.press_key(code);
                    }
                } else if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    self.input_state.release_key(code);
                }

                // Handle special keys
                if event.state.is_pressed() {
                    if let Some(text) = event.logical_key.to_text() {
                        match text {
                            "f" | "F" => {
                                self.show_frustum = !self.show_frustum;
                                println!(
                                    "Frustum culling visualization: {}",
                                    if self.show_frustum { "ON" } else { "OFF" }
                                );
                            }
                            "o" | "O" => {
                                self.show_octree = !self.show_octree;
                                println!(
                                    "Octree bounds visualization: {}",
                                    if self.show_octree { "ON" } else { "OFF" }
                                );
                            }
                            "l" | "L" => {
                                self.show_lod = !self.show_lod;
                                println!(
                                    "LOD level visualization: {}",
                                    if self.show_lod { "ON" } else { "OFF" }
                                );
                            }
                            "Escape" => {
                                info!("ESC pressed, exiting");
                                event_loop.exit();
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left && state.is_pressed() {
                    self.cursor_locked = !self.cursor_locked;
                    if let Some(window) = &self.window {
                        let _ = window.set_cursor_grab(if self.cursor_locked {
                            winit::window::CursorGrabMode::Confined
                        } else {
                            winit::window::CursorGrabMode::None
                        });
                        window.set_cursor_visible(!self.cursor_locked);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Update delta time (simplified - using fixed timestep for this demo)
                self.delta_time = 1.0 / 60.0;

                // Update camera
                self.update_camera(self.delta_time);
                self.update_camera_matrices();

                // Render
                if let Err(e) = self.render_scene() {
                    eprintln!("Render error: {e}");
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn setup_lod_groups(lod_manager: &mut SpatialLodManager) {
    // Tree LOD group
    lod_manager.register_lod_group(LodGroup::new(
        "tree",
        vec![
            LodLevel::new(0.0, "tree_high"),
            LodLevel::new(50.0, "tree_medium"),
            LodLevel::new(100.0, "tree_low"),
            LodLevel::new(200.0, "tree_billboard"),
        ],
    ));

    // Rock LOD group
    lod_manager.register_lod_group(LodGroup::new(
        "rock",
        vec![
            LodLevel::new(0.0, "rock_high"),
            LodLevel::new(40.0, "rock_low"),
        ],
    ));

    // Bush LOD group
    lod_manager.register_lod_group(LodGroup::new(
        "bush",
        vec![
            LodLevel::new(0.0, "bush_high"),
            LodLevel::new(30.0, "bush_low"),
        ],
    ));

    info!("Configured {} LOD groups", lod_manager.group_count());
}

#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;

    info!("Starting Spatial Optimization Demo");

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
    println!("spatial_optimization_demo requires graphics support and cannot run in headless mode");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_group_setup() {
        let mut lod_manager = SpatialLodManager::new();
        setup_lod_groups(&mut lod_manager);
        assert_eq!(lod_manager.group_count(), 3);
    }
}
