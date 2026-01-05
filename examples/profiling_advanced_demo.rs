//! Advanced profiling demonstration with visualization data generation.
//!
//! This example shows:
//! - Real-time profiling with frame time graphs
//! - Memory tracking with leak detection
//! - System bottleneck identification
//! - Chrome trace export for detailed analysis
//! - Visualization data generation

use praxis_profiling::{
    FramePhase, ProfileScope, Profiler, ProfilerConfig, ProfilingVisualization,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    praxis_utils::init()?;

    println!("=== Advanced Profiling Demo ===\n");

    // Create profiler
    let config = ProfilerConfig {
        enable_cpu: true,
        enable_gpu: false,
        enable_memory: true,
        enable_systems: true,
        max_frame_history: 300,
        bottleneck_threshold: 0.10, // 10% threshold for bottleneck detection
    };

    let mut profiler = Profiler::new(config);
    let mut visualization = ProfilingVisualization::new();

    // Register custom phase mappings
    profiler.register_phase_mapping("ai_update".to_string(), FramePhase::SystemUpdate);
    profiler.register_phase_mapping("particle_update".to_string(), FramePhase::PostProcess);

    // Start trace export
    profiler.begin_trace_export();

    println!("Running simulation for 100 frames...\n");

    // Simulate 100 frames with varying workloads
    for frame in 0..100 {
        profiler.begin_frame();

        simulate_frame_advanced(&profiler, frame);

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
    println!("\n✓ Chrome trace exported to: profiling_advanced_trace.json\n");

    // Final analysis
    print_final_analysis(&profiler, &visualization);

    // Generate performance report
    generate_performance_report(&profiler)?;

    Ok(())
}

fn simulate_frame_advanced(profiler: &Profiler, frame: u64) {
    let memory_tracker = profiler.memory_tracker();
    let system_profiler = profiler.system_profiler();

    // Simulate variable workload based on frame number
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
            format!("frame_{}_physics", frame),
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

    // Particle system (varies with frame)
    if frame % 3 == 0 {
        let _scope = ProfileScope::new("particle_update");
        system_profiler.begin_system("particle_update");

        let _alloc = memory_tracker.track_allocation(
            256 * 1024,
            format!("frame_{}_particles", frame),
            "Particles".to_string(),
        );

        simulate_work(Duration::from_micros(800));

        system_profiler.end_system("particle_update");
    }

    // Rendering preparation
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
                format!("frame_{}_uniform_buffer", frame),
                "Rendering".to_string(),
            );
            simulate_work(Duration::from_micros(400));
        }

        system_profiler.end_system("render_prep");
    }

    // Main rendering
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

    // Occasionally simulate a spike (bottleneck)
    if frame % 25 == 0 {
        let _scope = ProfileScope::new("expensive_operation");
        system_profiler.begin_system("expensive_operation");

        simulate_work(Duration::from_micros(5000)); // 5ms spike

        system_profiler.end_system("expensive_operation");
    }
}

fn simulate_work(duration: Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        std::hint::spin_loop();
    }
}

fn print_advanced_report(profiler: &Profiler, visualization: &ProfilingVisualization, frame: u64) {
    println!("=== Frame {} Report ===", frame);

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
    println!("  Top systems:");
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

fn print_final_analysis(profiler: &Profiler, visualization: &ProfilingVisualization) {
    println!("=== Final Performance Analysis ===\n");

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
