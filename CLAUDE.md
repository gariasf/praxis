# CLAUDE.md

Guidance for Claude Code when working with this repository.

## Project Overview

Praxis is a 3D game engine in Rust using `vulkano` (Vulkan), `bevy_ecs`, and `rapier3d`.

**Documentation**: See `docs/README.md` for comprehensive guides, concepts, and reference.

## Essential Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build
cargo build -p praxis_editor   # Specific crate

# Test & Quality
cargo test --workspace         # All tests
cargo fmt --all                # Format code
cargo clippy --all -- -D warnings  # Lint (must pass)

# Run Examples
cargo run --example deferred_demo
cargo run --example hdr_demo
cargo run --example physics_demo
cargo run --example editor_demo

# Documentation
cargo doc --workspace --no-deps --open
```

## Architecture

12-crate workspace organized by subsystem:

| Crate | Purpose |
|-------|---------|
| `praxis_core` | Engine lifecycle, main loop |
| `praxis_window` | Window management (winit) |
| `praxis_graphics` | Vulkan rendering |
| `praxis_ecs` | ECS (bevy_ecs) |
| `praxis_math` | Math (glam) |
| `praxis_scene` | Transform hierarchy, animation |
| `praxis_assets` | Asset loading (OBJ, GLTF) |
| `praxis_input` | Keyboard, mouse, gamepad |
| `praxis_gui` | Editor GUI (egui) |
| `praxis_physics` | Physics (Rapier3D) |
| `praxis_audio` | Audio (Kira) |
| `praxis_editor` | Editor tools |
| `praxis_utils` | Logging, errors, timing |

**Details**: `docs/reference/crates.md`, `docs/ARCHITECTURE.md`

## Key Patterns

### ECS
```rust
#[derive(Component)]
struct Health(f32);

fn damage_system(mut query: Query<&mut Health>) {
    for mut hp in query.iter_mut() {
        hp.0 -= 1.0;
    }
}
```

### Transform Hierarchy
- `Transform`: Local position, rotation, scale
- `GlobalTransform`: Computed world-space
- `Parent`/`Children`: Hierarchy relationships

### Rendering
- Forward rendering: `RenderContext::render()`
- Deferred rendering: `DeferredRenderer`
- HDR: `ToneMapper` with ACES/Reinhard
- Shadows: Cascaded shadow maps

**Details**: `docs/guides/rendering.md`, `docs/guides/hdr-and-tonemapping.md`

### Physics
- `RigidBody`: Dynamic, Static, Kinematic
- `Collider`: Boxes, spheres, capsules
- Fixed timestep (60 Hz default)
- Bidirectional transform sync with ECS

**Details**: `crates/praxis_physics/README.md`

### Animation
- `Skeleton`, `AnimationClip`, `AnimationPlayer`
- Blend trees, layered animation, cross-fading

**Details**: `docs/animation_system.md`

### Editor
- Selection: `SelectionSystem`, `Selectable` component
- Undo/Redo: `UndoRedoSystem`, command history
- Gizmos: Transform manipulation
- Camera: Orbit controller

**Details**: `docs/editor/README.md`

## Code Quality

### Linting (must pass CI)
```toml
clippy::all = "warn"
clippy::pedantic = "warn"
clippy::nursery = "warn"
unsafe_code = "warn"
missing_docs = "warn"
```

All public items require rustdoc comments.

### CI Checks
1. `cargo check --all`
2. `cargo fmt --all -- --check`
3. `cargo clippy --all -- -D warnings`

## Common Tasks

### Adding Components
1. Define in `crates/praxis_ecs/src/components.rs`
2. Derive `Component`
3. Add rustdoc
4. Export from `lib.rs`

### Graphics Changes
- Shaders: `crates/praxis_graphics/src/shaders/`
- Compiled via `vulkano-shaders` macro
- `RenderContext` manages Vulkan resources

### Physics
- Components: `RigidBody`, `Collider`, `PhysicsVelocity`
- Resources: `PhysicsWorld`, `PhysicsConfig`
- System order matters: sync → step → sync → events

## Dependencies

| Category | Crates |
|----------|--------|
| Graphics | vulkano, vulkano-shaders |
| Window | winit 0.30 |
| Math | glam |
| ECS | bevy_ecs |
| Physics | rapier3d |
| Audio | kira |
| GUI | egui, egui_vulkano |
| Logging | tracing |
| Errors | color-eyre |

## Documentation Structure

```
docs/
├── README.md              # Main index
├── BEGINNERS_GUIDE.md     # Learning resource
├── getting-started/       # Installation, setup
├── guides/                # How-to guides
├── concepts/              # Theory explanations
├── reference/             # API reference
└── editor/                # Editor docs
```

## Philosophy

- Use battle-proven libraries
- Prioritize clarity over abstraction
- Pragmatic, iterative development
- Leverage Rust's safety guarantees
