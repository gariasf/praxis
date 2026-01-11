# Testing Guide for Praxis Engine

This document outlines the testing strategy and practices for the Praxis game engine.

## Overview

Praxis uses a pragmatic testing approach that focuses on catching real issues while avoiding overly complex test setups that require GPU hardware or complex mocking.

## Table of Contents

- [Test Categories](#test-categories)
- [Running Tests](#running-tests)
- [Headless Testing](#headless-testing)
- [Mock Components](#mock-components)
- [Resource Cleanup Best Practices](#resource-cleanup-best-practices)
- [Test Isolation Strategies](#test-isolation-strategies)
- [Running Graphics and Physics Tests](#running-graphics-and-physics-tests)
- [Test Structure](#test-structure)
- [Benchmarks](#benchmarks)
- [CI Requirements](#ci-requirements)
- [Writing New Tests](#writing-new-tests)
- [Troubleshooting Tests](#troubleshooting-tests)

## Test Categories

### 1. Unit Tests (`#[cfg(test)] mod tests`)

Located within each crate's source files, these test individual functions and logic without external dependencies.

**Current Coverage:**

- `praxis_window`: Window resize logic, dimension validation, debouncing
- `praxis_graphics`: Device selection logic, parameter validation, error handling, post-processing
- `praxis_utils`: Tracing initialization
- `praxis_editor`: Selection system (15+ tests), undo/redo commands (12+ tests), viewport panel (18+ tests), camera controller (10+ tests)
- `praxis_spatial`: Octree operations (10+ tests), BVH operations (10+ tests), AABB calculations, frustum culling
- `praxis_physics`: Component creation, rigid body states, collision detection, spatial queries, transform sync

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

## Headless Testing

Praxis supports headless testing for CI environments and systems without GPU access. This enables running tests on cloud CI runners, Docker containers, and development machines without graphics hardware.

### Overview

Headless mode allows the engine to run tests and build in environments that lack:
- GPU hardware (e.g., cloud CI runners like GitHub Actions)
- Display/windowing capabilities (e.g., Docker containers, SSH sessions)
- Vulkan drivers or graphics APIs

The headless feature flag is designed for:
1. **CI/CD pipelines** - Running automated tests on GitHub Actions, GitLab CI, etc.
2. **Server environments** - Building dedicated game servers without graphics dependencies
3. **Development workflows** - Testing core logic without GPU dependencies
4. **Cross-platform builds** - Ensuring code compiles on all platforms

### Enabling Headless Mode

The `headless` feature is defined in the workspace `Cargo.toml`:

```toml
[features]
default = []
headless = []
```

To enable headless mode, add the `--features headless` flag to any cargo command:

```bash
# Run tests in headless mode
cargo test --workspace --features headless

# Build in headless mode
cargo build --features headless

# Check compilation in headless mode
cargo check --all --features headless

# Run clippy in headless mode
cargo clippy --all --features headless -- -D warnings

# Build examples in headless mode
cargo build --examples --features headless
```

### CI Configuration

The GitHub Actions workflow (`.github/workflows/rust-ci.yml`) uses headless mode for all CI operations:

```yaml
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - name: Cargo Check
        run: cargo check --all --features headless

      - name: Cargo Clippy
        run: cargo clippy --all --features headless -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - name: Run Tests
        run: cargo test --workspace --features headless

  build_examples:
    runs-on: ubuntu-latest
    steps:
      - name: Build Examples
        run: cargo build --examples --features headless
```

### Conditional Compilation Patterns

#### Examples with Graphics Requirements

Examples that require GPU rendering use conditional compilation to provide a fallback in headless mode:

```rust
// Main implementation (requires GPU)
#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    // Initialize graphics
    let mut render_context = RenderContext::new(window.clone()).await?;
    
    // Run event loop with rendering
    event_loop.run(app)
        .map_err(|e| eyre::eyre!("Event loop error: {}", e))?;
    
    Ok(())
}

// Headless fallback
#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("This example requires graphics support and cannot run in headless mode");
    Ok(())
}
```

**Examples using this pattern:**
- `examples/hello_triangle.rs`
- `examples/editor_demo.rs`
- `examples/comprehensive_scene_demo.rs`
- `examples/material_demo.rs`
- `examples/particles_demo.rs`
- `examples/terrain_demo.rs`
- All animation and lighting demos

### Testing Without GPU

#### Unit Tests (No GPU Required)

Most unit tests don't require GPU access and work seamlessly in headless mode:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_calculation() {
        let transform = Transform::from_xyz(1.0, 2.0, 3.0);
        assert_eq!(transform.translation.x, 1.0);
    }

    #[test]
    fn test_selection_system_logic() {
        let mut world = World::new();
        world.insert_resource(SelectionSystem::new());
        
        let entity = world.spawn(Selectable).id();
        
        world
            .get_resource_mut::<SelectionSystem>()
            .unwrap()
            .select_entity(entity, SelectionMode::Replace);
        
        assert!(world
            .get_resource::<SelectionSystem>()
            .unwrap()
            .is_selected(entity));
    }
}
```

**Test categories that work in headless mode:**
- Pure logic tests (math, transforms, algorithms)
- ECS system tests (component queries, resource management)
- State management tests (selection, undo/redo, input)
- Data structure tests (octree, BVH, frustum culling)
- Asset loading tests (parsing, validation, error handling)
- Scene graph tests (hierarchy, traversal, serialization)

#### Integration Tests (Minimal GPU Dependency)

Integration tests in the `tests/` directory are designed to work without actual rendering:

```rust
// tests/integration_test.rs
#[test]
fn test_cross_crate_initialization_order() {
    let _ = praxis_utils::init();
    
    let ecs_result = praxis_ecs::init();
    assert!(ecs_result.is_ok());
    
    let input_result = praxis_input::init();
    assert!(input_result.is_ok());
    
    let physics_result = praxis_physics::init();
    assert!(physics_result.is_ok());
}
```

**Integration test suites:**
- `integration_test.rs` - Cross-crate initialization
- `asset_integration_test.rs` - Asset loading and caching
- `editor_integration_test.rs` - Editor panel coordination
- `spatial_optimization_test.rs` - Spatial data structures
- `animation_comprehensive_test.rs` - Animation system
- `audio_comprehensive_test.rs` - Audio system

### Best Practices for Headless Testing

#### 1. Separate Logic from Rendering

Structure code to isolate pure logic from GPU operations:

```rust
// Good: Logic separated from rendering
pub struct Camera {
    position: Vec3,
    target: Vec3,
    fov: f32,
}

impl Camera {
    // Pure logic - easily testable
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }
    
    // Pure logic - easily testable
    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov, aspect, 0.1, 1000.0)
    }
}

// Rendering happens elsewhere
impl RenderContext {
    pub fn render_camera(&mut self, camera: &Camera) {
        // GPU operations here
    }
}
```

#### 2. Use Trait-Based Abstractions

Define traits for GPU-dependent operations:

```rust
pub trait RenderPass {
    fn execute(&mut self, cmd_buffer: &mut CommandBuffer) -> Result<()>;
    fn name(&self) -> &str;
}

// Real implementation
pub struct BloomPass { /* Vulkan resources */ }
impl RenderPass for BloomPass { /* GPU code */ }

// Mock implementation for tests
#[cfg(test)]
pub struct MockRenderPass;
#[cfg(test)]
impl RenderPass for MockRenderPass {
    fn execute(&mut self, _cmd_buffer: &mut CommandBuffer) -> Result<()> {
        Ok(())
    }
    fn name(&self) -> &str { "Mock" }
}
```

#### 3. Test Data Structures, Not Operations

Focus tests on data correctness rather than GPU side effects:

```rust
#[test]
fn test_draw_command_construction() {
    let command = DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        texture_name: Some("checkerboard".to_string()),
        material_properties: None,
    };
    
    assert_eq!(command.mesh_id, "cube");
    assert_eq!(command.texture_name, Some("checkerboard".to_string()));
}
```

#### 4. Validate Configuration and Parameters

Test setup and validation logic without execution:

```rust
#[test]
fn test_shadow_config_validation() {
    let config = ShadowConfig {
        map_size: 2048,
        cascade_count: 4,
        split_lambda: 0.95,
    };
    
    assert_eq!(config.map_size, 2048);
    assert!(config.map_size.is_power_of_two());
    assert!(config.cascade_count <= 8);
    assert!(config.split_lambda >= 0.0 && config.split_lambda <= 1.0);
}
```

### Local Development with Headless Mode

Even with GPU available, headless mode is useful for:

1. **Fast iteration on logic changes:**
   ```bash
   cargo test --workspace --features headless -- test_selection
   ```

2. **Verifying CI compatibility:**
   ```bash
   cargo check --all --features headless
   cargo clippy --all --features headless
   ```

3. **Building without graphics dependencies:**
   ```bash
   cargo build --release --features headless
   ```

## Mock Components

Mock components are essential for testing code that interacts with complex systems like graphics, physics, or audio without requiring actual hardware or external dependencies.

### When to Use Mocks

Use mock components when:
- Testing logic that interacts with GPU/graphics APIs
- Avoiding heavy external dependencies in unit tests
- Isolating specific behaviors from complex systems
- Running tests in CI without hardware access
- Testing error conditions that are hard to reproduce with real systems

### Mock Component Patterns

#### 1. Mock PostProcessPass

From `crates/praxis_graphics/src/post_process/tests.rs`:

```rust
#[test]
fn test_post_process_pass_trait() {
    struct MockCopyPass;
    
    impl PostProcessPass for MockCopyPass {
        fn execute(
            &mut self,
            _builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
            _input: &RenderTarget,
            _output: &RenderTarget,
        ) -> Result<()> {
            Ok(())
        }
        
        fn name(&self) -> &str {
            "Copy"
        }
    }
    
    let pass = MockCopyPass;
    assert_eq!(pass.name(), "Copy");
    assert!(!pass.requires_depth());
    assert!(!pass.modifies_alpha());
}
```

**Key Features:**
- Implements the same trait as production code
- Returns success without performing actual GPU operations
- Allows testing trait method behavior and default implementations

#### 2. Mock Rendering State

```rust
#[test]
fn test_render_command_construction() {
    struct MockMesh {
        vertex_count: usize,
        index_count: usize,
    }
    
    struct MockMaterial {
        albedo: [f32; 3],
        metallic: f32,
        roughness: f32,
    }
    
    let mesh = MockMesh {
        vertex_count: 36,
        index_count: 36,
    };
    
    let material = MockMaterial {
        albedo: [1.0, 0.0, 0.0],
        metallic: 0.0,
        roughness: 0.5,
    };
    
    // Test logic without actual GPU operations
    assert_eq!(mesh.vertex_count, 36);
    assert_eq!(material.metallic, 0.0);
}
```

#### 3. Mock Framebuffer and Texture

```rust
#[test]
fn test_framebuffer_dimensions() {
    struct MockFramebuffer {
        width: u32,
        height: u32,
    }
    
    struct MockTexture {
        format: String,
        mip_levels: u32,
    }
    
    let framebuffer = MockFramebuffer {
        width: 1920,
        height: 1080,
    };
    
    let texture = MockTexture {
        format: "R8G8B8A8_UNORM".to_string(),
        mip_levels: 1,
    };
    
    assert_eq!(framebuffer.width, 1920);
    assert_eq!(texture.format, "R8G8B8A8_UNORM");
}
```

#### 4. Mock Device and Hardware

```rust
#[cfg(test)]
mod test_mocks {
    pub struct MockDevice;
    pub struct MockQueue;
    pub struct MockBuffer {
        size: usize,
    }
    
    impl MockDevice {
        pub fn new() -> Self { Self }
    }
    
    impl MockBuffer {
        pub fn new(size: usize) -> Self {
            Self { size }
        }
        
        pub fn size(&self) -> usize { 
            self.size 
        }
    }
}

#[test]
fn test_buffer_allocation() {
    let buffer = test_mocks::MockBuffer::new(1024);
    assert_eq!(buffer.size(), 1024);
}
```

### Testing Strategies by Component

#### Graphics System Testing

**What to test:**
- Device selection logic (without actual Vulkan instance)
- Pipeline configuration validation
- Shader parameter validation
- Buffer size calculations
- Format conversions

**What to mock:**
- Vulkan instances and devices
- Command buffers
- Render targets and framebuffers
- Descriptor sets

**Example:**
```rust
#[test]
fn test_mesh_vertex_count_validation() {
    // Test logic without GPU upload
    let vertices = vec![/* vertex data */];
    let indices = vec![/* index data */];
    
    assert_eq!(vertices.len(), 24);
    assert_eq!(indices.len(), 36);
    assert!(indices.len() % 3 == 0, "Indices must form triangles");
}
```

#### Window System Testing

**What to test:**
- Window resize logic
- Dimension validation
- Debouncing behavior
- Event handling state

**What to avoid:**
- Actual window creation (requires display)
- GPU surface creation
- Swapchain operations

#### Scene Graph Testing

**What to test:**
- Transform hierarchy propagation
- Parent-child relationships
- Component lifecycle
- Entity spawning and despawning

**Example:**
```rust
#[test]
fn test_transform_hierarchy() {
    let mut world = World::new();
    
    let parent = world.spawn(Transform::from_xyz(10.0, 0.0, 0.0)).id();
    let child = world.spawn(Transform::from_xyz(5.0, 0.0, 0.0)).id();
    
    // Add parent-child relationship
    world.entity_mut(child).set_parent(parent);
    
    // Test hierarchy without rendering
    assert!(world.get::<Parent>(child).is_some());
}
```

#### Physics System Testing

**What to test:**
- Collision detection logic
- Raycast queries
- Transform synchronization
- Body state management

**What to avoid:**
- Actual physics simulation (Rapier3D handles this)
- Visual debugging (requires rendering)

#### Editor System Testing

**What to test:**
- Selection system logic
- Undo/redo command execution
- Panel state management
- Camera controller math

**What to avoid:**
- Actual viewport rendering
- GUI drawing (egui operations)

**Example from `tests/editor_integration_test.rs`:**
```rust
#[test]
fn test_selection_system_multi_select() {
    let mut world = World::new();
    world.insert_resource(SelectionSystem::new());
    
    let entity1 = world.spawn(Selectable).id();
    let entity2 = world.spawn(Selectable).id();
    
    let mut selection = world.get_resource_mut::<SelectionSystem>().unwrap();
    selection.select_entity(entity1, SelectionMode::Replace);
    selection.select_entity(entity2, SelectionMode::Add);
    
    assert_eq!(selection.selected_entities().len(), 2);
}
```

### Common Mock Patterns

#### Trait-Based Mocking

```rust
pub trait AssetLoader {
    fn load(&self, path: &Path) -> Result<Asset>;
}

// Production implementation
pub struct FileAssetLoader;
impl AssetLoader for FileAssetLoader {
    fn load(&self, path: &Path) -> Result<Asset> {
        // Real file loading
    }
}

// Mock for testing
#[cfg(test)]
pub struct MockAssetLoader {
    pub assets: HashMap<PathBuf, Asset>,
}

#[cfg(test)]
impl AssetLoader for MockAssetLoader {
    fn load(&self, path: &Path) -> Result<Asset> {
        self.assets.get(path)
            .cloned()
            .ok_or_else(|| eyre!("Asset not found"))
    }
}
```

#### Builder Pattern for Complex Mocks

```rust
#[cfg(test)]
struct MockRenderContextBuilder {
    width: u32,
    height: u32,
    device_name: String,
}

#[cfg(test)]
impl MockRenderContextBuilder {
    fn new() -> Self {
        Self {
            width: 1920,
            height: 1080,
            device_name: "MockDevice".to_string(),
        }
    }
    
    fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    
    fn build(self) -> MockRenderContext {
        MockRenderContext {
            width: self.width,
            height: self.height,
            device_name: self.device_name,
        }
    }
}
```

## Resource Cleanup Best Practices

Proper resource cleanup is critical for preventing memory leaks, handle exhaustion, and test interference. This section covers patterns and best practices for resource management in tests.

### Why Resource Cleanup Matters

1. **Memory Leaks**: Unreleased resources accumulate over test runs
2. **Handle Exhaustion**: Physics handles, GPU resources, file handles can be exhausted
3. **Test Isolation**: Resources from one test shouldn't affect another
4. **CI Stability**: Resource leaks can cause flaky or failing tests in CI
5. **Performance**: Cleanup ensures tests run efficiently

### TestWorld Pattern (Physics Tests)

The `TestWorld` wrapper from `crates/praxis_physics/src/tests.rs` provides automatic cleanup:

```rust
/// Wrapper around `World` that tracks spawned entities and ensures proper
/// cleanup of physics resources after tests.
pub struct TestWorld {
    world: World,
    spawned_entities: Vec<praxis_ecs::Entity>,
}

impl TestWorld {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(PhysicsWorld::new());
        world.insert_resource(PhysicsConfig::default());
        world.insert_resource(PhysicsTime::new());
        world.insert_resource(ContactEvents::new());

        Self {
            world,
            spawned_entities: Vec::new(),
        }
    }

    pub fn spawn(&mut self, bundle: impl Bundle) -> Entity {
        let entity = self.world.spawn(bundle);
        self.spawned_entities.push(entity);
        entity
    }
}

impl Drop for TestWorld {
    fn drop(&mut self) {
        // Despawn all tracked entities
        for entity in self.spawned_entities.drain(..) {
            let _ = self.world.despawn(entity);
        }

        // Run cleanup system to ensure physics handles are properly released
        let mut schedule = Schedule::default();
        schedule.add_systems(cleanup_physics_entities);
        schedule.run(self.world.inner_mut());
    }
}
```

**Usage:**
```rust
#[test]
fn test_with_automatic_cleanup() {
    let mut test_world = TestWorld::new();
    
    let entity = test_world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
    ));
    
    // Test code...
    
    // Cleanup happens automatically when test_world is dropped
}
```

### Manual Cleanup Patterns

#### ECS Entity Cleanup

```rust
#[test]
fn test_entity_lifecycle_with_multiple_components() {
    let mut world = World::new();

    let parent_entity = world.spawn(Transform::default());
    let entity = world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        Parent(parent_entity),
    ));

    // Test operations...
    
    // Manual cleanup
    let _ = world.despawn(entity);
    assert!(world.get::<Transform>(entity).is_none());
    
    world.clear_entities();  // Clean up all remaining entities
}
```

#### Resource Cleanup

```rust
#[test]
fn test_resource_lifecycle() {
    let mut world = World::new();
    
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(InputState::new());
    
    // Test operations...
    
    // Remove specific resources
    world.remove_resource::<PhysicsWorld>();
    world.remove_resource::<InputState>();
    
    assert!(!world.contains_resource::<PhysicsWorld>());
    assert!(!world.contains_resource::<InputState>());
}
```

#### Temporary File Cleanup

```rust
#[test]
fn test_asset_loading_with_cleanup() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_asset.obj");
    
    // Create temporary file
    std::fs::write(&test_file, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3")
        .expect("Failed to write test file");
    
    // Test asset loading
    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok());
    
    // Clean up temporary file
    std::fs::remove_file(&test_file)
        .expect("Failed to remove test file");
    
    assert!(!test_file.exists());
}
```

### Batch Cleanup Operations

```rust
#[test]
fn test_batch_entity_cleanup() {
    let mut world = World::new();
    
    // Create batch of entities
    let entities: Vec<_> = (0..100)
        .map(|i| world.spawn(Transform::from_xyz(i as f32, 0.0, 0.0)))
        .collect();
    
    // Test operations...
    
    // Batch cleanup
    for entity in entities {
        let _ = world.despawn(entity);
    }
    
    let remaining = world.query::<&Transform>().iter(world.inner()).count();
    assert_eq!(remaining, 0);
}
```

### Resource Replacement Pattern

```rust
#[test]
fn test_resource_replacement() {
    let mut world = World::new();

    // Insert resource
    world.insert_resource(Counter { value: 1 });
    assert_eq!(world.get_resource::<Counter>().unwrap().value, 1);

    // Replace resource (automatic cleanup of old value)
    world.insert_resource(Counter { value: 2 });
    assert_eq!(world.get_resource::<Counter>().unwrap().value, 2);

    // Final cleanup
    world.remove_resource::<Counter>();
    assert!(!world.contains_resource::<Counter>());
}
```

### Physics-Specific Cleanup

Physics tests require special attention due to Rapier3D's internal handle management:

```rust
#[test]
#[serial]  // Ensure physics tests don't run concurrently
fn test_physics_cleanup() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    
    let entity = world.spawn((
        Transform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
    ));
    
    // Sync to create physics handles
    let mut schedule = Schedule::default();
    schedule.add_systems((sync_physics_transforms_system, sync_colliders).chain());
    schedule.run(world.inner_mut());
    
    // Verify physics handle exists
    {
        let physics_world = world.inner_mut().resource::<PhysicsWorld>();
        assert!(physics_world.get_body_handle(entity).is_some());
    }
    
    // Despawn entity
    world.despawn(entity).expect("Failed to despawn");
    
    // Run cleanup system
    let mut cleanup_schedule = Schedule::default();
    cleanup_schedule.add_systems(cleanup_physics_entities);
    cleanup_schedule.run(world.inner_mut());
    
    // Verify handle was removed
    let physics_world = world.inner_mut().resource::<PhysicsWorld>();
    assert!(physics_world.get_body_handle(entity).is_none());
}
```

### Common Cleanup Mistakes

#### ❌ Mistake: Not Cleaning Up Temporary Files

```rust
#[test]
fn test_bad_cleanup() {
    let test_file = "test.obj";
    std::fs::write(test_file, "data").unwrap();
    
    let result = load_obj(test_file);
    assert!(result.is_ok());
    
    // Missing: std::fs::remove_file(test_file)
    // File persists after test!
}
```

#### ✅ Correct: Always Clean Up

```rust
#[test]
fn test_good_cleanup() {
    let test_file = std::env::temp_dir().join("test.obj");
    std::fs::write(&test_file, "data").unwrap();
    
    let result = load_obj(&test_file);
    assert!(result.is_ok());
    
    std::fs::remove_file(&test_file).ok();  // Always clean up
}
```

#### ❌ Mistake: Not Despawning Entities

```rust
#[test]
fn test_bad_entity_cleanup() {
    let mut world = World::new();
    
    for i in 0..1000 {
        world.spawn(Transform::default());
    }
    
    // Missing: cleanup
    // Entities persist in World until it's dropped
}
```

#### ✅ Correct: Explicit Cleanup

```rust
#[test]
fn test_good_entity_cleanup() {
    let mut world = World::new();
    
    for i in 0..1000 {
        world.spawn(Transform::default());
    }
    
    world.clear_entities();  // Clean up all entities
}
```

### Cleanup Checklist

Before finishing a test, ensure:

- [ ] All spawned entities are despawned or `world.clear_entities()` is called
- [ ] All inserted resources are removed or World is dropped
- [ ] All temporary files are deleted
- [ ] All physics handles are cleaned up via `cleanup_physics_entities`
- [ ] Any external connections (network, audio) are closed
- [ ] Any allocated buffers or GPU resources are released

## Test Isolation Strategies

Test isolation ensures that tests don't interfere with each other, producing reliable and repeatable results. This is crucial for both local development and CI/CD pipelines.

### Why Test Isolation Matters

1. **Repeatability**: Tests should produce the same results every time
2. **Parallelization**: Tests should be able to run concurrently without conflicts
3. **Debugging**: Failures should be clearly attributable to a single test
4. **CI Reliability**: Prevents flaky tests caused by shared state

### Isolation Patterns

#### 1. Independent World Instances

Each test creates its own `World` instance:

```rust
#[test]
fn test_isolated_world_1() {
    let mut world = World::new();
    world.insert_resource(GameState::new());
    
    let entity = world.spawn(Transform::default());
    // This world is independent of other tests
}

