//! Viewport panel for 3D scene rendering with camera controls.

mod viewport_grid;

use super::EditorPanel;
use crate::gizmo::{Gizmo, GizmoSystem};
use crate::selection::{Selectable, SelectionMode, SelectionSystem};
use egui::Ui;
use praxis_ecs::{
    CameraMatrices, Entity, GlobalTransform, MeshHandle, Transform, With, World,
};
use praxis_graphics::{DrawCommand, RenderContext, RenderTarget};
use praxis_input::InputState;
use praxis_math::{Mat4, Vec2, Vec3};
use praxis_utils::Result;
use std::sync::Arc;
use viewport_grid::GridRenderer;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use winit::keyboard::KeyCode;

/// Viewport panel providing 3D scene rendering with camera controls.
///
/// Features:
/// - Offscreen framebuffer rendering to egui::Image
/// - Camera controls: pan, orbit, zoom
/// - Grid floor rendering
/// - Viewport-specific camera entity
/// - Mouse/keyboard event handling within viewport bounds
/// - Gizmo overlay rendering
/// - Entity selection via raycasting
#[allow(dead_code)]
pub struct ViewportPanel {
    title: String,
    /// Viewport-specific camera entity
    camera_entity: Option<Entity>,
    /// Offscreen render target for viewport rendering
    render_target: Option<RenderTarget>,
    /// Offscreen image for rendering the scene
    offscreen_image: Option<Arc<Image>>,
    /// Image view for the offscreen image
    offscreen_image_view: Option<Arc<ImageView>>,
    /// egui texture ID for displaying the rendered viewport
    texture_id: Option<egui::TextureId>,
    /// Camera distance from the origin (for orbit)
    camera_distance: f32,
    /// Camera pitch angle (up/down rotation)
    camera_pitch: f32,
    /// Camera yaw angle (left/right rotation)
    camera_yaw: f32,
    /// Camera target position (orbit center)
    camera_target: Vec3,
    /// Whether the mouse is currently dragging in the viewport
    is_dragging: bool,
    /// Last mouse position for delta calculations
    last_mouse_pos: Option<egui::Pos2>,
    /// Field of view in degrees
    fov: f32,
    /// Near clip plane
    near_clip: f32,
    /// Far clip plane
    far_clip: f32,
    /// Viewport dimensions
    viewport_size: [u32; 2],
    /// Whether to show the grid floor
    show_grid: bool,
    /// Grid renderer
    grid_renderer: Option<GridRenderer>,
    /// Whether gizmos are enabled
    show_gizmos: bool,
    /// Whether the viewport is currently hovered
    is_hovered: bool,
    /// Viewport rect in screen space
    viewport_rect: Option<egui::Rect>,
}

