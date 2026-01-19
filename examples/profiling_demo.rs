//! Consolidated profiling demonstration with progressive complexity sections.
//!
//! This example demonstrates profiling features in three progressive sections:
//! 1. Basic Profiling - Core features and simple usage
//! 2. Advanced Profiling - Visualization and detailed analysis  
//! 3. Production Patterns - Real-world integration and best practices
//!
//! Run with: `cargo run --example profiling_demo`

use praxis_profiling::{
    FramePhase, ProfileScope, Profiler, ProfilerConfig, ProfilingVisualization,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    praxis_utils::init()?;

    println!("=== Praxis Profiling Demo ===\n");
    println!("This demo showcases profiling features in progressive complexity:\n");
    println!("1. Basic Profiling");
    println!("2. Advanced Profiling with Visualization");
    println!("3. Production Patterns\n");

    // Section 1: Basic Profiling
    section_1_basic_profiling()?;

    println!("\n{}\n", "=".repeat(60));

    // Section 2: Advanced Profiling
    section_2_advanced_profiling()?;

    println!("\n{}\n", "=".repeat(60));

    // Section 3: Production Patterns
    section_3_production_patterns()?;

    println!("\n=== Demo Complete ===\n");
    println!("Generated files:");
    println!("  - profiling_basic_trace.json");
    println!("  - profiling_advanced_trace.json");
    println!("  - performance_report.txt");
    println!("\nView traces at chrome://tracing or https://ui.perfetto.dev/\n");

    Ok(())
}

// ============================================================================
// Section 1: Basic Profiling
// ============================================================================

fn section_1_basic_profiling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Section 1: Basic Profiling ===\n");
    println!("Demonstrating:");
    println!("  • CPU timing with ProfileScope");
    println!("  • Memory allocation tracking");
    println!("  • System performance measurement");
    println!("  • Frame statistics");
    println!("  • Chrome trace export\n");

    // Create profiler with basic config
    let config = ProfilerConfig {
        enable_cpu: true,
        enable_gpu: false,
        enable_memory: true,
        enable_systems: true,
        max_frame_history: 300,
        bottleneck_threshold: 0.15,
    };

    let mut profiler = Profiler::new(config);

    // Start Chrome trace export
    profiler.begin_trace_export();

    println!("Running 10 frames with basic profiling...\n");

    // Simulate 10 frames
    for frame in 0..10 {
        profiler.begin_frame();

        simulate_basic_frame(&profiler, frame);

        profiler.end_frame();

        // Print stats every few frames
        if frame % 3 == 0 {
            print_basic_stats(&profiler);
        }

        thread::sleep(Duration::from_millis(16)); // ~60 FPS
    }

    // Export trace
    profiler.end_trace_export("profiling_basic_trace.json")?;
    println!("✓ Basic trace exported to: profiling_basic_trace.json\n");

    // Print final statistics
    print_basic_final_stats(&profiler);

    // Demonstrate leak detection
    demonstrate_leak_detection(&profiler);

    Ok(())
}

fn simulate_basic_frame(profiler: &Profiler, frame: u64) {
    let memory_tracker = profiler.memory_tracker();
    let system_profiler = profiler.system_profiler();

    // Physics system
    {
        let _scope = ProfileScope::new("physics_update");
        system_profiler.begin_system("physics_update");

        let _alloc = memory_tracker.track_allocation(
            1024 * 1024,
            format!("frame_{frame}_physics_buffer"),
            "Physics".to_string(),
        );

        simulate_work(Duration::from_micros(800));

        system_profiler.end_system("physics_update");
    }

    // Entity update system
    {
        let _scope = ProfileScope::new("entity_update");
        system_profiler.begin_system("entity_update");

        simulate_work(Duration::from_micros(1200));

        system_profiler.end_system("entity_update");
    }

    // Rendering preparation
    {
        let _scope = ProfileScope::new("render_prep");
        system_profiler.begin_system("render_prep");

        let _alloc = memory_tracker.track_allocation(
            2 * 1024 * 1024,
            format!("frame_{frame}_vertex_buffer"),
            "Rendering".to_string(),
        );

        simulate_work(Duration::from_micros(600));

        system_profiler.end_system("render_prep");
    }

    // Rendering
    {
        let _scope = ProfileScope::new("render");
        system_profiler.begin_system("render");

        simulate_work(Duration::from_micros(2000));

        system_profiler.end_system("render");
    }

    // GUI
    {
        let _scope = ProfileScope::new("gui");
        system_profiler.begin_system("gui");

        simulate_work(Duration::from_micros(500));

        system_profiler.end_system("gui");
    }

    // Simulate bottleneck every other frame
    if frame % 2 == 0 {
        let _scope = ProfileScope::new("expensive_system");
        system_profiler.begin_system("expensive_system");

        simulate_work(Duration::from_micros(3000));

        system_profiler.end_system("expensive_system");
    }
}

