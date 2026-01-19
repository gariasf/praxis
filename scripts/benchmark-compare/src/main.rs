//! Benchmark Comparison Tool
//!
//! Compares Criterion benchmark results between two baselines to detect performance regressions.
//! Specifically designed to track critical performance metrics:
//! - Multi-draw indirect batching performance
//! - GPU culling overhead
//! - Descriptor set allocation rate

use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory containing Criterion benchmark results
    #[arg(long, default_value = "target/criterion")]
    baseline_dir: PathBuf,

    /// Name of the baseline to compare against (e.g., "main")
    #[arg(long, default_value = "main")]
    current_baseline: String,

    /// Name of the new baseline being tested (e.g., "current")
    #[arg(long, default_value = "current")]
    new_baseline: String,

    /// Regression threshold as a percentage (e.g., 10.0 for 10%)
    #[arg(long, default_value = "10.0")]
    threshold: f64,

    /// Output JSON file path
    #[arg(long)]
    output: Option<PathBuf>,

    /// Output Markdown file path
    #[arg(long)]
    output_markdown: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkEstimate {
    #[serde(default)]
    point_estimate: Option<f64>,
    #[serde(default)]
    standard_error: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResult {
    #[serde(default)]
    mean: Option<BenchmarkEstimate>,
    #[serde(default)]
    median: Option<BenchmarkEstimate>,
}

#[derive(Debug, Clone)]
struct ComparisonResult {
    benchmark_name: String,
    baseline_time_ns: f64,
    current_time_ns: f64,
    change_percent: f64,
    is_regression: bool,
    is_improvement: bool,
}

#[derive(Debug, Serialize)]
struct SummaryReport {
    total_benchmarks: usize,
    regressions: Vec<ComparisonResult>,
    improvements: Vec<ComparisonResult>,
    unchanged: Vec<ComparisonResult>,
    regression_threshold: f64,
    has_critical_regressions: bool,
}

impl Serialize for ComparisonResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ComparisonResult", 6)?;
        state.serialize_field("benchmark_name", &self.benchmark_name)?;
        state.serialize_field("baseline_time_ns", &self.baseline_time_ns)?;
        state.serialize_field("current_time_ns", &self.current_time_ns)?;
        state.serialize_field("change_percent", &self.change_percent)?;
        state.serialize_field("is_regression", &self.is_regression)?;
        state.serialize_field("is_improvement", &self.is_improvement)?;
        state.end()
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🔬 Benchmark Comparison Tool");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Baseline dir: {}", args.baseline_dir.display());
    println!(
        "Comparing: {} vs {}",
        args.current_baseline, args.new_baseline
    );
    println!("Threshold: {}%", args.threshold);
    println!();

    // Define critical benchmarks to track
    let critical_benchmarks = vec![
        "multi_draw_indirect_rendering",
        "gpu_vs_cpu_culling",
        "descriptor_set_caching_lru",
    ];

    // Collect all benchmark comparisons
    let comparisons = compare_benchmarks(
        &args.baseline_dir,
        &args.current_baseline,
        &args.new_baseline,
        args.threshold,
    )?;

    // Generate summary report
    let summary = generate_summary(&comparisons, args.threshold, &critical_benchmarks);

    // Print results to console
    print_summary(&summary);

    // Save JSON output if requested
    if let Some(output_path) = &args.output {
        let json = serde_json::to_string_pretty(&summary)?;
        fs::write(output_path, json)
            .with_context(|| format!("Failed to write JSON output to {}", output_path.display()))?;
        println!("📊 JSON report saved to: {}", output_path.display());
    }

    // Save Markdown output if requested
    if let Some(markdown_path) = &args.output_markdown {
        let markdown = generate_markdown_report(&summary);
        fs::write(markdown_path, markdown).with_context(|| {
            format!(
                "Failed to write Markdown output to {}",
                markdown_path.display()
            )
        })?;
        println!("📝 Markdown report saved to: {}", markdown_path.display());
    }

    // Create a marker file if regressions were detected
    if summary.has_critical_regressions {
        let marker_path = Path::new("benchmark-results/regressions-detected");
        if let Some(parent) = marker_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(marker_path, "Regressions detected")?;
        println!("⚠️  Regression marker file created");
    }

    // Exit with error code if critical regressions detected
    if summary.has_critical_regressions {
        std::process::exit(1);
    }

    Ok(())
}

