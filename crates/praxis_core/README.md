# Praxis Core

The core engine crate providing lifecycle management, initialization orchestration, and the main entry point for the Praxis game engine.

## Overview

`praxis_core` is the foundation of the Praxis engine, responsible for coordinating all subsystems during startup, managing the application lifecycle, and providing the main execution entry point. While lightweight in implementation, it serves as the critical integration point that brings together graphics, input, audio, ECS, and windowing systems.

## Table of Contents

- [Architecture](#architecture)
- [Engine Lifecycle](#engine-lifecycle)
- [Initialization Patterns](#initialization-patterns)
- [Main Loop Phases](#main-loop-phases)
- [Integration Examples](#integration-examples)
- [Resource Management](#resource-management)
- [Best Practices](#best-practices)

---

## Architecture

### Role in the Engine

`praxis_core` coordinates the initialization and execution of all engine subsystems:

```
┌─────────────────────────────────────────────────────────────┐
│                        praxis_core                           │
│              (Orchestration & Lifecycle)                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
       ┌───────────────┼───────────────┬───────────────┐
       │               │               │               │
       ▼               ▼               ▼               ▼
 ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
 │ praxis_  │   │ praxis_  │   │ praxis_  │   │ praxis_  │
 │  utils   │   │   ecs    │   │  input   │   │  audio   │
 └──────────┘   └──────────┘   └──────────┘   └──────────┘
       │               │               │               │
       └───────────────┴───────────────┴───────────────┘
                       │
                       ▼
              ┌─────────────────┐
              │  praxis_window  │
              │   (Event Loop)  │
              └─────────────────┘
```

### Dependencies

The core crate depends on:
- **`praxis_utils`**: Logging, error handling, timing utilities
- **`praxis_ecs`**: Entity-Component-System initialization
- **`praxis_input`**: Input system initialization
- **`praxis_audio`**: Audio system initialization
- **`praxis_window`**: Window management and event loop

---

## Engine Lifecycle

The Praxis engine follows a well-defined lifecycle with distinct phases:

### Lifecycle Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      STARTUP PHASE                           │
│                                                              │
│  1. praxis_utils::init()     → Tracing, logging, errors    │
│  2. praxis_ecs::init()       → ECS world setup              │
│  3. praxis_input::init()     → Input system setup           │
│  4. praxis_audio::init()     → Audio backend init           │
│                                                              │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   EVENT LOOP CREATION                        │
│                                                              │
│  • Create winit EventLoop                                   │
│  • Set control flow (Poll mode)                             │
│  • Prepare ApplicationHandler                               │
│                                                              │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    WINDOW CREATION                           │
│                                                              │
│  • ApplicationHandler::resumed()                            │
│  • Create window with attributes                            │
│  • Initialize RenderContext (Vulkan)                        │
│  • Configure surface for rendering                          │
│                                                              │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                     RUNTIME LOOP                             │
│                                                              │
│  ┌────────────────────────────────────────────────┐        │
│  │  1. Input Phase                                 │        │
│  │     • Process window events                     │        │
│  │     • Update InputState                         │        │
│  │     • Handle device events (mouse motion)       │        │
│  └────────────────────────────────────────────────┘        │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────┐        │
│  │  2. Update Phase                                │        │
│  │     • Calculate delta time                      │        │
│  │     • Run ECS systems                           │        │
│  │     • Update physics simulation                 │        │
│  │     • Process game logic                        │        │
│  │     • Update animations                         │        │
│  └────────────────────────────────────────────────┘        │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────┐        │
│  │  3. Render Phase                                │        │
│  │     • Update camera matrices                    │        │
│  │     • Query renderable entities                 │        │
│  │     • Build draw commands                       │        │
│  │     • Submit to RenderContext                   │        │
│  │     • Present frame                             │        │
│  └────────────────────────────────────────────────┘        │
│                       │                                      │
│                       ▼                                      │
│  ┌────────────────────────────────────────────────┐        │
│  │  4. Audio Phase                                 │        │
│  │     • Update listener position                  │        │
│  │     • Process spatial audio                     │        │
│  │     • Handle sound playback                     │        │
│  └────────────────────────────────────────────────┘        │
│                       │                                      │
│                       │                                      │
│  └───────────────────┘ (Loop continues)                     │
│                                                              │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    SHUTDOWN PHASE                            │
│                                                              │
│  • ApplicationHandler receives exit signal                  │
│  • Cleanup resources                                        │
│  • Drop contexts and managers                               │
│  • Exit gracefully                                          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Phase Timing

- **Startup**: One-time initialization (~50-200ms depending on hardware)
- **Runtime Loop**: Typically targets 60 FPS (16.67ms per frame)
- **Shutdown**: Immediate cleanup when exit is requested

---

## Initialization Patterns

### Basic Initialization

The simplest initialization pattern using `praxis_core::run()`:

```rust
use praxis_core;
use praxis_utils::Result;

fn main() -> Result<()> {
    // Initialize all engine subsystems and run the event loop
    praxis_core::run()?;
    
    Ok(())
}
```

This pattern:
1. Calls `praxis_utils::init()` for logging setup
2. Calls `praxis_ecs::init()` for ECS initialization
3. Calls `praxis_input::init()` for input system setup
4. Calls `praxis_audio::init()` for audio backend initialization
5. Starts the window event loop via `praxis_window::run()`

### Custom Initialization

For more control, initialize subsystems individually:

```rust
use praxis_utils::{info, Result};
use praxis_ecs::World;
use praxis_input::{InputState, InputMap, Action};
use praxis_audio::AudioManager;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::keyboard::KeyCode;
use std::sync::Arc;

fn main() -> Result<()> {
    // 1. Initialize core utilities (logging, error handling)
    praxis_utils::init()?;
    info!("Starting custom application");

    // 2. Initialize ECS
    praxis_ecs::init()?;
    let mut world = World::new();

    // 3. Initialize input system with custom bindings
    praxis_input::init()?;
    let mut input_map = InputMap::default();
    input_map.bind_key(&Action::new("forward"), KeyCode::KeyW);
    input_map.bind_key(&Action::new("backward"), KeyCode::KeyS);
    input_map.bind_key(&Action::new("left"), KeyCode::KeyA);
    input_map.bind_key(&Action::new("right"), KeyCode::KeyD);
    input_map.bind_key(&Action::new("jump"), KeyCode::Space);
    
    let input_state = InputState::default();
    world.insert_resource(input_state);
    world.insert_resource(input_map);

    // 4. Initialize audio system
    praxis_audio::init()?;
    let audio_manager = AudioManager::new()?;
    world.insert_resource(audio_manager);

    // 5. Create event loop
    let event_loop = EventLoop::new()
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create event loop: {}", e))?;
    event_loop.set_control_flow(ControlFlow::Poll);

    // 6. Create custom application handler
    // ... implement ApplicationHandler trait ...

    // 7. Run the event loop
    event_loop.run_app(&mut app)
        .map_err(|e| praxis_utils::eyre::eyre!("Event loop error: {}", e))?;

    Ok(())
}
```

### Resource Initialization Pattern

Standard pattern for initializing engine resources:

```rust
use praxis_ecs::World;
use praxis_graphics::RenderContext;
use praxis_audio::AudioManager;
use praxis_input::{InputState, InputMap};
use std::sync::Arc;
use winit::window::Window;

async fn initialize_resources(
    window: Arc<Window>,
    world: &mut World,
) -> praxis_utils::Result<RenderContext> {
    // Graphics context (requires async for Vulkan initialization)
    let render_context = RenderContext::new(window.clone()).await?;

    // Audio manager
    let audio_manager = AudioManager::new()?;
    world.insert_resource(audio_manager);

    // Input state
    let input_state = InputState::default();
    world.insert_resource(input_state);

    // Input mapping
    let input_map = InputMap::default();
    world.insert_resource(input_map);

    Ok(render_context)
}
```

---

## Main Loop Phases

### 1. Input Phase

Process user input and system events:

```rust
use praxis_input::{InputState, InputMap, Action};
use winit::event::WindowEvent;

fn process_input(
    input_state: &mut InputState,
    event: &WindowEvent,
) {
    // Update input state from winit events
    praxis_input::winit_integration::process_window_event(input_state, event);
    
    // Update internal state (clear "just pressed" flags, etc.)
    input_state.update();
}

fn handle_device_events(
    camera_controller: &mut CameraController,
    event: &DeviceEvent,
    cursor_locked: bool,
) {
    if cursor_locked {
        if let DeviceEvent::MouseMotion { delta } = event {
            camera_controller.update_rotation(delta.0 as f32, delta.1 as f32);
        }
    }
}
```

### 2. Update Phase

Update game state and run systems:

```rust
use praxis_ecs::{World, Schedule};
use std::time::Duration;

fn update_game_state(
    world: &mut World,
    schedule: &mut Schedule,
    delta_time: Duration,
) {
    // Store delta time as a resource
    world.insert_resource(delta_time);
    
    // Run all registered systems
    schedule.run(world.inner_mut());
    
    // Systems can include:
    // - Physics simulation
    // - Animation updates
    // - AI logic
    // - Script execution
    // - Transform propagation
}
```

**Common System Examples:**

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};
use praxis_ecs::systems::*;
use praxis_physics::physics_systems::*;
use praxis_scene::animation_systems::*;

fn create_game_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    
    schedule.add_systems((
        // Transform hierarchy systems
        sync_parent_child_relationships,
        cleanup_removed_parents,
        propagate_transforms,
        propagate_transforms_for_reparented,
        propagate_transforms_for_changed_children,
    ).chain());
    
    schedule.add_systems((
        // Physics systems
        sync_transforms_to_physics,
        step_physics_simulation,
        sync_physics_to_transforms,
        handle_physics_events,
    ).chain());
    
    schedule.add_systems((
        // Animation systems
        update_animation_players,
        apply_skeletal_animations,
        blend_animations,
    ).chain());
    
    schedule
}
```

### 3. Render Phase

Query entities and submit draw commands:

```rust
use praxis_graphics::{RenderContext, RenderCommands, DrawCommand};
use praxis_ecs::{World, Transform, MeshHandle, TextureHandle, CameraMatrices};

fn render_frame(
    world: &World,
    render_context: &mut RenderContext,
    camera_entity: praxis_ecs::Entity,
) -> praxis_utils::Result<()> {
    // Get camera matrices
    let camera_matrices = world
        .inner()
        .get::<CameraMatrices>(camera_entity)
        .expect("Camera entity missing CameraMatrices");

    // Query all renderable entities
    let mut draw_commands = Vec::new();
    let mut query = world.inner_mut().query::<(
        &Transform,
        &MeshHandle,
        Option<&TextureHandle>,
    )>();

    for (transform, mesh_handle, texture_handle) in query.iter(world.inner()) {
        draw_commands.push(DrawCommand {
            mesh_id: mesh_handle.id.clone(),
            model: transform.compute_matrix(),
            texture_name: texture_handle.map(|t| t.id.clone()),
            material_properties: None,
        });
    }

    // Submit to renderer
    let commands = RenderCommands {
        view: camera_matrices.view,
        proj: camera_matrices.projection,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&commands)?;
    
    Ok(())
}
```

### 4. Audio Phase

Update spatial audio and process playback:

```rust
use praxis_audio::{AudioManager, AudioSource, AudioListener, play_sound_system};
use praxis_ecs::{World, Schedule};

fn update_audio(
    world: &mut World,
    audio_schedule: &mut Schedule,
) {
    // Run audio system to update spatial audio
    audio_schedule.run(world.inner_mut());
    
    // System handles:
    // - Distance attenuation
    // - Panning based on position
    // - Doppler effect (if enabled)
    // - Sound playback triggering
}
```

### Frame Timing

Control frame rate and measure performance:

```rust
use praxis_utils::timing::FrameTimer;
use std::time::{Duration, Instant};

struct GameState {
    frame_timer: FrameTimer,
    last_frame_time: Instant,
}

impl GameState {
    fn update_timing(&mut self) -> Duration {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        // Track FPS and frame time
        self.frame_timer.tick();
        
        delta
    }
    
    fn get_fps(&self) -> f64 {
        self.frame_timer.fps()
    }
}
```

---

## Integration Examples

### Minimal Application

A minimal Praxis application using the default initialization:

```rust
use praxis_core;

fn main() -> praxis_utils::Result<()> {
    praxis_core::run()
}
```

### Scene Rendering Application

A complete example with scene rendering:

```rust
use praxis_utils::{info, Result};
use praxis_ecs::{World, Transform, PerspectiveCameraBundle};
use praxis_graphics::{RenderContext, DrawCommand, RenderCommands};
use praxis_math::Vec3;
use winit::application::ApplicationHandler;
use winit::event_loop::{EventLoop, ControlFlow};
use std::sync::Arc;

struct App {
    window: Option<Arc<winit::window::Window>>,
    world: Option<World>,
    render_context: Option<RenderContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Create window
        let window = Arc::new(
            event_loop.create_window(
                winit::window::Window::default_attributes()
                    .with_title("Praxis Application")
            ).expect("Failed to create window")
        );
        
        // Initialize graphics
        let mut render_context = pollster::block_on(
            RenderContext::new(window.clone())
        ).expect("Failed to initialize graphics");
        
        // Load assets
        render_context.mesh_manager_mut()
            .load_mesh("cube", praxis_graphics::colored_cube_mesh())
            .expect("Failed to load mesh");
        
        // Create world and spawn entities
        let mut world = World::new();
        
        world.spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            praxis_ecs::MeshHandle::new("cube"),
        ));
        
        let _camera = world.spawn(PerspectiveCameraBundle::new(
            Vec3::new(0.0, 2.0, 5.0),
            60.0_f32.to_radians(),
            16.0 / 9.0,
        ));
        
        self.window = Some(window);
        self.world = Some(world);
        self.render_context = Some(render_context);
    }
    
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                // Render frame
                // ... rendering logic ...
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = App {
        window: None,
        world: None,
        render_context: None,
    };
    
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
```

### Physics-Enabled Application

Integrate physics simulation:

```rust
use praxis_ecs::{World, Schedule, Transform};
use praxis_physics::{PhysicsWorld, PhysicsConfig, RigidBody, Collider};
use praxis_physics::physics_systems::*;

fn setup_physics_world(world: &mut World) -> Schedule {
    // Initialize physics world resource
    let physics_config = PhysicsConfig::default();
    let physics_world = PhysicsWorld::new(physics_config);
    world.insert_resource(physics_world);
    
    // Create schedule with physics systems
    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_transforms_to_physics,
        step_physics_simulation,
        sync_physics_to_transforms,
        handle_physics_events,
    ).chain());
    
    // Spawn entities with physics components
    world.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        RigidBody::dynamic(),
        Collider::sphere(0.5),
    ));
    
    world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::static_body(),
        Collider::cuboid(10.0, 0.5, 10.0),
    ));
    
    schedule
}
```

### Multi-Subsystem Integration

Complete integration with multiple subsystems:

```rust
use praxis_ecs::{World, Schedule, IntoSystemConfigs};
use praxis_audio::{AudioManager, play_sound_system};
use praxis_scene::animation_systems::*;
use praxis_physics::physics_systems::*;
use praxis_ecs::systems::*;

