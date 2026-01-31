# Praxis Quick Start Guide

Get up and running with Praxis in minutes.

## Prerequisites

```bash
# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install components
rustup component add rustfmt clippy
```

### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install libasound2-dev libudev-dev pkg-config
```

**macOS:**
```bash
brew install pkg-config
```

**Windows:**
No additional dependencies needed.

## Clone and Build

```bash
git clone https://github.com/USERNAME/praxis.git
cd praxis
cargo build --workspace
```

## Run Examples

```bash
# Basic triangle rendering
cargo run --example hello_triangle

# ECS integration
cargo run --example ecs_integration

# Input handling
cargo run --example input_integration

# See all examples
cargo run --example <tab>
```

## Common Commands

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Specific crate
cargo build -p praxis_graphics
```

### Test

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p praxis_physics

# With output
cargo test -- --nocapture
```

### Format & Lint

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all --all-features -- -D warnings
```

### Documentation

```bash
# Generate and open docs
cargo doc --workspace --no-deps --open
```

## Using Task Runners

### Just

```bash
# Install
cargo install just

# Run commands
just build
just test
just ci
just doc
```

### Cargo Make

```bash
# Install
cargo install cargo-make

# Run commands
cargo make build
cargo make test
cargo make ci
cargo make doc
```

## Project Structure

```
praxis/
├── crates/              # 19 workspace crates
│   ├── praxis_core/     # Engine core
│   ├── praxis_graphics/ # Vulkan rendering
│   ├── praxis_ecs/      # Entity Component System
│   └── ...
├── examples/            # Example programs
├── docs/                # Documentation
├── .github/             # CI configuration
├── Cargo.toml           # Workspace configuration
├── WORKSPACE.md         # Workspace documentation
└── CONTRIBUTING.md      # Contribution guide
```

## Feature Flags

```bash
# Enable editor
cargo build --features editor

# Enable all optional features
cargo build --all-features

# Multiple features
cargo build --features "editor,networking,scripting"
```

## Verify Workspace

```bash
# Linux/macOS
bash scripts/verify-workspace.sh

# Windows (PowerShell)
.\scripts\verify-workspace.ps1
```

## Development Workflow

1. **Create branch**: `git checkout -b feature/my-feature`
2. **Make changes**: Edit code
3. **Format**: `cargo fmt --all`
4. **Lint**: `cargo clippy --all --all-features -- -D warnings`
5. **Test**: `cargo test --workspace`
6. **Commit**: `git commit -m "feat: add feature"`
7. **Push**: `git push origin feature/my-feature`
8. **Create PR**: On GitHub

## Troubleshooting

### Build Errors

```bash
# Clean build
cargo clean
cargo build
```

### Dependency Issues

```bash
# Update dependencies
cargo update

# Check dependency tree
cargo tree
```

### CI Failures

Run CI checks locally:

```bash
cargo check --all --all-features
cargo fmt --all -- --check
cargo clippy --all --all-features -- -D warnings
cargo test --workspace --all-features
```

## Next Steps

- Read [WORKSPACE.md](WORKSPACE.md) for workspace details
- Check [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines
- Explore [docs/](docs/) for in-depth documentation
- Read [CLAUDE.md](CLAUDE.md) for development guidelines

## Getting Help

- Open an issue for bugs
- Start a discussion for questions
- Check existing issues/PRs first

## Resources

- **Architecture**: [docs/architecture.md](docs/architecture.md)
- **Guides**: [docs/guides/](docs/guides/)
- **API Reference**: `cargo doc --open`
- **Examples**: [examples/](examples/)
