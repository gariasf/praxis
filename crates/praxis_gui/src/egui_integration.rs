//! Egui integration module for Vulkan rendering.

use egui_winit::winit::event::WindowEvent;
use std::sync::Arc;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::render_pass::RenderPass;
use vulkano::swapchain::Surface;

/// Manages egui integration with Vulkan rendering.
pub struct EguiIntegration {
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_winit_vulkano::Gui,
}

impl EguiIntegration {
    /// Creates a new egui integration.
    pub fn new(
        event_loop: &winit::event_loop::ActiveEventLoop,
        surface: Arc<Surface>,
        queue: Arc<Queue>,
        format: Format,
    ) -> Self {
        let egui_ctx = egui::Context::default();

        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            event_loop,
            None, // pixels_per_point
            None, // egui_zoom_factor
            None, // max_texture_side
        );

        let egui_renderer = egui_winit_vulkano::Gui::new(
            event_loop,
            surface,
            queue.clone(),
            format,
            egui_winit_vulkano::GuiConfig::default(),
        );

        Self {
            egui_ctx,
            egui_winit,
            egui_renderer,
        }
    }

    /// Handles window events for egui.
    pub fn handle_event(&mut self, window: &winit::window::Window, event: &WindowEvent) -> bool {
        self.egui_winit
            .on_window_event(window, event)
            .consumed
    }

    /// Begins a new egui frame.
    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.egui_winit.take_egui_input(window);
        self.egui_ctx.begin_pass(raw_input);
    }

    /// Ends the current egui frame and returns the output and render primitives.
    pub fn end_frame(
        &mut self,
        window: &winit::window::Window,
    ) -> (egui::FullOutput, Vec<egui::ClippedPrimitive>) {
        let full_output = self.egui_ctx.end_pass();
        
        self.egui_winit
            .handle_platform_output(window, full_output.platform_output.clone());

        let clipped_primitives = self
            .egui_ctx
            .tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

        (full_output, clipped_primitives)
    }

    /// Renders egui to the specified image view within a render pass.
    pub fn render(
        &mut self,
        _image_view: Arc<ImageView>,
        _render_pass: Arc<RenderPass>,
        _clipped_primitives: Vec<egui::ClippedPrimitive>,
        _textures_delta: egui::TexturesDelta,
        logical_size: [u32; 2],
        _scale_factor: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.egui_renderer.draw_on_subpass_image(logical_size);

        Ok(())
    }

    /// Gets a reference to the egui context.
    pub fn context(&self) -> &egui::Context {
        &self.egui_ctx
    }
}