fn print_basic_stats(profiler: &Profiler) {
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

fn print_basic_final_stats(profiler: &Profiler) {
    println!("--- Basic Profiling Results ---\n");

    let stats = profiler.statistics();
    println!("Performance:");
    println!("  Average FPS: {:.1}", stats.avg_fps);
    println!("  Min FPS: {:.1}", stats.min_fps);
    println!("  Max FPS: {:.1}", stats.max_fps);
    println!("  Avg CPU time: {:.2}ms", stats.cpu_time_ms);
    println!();

    println!("Memory:");
    println!(
        "  Current: {:.2} MB",
        stats.memory_allocated as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Peak: {:.2} MB",
        stats.memory_peak as f64 / (1024.0 * 1024.0)
    );
    println!();

    println!("Top 5 Systems:");
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
    println!("--- Leak Detection Demo ---\n");

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

// ============================================================================
// Section 2: Advanced Profiling
// ============================================================================

fn section_2_advanced_profiling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Section 2: Advanced Profiling ===\n");
    println!("Demonstrating:");
    println!("  • Custom phase mappings");
    println!("  • Frame time visualization data");
    println!("  • Memory tracking by category");
    println!("  • Variable workload profiling");
    println!("  • Nested profiling scopes");
    println!("  • Detailed bottleneck analysis\n");

    // Create profiler with advanced config
    let config = ProfilerConfig {
        enable_cpu: true,
        enable_gpu: false,
        enable_memory: true,
        enable_systems: true,
        max_frame_history: 300,
        bottleneck_threshold: 0.10, // More sensitive
    };

    let mut profiler = Profiler::new(config);
    let mut visualization = ProfilingVisualization::new();

    // Register custom phase mappings
    profiler.register_phase_mapping("ai_update".to_string(), FramePhase::SystemUpdate);
    profiler.register_phase_mapping("particle_update".to_string(), FramePhase::PostProcess);

    // Start trace export
    profiler.begin_trace_export();

    println!("Running 100 frames with variable workload...\n");

    // Simulate 100 frames with varying workloads
    for frame in 0..100 {
        profiler.begin_frame();

        simulate_advanced_frame(&profiler, frame);

        profiler.end_frame();

        // Update visualization data
        let breakdown = profiler.current_frame_breakdown();
        let stats = profiler.frame_statistics();
        let system_stats = profiler.system_profiler().system_statistics();
        let mem_stats = profiler.memory_tracker().statistics();

        visualization.update(
            breakdown.as_ref(),
            &stats,
            &system_stats,
            mem_stats.current_allocated,
        );

        // Print periodic reports
        if frame % 20 == 0 && frame > 0 {
            print_advanced_report(&profiler, &visualization, frame);
        }

        thread::sleep(Duration::from_millis(10));
    }

    // Export trace
    profiler.end_trace_export("profiling_advanced_trace.json")?;
    println!("\n✓ Advanced trace exported to: profiling_advanced_trace.json\n");

    // Final analysis
    print_advanced_final_analysis(&profiler, &visualization);

    Ok(())
}

fn simulate_advanced_frame(profiler: &Profiler, frame: u64) {
    let memory_tracker = profiler.memory_tracker();
    let system_profiler = profiler.system_profiler();

    // Variable workload based on sine wave
    let complexity_factor = (frame as f32 / 10.0).sin().abs() + 0.5;

    // AI update system
    {
        let _scope = ProfileScope::new("ai_update");
        system_profiler.begin_system("ai_update");

        simulate_work(Duration::from_micros((500.0 * complexity_factor) as u64));

        system_profiler.end_system("ai_update");
    }

    // Physics system with memory allocations
    {
        let _scope = ProfileScope::new("physics_update");
        system_profiler.begin_system("physics_update");

        let alloc_size = (512.0 * 1024.0 * complexity_factor) as usize;
        let _alloc = memory_tracker.track_allocation(
            alloc_size,
            format!("frame_{frame}_physics"),
            "Physics".to_string(),
        );

        simulate_work(Duration::from_micros((1200.0 * complexity_factor) as u64));

        system_profiler.end_system("physics_update");
    }

    // Animation system
    {
        let _scope = ProfileScope::new("animation_update");
        system_profiler.begin_system("animation_update");

        simulate_work(Duration::from_micros(400));

        system_profiler.end_system("animation_update");
    }

    // Particle system (intermittent)
    if frame % 3 == 0 {
        let _scope = ProfileScope::new("particle_update");
        system_profiler.begin_system("particle_update");

        let _alloc = memory_tracker.track_allocation(
            256 * 1024,
            format!("frame_{frame}_particles"),
            "Particles".to_string(),
        );

        simulate_work(Duration::from_micros(800));

        system_profiler.end_system("particle_update");
    }

    // Rendering preparation with nested scopes
    {
        let _scope = ProfileScope::new("render_prep");
        system_profiler.begin_system("render_prep");

        // Frustum culling
        {
            let _scope = ProfileScope::new("frustum_culling");
            simulate_work(Duration::from_micros(300));
        }

        // Buffer updates
        {
            let _scope = ProfileScope::new("buffer_updates");
            let _alloc = memory_tracker.track_allocation(
                1024 * 1024,
                format!("frame_{frame}_uniform_buffer"),
                "Rendering".to_string(),
            );
            simulate_work(Duration::from_micros(400));
        }

        system_profiler.end_system("render_prep");
    }

    // Main rendering with nested passes
    {
        let _scope = ProfileScope::new("render");
        system_profiler.begin_system("render");

        // Shadow pass
        {
            let _scope = ProfileScope::new("shadow_pass");
            simulate_work(Duration::from_micros(1500));
        }

        // Main pass
        {
            let _scope = ProfileScope::new("main_pass");
            simulate_work(Duration::from_micros(2500));
        }

        // Post-processing
        {
            let _scope = ProfileScope::new("post_process");
            simulate_work(Duration::from_micros(600));
        }

        system_profiler.end_system("render");
    }

    // GUI
    {
        let _scope = ProfileScope::new("gui");
        system_profiler.begin_system("gui");

        simulate_work(Duration::from_micros(300));

        system_profiler.end_system("gui");
    }

    // Occasional performance spike
    if frame % 25 == 0 {
        let _scope = ProfileScope::new("expensive_operation");
        system_profiler.begin_system("expensive_operation");

        simulate_work(Duration::from_micros(5000)); // 5ms spike

        system_profiler.end_system("expensive_operation");
    }
}

fn print_advanced_report(profiler: &Profiler, visualization: &ProfilingVisualization, frame: u64) {
    println!("--- Frame {frame} Report ---");

    let stats = profiler.statistics();
    println!(
        "  FPS: {:.1} (avg), {:.1} (min), {:.1} (max)",
        stats.avg_fps, stats.min_fps, stats.max_fps
    );
    println!("  CPU: {:.2}ms", stats.cpu_time_ms);
    println!(
        "  Memory: {:.2} MB",
        stats.memory_allocated as f64 / (1024.0 * 1024.0)
    );

    // Frame time statistics
    let ft_graph = &visualization.frame_time_graph;
    println!(
        "  Frame times: {:.2}ms avg, {:.2}ms min, {:.2}ms max",
        ft_graph.average(),
        ft_graph.min(),
        ft_graph.max()
    );

    // Top systems
    let top_systems = profiler.system_profiler().top_slowest_systems(3);
    println!("  Top 3 systems:");
    for system in top_systems {
        println!(
            "    {}: {:.2}ms ({:.1}%)",
            system.name,
            system.avg_time.as_secs_f64() * 1000.0,
            system.frame_percentage
        );
    }

    println!();
}

fn print_advanced_final_analysis(profiler: &Profiler, visualization: &ProfilingVisualization) {
    println!("--- Advanced Profiling Results ---\n");

    let stats = profiler.statistics();

    // Overall performance
    println!("Overall Performance:");
    println!("  Average FPS: {:.1}", stats.avg_fps);
    println!(
        "  Frame time: {:.2}ms (avg), {:.2}ms (min), {:.2}ms (max)",
        visualization.frame_time_graph.average(),
        visualization.frame_time_graph.min(),
        visualization.frame_time_graph.max()
    );
    println!();

    // Phase breakdown
    println!("Phase Breakdown:");
    if let Some(pie_chart) = &visualization.phase_pie_chart {
        for (phase, percentage, _color) in &pie_chart.segments {
            println!("  {:<15} {:>5.1}%", phase.name(), percentage);
        }
    }
    println!();

    // System performance
    println!("System Performance (Top 5):");
    let system_stats = profiler.system_profiler().system_statistics();
    for stat in system_stats.iter().take(5) {
        println!(
            "  {:<25} {:>7.2}ms ({:>5.1}%)",
            stat.name,
            stat.avg_time.as_secs_f64() * 1000.0,
            stat.frame_percentage
        );
    }
    println!();

    // Memory analysis
    let mem_stats = profiler.memory_tracker().statistics();
    println!("Memory Usage:");
    println!(
        "  Current: {:.2} MB",
        mem_stats.current_allocated as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Peak: {:.2} MB",
        mem_stats.peak_allocated as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Total allocated: {:.2} MB",
        mem_stats.total_allocated as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  Total deallocated: {:.2} MB",
        mem_stats.total_deallocated as f64 / (1024.0 * 1024.0)
    );
    println!("  Active allocations: {}", mem_stats.allocation_count);
    println!();

    println!("Memory by Category:");
    for (category, bytes) in &mem_stats.bytes_by_category {
        println!(
            "  {:<15} {:>7.2} MB",
            category,
            *bytes as f64 / (1024.0 * 1024.0)
        );
    }
    println!();

    // Bottleneck detection
    let bottlenecks = profiler.system_profiler().identify_bottlenecks();
    if !bottlenecks.is_empty() {
        println!("⚠ Performance Bottlenecks:");
        for bottleneck in bottlenecks {
            println!("  {} ({:?})", bottleneck.name, bottleneck.bottleneck_type);
            println!(
                "    Time: {:.2}ms ({:.1}%)",
                bottleneck.avg_time.as_secs_f64() * 1000.0,
                bottleneck.percentage
            );
            println!("    Severity: {:.0}%", bottleneck.severity * 100.0);
            println!("    Recommendation: {}", bottleneck.recommendation);
            println!();
        }
    } else {
        println!("✓ No significant bottlenecks detected\n");
    }
}

// ============================================================================
// Section 3: Production Patterns
// ============================================================================

fn section_3_production_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Section 3: Production Patterns ===\n");
    println!("Demonstrating:");
    println!("  • Performance report generation");
    println!("  • Long-running profiling session");
    println!("  • Statistical analysis");
    println!("  • Best practices\n");

    let config = ProfilerConfig {
        enable_cpu: true,
        enable_gpu: false,
        enable_memory: true,
        enable_systems: true,
        max_frame_history: 300,
        bottleneck_threshold: 0.12,
    };

    let mut profiler = Profiler::new(config);

    println!("Running production-like workload for 50 frames...\n");

    // Simulate realistic production workload
    for frame in 0..50 {
        profiler.begin_frame();

        simulate_production_frame(&profiler, frame);

        profiler.end_frame();

        if frame % 10 == 0 && frame > 0 {
            print_production_checkpoint(&profiler, frame);
        }

        thread::sleep(Duration::from_millis(16));
    }

    // Generate comprehensive report
    generate_performance_report(&profiler)?;

    print_production_summary(&profiler);

    Ok(())
}

