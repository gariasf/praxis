# Praxis Engine - Just Commands
# Install just: cargo install just
# Run: just <command>

# Default recipe to display help
default:
    @just --list

# Build all crates (debug)
build:
    cargo build --workspace

# Build all crates (release)
build-release:
    cargo build --workspace --release

# Build specific crate
build-crate CRATE:
    cargo build -p {{CRATE}}

# Check all crates compile
check:
    cargo check --all --all-features

# Check with no default features
check-minimal:
    cargo check --all --no-default-features

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run clippy linter
clippy:
    cargo clippy --all --all-features -- -D warnings

# Run clippy with auto-fix (where possible)
clippy-fix:
    cargo clippy --all --all-features --fix

# Run all tests
test:
    cargo test --workspace --all-features

# Run tests for specific crate
test-crate CRATE:
    cargo test -p {{CRATE}}

# Run tests with output
test-verbose:
    cargo test --workspace --all-features -- --nocapture

# Clean build artifacts
clean:
    cargo clean

# Generate and open documentation
doc:
    cargo doc --workspace --no-deps --open

# Generate documentation without opening
doc-build:
    cargo doc --workspace --no-deps

# Run all CI checks locally
ci: check fmt-check clippy test

# Run CI checks with minimal features
ci-minimal: check-minimal fmt-check test

# Build all examples (headless for CI)
examples:
    cargo build --examples --features headless

# Build all examples (with graphics)
examples-graphics:
    cargo build --examples --all-features

# Run specific example
example NAME:
    cargo run --example {{NAME}}

# Run all benchmarks
bench:
    cargo bench --workspace

# Run specific benchmark
bench-one NAME:
    cargo bench --bench {{NAME}}

# Update dependencies
update:
    cargo update

# Show dependency tree
tree:
    cargo tree

# Show duplicate dependencies
tree-duplicates:
    cargo tree -d

# Install development tools
install-dev-tools:
    cargo install cargo-edit cargo-outdated cargo-audit

# Check for outdated dependencies
outdated:
    cargo outdated

# Security audit
audit:
    cargo audit

# Count lines of code
loc:
    @echo "Counting lines of code..."
    @find crates -name '*.rs' | xargs wc -l | tail -n 1

# Profile compilation time
timings:
    cargo build --workspace --timings

# Check for unused dependencies
udeps:
    cargo +nightly udeps --all-targets

# Run cargo expand on specific file
expand FILE:
    cargo expand --lib {{FILE}}

# Create new crate in workspace
new-crate NAME:
    #!/usr/bin/env bash
    mkdir -p crates/{{NAME}}/src
    cat > crates/{{NAME}}/Cargo.toml << EOF
    [package]
    name = "{{NAME}}"
    version = "0.1.0"
    edition = "2021"
    license = "MIT"
    description = "TODO: Add description"

    [lints]
    workspace = true

    [dependencies]
    praxis_utils = { path = "../praxis_utils", version = "0.1.0" }
    EOF
    cat > crates/{{NAME}}/src/lib.rs << EOF
    //! TODO: Add crate documentation.

    #![warn(missing_docs)]
    #![warn(clippy::all, clippy::pedantic, clippy::nursery)]
    EOF
    echo "Created crate: crates/{{NAME}}"
    echo "Don't forget to add it to workspace members in root Cargo.toml!"
