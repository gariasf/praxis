//! Graphics system for the Praxis engine.
//!
//! This crate provides functionality for rendering and managing graphics.

use praxis_utils::{error, Result, info};
use wgpu::Instance;

const GPU_NOT_FOUND_ERROR_MESSAGE: &str =
    "Unable to find a GPU! Make sure you have installed required drivers!";

/// Initializes the graphics system.
pub fn init() {
    info!("Initializing renderer...");
    // Since wgpu functions are async, we need an async runtime.
    // `pollster::block_on` runs an async future to completion on the current thread.
    if let Err(e) = pollster::block_on(init_renderer()) {
        error!("Failed to initialize renderer: {:?}", e);
    }
}

async fn init_renderer() -> Result<()> {
    let instance = Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect( GPU_NOT_FOUND_ERROR_MESSAGE);

    let adapter_info = adapter.get_info();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("Praxis Render Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::Off,
        })
        .await
        .expect("Failed to request device");

    Ok(())
}
