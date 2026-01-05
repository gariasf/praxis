# Feature Flags

Praxis uses Cargo feature flags to make optional systems available. This keeps the default build lean while allowing you to opt into additional functionality as needed.

## Overview

By default, Praxis includes only the [core features](core-features.md): rendering, ECS, physics, audio, and more. Optional systems are gated behind feature flags to:

- **Reduce compile times** when you don't need certain features
- **Minimize binary size** for production builds
- **Avoid unnecessary dependencies** for simpler projects
- **Allow modular development** of game vs. editor tools

## Available Feature Flags

### `editor`

**Development tools and editor interface**

Enables the `praxis_editor` crate with comprehensive tooling:

- **Selection System**: Multi-entity selection with raycast picking
- **Undo/Redo**: Full command history with serialization
- **Gizmos**: Transform manipulation widgets (translate, rotate, scale)
- **Editor Camera**: Orbit controller with focus and framing
- **Hierarchy Panel**: Entity tree view with drag-and-drop reparenting
- **Inspector Panel**: Component property editing
- **Asset Browser**: Drag-and-drop asset management
- **Menu Bar**: File, edit, view menus with keyboard shortcuts
- **Console**: Debug command execution

**When to use**: Development and content creation workflows

**Enable with**:
```toml
# In your Cargo.toml
[dependencies]
praxis = { path = "../praxis", features = ["editor"] }
```

**Or build directly**:
```bash
cargo build --features editor
cargo run --features editor --example editor_demo
```

**Examples using this feature**:
- `editor_demo` - Full editor interface
- `editor_camera_demo` - Editor camera controls
- `selection_demo` - Entity selection system
- `undo_redo_system_demo` - Command history
- `command_system_demo` - Command pattern implementation
- `console_demo` - Debug console

**Documentation**: [Editor README](../editor/README.md)

---

### `scripting`

**Lua scripting integration**

Enables the `praxis_scripting` crate for runtime scripting:

- **Lua 5.4**: Full Lua integration via `mlua`
- **ECS Access**: Query and modify entities/components from scripts
- **Hot Reload**: Automatic script reloading on file changes
- **Sandboxing**: Configurable security levels (None/Moderate/Strict)
- **Performance Monitoring**: Track script execution times
- **Event System**: Script callbacks for game events
- **Component Bindings**: Expose custom components to Lua

**When to use**: Gameplay logic, modding support, rapid prototyping

**Enable with**:
```toml
[dependencies]
praxis = { path = "../praxis", features = ["scripting"] }
```

**Or build directly**:
```bash
cargo build --features scripting
cargo run --features scripting --example scripting_demo
```

**Examples using this feature**:
- `scripting_demo` - Basic Lua integration
- `scripting_advanced_demo` - Complex scripting patterns

**Documentation**: [Scripting Guide](../guides/scripting.md), [praxis_scripting README](../../crates/praxis_scripting/README.md)

---

### `networking`

**Multiplayer and networking**

Enables the `praxis_networking` crate for networked games:

- **Client-Server**: TCP/UDP transport with connection management
- **Entity Replication**: Automatic component synchronization
- **Interpolation/Extrapolation**: Smooth remote entity movement
- **Lag Compensation**: Server-side rewind for hit detection
- **Network Profiler**: Bandwidth and latency monitoring
- **Authority Model**: Server-authoritative gameplay
- **Delta Compression**: Efficient bandwidth usage

**When to use**: Multiplayer games, networked simulations

**Enable with**:
```toml
[dependencies]
praxis = { path = "../praxis", features = ["networking"] }
```

**Or build directly**:
```bash
cargo build --features networking
cargo run --features networking --example networking_demo
```

**Examples using this feature**:
- `networking_demo` - Client-server setup and entity replication

**Documentation**: [praxis_networking README](../../crates/praxis_networking/README.md)

---

### `terrain`

**Heightmap-based terrain rendering**

Enables the `praxis_terrain` crate for large-scale landscapes:

- **Heightmap Generation**: Procedural terrain via noise functions
- **LOD System**: Automatic level-of-detail based on distance
- **Texture Splatting**: Blend multiple terrain materials
- **Normal Mapping**: Per-pixel lighting for terrain detail
- **Chunking**: Efficient large-world streaming
- **Editor Integration**: Terrain editing tools (requires `editor` feature)

**When to use**: Open-world games, outdoor environments

**Enable with**:
```toml
[dependencies]
praxis = { path = "../praxis", features = ["terrain"] }
```

