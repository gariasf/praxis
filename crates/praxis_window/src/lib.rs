//! Window management system for the Praxis engine.
//!
//! This crate provides functionality for creating and managing windows.

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use praxis_utils::{Result, error, eyre, info};

#[derive(Default)]
struct WindowApp {
    window: Option<Window>,
}

// Implement the ApplicationHandler trait for our struct
impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attributes = WindowAttributes::default().with_title("Praxis Window");
        match event_loop.create_window(attributes) {
            Ok(window) => {
                self.window = Some(window);
                info!("Window created.");
            }
            Err(err) => {
                error!("Failed to create window: {}", err);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        // Ensure the event belongs to our window
        if self.window.as_ref().map_or(false, |w| w.id() == id) {
            match event {
                WindowEvent::CloseRequested => {
                    info!("Window close requested. Exiting.");
                    event_loop.exit();
                }
                // Handle other window events if needed
                _ => (),
            }
        }
    }

    // Add other ApplicationHandler methods if needed, default is fine for now
    // fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {}
    // fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {}
    // fn user_event(&mut self, event_loop: &ActiveEventLoop, event: T) {}
    // fn suspended(&mut self, event_loop: &ActiveEventLoop) {}
    // fn exiting(&mut self, event_loop: &ActiveEventLoop) {}
    // fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {}
    // fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {}
}

pub fn open_window() -> Result<()> {
    info!("Initializing event loop...");
    let event_loop =
        EventLoop::new().map_err(|err| eyre!("Failed to create event loop: {}", err))?;
    info!("Event loop created.");

    event_loop.set_control_flow(ControlFlow::Poll);

    info!("Launching application...");
    let mut app = WindowApp::default();
    info!("Application launched.");

    // Run the application and handle any errors
    event_loop
        .run_app(&mut app)
        .map_err(|err| eyre!("Event loop error: {}", err))?;

    info!("Event loop finished.");
    Ok(())
}
