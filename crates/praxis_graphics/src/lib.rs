//! Graphics system for the Praxis engine.
//!
//! This crate provides functionality for rendering and managing graphics.

use praxis_utils::{Result, info};
use wgpu::{Adapter, Device, Instance, Queue, Surface};

pub async fn init() -> Result<RenderContext> {
    info!("Initializing renderer...");
    RenderContext::new().await
}

pub struct RenderContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl RenderContext {
    pub async fn new() -> Result<Self> {
        info!("Creating wgpu instance...");
        let instance = wgpu::Instance::default();
        info!("Requesting adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await?;
        info!("Found adapter: {:?}", adapter.get_info());

        info!("Requesting device and queue...");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        info!("Device and queue obtained.");

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn render(&mut self, surface: &Surface<'static>) -> Result<()> {
        let frame = surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());

        let mut encoder =
            self
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
