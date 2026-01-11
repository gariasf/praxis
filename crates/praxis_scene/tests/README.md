# SaveManager Integration Tests

This directory contains comprehensive integration tests for the `SaveManager` in `praxis_scene`.

## Test Files

### `save_manager_tests.rs`
Core integration tests covering:

#### Entity Serialization
- Empty world save/load
- Single entity with transform
- Entity with all component types
- Multiple entities

#### Hierarchy Management
- Parent-child relationships
- Deep hierarchies (3+ levels)
- Multiple children per parent
- Complex multi-branch hierarchies

#### Component Coverage
- **Transform**: Position, rotation, scale
- **MeshHandle**: Mesh asset references
- **TextureHandle**: Texture asset references
- **MaterialHandle**: Material asset references
- **Camera**: Both perspective and orthographic projections
- **DirectionalLight**: Direction, color, intensity
- **PointLight**: Position, color, intensity, range
- **Visibility**: Visible/Hidden states
- **Active**: Active component marker

#### Special Features
- **NoSave marker**: Entities marked with `NoSave` are excluded from saves
- **Statistics tracking**: Entity count, component count, duration, file size
- **World clearing**: Load operation clears existing world first
- **Metadata preservation**: All metadata fields preserved through save/load

#### Version Migration
- Migration from version 0 to current
- Migration from version 1 to current
- Scene version migration testing

#### Advanced Scenarios
- Complex scenes with multiple entity types
- Transform with full rotation and scale
- Multiple save/load cycles
- Concurrent saves to different paths
- Nested directory creation
- Large world with 100+ entities

#### Configuration
- Pretty print vs compact format
- Validation on/off
- Custom save configuration

### `save_editor_data_tests.rs`
Editor-specific integration tests covering:

#### Editor Data Preservation
- **EditorCamera**: Orbit and free camera modes with full state
- **ViewportSettings**: Grid, gizmos, wireframe, colors, modes
- **EditorPreferences**: Auto-save, snap settings, asset paths
- **Selection state**: Selected entity tracking

#### Camera Modes
- Orbit camera with target, distance, pitch, yaw
- Free camera with position and orientation
- Fly mode support

#### Viewport Features
- Grid display settings
- Gizmo mode (Translate, Rotate, Scale)
- Wireframe overlay
- Bounding box display
- Light visualization
- Camera frustum display

#### Editor Preferences
- Auto-save configuration
- Snap-to-grid settings
- Rotation snap angles
- Asset browser state
- Hierarchy panel state (collapsed nodes)

#### Roundtrip Testing
- Complete editor data serialization/deserialization
- Runtime scene conversion (strips editor data)
- Editor data mutability
- Clear/reset operations

## Test Coverage Summary

### Total Tests: 40+
- **Basic operations**: 10 tests
- **Hierarchy handling**: 5 tests
- **Component types**: 8 tests
- **Version migration**: 4 tests
- **Configuration**: 4 tests
- **Editor data**: 20 tests
- **Advanced scenarios**: 8 tests

### Components Tested
- ✅ Name
- ✅ Transform (full: translation, rotation, scale)
- ✅ GlobalTransform
- ✅ MeshHandle
- ✅ TextureHandle
- ✅ MaterialHandle
- ✅ Camera (Perspective & Orthographic)
- ✅ DirectionalLight
- ✅ PointLight
- ✅ Visibility
- ✅ Active
- ✅ Parent/Children hierarchy
- ✅ NoSave marker

### Features Tested
- ✅ Full save/load cycles
- ✅ Hierarchy preservation (parent-child relationships)
- ✅ Deep hierarchies (3+ levels)
- ✅ Multiple children per parent
- ✅ NoSave entity exclusion
- ✅ Statistics tracking
- ✅ Metadata preservation (name, description, playtime, tags, custom data)
- ✅ Version migration (v0 → current, v1 → current)
- ✅ Configuration options (pretty print, validation)
- ✅ World clearing on load
- ✅ Nested directory creation
- ✅ Multiple save/load cycles
- ✅ Concurrent saves to different files
- ✅ Editor data preservation
- ✅ Runtime scene conversion
- ✅ Validation errors

## Running the Tests

### Run all tests
```bash
cargo test --package praxis_scene --test '*'
```

### Run specific test file
```bash
cargo test --package praxis_scene --test save_manager_tests
cargo test --package praxis_scene --test save_editor_data_tests
```

### Run specific test
```bash
cargo test --package praxis_scene --test save_manager_tests test_save_and_load_parent_child_hierarchy
```

### Run with output
```bash
cargo test --package praxis_scene --test save_manager_tests -- --nocapture
```

## Test Patterns

### Typical Test Structure
1. **Setup**: Create temp directory, world, manager
2. **Populate**: Add entities with components
3. **Save**: Write to temp file with metadata
4. **Verify Stats**: Check entity/component counts
5. **Load**: Read into new world
6. **Assert**: Verify all components and relationships preserved
7. **Cleanup**: Remove temp directory

### Helper Functions
- `temp_test_dir()`: Creates unique temporary directory
- `cleanup_test_dir(dir)`: Removes test directory and contents

## Dependencies

The tests require:
- `praxis_ecs`: For World and component types
- `praxis_scene`: For SaveManager and definitions
- `rand`: For generating unique temp directory names

## Notes

- All tests use temporary directories to avoid file conflicts
- Tests clean up after themselves (best effort)
- Some tests verify floating-point values with tolerance (0.001)
- Hierarchy tests verify both entity existence and relationship integrity
- Version migration tests create files manually to simulate old formats
- Editor data tests verify complete roundtrip serialization

## Future Enhancements

Potential areas for additional test coverage:
- Physics components (RigidBody, Collider, etc.) - when added to save system
- Audio components (AudioSource) - when added to save system
- Animation components (AnimationPlayer, Skeleton) - when added to save system
- Material properties - when added to save system
- Large-scale performance tests (1000+ entities)
- Corrupt file handling
- Disk space exhaustion scenarios
- Concurrent access patterns
