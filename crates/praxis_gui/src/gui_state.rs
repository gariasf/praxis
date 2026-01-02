//! Central GUI state manager that coordinates all GUI components.

use crate::{DebugUi, EguiIntegration, EntityInspector, TransformGizmos};
use praxis_ecs::World;
use std::sync::Arc;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::Surface;
use winit::event::WindowEvent;
use winit::window::Window;

/// Central GUI state that manages all GUI components.
pub struct GuiState {
    /// Egui integration layer.
    pub egui_integration: EguiIntegration,
    /// Debug UI for FPS counter and performance metrics.
    pub debug_ui: DebugUi,
    /// Entity inspector for viewing and editing ECS data.
    pub entity_inspector: EntityInspector,
    /// Transform gizmos for runtime scene editing.
    pub transform_gizmos: TransformGizmos,
}

impl GuiState {
    /// Creates a new GUI state.
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        surface: Arc<Surface>,
        queue: Arc<Queue>,
        format: Format,
    ) -> Self {
        let egui_integration = EguiIntegration::new(event_loop, surface, queue, format);

        Self {
            egui_integration,
            debug_ui: DebugUi::new(),
            entity_inspector: EntityInspector::new(),
            transform_gizmos: TransformGizmos::new(),
        }
    }

    /// Handles window events.
    ///
    /// Returns true if the event was consumed by the GUI.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.egui_integration.handle_event(window, event)
    }

    /// Updates and renders all GUI components.
    pub fn render(
        &mut self,
        window: &Window,
        world: &mut World,
        image_view: Arc<ImageView>,
        render_pass: Arc<RenderPass>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.egui_integration.begin_frame(window);

        let ctx = self.egui_integration.context();

        self.debug_ui.render(ctx);
        self.entity_inspector.render(ctx, world);
        self.transform_gizmos.render(ctx, world);

        let (full_output, clipped_primitives) = self.egui_integration.end_frame(window);

        let logical_size = [window.inner_size().width, window.inner_size().height];
        let scale_factor = window.scale_factor() as f32;

        self.egui_integration.render(
            image_view,
            render_pass,
            clipped_primitives,
            full_output.textures_delta,
            logical_size,
            scale_factor,
        )?;

        Ok(())
    }

    /// Gets a reference to the egui context.
    pub fn context(&self) -> &egui::Context {
        self.egui_integration.context()
    }
}
