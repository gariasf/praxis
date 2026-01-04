//! Console panel demonstration with log filtering and search.
//!
//! This example demonstrates the Praxis console panel with:
//! - Real-time log capture from the tracing system
//! - Log filtering by level (trace/debug/info/warn/error)
//! - Search functionality to filter messages
//! - Clear button to remove all logs
//! - Auto-scroll toggle for automatic scrolling to new messages
//! - Integration with praxis_utils tracing
//!
//! The console automatically captures all logs from the engine and displays
//! them with color-coded levels and timestamps.
//!
//! Note: This example demonstrates the console panel API and log capture.
//! For full UI rendering, the console panel should be integrated with an
//! egui-based application (see the editor examples).
//!
//! Usage:
//! ```bash
//! cargo run --example console_demo
//! ```

use praxis_editor::{init_with_console, LogBuffer, LogLevel};
use praxis_utils::{debug, error, info, trace, warn, Result};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // Create a shared log buffer
    let log_buffer = LogBuffer::new();

    // Initialize tracing with console capture
    init_with_console(log_buffer.clone())?;

    info!("Starting Console Demo");
    info!("Welcome to the Praxis Console Demo!");
    info!("The console captures all engine logs in real-time");

    println!("\n=== Praxis Console Demo ===");
    println!("Demonstration of the console panel with log filtering");
    println!("\nFeatures:");
    println!("  • Real-time log capture from tracing system");
    println!("  • Filter by log level (trace/debug/info/warn/error)");
    println!("  • Search functionality to filter messages");
    println!("  • Clear button to remove all logs");
    println!("  • Auto-scroll toggle for automatic scrolling");
    println!("  • Color-coded log levels with timestamps");
    println!("\nGenerating sample logs...\n");

    // Generate various log levels
    debug!("Debug message: Debugging information");
    info!("Info message: General information about operation");
    warn!("Warning message: Something might be wrong");
    error!("Error message: An error has occurred");
    trace!("Trace message: Very detailed debug information");

    // Generate logs from different modules
    info!(target: "praxis_graphics", "Graphics system initialized");
    info!(target: "praxis_ecs", "ECS system ready");
    info!(target: "praxis_audio", "Audio system started");

    // Generate a series of messages
    for i in 1..=10 {
        match i % 4 {
            0 => info!("Operation {} completed successfully", i),
            1 => debug!("Processing step {} of 10", i),
            2 => warn!("Resource {} needs attention", i),
            3 => error!("Failed to process item {}", i),
            _ => {}
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Display captured logs
    println!("\n=== Captured Logs ===");
    let messages = log_buffer.get_messages();
    println!("Total messages captured: {}", messages.len());

    // Group by level
    let mut counts = std::collections::HashMap::new();
    for msg in &messages {
        *counts.entry(msg.level).or_insert(0) += 1;
    }

    println!("\nLog counts by level:");
    if let Some(&count) = counts.get(&LogLevel::Trace) {
        println!("  TRACE: {}", count);
    }
    if let Some(&count) = counts.get(&LogLevel::Debug) {
        println!("  DEBUG: {}", count);
    }
    if let Some(&count) = counts.get(&LogLevel::Info) {
        println!("  INFO: {}", count);
    }
    if let Some(&count) = counts.get(&LogLevel::Warn) {
        println!("  WARN: {}", count);
    }
    if let Some(&count) = counts.get(&LogLevel::Error) {
        println!("  ERROR: {}", count);
    }

    // Show recent messages
    println!("\nRecent messages:");
    for msg in messages.iter().rev().take(10).rev() {
        println!(
            "[{}] [{}] {}: {}",
            msg.timestamp,
            msg.level.label(),
            msg.target,
            msg.message
        );
    }

    println!("\n=== Console Panel Features ===");
    println!("The ConsolePanel provides:");
    println!("  • Real-time log capture via custom tracing layer");
    println!("  • Thread-safe log buffer (Arc<Mutex<VecDeque>>)");
    println!("  • Maximum 1000 messages to prevent memory overflow");
    println!("  • Filter by log level with toggle buttons");
    println!("  • Search by message content or module name");
    println!("  • Auto-scroll or manual history review");
    println!("  • Clear all messages with one click");
    println!("  • Color-coded levels with timestamps");

    println!("\n=== Usage Example ===");
    println!("use praxis_editor::{{init_with_console, EditorState, LogBuffer}};");
    println!();
    println!("let log_buffer = LogBuffer::new();");
    println!("init_with_console(log_buffer.clone())?;");
    println!("let editor = EditorState::with_log_buffer(log_buffer);");
    println!();
    println!("// All engine logs now appear in the console panel");

    info!("Console demo completed successfully");

    Ok(())
}
