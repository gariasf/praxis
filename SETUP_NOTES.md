# Setup Notes

## Initial Setup Status

This Rust project requires the following system dependencies to be installed before it can be built:

### Required System Dependencies

1. **Rust** - ✓ Already installed (detected during build attempt)
2. **CMake** - ✗ Not installed (required for shaderc-sys dependency)
3. **Vulkan SDK** - Not verified (required for graphics rendering)

### Installation Instructions

#### Install CMake

**Using winget (Windows):**
```powershell
winget install --id Kitware.CMake --accept-package-agreements --accept-source-agreements
```

After installation, restart your terminal/PowerShell session to ensure `cmake` is in your PATH.

#### Install Vulkan SDK

Download and install from: https://vulkan.lunarg.com/sdk/home

### Build Commands (After Dependencies)

Once CMake and Vulkan SDK are installed:

```bash
# Check that everything compiles
cargo check --workspace

# Build the project
cargo build

# Run tests
cargo test --workspace

# Run formatting
cargo fmt --all

# Run linting
cargo clippy --all -- -D warnings
```

### Note

The automated setup encountered a security restriction preventing system package installation. Please install CMake manually using the instructions above, then run:

```bash
cargo build
```

This will download all Rust dependencies and compile the entire workspace (19 crates).