fn compare_benchmarks(
    baseline_dir: &Path,
    current_baseline: &str,
    new_baseline: &str,
    threshold: f64,
) -> Result<Vec<ComparisonResult>> {
    let mut comparisons = Vec::new();

    // Find all benchmark directories
    let entries = fs::read_dir(baseline_dir).with_context(|| {
        format!(
            "Failed to read baseline directory: {}",
            baseline_dir.display()
        )
    })?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let benchmark_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Try to load both baselines
        if let Ok(comparison) = compare_single_benchmark(
            &path,
            &benchmark_name,
            current_baseline,
            new_baseline,
            threshold,
        ) {
            comparisons.push(comparison);
        }
    }

    Ok(comparisons)
}

fn compare_single_benchmark(
    benchmark_dir: &Path,
    benchmark_name: &str,
    current_baseline: &str,
    new_baseline: &str,
    threshold: f64,
) -> Result<ComparisonResult> {
    // Load baseline estimates
    let baseline_path = benchmark_dir.join(current_baseline).join("estimates.json");
    let current_path = benchmark_dir.join(new_baseline).join("estimates.json");

    let baseline_result: BenchmarkResult = serde_json::from_str(
        &fs::read_to_string(&baseline_path)
            .with_context(|| format!("Failed to read baseline: {}", baseline_path.display()))?,
    )?;

    let current_result: BenchmarkResult = serde_json::from_str(
        &fs::read_to_string(&current_path)
            .with_context(|| format!("Failed to read current: {}", current_path.display()))?,
    )?;

    // Extract mean times
    let baseline_time = baseline_result
        .mean
        .and_then(|m| m.point_estimate)
        .ok_or_else(|| anyhow::anyhow!("Baseline mean not found"))?;

    let current_time = current_result
        .mean
        .and_then(|m| m.point_estimate)
        .ok_or_else(|| anyhow::anyhow!("Current mean not found"))?;

    // Calculate percentage change
    let change_percent = ((current_time - baseline_time) / baseline_time) * 100.0;

    // Determine if this is a regression or improvement
    let is_regression = change_percent > threshold;
    let is_improvement = change_percent < -threshold;

    Ok(ComparisonResult {
        benchmark_name: benchmark_name.to_string(),
        baseline_time_ns: baseline_time,
        current_time_ns: current_time,
        change_percent,
        is_regression,
        is_improvement,
    })
}

fn generate_summary(
    comparisons: &[ComparisonResult],
    threshold: f64,
    critical_benchmarks: &[&str],
) -> SummaryReport {
    let mut regressions = Vec::new();
    let mut improvements = Vec::new();
    let mut unchanged = Vec::new();

    for comparison in comparisons {
        if comparison.is_regression {
            regressions.push(comparison.clone());
        } else if comparison.is_improvement {
            improvements.push(comparison.clone());
        } else {
            unchanged.push(comparison.clone());
        }
    }

    // Check if any critical benchmarks regressed
    let has_critical_regressions = regressions.iter().any(|r| {
        critical_benchmarks
            .iter()
            .any(|critical| r.benchmark_name.contains(critical))
    });

    SummaryReport {
        total_benchmarks: comparisons.len(),
        regressions,
        improvements,
        unchanged,
        regression_threshold: threshold,
        has_critical_regressions,
    }
}

fn print_summary(summary: &SummaryReport) {
    println!("📊 Benchmark Comparison Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total benchmarks: {}", summary.total_benchmarks);
    println!(
        "Regressions (>{:.1}%): {}",
        summary.regression_threshold,
        summary.regressions.len()
    );
    println!(
        "Improvements (<-{:.1}%): {}",
        summary.regression_threshold,
        summary.improvements.len()
    );
    println!("Unchanged: {}", summary.unchanged.len());
    println!();

    if !summary.regressions.is_empty() {
        println!("{}", "❌ REGRESSIONS DETECTED".red().bold());
        println!("━━━━━━━━━━━━━━━━━━━━━━");
        for regression in &summary.regressions {
            print_comparison(regression, true);
        }
        println!();
    }

    if !summary.improvements.is_empty() {
        println!("{}", "✅ IMPROVEMENTS".green().bold());
        println!("━━━━━━━━━━━━━━━━━━━");
        for improvement in &summary.improvements {
            print_comparison(improvement, false);
        }
        println!();
    }

    if summary.has_critical_regressions {
        println!(
            "{}",
            "⚠️  CRITICAL REGRESSIONS IN KEY BENCHMARKS!".red().bold()
        );
        println!("The following critical performance areas have regressed:");
        for regression in &summary.regressions {
            if is_critical_benchmark(&regression.benchmark_name) {
                println!("  • {}", regression.benchmark_name.yellow());
            }
        }
        println!();
    }
}

