# Testing Guide for Praxis Engine

This document outlines the testing strategy and practices for the Praxis game engine.

## Overview

Praxis uses a pragmatic testing approach that focuses on catching real issues while avoiding overly complex test setups that require GPU hardware or complex mocking.

## Test Categories

### 1. Unit Tests (`#[cfg(test)] mod tests`)

Located within each crate's source files, these test individual functions and logic without external dependencies.

**Current Coverage:**

- `praxis_window`: Window resize logic, dimension validation, debouncing
- `praxis_graphics`: Device selection logic, parameter validation, error handling
- `praxis_utils`: Tracing initialization

**Focus Areas:**

- Pure logic functions that don't require external resources
- Data validation and boundary conditions
- Error handling and propagation
- Edge cases and corner cases

### 2. Integration Tests (`tests/`)

Located in the workspace `tests/` directory, these test interactions between components.

**Current Coverage:**

- Error propagation across crate boundaries
- Configuration validation patterns
- Resource management and cleanup
- Concurrent access safety
- Async initialization patterns

**Focus Areas:**

- Component interaction validation
- Cross-cutting concerns (logging, error handling)
- Memory safety and resource cleanup
- Threading and concurrency patterns

### Rationale

We focus on testing logic that:

1. **Can fail silently** - Logic errors that might not be immediately obvious
2. **Has complex edge cases** - Boundary conditions, error states
3. **Changes frequently** - Code that's actively developed and prone to regressions
4. **Is platform-dependent** - Code that behaves differently on different systems

We avoid testing:

1. **External library calls** - We trust well-established libraries like Vulkan
2. **Hardware-dependent operations** - GPU operations, audio playback
3. **Simple wrappers** - Thin wrappers around external APIs
4. **One-time initialization** - Setup code that runs once and fails obviously

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test --package praxis_window
cargo test --package praxis_graphics
```

### Integration Tests Only

```bash
cargo test --test integration_test
```

### With Output

```bash
cargo test --workspace -- --nocapture
```

## Test Structure

### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_resize_with_valid_dimensions() {
        let state = MockState::new(800, 600);
        assert!(state.should_resize(PhysicalSize::new(1024, 768)));
    }

    #[test]
    fn test_should_not_resize_with_zero_dimensions() {
        let state = MockState::new(800, 600);
        assert!(!state.should_resize(PhysicalSize::new(0, 600)));
    }
}
```

### Integration Test Example

```rust
#[test]
fn test_error_propagation() {
    let root_error = eyre::eyre!("Root cause error");
    let wrapped_error = root_error.wrap_err("Additional context");

    let error_string = format!("{:?}", wrapped_error);
    assert!(error_string.contains("Root cause error"));
    assert!(error_string.contains("Additional context"));
}
```
