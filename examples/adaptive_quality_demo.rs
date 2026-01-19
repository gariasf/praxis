//! Adaptive Quality System Demo
//!
//! This demo demonstrates the adaptive quality system that automatically adjusts
//! rendering parameters based on frame time to maintain target FPS.
//!
//! The system adjusts:
//! - LOD bias (level of detail preference)
//! - Mesh streaming priority threshold
//! - Shadow map resolution
//!
//! Controls:
//! - Space: Toggle artificial GPU load (simulates heavy rendering)
//! - R: Reset adaptive quality system
//! - E: Enable/disable adaptive quality
//! - Arrow Up/Down: Adjust target FPS
//! - Escape: Exit

use praxis_graphics::adaptive_quality::{AdaptiveQualityConfig, AdaptiveQualitySystem};
use std::time::Instant;

fn main() {
    println!("Adaptive Quality System Demo");
    println!("============================");
    println!();
    println!("This demo simulates the adaptive quality system.");
    println!("The system monitors frame times and automatically adjusts quality.");
    println!();
    println!("Controls:");
    println!("  Space: Toggle artificial GPU load");
    println!("  R: Reset system");
    println!("  E: Enable/disable adaptive quality");
    println!("  +/-: Adjust target FPS");
    println!("  Q: Quit");
    println!();

    // Create adaptive quality system targeting 60 FPS
    let config = AdaptiveQualityConfig {
        target_fps: 60.0,
        frame_history_size: 60,
        min_lod_bias: -1.0,
        max_lod_bias: 0.5,
        lod_bias_adjustment_rate: 0.05,
        min_shadow_resolution: 512,
        max_shadow_resolution: 2048,
        ..Default::default()
    };

    let mut quality_system = AdaptiveQualitySystem::new(config);

    // Simulation state
    let mut artificial_load = false;
    let mut frame_count = 0;
    let mut last_stats_print = Instant::now();

    println!(
        "System initialized. Target FPS: {}",
        quality_system.config().target_fps
    );
    println!("Press Space to toggle artificial load...");
    println!();

    // Simple simulation loop
    loop {
        let frame_start = Instant::now();

        // Simulate frame time based on load
        let base_frame_time = if artificial_load {
            // Simulate heavy load (33ms = ~30 FPS)
            0.033
        } else {
            // Simulate normal load (13ms = ~77 FPS)
            0.013
        };

        // Add some variance to make it realistic
        let variance = (frame_count as f32 * 0.1).sin() * 0.002;
        let frame_time = base_frame_time + variance;

        // Update adaptive quality system
        quality_system.update(frame_time);

        // Print statistics every second
        if last_stats_print.elapsed().as_secs() >= 1 {
            let stats = quality_system.statistics();

            println!("\n========================================");
            println!(
                "Frame {} | Load: {}",
                frame_count,
                if artificial_load { "HEAVY" } else { "LIGHT" }
            );
            println!("========================================");
            println!("Performance:");
            println!(
                "  Current FPS: {:.1} (target: {:.1})",
                stats.current_fps, stats.target_fps
            );
            println!(
                "  Avg frame time: {:.2}ms (target: {:.2}ms)",
                stats.average_frame_time_ms,
                quality_system.target_frame_time_ms()
            );
            println!();
            println!("Quality Settings:");
            println!("  LOD bias: {:.3}", stats.current_lod_bias);
            println!(
                "  Streaming threshold: {:.1}",
                stats.current_streaming_threshold
            );
            println!(
                "  Shadow resolution: {}x{}",
                stats.current_shadow_resolution, stats.current_shadow_resolution
            );
            println!();
            println!("Adjustments:");
            println!("  Total: {}", stats.adjustment_count);
            println!(
                "  Reductions: {} | Increases: {}",
                stats.reduction_count, stats.increase_count
            );

            if quality_system.shadow_resolution_changed() {
                println!();
                println!("⚠ Shadow resolution changed! Would recreate shadow maps.");
                quality_system.clear_shadow_resolution_changed();
            }

            last_stats_print = Instant::now();
        }

        // Simple command processing (in a real app, this would be actual input)
        // For demo purposes, we'll automatically toggle load every 5 seconds
        if frame_count % 300 == 150 {
            artificial_load = !artificial_load;
            println!(
                "\n>>> Toggling artificial load: {}",
                if artificial_load { "ON" } else { "OFF" }
            );
        }

        // Exit after a reasonable demo duration
        if frame_count > 1800 {
            // ~30 seconds at 60 FPS
            println!("\n\nDemo complete!");
            println!("Final statistics:");
            println!("{}", quality_system.statistics().summary());
            break;
        }

        frame_count += 1;

        // Sleep to simulate frame timing
        let elapsed = frame_start.elapsed();
        let target_frame_time = std::time::Duration::from_secs_f32(frame_time);
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }
}