fn print_comparison(comparison: &ComparisonResult, is_regression: bool) {
    let sign = if comparison.change_percent > 0.0 {
        "+"
    } else {
        ""
    };
    let color_fn: fn(&str) -> ColoredString = if is_regression {
        |s| s.red()
    } else {
        |s| s.green()
    };

    println!(
        "  {} {}",
        if is_regression { "❌" } else { "✅" },
        comparison.benchmark_name.bold()
    );
    println!(
        "     Baseline: {:.2} µs",
        comparison.baseline_time_ns / 1000.0
    );
    println!(
        "     Current:  {:.2} µs",
        comparison.current_time_ns / 1000.0
    );
    println!(
        "     Change:   {}",
        color_fn(&format!("{}{:.2}%", sign, comparison.change_percent))
    );
    println!();
}

fn is_critical_benchmark(name: &str) -> bool {
    name.contains("multi_draw_indirect")
        || name.contains("gpu_culling")
        || name.contains("descriptor_set_caching_lru")
}

fn generate_markdown_report(summary: &SummaryReport) -> String {
    let mut md = String::new();

    md.push_str(&format!(
        "### 📊 Benchmark Results ({} total)\n\n",
        summary.total_benchmarks
    ));

    if summary.has_critical_regressions {
        md.push_str("#### ⚠️ CRITICAL REGRESSIONS DETECTED\n\n");
        md.push_str(
            "**The following critical benchmarks have regressed beyond the threshold:**\n\n",
        );

        for regression in &summary.regressions {
            if is_critical_benchmark(&regression.benchmark_name) {
                md.push_str(&format!(
                    "- 🚨 **{}**: {:.2}% slower ({:.2} µs → {:.2} µs)\n",
                    regression.benchmark_name,
                    regression.change_percent,
                    regression.baseline_time_ns / 1000.0,
                    regression.current_time_ns / 1000.0
                ));
            }
        }
        md.push('\n');
    }

    if !summary.regressions.is_empty() {
        md.push_str(&format!(
            "#### ❌ Regressions ({})\n\n",
            summary.regressions.len()
        ));
        md.push_str("| Benchmark | Baseline | Current | Change |\n");
        md.push_str("|-----------|----------|---------|--------|\n");

        for regression in &summary.regressions {
            let critical_marker = if is_critical_benchmark(&regression.benchmark_name) {
                " 🚨"
            } else {
                ""
            };
            md.push_str(&format!(
                "| {}{} | {:.2} µs | {:.2} µs | +{:.2}% |\n",
                regression.benchmark_name,
                critical_marker,
                regression.baseline_time_ns / 1000.0,
                regression.current_time_ns / 1000.0,
                regression.change_percent
            ));
        }
        md.push('\n');
    }

    if !summary.improvements.is_empty() {
        md.push_str(&format!(
            "#### ✅ Improvements ({})\n\n",
            summary.improvements.len()
        ));
        md.push_str("| Benchmark | Baseline | Current | Change |\n");
        md.push_str("|-----------|----------|---------|--------|\n");

        for improvement in &summary.improvements {
            md.push_str(&format!(
                "| {} | {:.2} µs | {:.2} µs | {:.2}% |\n",
                improvement.benchmark_name,
                improvement.baseline_time_ns / 1000.0,
                improvement.current_time_ns / 1000.0,
                improvement.change_percent
            ));
        }
        md.push('\n');
    }

    if !summary.unchanged.is_empty() {
        md.push_str(&format!(
            "#### ➡️ Unchanged (within ±{}%) - {}\n\n",
            summary.regression_threshold,
            summary.unchanged.len()
        ));
        md.push_str("<details>\n<summary>View unchanged benchmarks</summary>\n\n");
        md.push_str("| Benchmark | Time | Change |\n");
        md.push_str("|-----------|------|--------|\n");

        for unchanged in &summary.unchanged {
            md.push_str(&format!(
                "| {} | {:.2} µs | {:.2}% |\n",
                unchanged.benchmark_name,
                unchanged.current_time_ns / 1000.0,
                unchanged.change_percent
            ));
        }
        md.push_str("\n</details>\n\n");
    }

    md.push_str("---\n\n");
    md.push_str("**Critical Benchmarks Tracked:**\n");
    md.push_str("- Multi-draw indirect batching (🎯 target: minimize draw call overhead)\n");
    md.push_str("- GPU culling overhead (🎯 target: <1ms for 10k objects)\n");
    md.push_str("- Descriptor allocation rate (🎯 target: 100x reduction with caching)\n");

    md
}