#[test]
fn test_isolated_world_2() {
    let mut world = World::new();
    world.insert_resource(GameState::new());
    
    let entity = world.spawn(Transform::default());
    // This world is completely separate from test_isolated_world_1
}
```

#### 2. Multiple Concurrent Worlds

Tests can create multiple worlds simultaneously without interference:

```rust
#[test]
fn test_multiple_worlds_isolation() {
    let mut world1 = World::new();
    let mut world2 = World::new();

    for i in 0..100 {
        world1.spawn(Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)));
        world2.spawn(Transform::from_translation(Vec3::new((i * 2) as f32, 0.0, 0.0)));
    }

    let count1: Vec<_> = world1
        .query::<&Transform>()
        .iter(world1.inner())
        .map(|t| t.translation.x)
        .collect();

    let count2: Vec<_> = world2
        .query::<&Transform>()
        .iter(world2.inner())
        .map(|t| t.translation.x)
        .collect();

    assert_eq!(count1.len(), 100);
    assert_eq!(count2.len(), 100);
    assert_ne!(count1, count2);  // Data is isolated between worlds
}
```

#### 3. Serial Test Execution for Shared Resources

Use `#[serial]` attribute for tests that access shared global state:

```rust
use serial_test::serial;

#[test]
#[serial]  // This test runs serially, not in parallel
fn test_physics_world_1() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    
    // Physics world uses global state internally
}

#[test]
#[serial]  // Also runs serially
fn test_physics_world_2() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    
    // Won't interfere with test_physics_world_1
}
```

