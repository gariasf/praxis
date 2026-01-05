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
- `praxis_editor`: Selection system (15+ tests), undo/redo commands (12+ tests), viewport panel (18+ tests), camera controller (10+ tests)
- `praxis_spatial`: Octree operations (10+ tests), BVH operations (10+ tests), AABB calculations, frustum culling

**Focus Areas:**

- Pure logic functions that don't require external resources
- Data validation and boundary conditions
- Error handling and propagation
- Edge cases and corner cases
- Component state management
- Command pattern execution and reversal
- Spatial query accuracy

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

#### Editor Integration (`tests/editor_integration_test.rs`)
- **Hierarchy Panel**: Entity tree operations, reparenting, expansion/collapse, parent-child relationship management
- **Inspector Panel**: Component editing, multi-component entities, Transform/Physics/Audio/Camera component modification
- **Viewport Panel**: Initialization, camera controls, selection integration, camera presets, grid/gizmo visibility, focus-on-selection
- **Drag-and-Drop System**: Asset instantiation workflow, entity hierarchy operations, cancellation, frame reset
- **Editor Camera Controller**: Target/distance controls, angle manipulation, focus functionality, input processing, smooth interpolation
- **Multi-Panel Integration**: Hierarchy selection affecting inspector, viewport camera following selection, asset drag to viewport
- **Undo/Redo Integration**: Command execution across editor panels, state restoration

#### Spatial Optimization (`tests/spatial_optimization_test.rs`)
- **Frustum Culling**: Basic visibility, edge cases, sphere/point visibility, near/far plane handling
- **Octree**: Insertion/query/removal, radius queries, ray queries, sorted ray queries, update operations
- **BVH**: Build/query operations, radius queries, ray queries, sorted ray queries, insert/remove/update
- **LOD System**: Level creation, distance-based selection, boundary cases, manager registration, entity assignment, batch selection
- **Visibility System**: Frustum update, distance culling, LOD integration, culling statistics
- **Integration Tests**: Complete culling pipeline, moving objects, camera movement with LOD

**Focus Areas:**

- Component interaction validation
- Cross-cutting concerns (logging, error handling)
- Memory safety and resource cleanup
- Threading and concurrency patterns
- Asset loading and management
- Transform hierarchy propagation
- Physics-ECS synchronization
- Editor panel coordination
- Spatial query correctness
- LOD selection logic
- Culling system accuracy

**Test Statistics:**
- 9 integration test files
- 150+ individual test cases
- Coverage across all major engine subsystems
- Focus on critical engine subsystem interactions
- Extensive editor workflow testing
- Comprehensive spatial optimization validation

### 3. Rationale

We focus on testing logic that:

1. **Can fail silently** - Logic errors that might not be immediately obvious
2. **Has complex edge cases** - Boundary conditions, error states
3. **Changes frequently** - Code that's actively developed and prone to regressions
4. **Is platform-dependent** - Code that behaves differently on different systems
5. **Involves cross-crate interactions** - Integration between subsystems
6. **Requires correct state management** - ECS resources, command history, selection state

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
cargo test --package praxis_editor
cargo test --package praxis_spatial
```

### Integration Tests Only

```bash
cargo test --test integration_test
cargo test --test asset_integration_test
cargo test --test resource_cleanup_test
cargo test --test editor_integration_test
cargo test --test spatial_optimization_test
```

### With Output

```bash
cargo test --workspace -- --nocapture
```

### Specific Test

```bash
cargo test test_cross_crate_initialization_order
cargo test test_asset_loading_cleanup
cargo test test_viewport_panel_camera_controls
cargo test test_octree_insertion_and_query
cargo test test_frustum_culling_basic_visibility
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

### ECS-Integrated System Test Example

```rust
#[test]
fn test_selection_system_with_world() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    // Create selectable entity
    let entity = world.spawn((
        Transform::default(),
        Selectable,
    ));

    // Select entity
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity, SelectionMode::Replace);

    // Verify selection
    assert!(world
        .get_resource::<SelectionSystem>()
        .unwrap()
        .is_selected(entity));
}
```

### Viewport Rendering Test Example

```rust
#[test]
fn test_viewport_camera_focus_on_selection() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());

    let mut viewport = ViewportPanel::new();

    // Create entities at different positions
    let entity1 = world.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        MeshHandle::new("cube".to_string()),
        Selectable,
    ));

    // Select entity
    world
        .get_resource_mut::<SelectionSystem>()
        .unwrap()
        .select_entity(entity1, SelectionMode::Replace);

    // Focus camera
    viewport.focus_on_selection(&mut world);
    viewport.update_camera(1.0);

    // Verify camera target
    let target = viewport.camera_target();
    assert!((target - Vec3::ZERO).length() < 0.1);
}
```

### Editor Command Test Example