fn setup_complete_engine(world: &mut World) -> Schedule {
    // Initialize all resources
    let audio_manager = AudioManager::new().expect("Audio init failed");
    world.insert_resource(audio_manager);
    
    // Create comprehensive schedule
    let mut schedule = Schedule::default();
    
    // Transform hierarchy (must run first)
    schedule.add_systems((
        sync_parent_child_relationships,
        cleanup_removed_parents,
        propagate_transforms,
        propagate_transforms_for_reparented,
        propagate_transforms_for_changed_children,
    ).chain());
    
    // Physics simulation
    schedule.add_systems((
        sync_transforms_to_physics,
        step_physics_simulation,
        sync_physics_to_transforms,
        handle_physics_events,
    ).chain());
    
    // Animation
    schedule.add_systems((
        update_animation_players,
        apply_skeletal_animations,
        blend_animations,
    ).chain());
    
    // Audio (can run in parallel with other systems)
    schedule.add_systems(play_sound_system);
    
    schedule
}
```

---

## Resource Management

### Core Resources

Resources are singleton data stored in the ECS `World`:

```rust
use praxis_ecs::World;
use praxis_input::{InputState, InputMap};
use praxis_audio::AudioManager;
use praxis_physics::PhysicsWorld;
use std::time::Duration;

