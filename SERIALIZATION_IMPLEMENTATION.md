# Serialization Implementation for Physics and Audio Components

## Summary

This implementation adds serialization support for `RigidBody`, `Collider`, and `AudioSource` components, enabling them to be saved and loaded through the Praxis ECS serialization system.

## Changes Made

### 1. `praxis_physics` Crate

#### `Cargo.toml`
- Added `serde` and `ron` as optional dependencies
- Created `serialization` feature (enabled by default)

#### `components.rs`
- Added `#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]` to:
  - `RigidBody` enum
  - `Collider` enum
- Implemented `SerializableComponent` trait for both types in a feature-gated module
- Both implementations use standard RON serialization/deserialization

### 2. `praxis_audio` Crate

#### `Cargo.toml`
- Added `serde` and `ron` as optional dependencies
- Created `serialization` feature (enabled by default)

#### `components.rs`
- Added `#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]` to `AudioState` enum
- Created custom serialization for `AudioSource` that:
  - Excludes internal runtime fields (`sound_handle`, `previous_position`)
  - Serializes all user-configurable fields (path, volume, spatial settings, etc.)
  - Resets internal fields to `None` on deserialization
- Fixed `with_volume` to be a non-const function (due to `clamp` usage)

### 3. `praxis_ecs` Crate

#### `Cargo.toml`
- Added `praxis_physics` and `praxis_audio` as dev-dependencies
- Created `test_physics` and `test_audio` features for optional test compilation

#### `serialization.rs`
- Added comprehensive roundtrip tests for:
  - `RigidBody` (Dynamic, Static, Kinematic variants)
  - `Collider` (Cuboid, Sphere, CapsuleY variants)
  - `AudioSource` (all configurable fields)
  - Combined physics and audio components on the same entity

## Test Coverage

All tests verify roundtrip serialization:
1. Create component with specific values
2. Serialize to RON string
3. Deserialize into new world
4. Verify all values match original

Tests are feature-gated with `#[cfg(all(feature = "test_physics", feature = "test_audio"))]` to avoid circular dependencies in normal builds.

## Usage Example

```rust
use praxis_ecs::{World, ComponentRegistry, Name, Transform};
use praxis_physics::{RigidBody, Collider};
use praxis_audio::AudioSource;

let mut world = World::new();

// Spawn entity with all component types
world.spawn((
    Name::new("Player"),
    Transform::from_xyz(0.0, 5.0, 0.0),
    RigidBody::Dynamic,
    Collider::capsule_y(1.0, 0.5),
    AudioSource::new("footsteps.ogg").with_volume(0.7),
));

// Register all types
let mut registry = ComponentRegistry::new();
registry.register::<Name>();
registry.register::<Transform>();
registry.register::<RigidBody>();
registry.register::<Collider>();
registry.register::<AudioSource>();

// Serialize
let ron_string = registry.serialize_world(&world)?;

// Deserialize
let mut new_world = World::new();
registry.deserialize_world(&ron_string, &mut new_world)?;
```

## Technical Details

### RigidBody Serialization
- Simple enum with three variants: Dynamic, Static, Kinematic
- Uses standard RON enum serialization

### Collider Serialization
- Enum with struct variants containing field data
- Each variant serializes its shape parameters (half-extents, radius, etc.)
- All primitive types (f32) serialize directly

### AudioSource Serialization
- Custom implementation to handle internal state
- Serializes: path, volume, spatial settings, looping, distances, doppler settings
- Excludes: sound_handle (runtime Kira handle), previous_position (internal tracking)
- On deserialization, internal fields reset to None for proper initialization

## Design Decisions

1. **Feature-gated**: Serialization is optional via cargo features, avoiding bloat for users who don't need it
2. **Default-enabled**: The serialization feature is enabled by default for convenience
3. **Trait-based**: Uses the `SerializableComponent` trait for consistent API
4. **RON format**: Uses Rust Object Notation for human-readable serialization
5. **Runtime-safe**: Internal runtime state is properly excluded from serialization

## Running Tests

To run the serialization tests:

```bash
# Run all ECS tests including serialization roundtrip tests
cargo test --package praxis_ecs --features test_physics,test_audio

# Run only the serialization tests
cargo test --package praxis_ecs serialization --features test_physics,test_audio
```
