# Performance Regression Testing Script (PowerShell)
# Runs critical benchmarks and compares against a baseline

param(
    [string]$Baseline = "main",
    [string]$Current = "current",
    [double]$Threshold = 10.0,
    [string]$OutputDir = "benchmark-results",
    [switch]$Help
)

if ($Help) {
    Write-Host "Performance Regression Testing Script"
    Write-Host ""
    Write-Host "Usage: .\run-performance-regression.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Baseline NAME      Baseline name (default: main)"
    Write-Host "  -Current NAME       Current baseline name (default: current)"
    Write-Host "  -Threshold PCT      Regression threshold percentage (default: 10.0)"
    Write-Host "  -OutputDir DIR      Output directory (default: benchmark-results)"
    Write-Host "  -Help               Show this help message"
    exit 0
}

$ErrorActionPreference = "Stop"

Write-Host "🔬 Performance Regression Testing" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host "Baseline: $Baseline"
Write-Host "Current:  $Current"
Write-Host "Threshold: ${Threshold}%"
Write-Host ""

# Create output directory
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Function to run a benchmark
function Run-Benchmark {
    param(
        [string]$BenchName,
        [string]$Filter,
        [string]$BaselineName
    )
    
    Write-Host "Running $BenchName benchmark (baseline: $BaselineName)..." -ForegroundColor Yellow
    cargo bench --bench $BenchName -- $Filter --save-baseline $BaselineName --noplot
    Write-Host ""
}

# Step 1: Run current benchmarks
Write-Host "Step 1: Running benchmarks on current code" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

Run-Benchmark "graphics_optimization" "multi_draw_indirect" $Current
Run-Benchmark "graphics_optimization" "gpu_vs_cpu_culling/gpu_culling" $Current
Run-Benchmark "descriptor_set_allocation" "descriptor_set_caching_lru" $Current

# Step 2: Check if baseline exists
$baselineExists = $false
if (Test-Path "target/criterion") {
    $baselineExists = (Get-ChildItem -Path "target/criterion" -Directory -Recurse | Where-Object { $_.Name -eq $Baseline }).Count -gt 0
}

if (-not $baselineExists) {
    Write-Host "Baseline '$Baseline' not found. Creating baseline..." -ForegroundColor Yellow
    Write-Host ""
    
    if ($Baseline -eq "main") {
        # Save current state
        $currentBranch = git rev-parse --abbrev-ref HEAD
        
        Write-Host "Checking out main branch for baseline..."
        git stash push -u -m "Performance regression test - stashing changes"
        git checkout main
        
        Write-Host "Step 2: Running benchmarks on baseline (main branch)" -ForegroundColor Green
        Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        
        Run-Benchmark "graphics_optimization" "multi_draw_indirect" $Baseline
        Run-Benchmark "graphics_optimization" "gpu_vs_cpu_culling/gpu_culling" $Baseline
        Run-Benchmark "descriptor_set_allocation" "descriptor_set_caching_lru" $Baseline
        
        # Return to original branch
        git checkout $currentBranch
        try {
            git stash pop
        } catch {
            # Ignore errors if nothing to pop
        }
    } else {
        Write-Host "Error: Baseline '$Baseline' not found and automatic creation only works for 'main'" -ForegroundColor Red
        Write-Host "Please create the baseline manually by running:"
        Write-Host "  cargo bench --bench <benchmark_name> -- --save-baseline $Baseline"
        exit 1
    }
} else {
    Write-Host "Baseline '$Baseline' found, skipping baseline creation" -ForegroundColor Green
    Write-Host ""
}

# Step 3: Compare results
Write-Host "Step 3: Comparing results" -ForegroundColor Green
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cargo run --manifest-path scripts/benchmark-compare/Cargo.toml -- `
    --baseline-dir target/criterion `
    --current-baseline $Baseline `
    --new-baseline $Current `
    --threshold $Threshold `
    --output "$OutputDir/comparison.json" `
    --output-markdown "$OutputDir/comparison.md"

# Step 4: Show results
Write-Host ""
Write-Host "Results saved to:" -ForegroundColor Green
Write-Host "  JSON:     $OutputDir/comparison.json"
Write-Host "  Markdown: $OutputDir/comparison.md"
Write-Host ""

# Check for regressions
if (Test-Path "$OutputDir/regressions-detected") {
    Write-Host "❌ Performance regressions detected!" -ForegroundColor Red
    Write-Host ""
    Get-Content "$OutputDir/comparison.md"
    exit 1
} else {
    Write-Host "✅ No performance regressions detected" -ForegroundColor Green
    Write-Host ""
    Get-Content "$OutputDir/comparison.md"
    exit 0
}