#### 4. One-Time Initialization

Use `std::sync::Once` for truly global initialization:

```rust
use std::sync::Once;

static INIT: Once = Once::new();

fn init_global_resources() {
    INIT.call_once(|| {
        // This runs exactly once, even if called from multiple tests
        praxis_utils::init();
    });
}

#[test]
fn test_with_initialization_1() {
    init_global_resources();
    // Test code
}

#[test]
fn test_with_initialization_2() {
    init_global_resources();
    // Same initialization, but safe to call multiple times
}
```

#### 5. Temporary Directory Isolation

Use unique temporary directories for file-based tests:

```rust
#[test]
fn test_with_unique_temp_dir() {
    use std::time::SystemTime;
    
    let temp_dir = std::env::temp_dir()
        .join(format!("praxis_test_{}", 
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    
    std::fs::create_dir_all(&temp_dir).unwrap();
    
    let test_file = temp_dir.join("test.obj");
    std::fs::write(&test_file, "data").unwrap();
    
    // Test operations...
    
    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}
```

#### 6. Resource Scoping

Use explicit scoping to control resource lifetimes:

```rust
#[test]
fn test_resource_scoping() {
    let mut world = World::new();
    
    // Inner scope for temporary resource access
    {
        let mut selection = SelectionSystem::new();
        selection.select_entity(entity, SelectionMode::Replace);
        world.insert_resource(selection);
    }
    
    // Resource still exists but inner scope variables are dropped
    assert!(world.contains_resource::<SelectionSystem>());
    
    {
        let selection = world.get_resource::<SelectionSystem>().unwrap();
        assert_eq!(selection.selected_count(), 1);
    }  // Reference dropped here
    
    // Can now get mutable reference
    let mut selection = world.get_resource_mut::<SelectionSystem>().unwrap();
    selection.clear();
}
```

