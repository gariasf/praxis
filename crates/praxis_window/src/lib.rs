//! Window management system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing windows.

use std::sync::Arc;

use praxis_graphics::RenderContext;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowId},
};

use praxis_utils::Result;
use praxis_utils::info;

struct State {
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    size: winit::dpi::PhysicalSize<u32>,
    render_context: RenderContext,
}


#[derive(Default)]
struct App {
    state: Option<State>,
}


impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes()
                    .with_fullscreen(Some(Fullscreen::Borderless(None))))
                .unwrap(),
        );
        info!("Window created successfully.");

        let state = pollster::block_on(State::new(window.clone()));
        self.state = Some(state);
        info!("Window and graphics state initialized.");

        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                info!("Close requested, exiting event loop...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let _ = state.render_context.render(&state.surface);
            }
            WindowEvent::Resized(size) => {
                info!("Window resized to: {:?}", size);
                state.resize(size);
            }
            _ => (),
        }
    }

    // Add other ApplicationHandler methods if needed, default is fine for now
    // fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {}
    // fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {}
    // fn user_event(&mut self, event_loop: &ActiveEventLoop, event: T) {}
    // fn suspended(&mut self, event_loop: &ActiveEventLoop) {}
    // fn exiting(&mut self, event_loop: &ActiveEventLoop) {}
    // fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {}
}
impl State {
    async fn new(window: Arc<Window>) -> State {
        info!("Initializing graphics render context...");
        let render_context = RenderContext::new().await.unwrap();

        info!("Getting initial window size...");
        let size = window.inner_size();

        info!("Creating surface...");
        let surface = render_context.instance.create_surface(window.clone()).unwrap();
        info!("Querying surface capabilities...");
        let cap = surface.get_capabilities(&render_context.adapter);
        let surface_format = cap.formats[0];
        info!("Selected surface format: {:?}", surface_format);

        let state = State {
            size,
            surface,
            render_context,
            surface_format,
        };

        info!("Configuring surface for the first time...");
        state.configure_surface();

        state
    }

    fn configure_surface(&self) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we're going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        info!("Applying surface configuration: {:?}", surface_config);
        self.surface.configure(&self.render_context.device, &surface_config);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        info!("Updating state with new size: {:?}", new_size);
        self.size = new_size;

        info!("Reconfiguring surface due to resize...");
        self.configure_surface();
    }
}

pub fn run() -> Result<()> {
    info!("Creating event loop...");
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    info!("Running application...");
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();

    Ok(())
}
