use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::camera::Camera;
use crate::gpu::GpuContext;
use crate::input::Input;
use crate::state::State;

mod assets;
mod camera;
mod capture;
mod components;
pub mod gpu;
mod helmet;
mod input;
mod render;
mod scenes;
mod state;
mod stats;
mod time;

pub use capture::capture;
pub use scenes::ReferenceScene;
pub use state::DEFAULT_MODEL;
pub use stats::FrameStats;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes =
            Window::default_attributes().with_inner_size(winit::dpi::PhysicalSize::new(2560, 1440));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let size = window.inner_size();
        tracing::info!(width = size.width, height = size.height, "window created");
        window
            .set_cursor_grab(winit::window::CursorGrabMode::Confined)
            .ok();
        window.set_cursor_visible(false);

        self.shell = Some(pollster::block_on(WindowShell::new(window)).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let shell = match &mut self.shell {
            Some(shell) => shell,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => shell.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                if let Err(error) = shell.redraw() {
                    tracing::error!(%error, "redraw failed");
                    event_loop.exit();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        repeat,
                        ..
                    },
                ..
            } => {
                let mut input = shell.state.world.resource_mut::<Input>();
                if key_state.is_pressed() {
                    input.pressed.insert(code);
                    if !repeat {
                        input.just_pressed.insert(code);
                    }
                } else {
                    input.pressed.remove(&code);
                }
                if code == KeyCode::Escape && key_state.is_pressed() {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(shell) = &mut self.shell
            && let winit::event::DeviceEvent::MouseMotion { delta } = event
        {
            let mut camera = shell.state.world.resource_mut::<Camera>();
            camera.yaw += delta.0 as f32 * camera.sensitivity;
            camera.pitch -= delta.1 as f32 * camera.sensitivity;
            camera.pitch = camera.pitch.clamp(-1.5, 1.5); // ~86 degrees
        }
    }
}

#[derive(Default)]
struct App {
    shell: Option<WindowShell>,
}

/// The window, the surface it presents through, and the State that renders
/// into that surface on wall-clock time.
struct WindowShell {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    last_frame: Option<Instant>,
    state: State,
}

impl WindowShell {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let (context, surface) = GpuContext::with_surface(window.clone()).await?;

        let surface_caps = surface.get_capabilities(&context.adapter);

        // Assuming an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        tracing::debug!(
            ?surface_format,
            srgb = surface_format.is_srgb(),
            "surface format chosen"
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let is_surface_configured = size.width > 0 && size.height > 0;
        if is_surface_configured {
            surface.configure(&context.device, &config);
        }

        // A minimised window reports a zero size; State takes the real one on resize.
        let state = State::new(
            &context,
            surface_format,
            size.width.max(1),
            size.height.max(1),
            ReferenceScene::Helmet,
        )?;

        Ok(Self {
            window,
            surface,
            device: context.device,
            queue: context.queue,
            config,
            is_surface_configured,
            last_frame: None,
            state,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        tracing::debug!(width, height, "resize requested");
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.state.resize(width, height);
            self.is_surface_configured = true;
        }
    }

    /// Advances the State by the wall-clock time since the previous frame and
    /// presents its render.
    fn redraw(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let now = Instant::now();
        let delta_time = self
            .last_frame
            .map_or(0.0, |last_frame| (now - last_frame).as_secs_f32());
        self.last_frame = Some(now);
        self.state.update(delta_time);

        let (output, is_suboptimal) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => (surface_texture, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => (surface_texture, true),
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                tracing::warn!("surface outdated, reconfiguring");
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                tracing::error!("surface texture lost; bailing");
                anyhow::bail!("Surface texture lost");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.state.render(&view, None);
        self.queue.present(output);

        // Only after presenting: wgpu rejects a reconfigure while a texture
        // acquired from the surface is alive.
        if is_suboptimal {
            tracing::debug!("surface suboptimal, reconfiguring");
            self.surface.configure(&self.device, &self.config);
        }

        Ok(())
    }
}

pub fn run() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