### Handling Global State

#### Input State Isolation

```rust
#[test]
fn test_input_state_reset() {
    let mut input_state = InputState::new();
    
    input_state.press_key(KeyCode::KeyW);
    input_state.press_key(KeyCode::KeyA);
    
    assert!(input_state.is_key_pressed(KeyCode::KeyW));
    
    // Reset for next test section
    input_state.clear();
    
    assert!(!input_state.is_key_pressed(KeyCode::KeyW));
    assert!(!input_state.is_key_pressed(KeyCode::KeyA));
}
```

#### Tracing Initialization Handling

```rust
static TRACING_INIT: Once = Once::new();

fn init_tracing_once() {
    TRACING_INIT.call_once(|| {
        let _ = praxis_utils::init();
    });
}

#[test]
fn test_with_tracing_1() {
    init_tracing_once();
    // Tracing is initialized exactly once
}

#[test]
fn test_with_tracing_2() {
    init_tracing_once();
    // Safe to call again, will not re-initialize
}
```

### Anti-Patterns to Avoid

#### ❌ Sharing Mutable State Between Tests

```rust
// DON'T DO THIS
static mut SHARED_WORLD: Option<World> = None;

#[test]
fn test_bad_1() {
    unsafe {
        SHARED_WORLD = Some(World::new());
        // Other tests might modify this!
    }
}

#[test]
fn test_bad_2() {
    unsafe {
        // This might see state from test_bad_1
        if let Some(world) = &mut SHARED_WORLD {
            // Dangerous!
        }
    }
}
```