fn simulate_production_frame(profiler: &Profiler, frame: u64) {
    let system_profiler = profiler.system_profiler();
    let memory_tracker = profiler.memory_tracker();

    // Realistic system execution patterns
    {
        let _scope = ProfileScope::new("input_system");
        system_profiler.begin_system("input_system");
        simulate_work(Duration::from_micros(100));
        system_profiler.end_system("input_system");
    }

    {
        let _scope = ProfileScope::new("physics_system");
        system_profiler.begin_system("physics_system");
        let _alloc = memory_tracker.track_allocation(
            512 * 1024,
            format!("physics_{frame}"),
            "Physics".to_string(),
        );
        simulate_work(Duration::from_micros(1500));
        system_profiler.end_system("physics_system");
    }

    {
        let _scope = ProfileScope::new("animation_system");
        system_profiler.begin_system("animation_system");
        simulate_work(Duration::from_micros(800));
        system_profiler.end_system("animation_system");
    }

    {
        let _scope = ProfileScope::new("render_system");
        system_profiler.begin_system("render_system");
        let _alloc = memory_tracker.track_allocation(
            2 * 1024 * 1024,
            format!("render_buffers_{frame}"),
            "Rendering".to_string(),
        );
        simulate_work(Duration::from_micros(3000));
        system_profiler.end_system("render_system");
    }

    {
        let _scope = ProfileScope::new("audio_system");
        system_profiler.begin_system("audio_system");
        simulate_work(Duration::from_micros(200));
        system_profiler.end_system("audio_system");
    }

    {
        let _scope = ProfileScope::new("ui_system");
        system_profiler.begin_system("ui_system");
        simulate_work(Duration::from_micros(400));
        system_profiler.end_system("ui_system");
    }
}

