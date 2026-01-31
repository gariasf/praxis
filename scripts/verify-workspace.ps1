# Verify workspace structure and configuration

$ErrorActionPreference = "Stop"

Write-Host "Praxis Workspace Verification" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host

# Track results
$script:Errors = 0
$script:Warnings = 0

# Function to check if file exists
function Check-File {
    param([string]$Path)
    
    if (Test-Path $Path -PathType Leaf) {
        Write-Host "✓ $Path" -ForegroundColor Green
    } else {
        Write-Host "✗ $Path (missing)" -ForegroundColor Red
        $script:Errors++
    }
}

# Function to check if directory exists
function Check-Dir {
    param([string]$Path)
    
    if (Test-Path $Path -PathType Container) {
        Write-Host "✓ $Path/" -ForegroundColor Green
    } else {
        Write-Host "✗ $Path/ (missing)" -ForegroundColor Red
        $script:Errors++
    }
}

# Function to check if crate has required files
function Check-Crate {
    param([string]$CrateName)
    
    $cratePath = "crates/$CrateName"
    Write-Host "Checking $CrateName..."
    
    if (!(Test-Path $cratePath -PathType Container)) {
        Write-Host "  ✗ Directory missing" -ForegroundColor Red
        $script:Errors++
        return
    }
    
    # Check Cargo.toml
    $cargoToml = "$cratePath/Cargo.toml"
    if (Test-Path $cargoToml) {
        Write-Host "  ✓ Cargo.toml" -ForegroundColor Green
        
        # Check for workspace lints
        $content = Get-Content $cargoToml -Raw
        if ($content -match "workspace = true") {
            Write-Host "  ✓ Uses workspace lints" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ Missing workspace lints" -ForegroundColor Yellow
            $script:Warnings++
        }
    } else {
        Write-Host "  ✗ Cargo.toml missing" -ForegroundColor Red
        $script:Errors++
    }
    
    # Check lib.rs
    $libRs = "$cratePath/src/lib.rs"
    if (Test-Path $libRs) {
        Write-Host "  ✓ src/lib.rs" -ForegroundColor Green
        
        # Check for documentation warnings
        $content = Get-Content $libRs -Raw
        if ($content -match "#!\[warn\(missing_docs\)\]") {
            Write-Host "  ✓ Enforces missing_docs" -ForegroundColor Green
        } else {
            Write-Host "  ⚠ Missing missing_docs lint" -ForegroundColor Yellow
            $script:Warnings++
        }
    } else {
        Write-Host "  ✗ src/lib.rs missing" -ForegroundColor Red
        $script:Errors++
    }
    
    # Check README
    $readme = "$cratePath/README.md"
    if (Test-Path $readme) {
        Write-Host "  ✓ README.md" -ForegroundColor Green
    } else {
        Write-Host "  ⚠ README.md missing (recommended)" -ForegroundColor Yellow
        $script:Warnings++
    }
    
    Write-Host
}

# Root files
Write-Host "Root Configuration:"
Check-File "Cargo.toml"
Check-File "Cargo.lock"
Check-File ".gitignore"
Check-File "WORKSPACE.md"
Check-File "CONTRIBUTING.md"
Check-File "justfile"
Check-File "Makefile.toml"
Write-Host

# CI configuration
Write-Host "CI Configuration:"
Check-File ".github/workflows/rust-ci.yml"
Check-File ".github/workflows/README.md"
Write-Host

# Crate directories
Write-Host "Crate Directories:"
$crates = @(
    "praxis_core",
    "praxis_assets",
    "praxis_audio",
    "praxis_ecs",
    "praxis_editor",
    "praxis_graphics",
    "praxis_gui",
    "praxis_input",
    "praxis_math",
    "praxis_networking",
    "praxis_physics",
    "praxis_procedural",
    "praxis_profiling",
    "praxis_scene",
    "praxis_scripting",
    "praxis_spatial",
    "praxis_terrain",
    "praxis_utils",
    "praxis_window"
)

foreach ($crate in $crates) {
    Check-Dir "crates/$crate"
}
Write-Host

# Check each crate in detail
Write-Host "Detailed Crate Checks:"
Write-Host "====================="
foreach ($crate in $crates) {
    Check-Crate $crate
}

# Check workspace member list
Write-Host "Workspace Member Verification:"
if (Test-Path "Cargo.toml") {
    $cargoContent = Get-Content "Cargo.toml" -Raw
    foreach ($crate in $crates) {
        if ($cargoContent -match "`"crates/$crate`"") {
            Write-Host "✓ $crate in workspace members" -ForegroundColor Green
        } else {
            Write-Host "✗ $crate NOT in workspace members" -ForegroundColor Red
            $script:Errors++
        }
    }
}
Write-Host

# Check workspace lints configuration
Write-Host "Workspace Lints Configuration:"
if (Test-Path "Cargo.toml") {
    $cargoContent = Get-Content "Cargo.toml" -Raw
    if ($cargoContent -match "\[workspace\.lints\.clippy\]") {
        Write-Host "✓ Clippy lints configured" -ForegroundColor Green
    } else {
        Write-Host "✗ Clippy lints not configured" -ForegroundColor Red
        $script:Errors++
    }
    
    if ($cargoContent -match "\[workspace\.lints\.rust\]") {
        Write-Host "✓ Rust lints configured" -ForegroundColor Green
    } else {
        Write-Host "✗ Rust lints not configured" -ForegroundColor Red
        $script:Errors++
    }
}
Write-Host

# Summary
Write-Host "==============================" -ForegroundColor Cyan
Write-Host "Verification Summary" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan
Write-Host "Errors:   $script:Errors" -ForegroundColor $(if ($script:Errors -eq 0) { "Green" } else { "Red" })
Write-Host "Warnings: $script:Warnings" -ForegroundColor $(if ($script:Warnings -eq 0) { "Green" } else { "Yellow" })
Write-Host

if ($script:Errors -eq 0) {
    Write-Host "✓ Workspace structure is valid!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "✗ Workspace structure has errors!" -ForegroundColor Red
    exit 1
}