#### ✅ Create Independent Instances

```rust
// DO THIS INSTEAD
#[test]
fn test_good_1() {
    let mut world = World::new();
    // Independent instance
}

#[test]
fn test_good_2() {
    let mut world = World::new();
    // Completely separate instance
}
```

#### ❌ Leaving Files Without Cleanup

```rust
// DON'T DO THIS
#[test]
fn test_bad_file_handling() {
    std::fs::write("test.txt", "data").unwrap();
    // File persists after test
}
```

#### ✅ Use Unique Names and Clean Up

```rust
// DO THIS INSTEAD
#[test]
fn test_good_file_handling() {
    let file_path = std::env::temp_dir().join(format!("test_{}.txt", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    
    std::fs::write(&file_path, "data").unwrap();
    
    // Test operations...
    
    std::fs::remove_file(&file_path).ok();
}
```

### Testing Concurrent Operations

```rust
#[test]
fn test_concurrent_world_operations() {
    use std::thread;
    
    let handle1 = thread::spawn(|| {
        let mut world = World::new();
        for i in 0..100 {
            world.spawn(Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)));
        }
        world.query::<&Transform>().iter(world.inner()).count()
    });
    
    let handle2 = thread::spawn(|| {
        let mut world = World::new();
        for i in 0..100 {
            world.spawn(Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)));
        }
        world.query::<&Transform>().iter(world.inner()).count()
    });
    
    let count1 = handle1.join().unwrap();
    let count2 = handle2.join().unwrap();
    
    assert_eq!(count1, 100);
    assert_eq!(count2, 100);
}
```