#[allow(dead_code)]
impl ViewportPanel {
    /// Creates a new viewport panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Viewport".to_string(),
            camera_entity: None,
            render_target: None,
            offscreen_image: None,
            offscreen_image_view: None,
            texture_id: None,
            camera_distance: 10.0,
            camera_pitch: -30.0_f32.to_radians(),
            camera_yaw: 45.0_f32.to_radians(),
            camera_target: Vec3::ZERO,
            is_dragging: false,
            last_mouse_pos: None,
            fov: 60.0,
            near_clip: 0.1,
            far_clip: 1000.0,
            viewport_size: [800, 600],
            show_grid: true,
            grid_renderer: None,
            show_gizmos: true,
            is_hovered: false,
            viewport_rect: None,
        }
    }

    /// Initializes the viewport with a render context.
    ///
    /// This creates the offscreen render target, viewport camera, and grid.
    pub fn initialize(&mut self, render_context: &mut RenderContext) -> Result<()> {
        // Create render pass for viewport
        let render_pass = render_context.create_post_process_render_pass()?;

        // Create offscreen image
        let offscreen_image = Image::new(
            render_context.memory_allocator().clone(),
            vulkano::image::ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [self.viewport_size[0], self.viewport_size[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::SAMPLED
                    | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create offscreen image: {}", e))?;

        let offscreen_image_view = ImageView::new_default(offscreen_image.clone())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create image view: {}", e))?;

        // Create offscreen render target
        let render_target = RenderTarget::new(
            render_context.memory_allocator().clone(),
            render_pass,
            self.viewport_size,
            Format::R8G8B8A8_UNORM,
        )?;

        self.offscreen_image = Some(offscreen_image);
        self.offscreen_image_view = Some(offscreen_image_view);
        self.render_target = Some(render_target);

        // Initialize grid renderer
        let grid_renderer = GridRenderer::new();
        grid_renderer.initialize(render_context)?;
        self.grid_renderer = Some(grid_renderer);

        Ok(())
    }

    /// Sets the viewport camera entity.
    pub fn set_camera_entity(&mut self, entity: Entity) {
        self.camera_entity = Some(entity);
    }

    /// Gets the viewport camera entity.
    pub fn camera_entity(&self) -> Option<Entity> {
        self.camera_entity
    }

    /// Computes the camera position from orbit parameters.
    fn compute_camera_position(&self) -> Vec3 {
        let x = self.camera_distance * self.camera_pitch.cos() * self.camera_yaw.sin();
        let y = self.camera_distance * self.camera_pitch.sin();
        let z = self.camera_distance * self.camera_pitch.cos() * self.camera_yaw.cos();
        self.camera_target + Vec3::new(x, y, z)
    }

    /// Computes the camera transform from orbit parameters.
    pub fn compute_camera_transform(&self) -> Transform {
        let position = self.compute_camera_position();
        let mut transform = Transform::from_translation(position);
        transform.look_at(self.camera_target, Vec3::Y);
        transform
    }

    /// Computes camera matrices (view and projection).
    fn compute_camera_matrices(&self) -> CameraMatrices {
        let camera_transform = self.compute_camera_transform();
        let view = camera_transform.compute_inverse_matrix();
        let aspect_ratio = self.viewport_size[0] as f32 / self.viewport_size[1] as f32;
        let proj = Mat4::perspective_rh(
            self.fov.to_radians(),
            aspect_ratio,
            self.near_clip,
            self.far_clip,
        );

        CameraMatrices {
            view,
            projection: proj,
            view_projection: proj * view,
        }
    }

    /// Handles mouse input for camera controls and entity selection.
    fn handle_viewport_input(
        &mut self,
        ui: &mut Ui,
        viewport_rect: egui::Rect,
        world: &mut World,
        input_state: &InputState,
    ) {
        let response = ui.allocate_rect(viewport_rect, egui::Sense::click_and_drag());

        self.is_hovered = response.hovered();
        self.viewport_rect = Some(viewport_rect);

        // Handle left-click for entity selection (only when not dragging)
        if response.clicked_by(egui::PointerButton::Primary) && !self.is_dragging {
            if let Some(click_pos) = response.interact_pointer_pos() {
                self.handle_entity_selection(click_pos, viewport_rect, world, input_state);
            }
        }

        // Handle right-click drag for orbit
        if response.dragged_by(egui::PointerButton::Secondary) {
            if let Some(current_pos) = response.interact_pointer_pos() {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = current_pos - last_pos;
                    self.camera_yaw -= delta.x * 0.005;
                    self.camera_pitch -= delta.y * 0.005;

                    // Clamp pitch to avoid gimbal lock
                    const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
                    self.camera_pitch = self.camera_pitch.clamp(-MAX_PITCH, MAX_PITCH);
                }
                self.last_mouse_pos = Some(current_pos);
                self.is_dragging = true;
            }
        } else if response.dragged_by(egui::PointerButton::Middle) {
            // Middle mouse button for panning
            if let Some(current_pos) = response.interact_pointer_pos() {
                if let Some(last_pos) = self.last_mouse_pos {
                    let delta = current_pos - last_pos;

                    // Pan perpendicular to view direction
                    let right =
                        Vec3::new(self.camera_yaw.cos(), 0.0, -self.camera_yaw.sin()).normalize();
                    let up = Vec3::Y;

                    let pan_speed = self.camera_distance * 0.001;
                    self.camera_target -= right * delta.x * pan_speed;
                    self.camera_target += up * delta.y * pan_speed;
                }
                self.last_mouse_pos = Some(current_pos);
                self.is_dragging = true;
            }
        } else {
            self.is_dragging = false;
            self.last_mouse_pos = response.interact_pointer_pos();
        }

        // Handle mouse wheel for zoom (only if hovering over viewport)
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll_delta.abs() > 0.01 {
                self.camera_distance *= (1.0 - scroll_delta * 0.001).max(0.1);
                self.camera_distance = self.camera_distance.clamp(1.0, 1000.0);
            }
        }

        // Update gizmo hover if enabled
        if self.show_gizmos && self.is_hovered {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                self.update_gizmo_hover(mouse_pos, viewport_rect, world);
            }
        }
    }

    /// Handles entity selection via raycasting.
    fn handle_entity_selection(
        &mut self,
        click_pos: egui::Pos2,
        viewport_rect: egui::Rect,
        world: &mut World,
        input_state: &InputState,
    ) {
        // Convert click position to viewport-relative coordinates
        let viewport_pos = Vec2::new(
            click_pos.x - viewport_rect.min.x,
            click_pos.y - viewport_rect.min.y,
        );
        let viewport_size = Vec2::new(viewport_rect.width(), viewport_rect.height());

        // Get camera matrices
        let camera_matrices = self.compute_camera_matrices();
        let camera_transform = self.compute_camera_transform();

        // Determine selection mode based on modifiers
        let ctrl = input_state.is_key_pressed(KeyCode::ControlLeft)
            || input_state.is_key_pressed(KeyCode::ControlRight);
        let shift = input_state.is_key_pressed(KeyCode::ShiftLeft)
            || input_state.is_key_pressed(KeyCode::ShiftRight);
        let alt = input_state.is_key_pressed(KeyCode::AltLeft)
            || input_state.is_key_pressed(KeyCode::AltRight);

        let mode = if ctrl {
            SelectionMode::Remove
        } else if shift {
            SelectionMode::Add
        } else if alt {
            SelectionMode::Toggle
        } else {
            SelectionMode::Replace
        };

        // Perform simple raycast picking directly
        // Convert screen space to NDC
        let ndc_x = (2.0 * viewport_pos.x) / viewport_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * viewport_pos.y) / viewport_size.y;

        // Compute ray in world space
        let ray_origin = camera_transform.translation;
        let inv_vp = camera_matrices.view_projection.inverse();
        let near_point = inv_vp * praxis_math::Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let near_point = near_point.truncate() / near_point.w;
        let far_point = inv_vp * praxis_math::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        let far_point = far_point.truncate() / far_point.w;
        let ray_dir = (far_point - near_point).normalize();

        // Find closest entity intersecting the ray
        let mut closest_entity = None;
        let mut closest_distance = f32::MAX;

        let mut query = world.query_filtered::<(Entity, &GlobalTransform), With<Selectable>>();
        for (entity, global_transform) in query.iter(world.inner()) {
            let entity_pos = global_transform.translation();
            let to_entity = entity_pos - ray_origin;
            let projection = to_entity.dot(ray_dir);

            if projection < 0.0 {
                continue; // Behind camera
            }

            let closest_point = ray_origin + ray_dir * projection;
            let distance_to_ray = (entity_pos - closest_point).length();
            let pick_radius = 1.0; // Simple sphere-based picking

            if distance_to_ray <= pick_radius && projection < closest_distance {
                closest_distance = projection;
                closest_entity = Some(entity);
            }
        }

        // Apply selection
        let selection_system = world.get_resource_mut::<SelectionSystem>().unwrap();
        if let Some(entity) = closest_entity {
            selection_system.select_entity(entity, mode);
        } else if mode == SelectionMode::Replace {
            // Clear selection if clicking on empty space in replace mode
            selection_system.clear();
        }
    }

    /// Updates gizmo hover state based on mouse position.
    fn update_gizmo_hover(
        &self,
        mouse_pos: egui::Pos2,
        viewport_rect: egui::Rect,
        world: &mut World,
    ) {
        let gizmo_system = world.get_resource_mut::<GizmoSystem>();
        if gizmo_system.is_none() {
            return;
        }
        let gizmo_system = gizmo_system.unwrap();

        // Convert mouse position to viewport-relative coordinates
        let viewport_pos = Vec2::new(
            (mouse_pos.x - viewport_rect.min.x) / viewport_rect.width(),
            (mouse_pos.y - viewport_rect.min.y) / viewport_rect.height(),
        );

        let camera_matrices = self.compute_camera_matrices();
        let camera_position = self.compute_camera_position();

        gizmo_system.update_hover(viewport_pos, &camera_matrices, camera_position);
    }

    /// Handles keyboard input for camera controls.
    pub fn handle_keyboard_input(&mut self, input_state: &InputState, delta_time: f32) {
        let move_speed = 5.0 * delta_time;

        // WASD for camera target movement
        let forward = Vec3::new(self.camera_yaw.sin(), 0.0, self.camera_yaw.cos()).normalize();
        let right = Vec3::new(self.camera_yaw.cos(), 0.0, -self.camera_yaw.sin()).normalize();

        if input_state.is_key_pressed(KeyCode::KeyW) {
            self.camera_target += forward * move_speed;
        }
        if input_state.is_key_pressed(KeyCode::KeyS) {
            self.camera_target -= forward * move_speed;
        }
        if input_state.is_key_pressed(KeyCode::KeyA) {
            self.camera_target -= right * move_speed;
        }
        if input_state.is_key_pressed(KeyCode::KeyD) {
            self.camera_target += right * move_speed;
        }
        if input_state.is_key_pressed(KeyCode::KeyQ) {
            self.camera_target.y -= move_speed;
        }
        if input_state.is_key_pressed(KeyCode::KeyE) {
            self.camera_target.y += move_speed;
        }
    }

    /// Builds draw commands for the viewport scene.
    ///
    /// This queries the world for entities with meshes and builds DrawCommands.
    pub fn build_draw_commands(&self, world: &mut World) -> Vec<DrawCommand> {
        let mut draw_commands = Vec::new();

        // Add grid if enabled
        if self.show_grid {
            if let Some(ref grid_renderer) = self.grid_renderer {
                draw_commands.push(DrawCommand {
                    mesh_id: grid_renderer.mesh_id().to_string(),
                    model: grid_renderer.model_matrix(),
                    texture_name: None,
                    material_properties: None,
                });
            }
        }

        // Query scene entities with meshes
        let mut mesh_query = world.query::<(&Transform, &MeshHandle)>();
        for (transform, mesh_handle) in mesh_query.iter(world.inner()) {
            draw_commands.push(DrawCommand {
                mesh_id: mesh_handle.id.clone(),
                model: transform.compute_matrix(),
                texture_name: None,
                material_properties: None,
            });
        }

        // Add gizmo rendering if enabled
        if self.show_gizmos {
            if let Some(gizmo_system) = world.get_resource::<GizmoSystem>() {
                if let Some(gizmo) = gizmo_system.active_gizmo() {
                    let gizmo_draw_commands = self.build_gizmo_draw_commands(gizmo, gizmo_system);
                    draw_commands.extend(gizmo_draw_commands);
                }
            }
        }

        draw_commands
    }

    /// Builds draw commands for gizmo rendering.
    fn build_gizmo_draw_commands(
        &self,
        gizmo: &Gizmo,
        gizmo_system: &GizmoSystem,
    ) -> Vec<DrawCommand> {
        let draw_commands = Vec::new();

        // Get gizmo lines
        let lines = gizmo.get_lines(gizmo_system.mode(), gizmo_system.space());

        // For each line, we would create a mesh and draw command
        // This is a placeholder - actual implementation would need
        // line rendering support in the graphics system
        for (_start, _end, _color) in lines {
            // TODO: Create line mesh and add draw command
            // This requires line rendering primitive support
        }

        draw_commands
    }

    /// Registers the viewport texture with egui.
    /// 
    /// This method provides the integration point for registering the offscreen
    /// texture with the egui renderer. The actual implementation depends on the
    /// egui integration library being used (e.g., egui_winit_vulkano).
    /// 
    /// # Arguments
    /// 
    /// * `texture_id` - The egui texture ID to assign to this viewport's texture
    pub fn set_texture_id(&mut self, texture_id: egui::TextureId) {
        self.texture_id = Some(texture_id);
    }
    
    /// Gets the texture ID if registered.
    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture_id
    }

    /// Resizes the viewport render target.
    pub fn resize_viewport(
        &mut self,
        render_context: &RenderContext,
        new_size: [u32; 2],
    ) -> Result<()> {
        if new_size[0] == 0 || new_size[1] == 0 {
            return Ok(());
        }

        self.viewport_size = new_size;

        // Recreate offscreen image
        let offscreen_image = Image::new(
            render_context.memory_allocator().clone(),
            vulkano::image::ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [new_size[0], new_size[1], 1],
                usage: ImageUsage::COLOR_ATTACHMENT
                    | ImageUsage::SAMPLED
                    | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create offscreen image: {}", e))?;

        let offscreen_image_view = ImageView::new_default(offscreen_image.clone())
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create image view: {}", e))?;

        self.offscreen_image = Some(offscreen_image);
        self.offscreen_image_view = Some(offscreen_image_view);

        // Recreate render target with new size
        if self.render_target.is_some() {
            let render_pass = render_context.create_post_process_render_pass()?;
            let render_target = RenderTarget::new(
                render_context.memory_allocator().clone(),
                render_pass,
                self.viewport_size,
                Format::R8G8B8A8_UNORM,
            )?;
            self.render_target = Some(render_target);
        }

        Ok(())
    }

    /// Gets the render target for external rendering.
    pub fn render_target(&self) -> Option<&RenderTarget> {
        self.render_target.as_ref()
    }

    /// Gets the offscreen image view.
    pub fn offscreen_image_view(&self) -> Option<&Arc<ImageView>> {
        self.offscreen_image_view.as_ref()
    }

    /// Sets whether to show the grid.
    pub fn set_show_grid(&mut self, show: bool) {
        self.show_grid = show;
    }

    /// Gets whether the grid is shown.
    pub fn show_grid(&self) -> bool {
        self.show_grid
    }

    /// Sets whether to show gizmos.
    pub fn set_show_gizmos(&mut self, show: bool) {
        self.show_gizmos = show;
    }

    /// Gets whether gizmos are shown.
    pub fn show_gizmos(&self) -> bool {
        self.show_gizmos
    }

    /// Gets the camera distance.
    pub fn camera_distance(&self) -> f32 {
        self.camera_distance
    }

    /// Sets the camera distance.
    pub fn set_camera_distance(&mut self, distance: f32) {
        self.camera_distance = distance.clamp(1.0, 1000.0);
    }

    /// Gets the camera target position.
    pub fn camera_target(&self) -> Vec3 {
        self.camera_target
    }

    /// Sets the camera target position.
    pub fn set_camera_target(&mut self, target: Vec3) {
        self.camera_target = target;
    }

    /// Resets the camera to default position.
    pub fn reset_camera(&mut self) {
        self.camera_distance = 10.0;
        self.camera_pitch = -30.0_f32.to_radians();
        self.camera_yaw = 45.0_f32.to_radians();
        self.camera_target = Vec3::ZERO;
    }

    /// Returns whether the viewport is currently hovered.
    pub fn is_hovered(&self) -> bool {
        self.is_hovered
    }

    /// Gets the viewport rect in screen space.
    pub fn viewport_rect(&self) -> Option<egui::Rect> {
        self.viewport_rect
    }
}

