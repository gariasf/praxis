//! Basic window example demonstrating the praxis_window crate.
//!
//! This example shows how to create a simple window with custom event handling.
//! The window displays frame count and responds to input events.
//!
//! Controls:
//! - ESC: Exit application
//! - Any key: Prints key press to console
//! - Mouse: Prints position to console

use praxis_window::{Key, MouseButton, Window, WindowConfig, WindowEventHandler, WindowManager};

struct BasicWindowApp {
    frame_count: u32,
    show_logs: bool,
}

impl BasicWindowApp {
    fn new() -> Self {
        Self {
            frame_count: 0,
            show_logs: false,
        }
    }
}

impl WindowEventHandler for BasicWindowApp {
    fn on_init(&mut self, window: &Window) {
        let size = window.inner_size();
        println!("✓ Window initialized");
        println!("  Size: {}x{}", size.width, size.height);
        println!("  Title: {:?}", window.title());
        println!("\nPress ESC to exit");
        println!("Press 'L' to toggle verbose logging\n");
    }

    fn on_update(&mut self, _delta_time: f32) {
        // Game logic would go here
    }

    fn on_render(&mut self, _window: &Window) {
        self.frame_count += 1;

        if self.show_logs && self.frame_count % 60 == 0 {
            println!("Frame {} rendered", self.frame_count);
        }
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        println!("Window resized to {}x{}", width, height);
    }

    fn on_close(&mut self) -> bool {
        println!("\n✓ Window closed after {} frames", self.frame_count);
        true
    }

    fn on_focused(&mut self) {
        if self.show_logs {
            println!("Window gained focus");
        }
    }

    fn on_unfocused(&mut self) {
        if self.show_logs {
            println!("Window lost focus");
        }
    }

    fn on_key_pressed(&mut self, key: Key, is_repeat: bool) {
        if is_repeat {
            return;
        }

        match key {
            Key::Character(ref c) if c.as_str() == "l" => {
                self.show_logs = !self.show_logs;
                println!("Verbose logging: {}", self.show_logs);
            }
            _ => {
                if self.show_logs {
                    println!("Key pressed: {:?}", key);
                }
            }
        }
    }

    fn on_mouse_moved(&mut self, x: f64, y: f64) {
        if self.show_logs && self.frame_count % 60 == 0 {
            println!("Mouse position: ({:.1}, {:.1})", x, y);
        }
    }

    fn on_mouse_button_pressed(&mut self, button: MouseButton) {
        if self.show_logs {
            println!("Mouse button pressed: {:?}", button);
        }
    }

    fn on_mouse_wheel(&mut self, _delta_x: f32, delta_y: f32) {
        if self.show_logs {
            println!("Mouse wheel: {:.2}", delta_y);
        }
    }
}

fn main() -> praxis_utils::Result<()> {
    praxis_utils::init()?;

    println!("=== Basic Window Example ===\n");

    let config = WindowConfig::default()
        .with_title("Praxis - Basic Window")
        .with_size(1280, 720)
        .with_resizable(true);

    let app = BasicWindowApp::new();
    let manager = WindowManager::with_handler(config, app)?;

    manager.run()?;

    Ok(())
}