## Running Graphics and Physics Tests

### Local Development (With GPU)

When you have a GPU available locally, you can run all tests including those that interact with graphics systems:

```bash
# Run all tests (including graphics tests if GPU available)
cargo test --workspace

# Run specific graphics tests
cargo test --package praxis_graphics
cargo test --package praxis_editor

# Run physics tests (require serial execution)
cargo test --package praxis_physics
```

### Local Development (Without GPU / Headless)

When developing on a machine without GPU or in SSH/remote sessions:

```bash
# Run tests without graphics dependencies
cargo test --workspace --features headless

# Check that code compiles in headless mode
cargo check --all --features headless

# Run lints in headless mode
cargo clippy --all --features headless -- -D warnings
```

### CI Environment

CI runs use headless mode exclusively since GitHub Actions runners don't have GPU access:

```yaml
# From .github/workflows/rust-ci.yml
- name: Run Tests
  run: cargo test --workspace --features headless

- name: Cargo Check
  run: cargo check --all --features headless

- name: Cargo Clippy
  run: cargo clippy --all --features headless -- -D warnings
```

### Physics Tests Considerations

Physics tests use `#[serial]` because Rapier3D's physics world has internal global state:

```rust
use serial_test::serial;

#[test]
#[serial]  // Must run serially to avoid conflicts
fn test_physics_world_creation() {
    let mut world = World::new();
    world.insert_resource(PhysicsWorld::new());
    // Test code...
}
```