```rust
#[test]
fn test_undo_redo_transform_edit() {
    let mut world = World::new();
    let mut history = CommandHistory::new();
    let entity = world.spawn(Transform::default()).id();

    let old_transform = Transform::default();
    let new_transform = Transform::from_xyz(10.0, 5.0, 3.0);

    // Execute command
    let command = Box::new(TransformEditCommand::new(
        entity,
        old_transform,
        new_transform,
    ));
    history.execute(&mut world, command).unwrap();

    // Verify edit
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation.x, 10.0);

    // Undo
    history.undo(&mut world).unwrap();
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation.x, 0.0);

    // Redo
    history.redo(&mut world).unwrap();
    let transform = world.get::<Transform>(entity).unwrap();
    assert_eq!(transform.translation.x, 10.0);
}
```

### Spatial Optimization Test Example

```rust
#[test]
fn test_frustum_culling_with_visibility_system() {
    let mut visibility_system = VisibilitySystem::with_max_distance(200.0);

    // Set up camera
    let camera_pos = Vec3::new(0.0, 0.0, 10.0);
    let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
    let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
    let view_proj = proj * view;

    visibility_system.update_frustum(view_proj);

    // Create test entities
    let entities = vec![
        (
            Entity::from_raw(1),
            Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0)),
            Vec3::ZERO,
        ),
        (
            Entity::from_raw(2),
            Aabb::from_center_half_extents(Vec3::new(50.0, 0.0, 0.0), Vec3::splat(1.0)),
            Vec3::new(50.0, 0.0, 0.0),
        ),
    ];

    // Run culling
    let (results, stats) = visibility_system.cull_entities(&entities, camera_pos);

    // Verify results
    assert_eq!(stats.total_objects, 2);
    assert!(stats.visible_objects > 0);
    assert!(results[0].is_visible); // Object at origin should be visible
    assert!(!results[1].is_visible); // Object to side should be culled
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
- **Integration Tests**: Comprehensive (150+ test cases across 9 files)
- **Unit Tests**: Good (80+ test cases in core modules)
- **Benchmark Coverage**: Excellent (4 comprehensive suites)
- **Editor Tests**: Extensive (60+ tests across selection, undo/redo, viewport, camera)
- **Spatial Tests**: Comprehensive (45+ tests across octree, BVH, frustum, LOD)

### Future Goals
- Achieve 60%+ test coverage on core systems
- Add unit tests for critical rendering paths
- Expand physics system test coverage
- Add scene serialization tests
- Implement render comparison tests using pixel hashing
- Add performance regression tests for editor operations

## Writing New Tests

### Guidelines

1. **Test One Thing**: Each test should verify a single behavior
2. **Clear Names**: Use descriptive test function names (e.g., `test_viewport_camera_focus_on_selection`)
3. **Arrange-Act-Assert**: Structure tests in three clear phases
4. **Clean Up**: Remove temporary files and resources
5. **No Flakiness**: Avoid timing-dependent or environment-dependent tests
6. **Document Why**: Add comments explaining non-obvious test logic
7. **ECS Patterns**: Initialize World and resources properly for system tests
8. **State Verification**: Always verify both positive and negative cases

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

### Testing ECS-Integrated Systems

When testing systems that interact with the ECS:

1. **Create a World**: Always start with `World::new()`
2. **Insert Resources**: Add any required resources before spawning entities
3. **Spawn Test Entities**: Create entities with the components needed for testing
4. **Execute Systems**: Call system functions or manually perform operations
5. **Query and Verify**: Use queries to verify component state
6. **Clean Up**: Clear entities and resources at the end

```rust
#[test]
fn test_ecs_system_behavior() {
    let mut world = World::new();
    world.insert_resource(SystemResource::default());
    
    let entity = world.spawn((
        Component1::default(),
        Component2::default(),
    ));
    
    // Run system
    my_system(&mut world);
    
    // Verify results
    let result = world.get::<Component1>(entity).unwrap();
    assert_eq!(result.value, expected_value);
}
```

### Testing Viewport Rendering

Viewport tests focus on camera controls, selection integration, and visual state without actual rendering:

```rust
#[test]
fn test_viewport_feature() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    let mut viewport = ViewportPanel::new();
    
    // Set up scene
    let entity = world.spawn((
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        GlobalTransform::default(),
        Selectable,
    ));
    
    // Interact with viewport
    viewport.focus_on_selection(&mut world);
    viewport.update_camera(1.0);
    
    // Verify camera state
    let target = viewport.camera_target();
    assert!((target.x - 10.0).abs() < 0.1);
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

**ECS test has unexpected state:**
- Ensure World is properly initialized
- Check that all required resources are inserted
- Verify component queries are correct
- Look for leftover state from previous operations

## References

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Integration Testing Patterns](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Bevy ECS Testing Patterns](https://bevyengine.org/learn/book/getting-started/ecs/)
- Project benchmarking guide: `docs/benchmarking.md`
