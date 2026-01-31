# Contributing to Praxis

Thank you for your interest in contributing to Praxis! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)

## Code of Conduct

Be respectful, constructive, and professional. This is an educational project - we're all here to learn.

## Getting Started

### Prerequisites

```bash
# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required components
rustup component add rustfmt clippy

# Install optional development tools
cargo install cargo-make just cargo-edit cargo-outdated cargo-audit
```

### System Dependencies

#### Ubuntu/Debian
```bash
sudo apt-get install libasound2-dev libudev-dev pkg-config
```

#### macOS
```bash
brew install pkg-config
```

#### Windows
No additional dependencies required.

### Clone and Build

```bash
git clone https://github.com/USERNAME/praxis.git
cd praxis
cargo build --workspace
```

### Run Tests

```bash
cargo test --workspace --all-features
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

### 2. Make Changes

Follow the [Code Style](#code-style) guidelines.

### 3. Run Checks

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all --all-features -- -D warnings

# Run tests
cargo test --workspace --all-features

# Or run all at once
cargo make ci  # or: just ci
```

### 4. Commit Changes

```bash
git add .
git commit -m "feat: add new feature"
# or
git commit -m "fix: resolve issue with X"
```

**Commit Message Format:**
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting)
- `refactor:` Code refactoring
- `test:` Adding or updating tests
- `chore:` Maintenance tasks

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

## Code Style

### Formatting

All code must be formatted with `rustfmt`:

```bash
cargo fmt --all
```

### Linting

All code must pass clippy with workspace lints:

```toml
[lints]
workspace = true
```

This enforces:
- `clippy::all`
- `clippy::pedantic`
- `clippy::nursery`
- `unsafe_code = warn`
- `missing_docs = warn`

### Documentation

All public items must have rustdoc comments:

```rust
/// Brief description of what this does.
///
/// More detailed explanation if needed.
///
/// # Examples
///
/// ```
/// use praxis_math::Vec3;
/// let v = Vec3::new(1.0, 2.0, 3.0);
/// ```
pub fn example() {}
```

Module-level documentation:

```rust
//! Module description.
//!
//! More details about what this module provides.
```

### Naming Conventions

See [CLAUDE.md - Naming Conventions](CLAUDE.md#naming-conventions) for detailed guidelines.

**Summary:**
- **Manager**: Resource caching, lifetime management
- **Renderer**: GPU rendering, draw calls
- **System**: ECS behavior, component processing
- **Context**: Top-level API coordinator (rare)

### Code Organization

```rust
// Imports
use std::collections::HashMap;
use praxis_math::Vec3;

// Constants
const MAX_ENTITIES: usize = 1000;

// Type aliases
type EntityId = u64;

// Structs/Enums
pub struct Example {
    field: i32,
}

// Implementations
impl Example {
    pub fn new() -> Self {
        Self { field: 0 }
    }
}

// Functions
pub fn helper() {}
```

## Testing

### Unit Tests

Place unit tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(2 + 2, 4);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/integration_test.rs
use praxis_graphics::RenderContext;

#[test]
fn test_render_context() {
    // Test code
}
```

### Doc Tests

Include in documentation:

```rust
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// use my_crate::add;
/// assert_eq!(add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Running Tests

```bash
# All tests
cargo test --workspace --all-features

# Specific crate
cargo test -p praxis_graphics

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture
```

## Documentation

### Crate-Level Documentation

Each crate should have:
- `README.md`: Overview, examples, usage
- `lib.rs`: Module-level docs (`//!`)

### Adding Examples

1. Create file in `examples/` directory
2. Add to root `Cargo.toml`:

```toml
[[example]]
name = "my_example"
path = "examples/my_example.rs"
required-features = ["feature-name"]  # if needed
```

3. Document in `CLAUDE.md` command list

### Building Documentation

```bash
# Generate and open
cargo doc --workspace --no-deps --open

# Just generate
cargo doc --workspace --no-deps
```

## Submitting Changes

### Pull Request Checklist

- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Clippy passes (`cargo clippy --all --all-features -- -D warnings`)
- [ ] Tests pass (`cargo test --workspace --all-features`)
- [ ] Documentation is updated
- [ ] Examples are updated (if applicable)
- [ ] Commit messages follow convention
- [ ] PR description explains the change

### PR Description Template

```markdown
## Description
Brief description of the change.

## Motivation
Why is this change needed?

## Changes
- Change 1
- Change 2
- Change 3

## Testing
How was this tested?

## Checklist
- [ ] Code formatted
- [ ] Clippy passed
- [ ] Tests passed
- [ ] Documentation updated
```

### Review Process

1. CI checks must pass
2. At least one maintainer approval required
3. Address review comments
4. Squash commits if needed
5. Maintainer will merge

## Adding New Crates

### Using justfile

```bash
just new-crate praxis_my_feature
```

### Manual Creation

1. Create directory and files:

```bash
mkdir -p crates/praxis_my_feature/src
```

2. Create `Cargo.toml`:

```toml
[package]
name = "praxis_my_feature"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "Brief description"

[lints]
workspace = true

[dependencies]
praxis_utils = { path = "../praxis_utils", version = "0.1.0" }
```

3. Create `src/lib.rs`:

```rust
//! Crate documentation.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
```

4. Add to workspace in root `Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing crates ...
    "crates/praxis_my_feature",
]
```

5. Create `README.md` documenting the crate

6. Update `CLAUDE.md` architecture table

## Educational Focus

Praxis is an educational engine. When contributing:

1. **Prioritize clarity over cleverness**: Code should teach, not obscure
2. **Document the "why"**: Explain design decisions
3. **Use proven patterns**: Demonstrate industry-standard approaches
4. **Focus on fundamentals**: Core concepts over edge cases
5. **Avoid over-engineering**: Keep it practical

See [CLAUDE.md - Educational Value](CLAUDE.md#educational-value--design-rationale) for detailed guidance.

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing issues and PRs first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
