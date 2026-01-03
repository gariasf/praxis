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

The integration test suite includes comprehensive tests across multiple areas:

#### Core System Integration (`tests/integration_test.rs`)
- **Initialization Tests**: Cross-crate initialization order, independent subsystem initialization, repeated initialization safety
- **ECS World Tests**: Entity creation/cleanup, component lifecycle, resource management
- **Input State Tests**: Key state management, input cleanup
- **Physics World Tests**: Physics entity lifecycle, collision component management
- **Scene Graph Tests**: Transform hierarchy, parent-child relationships, entity cleanup
- **Concurrent Operations**: Multiple world isolation, resource isolation
- **Error Handling**: Cross-crate error propagation, descriptive error messages

#### Asset System Integration (`tests/asset_integration_test.rs`)
- **OBJ Loading**: Basic loading, sequential loading, multiple file formats
- **Path Resolution**: Absolute paths, relative paths, special characters
- **Error Handling**: Nonexistent files, empty files, malformed data
- **Asset Caching**: Simulated cache behavior, reuse patterns
- **Attribute Variations**: Position-only meshes, normals, UVs, complete vertex data
- **Large Mesh Handling**: Memory handling for 1000+ vertex meshes
- **Loader Independence**: Multiple loader instances, reusability
- **Data Structure Validation**: Correct vertex/index parsing, attribute alignment
- **Comments and Edge Cases**: OBJ file comments, file path edge cases

#### Asset Path Resolution (`tests/asset_path_resolution_test.rs`)
- Path handling across different input types
- Canonicalization and normalization

#### Asset Loader Traits (`tests/asset_loader_trait_test.rs`)
- Loader interface compliance
- Extension support verification

#### Resource Cleanup (`tests/resource_cleanup_test.rs`)
- Memory leak detection
- Proper resource disposal

**Focus Areas:**

- Component interaction validation
- Cross-cutting concerns (logging, error handling)
- Memory safety and resource cleanup
- Threading and concurrency patterns
- Asset loading and management
- Transform hierarchy propagation
- Physics-ECS synchronization

**Test Statistics:**
- 5 integration test files
- 50+ individual test cases
- Coverage across all 11 engine crates
- Focus on critical engine subsystem interactions

### 3. Rationale

We focus on testing logic that:

1. **Can fail silently** - Logic errors that might not be immediately obvious
2. **Has complex edge cases** - Boundary conditions, error states
3. **Changes frequently** - Code that's actively developed and prone to regressions
4. **Is platform-dependent** - Code that behaves differently on different systems
5. **Involves cross-crate interactions** - Integration between subsystems

We avoid testing:

1. **External library calls** - We trust well-established libraries like Vulkan, Rapier3D
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
cargo test --package praxis_physics
```

### Integration Tests Only

```bash
cargo test --test integration_test
cargo test --test asset_integration_test
cargo test --test resource_cleanup_test
```

### With Output

```bash
cargo test --workspace -- --nocapture
```

### Specific Test

```bash
cargo test test_cross_crate_initialization_order
cargo test test_asset_loading_cleanup
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

## Benchmarks

In addition to tests, Praxis maintains a comprehensive benchmark suite using Criterion.rs for performance regression detection.

### Benchmark Suites

**Location:** `benches/` directory

1. **`mesh_upload.rs`** - Graphics memory management performance
   - Mesh upload performance (100 to 50,000 vertices)
   - Textured mesh overhead
   - Primitive generation benchmarks

2. **`render_loop.rs`** - Camera and frame timing systems
   - Camera matrix updates (1 to 50 cameras)
   - Primary camera selection
   - Sorted camera queries
   - Frame timer performance

3. **`physics_step.rs`** - Rapier3D integration performance
   - Physics simulation (10 to 500 objects)
   - Collision event detection
   - Raycast queries
   - Point-inside queries
   - Transform synchronization (10 to 1,000 objects)

4. **`transform_propagation.rs`** - Hierarchical transform system
   - Flat hierarchy propagation (10 to 1,000 entities)
   - Tree hierarchies (various depths and breadths)
   - Rotation and scale overhead
   - Deep hierarchy chains (5 to 50 levels)
   - Parent-child sync
   - Incremental transform updates

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific suite
cargo bench --bench mesh_upload
cargo bench --bench render_loop
cargo bench --bench physics_step
cargo bench --bench transform_propagation