fn initialize_resources(world: &mut World) -> praxis_utils::Result<()> {
    // Input system resources
    world.insert_resource(InputState::default());
    world.insert_resource(InputMap::default());
    
    // Audio system resource
    let audio_manager = AudioManager::new()?;
    world.insert_resource(audio_manager);
    
    // Physics system resource
    let physics_world = PhysicsWorld::new(Default::default());
    world.insert_resource(physics_world);
    
    // Timing resource
    world.insert_resource(Duration::ZERO);
    
    Ok(())
}
```

### Resource Access

Access resources in systems or application code:

```rust
use praxis_ecs::World;
use praxis_input::{InputState, InputMap, Action};

fn check_input(world: &World) {
    let input_state = world.get_resource::<InputState>()
        .expect("InputState not found");
    let input_map = world.get_resource::<InputMap>()
        .expect("InputMap not found");
    
    if input_map.is_action_pressed(&Action::new("jump"), input_state) {
        // Handle jump action
    }
}

fn update_resource(world: &mut World) {
    let mut input_state = world.get_resource_mut::<InputState>()
        .expect("InputState not found");
    input_state.update();
}
```

### Resource Cleanup

Resources are automatically cleaned up when the `World` is dropped. For explicit cleanup:

```rust
use praxis_ecs::World;
use praxis_audio::AudioManager;