impl Default for ViewportPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for ViewportPanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&praxis_graphics::RenderContext>,
    ) {
        ui.horizontal(|ui| {
            ui.heading("Viewport");
            ui.separator();
            ui.checkbox(&mut self.show_grid, "Show Grid");
            ui.separator();
            ui.checkbox(&mut self.show_gizmos, "Show Gizmos");

            if ui.button("Reset Camera").clicked() {
                self.reset_camera();
            }
        });

        ui.separator();

        // Calculate available space for viewport
        let available_size = ui.available_size();
        let viewport_size = egui::vec2(available_size.x, available_size.y - 30.0); // Leave space for controls

        // Allocate space for viewport
        let (rect, _response) = ui.allocate_exact_size(viewport_size, egui::Sense::hover());

        // Display the rendered texture if available
        if let Some(texture_id) = self.texture_id {
            ui.painter().image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        } else {
            // Render viewport background
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 35));

            // Draw a placeholder message
            let text = if self.render_target.is_some() {
                "Viewport (3D Scene)"
            } else {
                "Viewport (Not Initialized)"
            };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::proportional(16.0),
                egui::Color32::GRAY,
            );
        }

        // Camera info overlay
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                rect.left_top() + egui::vec2(5.0, 5.0),
                egui::vec2(200.0, 90.0),
            ),
            3.0,
            egui::Color32::from_rgba_premultiplied(20, 20, 25, 200),
        );

        ui.painter().text(
            rect.left_top() + egui::vec2(10.0, 10.0),
            egui::Align2::LEFT_TOP,
            format!(
                "Distance: {:.1}\nPitch: {:.1}°\nYaw: {:.1}°\nTarget: ({:.1}, {:.1}, {:.1})",
                self.camera_distance,
                self.camera_pitch.to_degrees(),
                self.camera_yaw.to_degrees(),
                self.camera_target.x,
                self.camera_target.y,
                self.camera_target.z
            ),
            egui::FontId::monospace(11.0),
            egui::Color32::from_rgb(200, 200, 200),
        );

        // Bottom control bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Controls:");
            ui.separator();
            ui.label("🖱 Left-Click: Select");
            ui.separator();
            ui.label("🖱 Right-Click+Drag: Orbit");
            ui.separator();
            ui.label("🖱 Middle-Click+Drag: Pan");
            ui.separator();
            ui.label("🖱 Scroll: Zoom");
            ui.separator();
            ui.label("⌨ WASD/QE: Move Target");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_panel_creation() {
        let panel = ViewportPanel::new();
        assert_eq!(panel.title(), "Viewport");
        assert_eq!(panel.camera_distance(), 10.0);
        assert!(panel.show_grid());
        assert!(panel.show_gizmos());
    }

    #[test]
    fn test_viewport_panel_default() {
        let panel = ViewportPanel::default();
        assert_eq!(panel.camera_distance(), 10.0);
    }

    #[test]
    fn test_camera_distance_clamping() {
        let mut panel = ViewportPanel::new();
        panel.set_camera_distance(0.5); // Below minimum
        assert_eq!(panel.camera_distance(), 1.0);

        panel.set_camera_distance(2000.0); // Above maximum
        assert_eq!(panel.camera_distance(), 1000.0);

        panel.set_camera_distance(50.0); // Within range
        assert_eq!(panel.camera_distance(), 50.0);
    }

    #[test]
    fn test_camera_target() {
        let mut panel = ViewportPanel::new();
        let target = Vec3::new(5.0, 2.0, 3.0);
        panel.set_camera_target(target);
        assert_eq!(panel.camera_target(), target);
    }

    #[test]
    fn test_reset_camera() {
        let mut panel = ViewportPanel::new();

        // Modify camera
        panel.set_camera_distance(50.0);
        panel.set_camera_target(Vec3::new(10.0, 5.0, 8.0));

        // Reset
        panel.reset_camera();

        // Verify defaults
        assert_eq!(panel.camera_distance(), 10.0);
        assert_eq!(panel.camera_target(), Vec3::ZERO);
    }

    #[test]
    fn test_grid_visibility() {
        let mut panel = ViewportPanel::new();
        assert!(panel.show_grid());

        panel.set_show_grid(false);
        assert!(!panel.show_grid());

        panel.set_show_grid(true);
        assert!(panel.show_grid());
    }

    #[test]
    fn test_gizmo_visibility() {
        let mut panel = ViewportPanel::new();
        assert!(panel.show_gizmos());

        panel.set_show_gizmos(false);
        assert!(!panel.show_gizmos());

        panel.set_show_gizmos(true);
        assert!(panel.show_gizmos());
    }

    #[test]
    fn test_compute_camera_position() {
        let panel = ViewportPanel::new();
        let position = panel.compute_camera_position();

        // Should be at some distance from origin
        assert!(position.length() > 0.0);

        // Distance should match camera_distance
        let distance_from_target = (position - panel.camera_target()).length();
        assert!((distance_from_target - panel.camera_distance()).abs() < 0.01);
    }

    #[test]
    fn test_compute_camera_transform() {
        let panel = ViewportPanel::new();
        let transform = panel.compute_camera_transform();

        // Transform should have the computed position
        let expected_pos = panel.compute_camera_position();
        let actual_pos = transform.translation;

        assert!((expected_pos - actual_pos).length() < 0.01);
    }

    #[test]
    fn test_camera_entity() {
        use praxis_ecs::Entity;

        let mut panel = ViewportPanel::new();
        assert!(panel.camera_entity().is_none());

        let entity = Entity::from_raw(42);
        panel.set_camera_entity(entity);
        assert_eq!(panel.camera_entity(), Some(entity));
    }

    #[test]
    fn test_hover_state() {
        let panel = ViewportPanel::new();
        assert!(!panel.is_hovered());
    }
}
