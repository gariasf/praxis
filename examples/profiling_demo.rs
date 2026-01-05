//! Demonstration of profiling and performance analysis tools.
//!
//! This example shows how to:
//! - Profile CPU and GPU performance
//! - Track memory allocations
//! - Identify system bottlenecks
//! - Export to Chrome tracing format

use praxis_profiling::{
    AllocationTracker, FramePhase, GpuProfiler, LeakDetector, ProfileScope, Profiler,
    ProfilerConfig, SystemProfiler,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    praxis_utils::init()?;

    println!("=== Praxis Profiling Demo ===\n");

    // Create profiler with default config
    let config = ProfilerConfig {
        enable_cpu: true,
        enable_gpu: false, // Would need Vulkan setup
        enable_memory: true,
        enable_systems: true,
        max_frame_history: 300,
        bottleneck_threshold: 0.15,
    };

    let mut profiler = Profiler::new(config);

    // Start Chrome trace export
    profiler.begin_trace_export();

    println!("Running profiling test for 10 frames...\n");

    // Simulate 10 frames
    for frame in 0..10 {
        profiler.begin_frame();

        simulate_frame(&profiler, frame);

        profiler.end_frame();

        // Print stats every few frames
        if frame % 3 == 0 {
            print_profiler_stats(&profiler);
        }

        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    // Export trace
    profiler.end_trace_export("profiling_trace.json")?;
    println!("\n✓ Chrome trace exported to: profiling_trace.json");
    println!("  Open in chrome://tracing or https://ui.perfetto.dev/\n");

    // Print final statistics
    print_final_stats(&profiler);

    // Demonstrate leak detection
    demonstrate_leak_detection(&profiler);

    Ok(())
}

fn simulate_frame(profiler: &Profiler, frame: u64) {
    let memory_tracker = profiler.memory_tracker();
    let system_profiler = profiler.system_profiler();

    // Simulate physics system
    {
        let _scope = ProfileScope::new("physics_update");
        system_profiler.begin_system("physics_update");

        // Track some memory allocations
        let _alloc1 = memory_tracker.track_allocation(
            1024 * 1024,
            format!("frame_{}_physics_buffer", frame),
            "Physics".to_string(),
        );

        simulate_work(Duration::from_micros(800));

        system_profiler.end_system("physics_update");
    }

    // Simulate entity update system
    {
        let _scope = ProfileScope::new("entity_update");
        system_profiler.begin_system("entity_update");

        simulate_work(Duration::from_micros(1200));

        system_profiler.end_system("entity_update");
    }

    // Simulate rendering prep
    {
        let _scope = ProfileScope::new("render_prep");
        system_profiler.begin_system("render_prep");

        // Track GPU-related allocations
        let _alloc2 = memory_tracker.track_allocation(
            2 * 1024 * 1024,
            format!("frame_{}_vertex_buffer", frame),
            "Rendering".to_string(),
        );

        simulate_work(Duration::from_micros(600));

        system_profiler.end_system("render_prep");
    }

    // Simulate rendering (would be GPU in real usage)
    {
        let _scope = ProfileScope::new("render");
        system_profiler.begin_system("render");

        simulate_work(Duration::from_micros(2000));

        system_profiler.end_system("render");
    }

    // Simulate GUI
    {
        let _scope = ProfileScope::new("gui");
        system_profiler.begin_system("gui");

        simulate_work(Duration::from_micros(500));

        system_profiler.end_system("gui");
    }

    // Simulate an expensive system (bottleneck)
    if frame % 2 == 0 {
        let _scope = ProfileScope::new("expensive_system");
        system_profiler.begin_system("expensive_system");

        simulate_work(Duration::from_micros(3000));

        system_profiler.end_system("expensive_system");
    }
}

fn simulate_work(duration: Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        // Busy wait to simulate work
        std::hint::spin_loop();
    }
}

fn print_profiler_stats(profiler: &Profiler) {
    let stats = profiler.statistics();

    println!(
        "Frame {}: {:.1} FPS, {:.2}ms CPU, {} bytes allocated",
        stats.frame_number, stats.avg_fps, stats.cpu_time_ms, stats.memory_allocated
    );

    // Print frame breakdown
    if let Some(breakdown) = profiler.current_frame_breakdown() {
        println!("  Phase breakdown:");
        let phases = [
            FramePhase::Physics,
            FramePhase::SystemUpdate,
            FramePhase::RenderPrep,
            FramePhase::Rendering,
            FramePhase::Gui,
            FramePhase::Other,
        ];

        for phase in phases {
            let pct = breakdown.phase_percentage(phase);
            if pct > 0.0 {
                println!("    {:<15} {:>5.1}%", phase.name(), pct);
            }
        }
    }

    println!();
}

fn print_final_stats(profiler: &Profiler) {
    println!("=== Final Statistics ===\n");

    let stats = profiler.statistics();
    println!("Performance:");
    println!("  Average FPS: {:.1}", stats.avg_fps);
    println!("  Min FPS: {:.1}", stats.min_fps);
    println!("  Max FPS: {:.1}", stats.max_fps);
    println!("  Avg CPU time: {:.2}ms", stats.cpu_time_ms);
    println!();

    println!("Memory:");
    println!(
        "  Current allocated: {:.2} MB",
        stats.memory_allocated as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Peak allocated: {:.2} MB",
        stats.memory_peak as f64 / (1024.0 * 1024.0)
    );
    println!();

    // Print system statistics
    println!("System Performance:");
    let system_stats = profiler.system_profiler().system_statistics();
    for stat in system_stats.iter().take(5) {
        println!(
            "  {:<20} avg: {:>7.2}ms ({:>5.1}%)",
            stat.name,
            stat.avg_time.as_secs_f64() * 1000.0,
            stat.frame_percentage
        );
    }
    println!();

    // Print bottlenecks
    let bottlenecks = profiler.system_profiler().identify_bottlenecks();
    if !bottlenecks.is_empty() {
        println!("⚠ Bottlenecks Detected:");
        for bottleneck in bottlenecks {
            println!("  {} ({:?})", bottleneck.name, bottleneck.bottleneck_type);
            println!(
                "    Time: {:.2}ms ({:.1}%)",
                bottleneck.avg_time.as_secs_f64() * 1000.0,
                bottleneck.percentage
            );
            println!("    Severity: {:.0}%", bottleneck.severity * 100.0);
            println!("    Recommendation: {}", bottleneck.recommendation);
        }
        println!();
    }
}

fn demonstrate_leak_detection(profiler: &Profiler) {
    println!("=== Leak Detection Demo ===\n");

    let leak_detector = profiler.leak_detector();
    let memory_tracker = profiler.memory_tracker();

    // Create checkpoint
    leak_detector.checkpoint();
    println!("Created memory checkpoint");

    // Allocate some memory
    let alloc1 =
        memory_tracker.track_allocation(1024, "test_leak_1".to_string(), "Test".to_string());
    let alloc2 =
        memory_tracker.track_allocation(2048, "test_leak_2".to_string(), "Test".to_string());

    thread::sleep(Duration::from_millis(100));

    // Free one allocation
    memory_tracker.track_deallocation(alloc1);

    // Detect leaks
    let leaks = leak_detector.detect_leaks(Duration::from_millis(50));
    println!("Detected {} potential leak(s)", leaks.len());

    for (id, alloc) in leaks {
        println!(
            "  Allocation {} at {}: {} bytes (category: {})",
            id, alloc.location, alloc.size, alloc.category
        );
    }

    // Clean up
    memory_tracker.track_deallocation(alloc2);

    println!();
}
