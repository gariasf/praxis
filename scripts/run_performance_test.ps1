# Automated performance testing script for Praxis engine (PowerShell)
#
# Usage:
#   .\scripts\run_performance_test.ps1
#   .\scripts\run_performance_test.ps1 -Release
#
# This script runs the comprehensive performance profiling demo and
# validates that all optimizations provide expected improvements.

param(
    [switch]$Release
)

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "Praxis Engine Performance Test" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan
Write-Host ""

# Check if running in release mode
if (-not $Release) {
    Write-Host "WARNING: Running in debug mode" -ForegroundColor Yellow
    Write-Host "For accurate performance measurement, run:"
    Write-Host "  .\scripts\run_performance_test.ps1 -Release"
    Write-Host ""
    $continue = Read-Host "Continue anyway? (y/N)"
    if ($continue -ne "y" -and $continue -ne "Y") {
        exit 1
    }
    $mode = ""
} else {
    $mode = "--release"
    Write-Host "Running in release mode" -ForegroundColor Green
}

Write-Host ""
Write-Host "Building example..."

if ($mode -eq "--release") {
    cargo build --release --example performance_profiling_comprehensive
} else {
    cargo build --example performance_profiling_comprehensive
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Build successful" -ForegroundColor Green
Write-Host ""

# System information
Write-Host "System Information:"
Write-Host "===================="

# Get GPU info (Windows)
try {
    $gpu = Get-WmiObject Win32_VideoController | Select-Object -First 1 -ExpandProperty Name
    Write-Host "GPU: $gpu"
} catch {
    Write-Host "GPU: Unable to detect"
}

# Get CPU info
try {
    $cpu = Get-WmiObject Win32_Processor | Select-Object -First 1 -ExpandProperty Name
    Write-Host "CPU: $cpu"
} catch {
    Write-Host "CPU: Unable to detect"
}

# Get memory info
try {
    $mem = Get-WmiObject Win32_ComputerSystem | Select-Object -ExpandProperty TotalPhysicalMemory
    $memGB = [math]::Round($mem / 1GB, 1)
    Write-Host "RAM: $memGB GB"
} catch {
    Write-Host "RAM: Unable to detect"
}

Write-Host ""
Write-Host "Running performance test..."
Write-Host "============================="
Write-Host ""
Write-Host "Instructions:"
Write-Host "1. The demo will start with baseline (no optimizations)"
Write-Host "2. Wait 2-3 seconds for warmup"
Write-Host "3. Press '1' through '7' to test each optimization level"
Write-Host "4. Press 'P' after each level to see results"
Write-Host "5. Press 'P' at the end for a full comparison report"
Write-Host "6. Press 'E' to export Chrome trace (optional)"
Write-Host "7. Press 'ESC' to exit"
Write-Host ""
Write-Host "Starting in 3 seconds..."
Start-Sleep -Seconds 3

# Run the example
if ($mode -eq "--release") {
    cargo run --release --example performance_profiling_comprehensive
} else {
    cargo run --example performance_profiling_comprehensive
}

Write-Host ""
Write-Host "Performance test complete!"
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Review the performance comparison report"
Write-Host "2. If you exported a Chrome trace, open it in chrome://tracing"
Write-Host "3. Check that each optimization shows improvement"
Write-Host "4. Validate against expected results in docs/performance_profiling_guide.md"
Write-Host ""

# Check if trace file was generated
if (Test-Path "performance_trace.json") {
    Write-Host "Chrome trace exported: performance_trace.json" -ForegroundColor Green
    Write-Host "Open in Chrome: chrome://tracing"
    Write-Host ""
}
