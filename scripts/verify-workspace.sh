#!/usr/bin/env bash
# Verify workspace structure and configuration

set -e

echo "Praxis Workspace Verification"
echo "=============================="
echo

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track results
ERRORS=0
WARNINGS=0

# Function to check if file exists
check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1"
    else
        echo -e "${RED}✗${NC} $1 (missing)"
        ((ERRORS++))
    fi
}

# Function to check if directory exists
check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $1/"
    else
        echo -e "${RED}✗${NC} $1/ (missing)"
        ((ERRORS++))
    fi
}

# Function to check if crate has required files
check_crate() {
    local crate_name=$1
    local crate_path="crates/$crate_name"
    
    echo "Checking $crate_name..."
    
    if [ ! -d "$crate_path" ]; then
        echo -e "  ${RED}✗${NC} Directory missing"
        ((ERRORS++))
        return
    fi
    
    # Check Cargo.toml
    if [ -f "$crate_path/Cargo.toml" ]; then
        echo -e "  ${GREEN}✓${NC} Cargo.toml"
        
        # Check for workspace lints
        if grep -q "workspace = true" "$crate_path/Cargo.toml"; then
            echo -e "  ${GREEN}✓${NC} Uses workspace lints"
        else
            echo -e "  ${YELLOW}⚠${NC} Missing workspace lints"
            ((WARNINGS++))
        fi
    else
        echo -e "  ${RED}✗${NC} Cargo.toml missing"
        ((ERRORS++))
    fi
    
    # Check lib.rs
    if [ -f "$crate_path/src/lib.rs" ]; then
        echo -e "  ${GREEN}✓${NC} src/lib.rs"
        
        # Check for documentation warnings
        if grep -q "#!\[warn(missing_docs)\]" "$crate_path/src/lib.rs"; then
            echo -e "  ${GREEN}✓${NC} Enforces missing_docs"
        else
            echo -e "  ${YELLOW}⚠${NC} Missing missing_docs lint"
            ((WARNINGS++))
        fi
    else
        echo -e "  ${RED}✗${NC} src/lib.rs missing"
        ((ERRORS++))
    fi
    
    # Check README
    if [ -f "$crate_path/README.md" ]; then
        echo -e "  ${GREEN}✓${NC} README.md"
    else
        echo -e "  ${YELLOW}⚠${NC} README.md missing (recommended)"
        ((WARNINGS++))
    fi
    
    echo
}

# Root files
echo "Root Configuration:"
check_file "Cargo.toml"
check_file "Cargo.lock"
check_file ".gitignore"
check_file "WORKSPACE.md"
check_file "CONTRIBUTING.md"
check_file "justfile"
check_file "Makefile.toml"
echo

# CI configuration
echo "CI Configuration:"
check_file ".github/workflows/rust-ci.yml"
check_file ".github/workflows/README.md"
echo

# Crate directories
echo "Crate Directories:"
CRATES=(
    "praxis_core"
    "praxis_assets"
    "praxis_audio"
    "praxis_ecs"
    "praxis_editor"
    "praxis_graphics"
    "praxis_gui"
    "praxis_input"
    "praxis_math"
    "praxis_networking"
    "praxis_physics"
    "praxis_procedural"
    "praxis_profiling"
    "praxis_scene"
    "praxis_scripting"
    "praxis_spatial"
    "praxis_terrain"
    "praxis_utils"
    "praxis_window"
)

for crate in "${CRATES[@]}"; do
    check_dir "crates/$crate"
done
echo

# Check each crate in detail
echo "Detailed Crate Checks:"
echo "====================="
for crate in "${CRATES[@]}"; do
    check_crate "$crate"
done

# Check workspace member list
echo "Workspace Member Verification:"
if [ -f "Cargo.toml" ]; then
    for crate in "${CRATES[@]}"; do
        if grep -q "\"crates/$crate\"" Cargo.toml; then
            echo -e "${GREEN}✓${NC} $crate in workspace members"
        else
            echo -e "${RED}✗${NC} $crate NOT in workspace members"
            ((ERRORS++))
        fi
    done
fi
echo

# Check workspace lints configuration
echo "Workspace Lints Configuration:"
if [ -f "Cargo.toml" ]; then
    if grep -q "\[workspace.lints.clippy\]" Cargo.toml; then
        echo -e "${GREEN}✓${NC} Clippy lints configured"
    else
        echo -e "${RED}✗${NC} Clippy lints not configured"
        ((ERRORS++))
    fi
    
    if grep -q "\[workspace.lints.rust\]" Cargo.toml; then
        echo -e "${GREEN}✓${NC} Rust lints configured"
    else
        echo -e "${RED}✗${NC} Rust lints not configured"
        ((ERRORS++))
    fi
fi
echo

# Summary
echo "=============================="
echo "Verification Summary"
echo "=============================="
echo -e "Errors:   ${RED}$ERRORS${NC}"
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"
echo

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}✓ Workspace structure is valid!${NC}"
    exit 0
else
    echo -e "${RED}✗ Workspace structure has errors!${NC}"
    exit 1
fi
