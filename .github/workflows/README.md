# GitHub Actions CI Configuration

This directory contains the CI/CD workflows for the Praxis engine.

## Workflows

### rust-ci.yml - Main Rust CI

Primary continuous integration workflow that runs on all PRs and pushes to main.

#### Jobs

1. **check** - Cargo Check
   - Runs `cargo check --all --all-features`
   - Runs `cargo check --all --no-default-features`
   - Ensures code compiles with all feature combinations

2. **fmt** - Format Check
   - Runs `cargo fmt --all -- --check`
   - Ensures code follows Rust formatting standards
   - Must pass for PR to be merged

3. **clippy** - Lint Check
   - Runs `cargo clippy --all --all-features -- -D warnings`
   - Enforces code quality standards
   - Treats all warnings as errors
   - Must pass for PR to be merged

4. **test** - Test Suite
   - Runs `cargo test --workspace --all-features`
   - Executes all unit and integration tests
   - Must pass for PR to be merged

5. **build_examples** - Example Build
   - Runs `cargo build --examples --features headless`
   - Ensures all examples compile
   - Uses headless mode for CI (no GPU)

### docs.yml - Documentation

Builds and deploys documentation to GitHub Pages.

### performance-regression.yml - Performance Testing

Runs benchmarks and detects performance regressions.

## System Dependencies

The CI requires these system packages on Ubuntu:

- `libasound2-dev`: Audio support (ALSA)
- `libudev-dev`: Device detection
- `pkg-config`: Build configuration

## Running CI Locally

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install components
rustup component add rustfmt clippy
```

### Run All Checks

```bash
# Option 1: Using cargo-make
cargo install cargo-make
cargo make ci

# Option 2: Using just
cargo install just
just ci

# Option 3: Manual
cargo check --all --all-features
cargo fmt --all -- --check
cargo clippy --all --all-features -- -D warnings
cargo test --workspace --all-features
```

### Individual Checks

```bash
# Check compilation
cargo check --all --all-features
cargo check --all --no-default-features

# Format
cargo fmt --all

# Lint
cargo clippy --all --all-features -- -D warnings

# Test
cargo test --workspace --all-features

# Examples
cargo build --examples --features headless
```

## Caching

The CI uses `Swatinem/rust-cache@v2` to cache:
- Cargo registry
- Cargo index
- Target directory
- Build artifacts

This significantly speeds up CI runs.

## Feature Flags in CI

The CI tests multiple feature combinations:

1. **All features**: `--all-features`
2. **No default features**: `--no-default-features`
3. **Headless mode**: `--features headless` (for examples)

## Troubleshooting

### CI Passes Locally But Fails on GitHub

1. **Different Rust version**: CI uses stable, ensure you're on stable
2. **Cache issues**: Clear GitHub cache or add `[cache]` to workflow
3. **Platform differences**: CI runs on Ubuntu, may behave differently

### Clippy Warnings

If clippy fails:

```bash
# See all warnings
cargo clippy --all --all-features

# Auto-fix where possible
cargo clippy --all --all-features --fix
```

### Format Issues

If format check fails:

```bash
# Format all code
cargo fmt --all

# Check what would change
cargo fmt --all -- --check
```

### Test Failures

```bash
# Run tests with output
cargo test --workspace --all-features -- --nocapture

# Run specific test
cargo test test_name -- --nocapture

# Run tests for specific crate
cargo test -p praxis_graphics
```

## Adding New Checks

To add a new CI job, edit `.github/workflows/rust-ci.yml`:

```yaml
new_job:
  name: New Check
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y libasound2-dev libudev-dev pkg-config
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Rust Cache
      uses: Swatinem/rust-cache@v2
    
    - name: Run Check
      run: cargo your-command
```

## Badge

Add to README.md:

```markdown
[![CI](https://github.com/USERNAME/praxis/workflows/Rust%20CI/badge.svg)](https://github.com/USERNAME/praxis/actions)
```

## Best Practices

1. **Keep CI fast**: Use caching, run checks in parallel
2. **Fail fast**: Run quick checks (fmt) before slow ones (clippy, test)
3. **Test feature combinations**: Ensure all features work independently
4. **Use headless mode**: For examples that need GPU
5. **Document requirements**: List system dependencies clearly
