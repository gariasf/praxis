//! Viewport panel for 3D scene rendering with camera controls.

mod viewport_grid;

use super::EditorPanel;
use egui::Ui;
use praxis_ecs::{Entity, Transform};
use praxis_graphics::{DrawCommand, RenderCommands, RenderContext, RenderTarget};
use praxis_input::InputState;
use praxis_math::{Mat4, Vec3};
use praxis_utils::Result;
use viewport_grid::GridRenderer;
use winit::keyboard::KeyCode;

/// Viewport panel providing 3D scene rendering with camera controls.
///
/// Features:
/// - Offscreen framebuffer rendering to egui::Image
/// - Camera controls: pan, orbit, zoom
/// - Grid floor rendering
/// - Viewport-specific camera entity
/// - Mouse/keyboard event handling within viewport bounds
pub struct ViewportPanel {
    title: String,
    /// Viewport-specific camera entity
    camera_entity: Option<Entity>,
    /// Offscreen render target for viewport rendering
    render_target: Option<RenderTarget>,
    /// egui texture ID for displaying the rendered viewport
    #[allow(dead_code)] // Will be used when texture display is implemented
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
}

impl ViewportPanel {
    /// Creates a new viewport panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Viewport".to_string(),
            camera_entity: None,
            render_target: None,
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
        }
    }

    /// Initializes the viewport with a render context.
    ///
    /// This creates the offscreen render target, viewport camera, and grid.
    pub fn initialize(&mut self, render_context: &mut RenderContext) -> Result<()> {
        // Create render pass for viewport
        let render_pass = render_context.create_post_process_render_pass()?;

        // Create offscreen render target
        let render_target = RenderTarget::new(
            render_context.memory_allocator().clone(),
            render_pass,
            self.viewport_size,
            vulkano::format::Format::R8G8B8A8_UNORM,
        )?;

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

    /// Handles mouse input for camera controls within the viewport bounds.
    fn handle_camera_input(&mut self, ui: &mut Ui, viewport_rect: egui::Rect) {
        let response = ui.allocate_rect(viewport_rect, egui::Sense::click_and_drag());

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

    /// Renders the viewport contents to the offscreen render target.
    pub fn render_viewport(&mut self, _render_context: &mut RenderContext) -> Result<()> {
        let _render_target = match &self.render_target {
            Some(rt) => rt,
            None => return Ok(()), // Not initialized yet
        };

        // Update camera transform
        let camera_transform = self.compute_camera_transform();

        // Compute view and projection matrices
        let view = camera_transform.compute_inverse_matrix();
        let aspect_ratio = self.viewport_size[0] as f32 / self.viewport_size[1] as f32;
        let proj = Mat4::perspective_rh(
            self.fov.to_radians(),
            aspect_ratio,
            self.near_clip,
            self.far_clip,
        );

        // Build draw commands
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

        // TODO: Query and add scene entities with mesh components
        // This would involve:
        // 1. Querying the ECS world for entities with MeshHandle and Transform
        // 2. Converting them to DrawCommand instances
        // 3. Adding them to draw_commands

        // Render the viewport
        let _render_commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None, // TODO: Add lighting from scene
        };

        // Note: This would ideally render to our offscreen target, but the current
        // RenderContext::render() renders to the swapchain. We'll need to extend
        // RenderContext to support rendering to arbitrary framebuffers.
        // For now, this is a placeholder for the architecture.

        Ok(())
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

        // Recreate render target with new size
        if self.render_target.is_some() {
            let render_pass = render_context.create_post_process_render_pass()?;
            let render_target = RenderTarget::new(
                render_context.memory_allocator().clone(),
                render_pass,
                self.viewport_size,
                vulkano::format::Format::R8G8B8A8_UNORM,
            )?;
            self.render_target = Some(render_target);
        }

        Ok(())
    }

    /// Gets the render target for external rendering.
    pub fn render_target(&self) -> Option<&RenderTarget> {
        self.render_target.as_ref()
    }

    /// Sets whether to show the grid.
    pub fn set_show_grid(&mut self, show: bool) {
        self.show_grid = show;
    }

    /// Gets whether the grid is shown.
    pub fn show_grid(&self) -> bool {
        self.show_grid
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

    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Viewport");
            ui.separator();
            ui.checkbox(&mut self.show_grid, "Show Grid");

            if ui.button("Reset Camera").clicked() {
                self.reset_camera();
            }
        });

        ui.separator();

        // Calculate available space for viewport
        let available_size = ui.available_size();
        let viewport_size = egui::vec2(available_size.x, available_size.y - 30.0); // Leave space for controls

        // Draw viewport area
        let (rect, _response) = ui.allocate_exact_size(viewport_size, egui::Sense::hover());

        // Handle camera input
        self.handle_camera_input(ui, rect);

        // Render viewport background
        ui.painter()
            .rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 30, 35));

        // TODO: Display the actual rendered texture from render_target
        // This would involve:
        // 1. Converting the Vulkan image to an egui texture
        // 2. Registering it with egui
        // 3. Drawing it with ui.image()

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
}
