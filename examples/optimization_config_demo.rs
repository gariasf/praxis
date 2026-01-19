//! Demonstration of the RenderingOptimizationConfig system.
//!
//! This example shows how to use the optimization config for runtime A/B
//! performance testing with GUI and keyboard controls.
//!
//! # Controls
//!
//! - `F1`: Toggle Multi-Draw Indirect
//! - `F2`: Toggle GPU Culling
//! - `F3`: Toggle GPU LOD Selection
//! - `F4`: Toggle Descriptor Caching
//! - `F5`: Toggle Hi-Z Occlusion
//! - `F6`: Toggle Mesh Streaming
//! - `F7`: Toggle Panel Visibility
//! - `F8`: Reset to Defaults
//! - `ESC`: Exit

use praxis_graphics::optimization_config::RenderingOptimizationConfig;
use praxis_utils::Result;

fn main() -> Result<()> {
    // Initialize logging
    praxis_utils::init_logging();

    println!("=== Rendering Optimization Config Demo ===\n");

    // Create config with default settings
    let mut config = RenderingOptimizationConfig::default();

    println!("Default configuration:");
    println!("{}\n", config.summary());
    println!(
        "Enabled: {}/{}",
        config.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    // Demonstrate toggling individual optimizations
    println!("\n--- Testing individual toggles ---");

    config.set_multi_draw_indirect(false);
    println!(
        "Multi-draw indirect disabled: {}",
        !config.multi_draw_indirect()
    );
    println!("Changed flag: {}", config.has_changed());

    config.clear_changed_flag();
    println!("Cleared changed flag: {}", !config.has_changed());

    // Test bulk operations
    println!("\n--- Testing bulk operations ---");

    config.enable_all();
    println!("After enable_all():");
    println!(
        "Enabled: {}/{}",
        config.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    config.disable_all();
    println!("After disable_all():");
    println!(
        "Enabled: {}/{}",
        config.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    config.reset_to_defaults();
    println!("After reset_to_defaults():");
    println!(
        "Enabled: {}/{}",
        config.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    // Demonstrate persistence
    println!("\n--- Testing serialization ---");

    let config = RenderingOptimizationConfig::all_enabled();
    let json = serde_json::to_string_pretty(&config)?;
    println!("Serialized config:\n{}", json);

    let loaded: RenderingOptimizationConfig = serde_json::from_str(&json)?;
    println!("\nDeserialized config:");
    println!(
        "Enabled: {}/{}",
        loaded.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    // Demonstrate creating custom profiles
    println!("\n--- Custom optimization profiles ---");

    // Performance profile (all enabled)
    let performance = RenderingOptimizationConfig::all_enabled();
    println!(
        "Performance profile: {}/{} optimizations",
        performance.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    // Debug profile (all disabled for easier debugging)
    let debug = RenderingOptimizationConfig::all_disabled();
    println!(
        "Debug profile: {}/{} optimizations",
        debug.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    // Balanced profile (some optimizations)
    let mut balanced = RenderingOptimizationConfig::default();
    balanced.set_hiz_occlusion(false); // Disable expensive occlusion
    balanced.set_mesh_streaming(false); // Disable streaming for simpler setup
    println!(
        "Balanced profile: {}/{} optimizations",
        balanced.enabled_count(),
        RenderingOptimizationConfig::TOTAL_OPTIMIZATIONS
    );

    println!("\n=== Demo Complete ===");
    println!("\nIn a real application:");
    println!("1. Call config.handle_keyboard_input(ctx) each frame");
    println!("2. Call config.show_gui(ctx) to render the control panel");
    println!("3. Check config.has_changed() to reset performance metrics");
    println!("4. Use config methods to conditionally enable optimizations");

    Ok(())
}