**Why Serial Execution?**
- Rapier3D internally manages physics handles
- Concurrent physics world creation can cause handle conflicts
- Tests run sequentially to ensure proper isolation

**Running Physics Tests:**
```bash
# Run all physics tests serially
cargo test --package praxis_physics

# Run specific physics test
cargo test --package praxis_physics test_rigid_body_creation
```

### Graphics Tests Considerations

Graphics tests that don't require actual GPU rendering work in both modes:

```rust
#[test]
fn test_post_process_configuration() {
    let config = BloomConfig {
        brightness_threshold: 1.5,
        bloom_intensity: 1.0,
        blur_iterations: 3,
        exposure: 1.0,
    };
    
    // Tests configuration logic without GPU
    assert_eq!(config.brightness_threshold, 1.5);
}
```

Tests requiring GPU are typically integration tests or examples:

```rust
// Example that requires GPU
#[cfg(not(feature = "headless"))]
fn main() -> Result<()> {
    let mut render_context = RenderContext::new(window).await?;
    // GPU-dependent rendering
}

#[cfg(feature = "headless")]
fn main() -> Result<()> {
    println!("This example requires graphics support");
    Ok(())
}
```

### Platform-Specific Considerations

#### Windows

```powershell
# Run all tests
cargo test --workspace

# Run with specific backend (if multiple GPUs)
$env:VK_ICD_FILENAMES="path\to\driver.json"
cargo test --workspace
```

#### Linux

```bash
# Run all tests
cargo test --workspace

# Headless mode (no X11/Wayland)
cargo test --workspace --features headless

# With specific Vulkan driver
VK_ICD_FILENAMES=/path/to/driver.json cargo test --workspace
```

#### macOS

```bash
# Run all tests (MoltenVK for Vulkan)
cargo test --workspace

# Headless mode
cargo test --workspace --features headless
```

### Test Categories and GPU Requirements

| Test Category | GPU Required? | Run in CI? | Serial Execution? |
|--------------|---------------|------------|------------------|
| Unit tests (math, ECS) | No | Yes | No |
| Unit tests (graphics config) | No | Yes | No |
| Integration tests | No | Yes | No |
| Physics tests | No | Yes | Yes (`#[serial]`) |
| Asset loading tests | No | Yes | No |
| Editor logic tests | No | Yes | No |
| Rendering examples | Yes | No | N/A |
| GPU benchmark tests | Yes | No | No |

### Running Specific Test Subsets

