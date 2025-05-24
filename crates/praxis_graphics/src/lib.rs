//! Graphics system for the Praxis engine.
//!
//! This crate provides functionality for rendering and managing graphics.

use praxis_utils::{Result, eyre, info};
use std::sync::Arc;
use wgpu::{Device, Queue, Surface};
use winit::window::Window;

/// Core graphics context containing the wgpu state.
///
/// This struct holds the main graphics backend components like the instance, adapter,
/// device, queue, and the surface linked to a window.
pub struct RenderContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
}

impl RenderContext {
    /// Creates a new `RenderContext` for a given window.
    ///
    /// Initializes graphics backend, creates a surface for the window, selects a compatible
    /// adapter and device, and determines a suitable surface format.
    ///
    /// # Arguments
    ///
    /// * `window` - An `Arc<Window>` representing the window to render onto.
    ///
    /// # Panics
    ///
    /// Panics if a compatible adapter or device cannot be found, or if surface
    /// creation fails.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::from_comma_list("vulkan, metal"),
            flags: wgpu::InstanceFlags::empty(),
            backend_options: wgpu::BackendOptions::default(),
        });

        info!("Creating surface...");
        let surface = instance
            .create_surface(window)
            .map_err(|e| eyre::eyre!("Failed to create surface: {}", e))?;

        info!("Requesting adapter...");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|e| eyre::eyre!("Failed to find a compatible graphics adapter: {}", e))?;

        info!("Found adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| eyre::eyre!("Failed to create device: {}", e))?;

        info!("Found device {:?}", device);
        info!("Found queue {:?}", queue);

        info!("Querying surface capabilities...");
        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities.formats[0];

        info!("Selected surface format: {:?}", surface_format);

        Ok(Self {
            device,
            queue,
            surface,
            surface_format,
        })
    }

    pub fn configure_surface(&self, width: u32, height: u32) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            view_formats: vec![self.surface_format],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width,
            height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        info!("Applying surface configuration: {:?}", surface_config);
        self.surface.configure(&self.device, &surface_config);
    }

    /// Renders a single frame to the configured surface.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or a `SurfaceError` if acquiring or presenting
    /// the frame fails (e.g., surface lost, outdated, or timeout).
    pub fn render(&mut self) -> Result<()> {
        // Get the current swap chain texture to render to
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        drop(render_pass);

        self.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }
}