fn print_production_checkpoint(profiler: &Profiler, frame: u64) {
    let stats = profiler.statistics();
    println!(
        "Checkpoint at frame {}: {:.1} FPS, {:.2}ms avg frame time",
        frame,
        stats.avg_fps,
        1000.0 / stats.avg_fps
    );
}

fn print_production_summary(profiler: &Profiler) {
    println!("\n--- Production Pattern Results ---\n");

    let stats = profiler.statistics();

    println!("Performance Summary:");
    println!("  Target: 60 FPS (16.67ms per frame)");
    println!(
        "  Achieved: {:.1} FPS ({:.2}ms per frame)",
        stats.avg_fps,
        1000.0 / stats.avg_fps
    );

    if stats.avg_fps >= 60.0 {
        println!("  Status: ✓ Meeting performance target");
    } else {
        println!("  Status: ⚠ Below performance target");
    }
    println!();

    println!("System Budget Analysis:");
    let system_stats = profiler.system_profiler().system_statistics();
    let frame_budget_ms = 16.67; // 60 FPS

    for stat in system_stats.iter() {
        let time_ms = stat.avg_time.as_secs_f64() * 1000.0;
        let budget_pct = (time_ms / frame_budget_ms) * 100.0;
        let status = if budget_pct < 10.0 {
            "✓"
        } else if budget_pct < 20.0 {
            "⚠"
        } else {
            "✗"
        };

        println!(
            "  {} {:<20} {:>7.2}ms ({:>5.1}% of frame budget)",
            status, stat.name, time_ms, budget_pct
        );
    }
    println!();
}