```bash
# Only unit tests (fast, no GPU)
cargo test --lib --all

# Only integration tests
cargo test --test '*'

# Only tests matching pattern
cargo test physics -- --nocapture

# Run tests with specific feature
cargo test --features headless

# Run tests excluding specific crate
cargo test --workspace --exclude praxis_graphics
```

### Debugging Graphics Tests

When graphics tests fail on local machine:

1. **Check Vulkan installation:**
   ```bash
   vulkaninfo  # Should show available devices
   ```

2. **Verify environment:**
   ```bash
   cargo test --package praxis_graphics -- --nocapture
   ```

3. **Check drivers:**
   - Update GPU drivers to latest version
   - Ensure Vulkan SDK is installed
   - Verify device has Vulkan 1.2+ support

4. **Fallback to headless:**
   ```bash
   cargo test --features headless
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

# Run benchmarks in headless mode (non-GPU benchmarks only)
cargo bench --features headless
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
   cargo check --all --features headless
   ```

2. **Format Check** - Enforce consistent code style
   ```bash
   cargo fmt --all -- --check
   ```

3. **Clippy Lints** - Catch common mistakes and enforce best practices
   ```bash
   cargo clippy --all --features headless -- -D warnings
   ```

#### Test Job
4. **Run Tests** - Execute full test suite
   ```bash
   cargo test --workspace --features headless
   ```

#### Build Examples Job
5. **Build Examples** - Ensure all examples compile
   ```bash
   cargo build --examples --features headless
   ```

### CI Configuration

- **Triggers**: Pull requests and pushes to `main` branch
- **Platform**: Ubuntu latest
- **Rust Toolchain**: Stable with rustfmt and clippy components
- **Caching**: Rust build cache via `Swatinem/rust-cache@v2`
- **Failure Policy**: All checks must pass for merge
- **Dependencies**: libasound2-dev, libudev-dev, pkg-config

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

# Optional: Verify headless mode
cargo check --all --features headless
cargo test --workspace --features headless

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
- **Physics Tests**: Extensive (50+ tests covering components, systems, queries)

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
- Ensure headless mode is properly configured
- Look for timing-dependent code
- Check file path assumptions (CI may have different directory structure)

**Test is flaky:**
- Identify source of non-determinism
- Add explicit synchronization if needed
- Consider using mocks for external dependencies
- Check for race conditions in concurrent code
- Ensure proper cleanup between test runs

**Test is slow:**
- Profile the test to find bottlenecks
- Consider moving to benchmarks if measuring performance
- Reduce test data size while maintaining coverage
- Check for unnecessary resource allocations
- Use mock components for expensive operations

**ECS test has unexpected state:**
- Ensure World is properly initialized
- Check that all required resources are inserted
- Verify component queries are correct
- Look for leftover state from previous operations
- Use `world.clear_entities()` between test sections

**Physics test fails:**
- Verify `#[serial]` attribute is present
- Check that `PhysicsWorld` is properly initialized
- Ensure cleanup systems are run
- Verify entity handles are properly released
- Check for floating point precision issues

**Graphics test fails locally:**
- Verify GPU drivers are up to date
- Check Vulkan installation with `vulkaninfo`
- Try running with headless mode: `cargo test --features headless`
- Check for hardware compatibility issues
- Verify swapchain creation is not attempted in tests

**File I/O test fails:**
- Use `std::env::temp_dir()` for temporary files
- Always clean up files after tests
- Use unique file names to avoid conflicts
- Check file permissions
- Verify path separators for cross-platform compatibility

### Debugging Tips

```bash
# Run test with output
cargo test test_name -- --nocapture

# Run test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run single test
cargo test --package praxis_physics test_rigid_body_creation

# Show ignored tests
cargo test -- --ignored

# List all tests without running
cargo test -- --list
```

## Summary

Praxis testing follows these principles:

1. **Use the `headless` feature flag** for all GPU-dependent operations
2. **Separate logic from rendering** to maximize testable code
3. **Mock GPU resources** using traits and test doubles
4. **Test data structures and algorithms** rather than GPU side effects
5. **Run unit and integration tests** in CI without GPU hardware
6. **Provide informative fallbacks** for examples that require graphics
7. **Ensure proper resource cleanup** to prevent leaks and test interference
8. **Isolate tests** using independent World instances and serial execution where needed
9. **Document test patterns** and maintain consistency across the codebase

This approach ensures:
- ✅ Tests run reliably in CI/CD pipelines
- ✅ Development doesn't require GPU for all testing
- ✅ Core engine logic is well-tested
- ✅ GPU-specific code is isolated and documented
- ✅ Tests are maintainable and easy to understand
- ✅ Resource leaks are prevented
- ✅ Tests can run in parallel when safe
- ✅ Flaky tests are minimized

## References

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Integration Testing Patterns](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Bevy ECS Testing Patterns](https://bevyengine.org/learn/book/getting-started/ecs/)
- [Cargo Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)
- Project benchmarking guide: `docs/benchmarking.md`