# Run specific benchmark
cargo bench --bench physics_step -- physics_raycast

# Save baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main
```

### Benchmark Reports

Criterion generates HTML reports in `target/criterion/`:
- Statistical analysis with confidence intervals
- Performance plots and visualizations
- Regression detection
- Throughput measurements

### Performance Targets

For 60 FPS (16.67ms frame budget):
- Transform propagation: < 1ms for 1,000 entities
- Physics step: < 16ms for 100 objects
- Mesh upload: < 5ms for 10,000 vertices
- Camera updates: < 100μs for 10 cameras

See `docs/benchmarking.md` for detailed benchmark documentation.

## CI Requirements

### GitHub Actions Workflow

The project enforces quality standards through CI (`.github/workflows/rust-ci.yml`):

#### Check Job
1. **Cargo Check** - Verify all crates compile
   ```bash
   cargo check --all
   ```

2. **Format Check** - Enforce consistent code style
   ```bash
   cargo fmt --all -- --check
   ```

3. **Clippy Lints** - Catch common mistakes and enforce best practices
   ```bash
   cargo clippy --all -- -D warnings
   ```

#### Test Job
4. **Run Tests** - Execute full test suite
   ```bash
   cargo test --workspace
   ```

### CI Configuration

- **Triggers**: Pull requests and pushes to `main` branch
- **Platform**: Ubuntu latest
- **Rust Toolchain**: Stable with rustfmt and clippy components
- **Caching**: Rust build cache via `Swatinem/rust-cache@v2`
- **Failure Policy**: All checks must pass for merge

### Linting Standards

Workspace-level lint configuration in `Cargo.toml`:

```toml
[workspace.lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"

[workspace.lints.rust]
unsafe_code = "warn"
missing_docs = "warn"
```

All public items must have rustdoc comments (`///` for items, `//!` for modules).

### Local Pre-commit Checks

Before committing, developers should run:

```bash
# Format code
cargo fmt --all

# Check for errors
cargo check --all

# Run clippy
cargo clippy --all -- -D warnings

# Run tests
cargo test --workspace

# Optional: Run benchmarks
cargo bench
```

## Test Coverage Goals

### Current State
- **Integration Tests**: Comprehensive (50+ test cases)
- **Unit Tests**: Minimal (basic coverage in core modules)
- **Benchmark Coverage**: Excellent (4 comprehensive suites)

### Future Goals
- Achieve 50%+ test coverage on core systems
- Add unit tests for critical rendering paths
- Expand physics system test coverage
- Add scene serialization tests
- Implement render comparison tests using pixel hashing

## Writing New Tests

### Guidelines

1. **Test One Thing**: Each test should verify a single behavior
2. **Clear Names**: Use descriptive test function names (e.g., `test_physics_world_cleanup`)
3. **Arrange-Act-Assert**: Structure tests in three clear phases
4. **Clean Up**: Remove temporary files and resources
5. **No Flakiness**: Avoid timing-dependent or environment-dependent tests
6. **Document Why**: Add comments explaining non-obvious test logic

### Example: Adding an Integration Test

```rust
#[test]
fn test_new_subsystem_integration() {
    // Arrange: Set up test environment
    let mut world = World::new();
    let resource = TestResource::new();
    world.insert_resource(resource);
    
    // Act: Perform the operation
    let entity = world.spawn(TestComponent::default()).id();
    
    // Assert: Verify expected behavior
    assert!(world.get::<TestComponent>(entity).is_some());
    
    // Clean up: Remove temporary resources
    world.clear_entities();
}
```

## Troubleshooting Tests

### Common Issues

**Test fails only in CI:**
- Check for platform-specific behavior
- Verify all dependencies are available in CI environment
- Look for timing-dependent code

**Test is flaky:**
- Identify source of non-determinism
- Add explicit synchronization if needed
- Consider using mocks for external dependencies

**Test is slow:**
- Profile the test to find bottlenecks
- Consider moving to benchmarks if measuring performance
- Reduce test data size while maintaining coverage

## References

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Integration Testing Patterns](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- Project benchmarking guide: `docs/benchmarking.md`