**For terrain with editor support**:
```toml
[dependencies]
praxis = { path = "../praxis", features = ["terrain", "editor"] }
```

**Or build directly**:
```bash
cargo build --features terrain
cargo run --features terrain --example terrain_demo
```

**Examples using this feature**:
- `terrain_demo` - Heightmap rendering with LOD

**Documentation**: [Terrain System](../terrain-system.md)

---

## Combining Features

You can enable multiple features together:

```toml
[dependencies]
praxis = { path = "../praxis", features = ["editor", "scripting"] }
```

```bash
cargo build --features "editor,scripting,networking"
cargo run --features "editor,terrain" --example editor_demo
```

## Common Combinations

### Game Development
```toml
features = ["editor", "scripting"]
```
Editor tools for content creation + scripting for gameplay logic.

### Multiplayer Game
```toml
features = ["networking", "scripting"]
```
Networked gameplay with scriptable game logic.

### Open World Editor
```toml
features = ["editor", "terrain"]
```
Full editor with terrain editing capabilities.

### Full Feature Set
```toml
features = ["editor", "scripting", "networking", "terrain"]
```
Everything enabled for maximum flexibility (slower compile times).

## Checking Active Features

In your code, you can conditionally compile based on features:

```rust
#[cfg(feature = "editor")]
use praxis_editor::{SelectionSystem, UndoRedoSystem};

#[cfg(feature = "scripting")]
use praxis_scripting::ScriptingContext;

fn setup(world: &mut World) {
    #[cfg(feature = "editor")]
    world.insert_resource(SelectionSystem::new());
    
    #[cfg(feature = "scripting")]
    world.insert_resource(ScriptingContext::new(Default::default()).unwrap());
}
```

## Feature Flag Dependencies

Some features have internal dependencies:

- **`terrain`** with **`editor`**: Enables terrain editing tools in the editor
  - Use: `features = ["terrain", "editor"]`
  - The terrain editor extensions are automatically enabled

## Default Features

Currently, **no features are enabled by default**. This means:

```bash
cargo build
```

Builds only the [core features](core-features.md) without editor, scripting, networking, or terrain.

To get optional features, you must explicitly enable them.

## Build Times and Binary Size

Enabling features increases compile time and binary size:

| Configuration | Approx. Compile Time | Binary Size Impact |
|---------------|---------------------|-------------------|
| Default (core only) | Baseline | Baseline |
| `+ editor` | +20-30% | +15-20% |
| `+ scripting` | +10-15% | +8-12% |
| `+ networking` | +10-15% | +8-10% |
| `+ terrain` | +5-10% | +5-8% |
| All features | +40-50% | +30-40% |

*Times and sizes are approximate and vary by platform.*

## Recommendations

- **Development**: Use `editor` for visual debugging and content creation
- **Prototyping**: Add `scripting` for fast gameplay iteration
- **Production**: Only enable features you actually use in the final game
- **Modding**: Expose `scripting` for user-created content
- **CI/CD**: Test builds with different feature combinations to ensure modularity

## Examples by Feature

Run examples to explore each feature flag:

```bash
# Core features (no flags needed)
cargo run --example comprehensive_scene_demo
cargo run --example animation_blending_demo
cargo run --example audio_demo

# Editor feature
cargo run --features editor --example editor_demo
cargo run --features editor --example selection_demo

# Scripting feature
cargo run --features scripting --example scripting_demo
cargo run --features scripting --example scripting_advanced_demo

# Networking feature
cargo run --features networking --example networking_demo

# Terrain feature
cargo run --features terrain --example terrain_demo
```

## Troubleshooting

### Feature Not Found
```
error: feature `xyz` is not defined
```
Check spelling and ensure the feature exists in the root `Cargo.toml`.

### Missing Types
```
error: cannot find type `SelectionSystem` in crate `praxis_editor`
```
You're trying to use a type from an optional crate. Either:
1. Enable the feature: `--features editor`
2. Guard usage with `#[cfg(feature = "editor")]`

### Example Won't Run
```
error: target `example_name` requires `--features scripting`
```
Some examples require specific features. Check the example's `required-features` in `Cargo.toml` and enable them:
```bash
cargo run --features scripting --example scripting_demo
```

## See Also

- [Core Features](core-features.md) - What's included by default
- [Project Structure](project-structure.md) - Crate organization
- [Installation](installation.md) - Setup and requirements
