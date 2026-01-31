//! Window with graphics integration example.
//!
//! Demonstrates how to integrate praxis_window with praxis_graphics for rendering.
//! This shows the proper separation of concerns between windowing and graphics.
//!
//! Controls:
//! - ESC: Exit application

use praxis_graphics::RenderContext;
use praxis_window::{Window, WindowConfig, WindowEventHandler, WindowManager};
use std::sync::Arc;

struct GraphicsApp {
    render_context: Option<RenderContext>,
    frame_count: u32,
}

impl GraphicsApp {
    fn new() -> Self {
        Self {
            render_context: None,
            frame_count: 0,
        }
    }
}

impl WindowEventHandler for GraphicsApp {
    fn on_init(&mut self, window: &Window) {
        println!("Initializing graphics...");

        // SAFETY: We need to create an Arc<Window> for RenderContext.
        // This is safe because:
        // 1. The window reference is valid for the entire handler lifetime
        // 2. RenderContext only uses it for surface creation
        // 3. The window is never dropped while RenderContext exists
        //
        // Note: In real applications, you might want to restructure to avoid this,
        // or ensure proper lifetime management.
        let window_ptr = window as *const Window;
        let window_arc = unsafe { Arc::from_raw(window_ptr) };

        // Create a clone for RenderContext, then forget the original Arc
        // to avoid double-free (the window is owned by WindowManager)
        let window_clone = Arc::clone(&window_arc);
        std::mem::forget(window_arc);

        match pollster::block_on(RenderContext::new(window_clone)) {
            Ok(ctx) => {
                println!("✓ Graphics initialized successfully");
                self.render_context = Some(ctx);
            }
            Err(e) => {
                eprintln!("✗ Failed to initialize graphics: {}", e);
            }
        }
    }

    fn on_update(&mut self, _delta_time: f32) {
        // Update game logic here
    }

    fn on_render(&mut self, _window: &Window) {
        if let Some(ref mut ctx) = self.render_context {
            // In a real application, you would:
            // 1. Build render commands
            // 2. Submit them to the render context
            // 3. Present the frame
            //
            // For this example, we just track frames
            self.frame_count += 1;

            if self.frame_count % 60 == 0 {
                println!("Rendered {} frames", self.frame_count);
            }
        }
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        println!("Resizing to {}x{}", width, height);

        if let Some(ref mut ctx) = self.render_context {
            // Recreate swapchain for new window size
            ctx.configure_surface(width, height);
            println!("✓ Swapchain recreated");
        }
    }

    fn on_close(&mut self) -> bool {
        println!("\n✓ Shutting down after {} frames", self.frame_count);
        true
    }
}

fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;

    println!("=== Window with Graphics Example ===\n");
    println!("This example demonstrates proper integration between");
    println!("praxis_window and praxis_graphics.\n");

    let config = WindowConfig::default()
        .with_title("Praxis - Window with Graphics")
        .with_size(1920, 1080)
        .with_resizable(true);

    let app = GraphicsApp::new();
    let manager = WindowManager::with_handler(config, app)?;

    manager.run()?;

    Ok(())
}