fn cleanup_resources(world: &mut World) {
    // Remove specific resources if needed
    world.remove_resource::<AudioManager>();
    
    // World drop will clean up remaining resources
}
```

---

## Best Practices

### Initialization Order

Always initialize subsystems in this order:

1. **Utils** (`praxis_utils::init()`) - Sets up logging first
2. **ECS** (`praxis_ecs::init()`) - Initializes the world
3. **Input** (`praxis_input::init()`) - Prepares input handling
4. **Audio** (`praxis_audio::init()`) - Starts audio backend
5. **Window/Graphics** - Created in event loop (requires async)

### Error Handling

Use `Result<T>` for all initialization and runtime operations:

```rust
use praxis_utils::Result;

fn initialize_game() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;
    
    // Additional setup with proper error propagation
    setup_scene()?;
    load_assets()?;
    
    Ok(())
}
```

### System Ordering

Maintain correct system execution order:

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};

// ✅ CORRECT: Systems run in dependency order
schedule.add_systems((
    input_system,        // 1. Process input
    game_logic_system,   // 2. Update game state
    physics_system,      // 3. Simulate physics
    animation_system,    // 4. Update animations
    transform_system,    // 5. Propagate transforms
).chain());

// ❌ INCORRECT: Random order may cause issues
schedule.add_systems((
    transform_system,
    input_system,
    animation_system,
    physics_system,
    game_logic_system,
).chain());
```