fn generate_performance_report(profiler: &Profiler) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create("performance_report.txt")?;

    writeln!(file, "Praxis Engine Performance Report")?;
    writeln!(file, "================================\n")?;

    let stats = profiler.statistics();
    writeln!(file, "Summary:")?;
    writeln!(file, "  Frames analyzed: {}", stats.frame_number)?;
    writeln!(file, "  Average FPS: {:.1}", stats.avg_fps)?;
    writeln!(file, "  Min FPS: {:.1}", stats.min_fps)?;
    writeln!(file, "  Max FPS: {:.1}", stats.max_fps)?;
    writeln!(file, "  Average CPU time: {:.2}ms", stats.cpu_time_ms)?;
    writeln!(
        file,
        "  Peak memory: {:.2} MB\n",
        stats.memory_peak as f64 / (1024.0 * 1024.0)
    )?;

    writeln!(file, "System Performance:")?;
    let system_stats = profiler.system_profiler().system_statistics();
    for stat in system_stats {
        writeln!(
            file,
            "  {}: avg {:.2}ms, min {:.2}ms, max {:.2}ms ({} calls)",
            stat.name,
            stat.avg_time.as_secs_f64() * 1000.0,
            stat.min_time.as_secs_f64() * 1000.0,
            stat.max_time.as_secs_f64() * 1000.0,
            stat.execution_count
        )?;
    }

    writeln!(file, "\nBottlenecks:")?;
    let bottlenecks = profiler.system_profiler().identify_bottlenecks();
    if bottlenecks.is_empty() {
        writeln!(file, "  None detected")?;
    } else {
        for bottleneck in bottlenecks {
            writeln!(
                file,
                "  {}: {:.2}ms ({:.1}%)",
                bottleneck.name,
                bottleneck.avg_time.as_secs_f64() * 1000.0,
                bottleneck.percentage
            )?;
            writeln!(file, "    Recommendation: {}", bottleneck.recommendation)?;
        }
    }

    println!("✓ Performance report saved to: performance_report.txt\n");

    Ok(())
}

// ============================================================================
// Utility Functions
// ============================================================================

fn simulate_work(duration: Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}