### Performance Considerations

**Frame Timing:**
```rust
use praxis_utils::timing::FrameTimer;

let mut timer = FrameTimer::new_with_global();

// In game loop
let delta = timer.tick();
if delta.as_secs_f64() > 0.033 {  // > 30 FPS
    praxis_utils::warn!("Frame took too long: {:.2}ms", delta.as_secs_f64() * 1000.0);
}
```

**Resource Pooling:**
```rust
// Reuse vectors to avoid allocations
let mut draw_commands = Vec::with_capacity(1000);

// In render loop
draw_commands.clear();  // Reuse allocation
for entity in query.iter() {
    draw_commands.push(/* ... */);
}
```

**Batch Operations:**
```rust
// ✅ GOOD: Query once, iterate efficiently
let mut query = world.query::<(&Transform, &Velocity)>();
for (transform, velocity) in query.iter(world) {
    // Process batch
}

// ❌ BAD: Individual entity lookups
for entity in entities {
    let transform = world.get::<Transform>(entity);
    let velocity = world.get::<Velocity>(entity);
}
```

### Logging Levels

Use appropriate logging levels:

```rust
use praxis_utils::{trace, debug, info, warn, error};

// TRACE: Very detailed, performance-sensitive info
trace!("Processing entity {:?}", entity);

// DEBUG: Useful for debugging, disabled in release
debug!("Updated transform: {:?}", transform);

// INFO: General information about program flow
info!("Loaded {} assets", count);

// WARN: Unexpected but recoverable situations
warn!("Asset not found, using fallback");

// ERROR: Serious problems that need attention
error!("Failed to initialize renderer: {}", e);
```

---

## Examples

The `examples/` directory contains comprehensive demonstrations:

### Basic Examples

```bash
# Simple scene rendering
cargo run --example scene_demo

# Input handling demonstration
cargo run --example input_integration

# ECS usage patterns
cargo run --example ecs_integration
```

### Advanced Examples

```bash
# Complete scene with multiple systems
cargo run --example comprehensive_scene_demo

# GUI integration
cargo run --example gui_demo

# Editor tools
cargo run --example editor_demo

# Physics simulation
cargo run --example physics_demo

# Animation system
cargo run --example animation_demo
```

### Performance Examples

```bash
# Profiling and optimization
cargo run --example profiling_demo

# Spatial partitioning
cargo run --example spatial_optimization_demo

# Level of detail
cargo run --example lod_demo
```

---

## See Also

### Documentation

- [Main Documentation](../../docs/README.md) - Complete engine documentation
- [Architecture Guide](../../docs/architecture.md) - Engine design and structure
- [Beginner's Guide](../../docs/beginners-guide.md) - Getting started tutorial
- [Getting Started](../../docs/getting-started/README.md) - Installation and setup

### Related Crates

- [praxis_window](../praxis_window/README.md) - Window management and event loop
- [praxis_ecs](../praxis_ecs/README.md) - Entity-Component-System
- [praxis_graphics](../praxis_graphics/README.md) - Rendering system
- [praxis_input](../praxis_input/README.md) - Input handling
- [praxis_audio](../praxis_audio/README.md) - Audio system
- [praxis_utils](../praxis_utils/README.md) - Utilities and logging

### Community

- [GitHub Repository](https://github.com/your-org/praxis)
- [Issue Tracker](https://github.com/your-org/praxis/issues)
- [Discussions](https://github.com/your-org/praxis/discussions)

---

## License

See the main [LICENSE](../../LICENSE) file in the repository root.
