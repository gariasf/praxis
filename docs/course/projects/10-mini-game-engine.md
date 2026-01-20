# Project 10: Mini Game Engine

**Difficulty**: Advanced  
**Estimated Time**: 6-8 weeks  
**Core Learning**: Engine architecture, subsystem integration, plugin system, complete game development pipeline

## Overview

Build a complete mini game engine from scratch, integrating all concepts from previous projects. This capstone project teaches engine architecture design, subsystem coordination, plugin systems, and the complete workflow from asset import to game deployment.

### Learning Objectives

- Design modular engine architecture
- Integrate multiple subsystems (rendering, physics, audio, etc.)
- Build asset pipeline (import, process, manage)
- Implement plugin/scripting system
- Create editor-runtime separation
- Develop complete game with your engine
- Optimize and profile engine performance

## Feature Requirements

### Core Features (Minimum Viable)

1. **Engine Core**
   - Application lifecycle (init, update, shutdown)
   - Main loop with delta time
   - Event system (input, window, custom events)
   - Resource manager (load, cache, unload)
   - Configuration system (settings file)

2. **Rendering Subsystem**
   - 3D mesh rendering
   - Basic lighting (directional, point)
   - Camera system
   - Material system
   - Skybox or solid background

3. **Scene Management**
   - Entity-component system (ECS) or scene graph
   - Transform hierarchy
   - Scene serialization (save/load)
   - Scene switching

4. **Asset Pipeline**
   - Load models (OBJ, GLTF)
   - Load textures (PNG, JPG)
   - Load audio (WAV, OGG, MP3)
   - Asset metadata (import settings)
   - Asset hot-reloading (optional)

5. **Input System**
   - Keyboard, mouse, gamepad support
   - Input mapping (rebindable controls)
   - Input events
   - Action system (high-level inputs)

### Extended Features (Recommended)

6. **Physics Integration**
   - Rigid body simulation
   - Collision detection and response
   - Character controller
   - Physics materials

7. **Audio System**
   - Background music playback
   - Sound effects (3D spatial audio)
   - Audio mixer (volume control, groups)
   - Audio events

8. **Animation System**
   - Skeletal animation playback
   - Animation blending
   - Animation state machine
   - Animation events

9. **Scripting/Gameplay**
   - Scripting language integration (Lua, Python, etc.)
   - Component scripting (attach scripts to entities)
   - Hot-reload scripts
   - API for engine systems

10. **Basic Editor**
    - Scene editor (from Project 08)
    - Transform gizmos
    - Property inspector
    - Asset browser
    - Play mode (test game in editor)

### Stretch Goals

11. **Advanced Rendering**
    - Deferred rendering
    - Shadow mapping
    - Post-processing (bloom, SSAO, etc.)
    - Particle systems

12. **Advanced Systems**
    - UI system (HUD, menus)
    - Terrain system
    - Networking (multiplayer support)
    - Profiling tools (built-in performance metrics)

13. **Build Pipeline**
    - Export standalone game
    - Multiple platform support
    - Asset bundling
    - Optimization passes

## Architecture Guidance

### High-Level Architecture

```
MiniGameEngine
├── Core
│   ├── Engine (lifecycle, main loop)
│   ├── EventBus (pub-sub system)
│   ├── ModuleRegistry (subsystems)
│   └── Configuration
├── Platform
│   ├── Window (abstraction over OS windowing)
│   ├── FileSystem (path resolution, I/O)
│   └── Time (high-precision timing)
├── Subsystems
│   ├── Renderer
│   │   ├── RenderContext
│   │   ├── MaterialSystem
│   │   └── ShaderManager
│   ├── Physics
│   │   ├── PhysicsWorld
│   │   └── CollisionResolver
│   ├── Audio
│   │   ├── AudioEngine
│   │   └── SoundManager
│   ├── Input
│   │   ├── InputManager
│   │   └── InputMapper
│   ├── Scene
│   │   ├── World (ECS or scene graph)
│   │   ├── EntityFactory
│   │   └── ComponentRegistry
│   └── Scripting
│       ├── ScriptEngine
│       └── ScriptBindings
├── Assets
│   ├── AssetManager
│   ├── AssetImporters (mesh, texture, audio, etc.)
│   └── AssetCache
├── Editor (optional)
│   ├── EditorCore
│   ├── Viewport
│   ├── Inspector
│   └── AssetBrowser
└── Game (your game code using the engine)
    ├── GameSystems
    ├── GameComponents
    └── GameScripts
```

### Core Engine Loop

```
struct Engine {
  modules: Vec<Box<dyn Module>>,
  event_bus: EventBus,
  is_running: bool,
  target_fps: u32,
}

trait Module {
  fn init(&mut self, engine: &Engine);
  fn update(&mut self, delta_time: f32);
  fn shutdown(&mut self);
}

impl Engine {
  fn run(&mut self) {
    self.init();
    
    while self.is_running {
      let delta_time = self.calculate_delta_time();
      
      self.process_events();
      self.update(delta_time);
      self.render();
      
      self.limit_frame_rate();
    }
    
    self.shutdown();
  }
  
  fn init(&mut self) {
    for module in &mut self.modules {
      module.init(self);
    }
    self.event_bus.emit(Event::EngineInitialized);
  }
  
  fn update(&mut self, delta_time: f32) {
    self.event_bus.emit(Event::BeginFrame { delta_time });
    
    for module in &mut self.modules {
      module.update(delta_time);
    }
    
    self.event_bus.emit(Event::EndFrame);
  }
  
  fn shutdown(&mut self) {
    for module in self.modules.iter_mut().rev() {
      module.shutdown();
    }
  }
}
```

### Event System

```
enum Event {
  # Engine lifecycle
  EngineInitialized,
  BeginFrame { delta_time: f32 },
  EndFrame,
  
  # Input
  KeyPressed { key: KeyCode },
  KeyReleased { key: KeyCode },
  MouseMoved { x: f32, y: f32 },
  MouseButton { button: MouseButton, pressed: bool },
  
  # Window
  WindowResized { width: u32, height: u32 },
  WindowClosed,
  
  # Scene
  SceneLoaded { scene_id: Uuid },
  EntitySpawned { entity_id: EntityId },
  EntityDestroyed { entity_id: EntityId },
  
  # Custom
  Custom { type_id: String, data: Value },
}

struct EventBus {
  listeners: HashMap<EventType, Vec<Box<dyn Fn(&Event)>>>,
}

impl EventBus {
  fn subscribe<F>(&mut self, event_type: EventType, callback: F)
  where F: Fn(&Event) + 'static {
    self.listeners.entry(event_type)
      .or_insert(Vec::new())
      .push(Box::new(callback));
  }
  
  fn emit(&mut self, event: Event) {
    let event_type = event.get_type();
    if let Some(listeners) = self.listeners.get(&event_type) {
      for listener in listeners {
        listener(&event);
      }
    }
  }
}
```

### Asset Management

```
struct AssetManager {
  cache: HashMap<AssetId, Arc<dyn Asset>>,
  importers: HashMap<String, Box<dyn AssetImporter>>,
  loader_threads: ThreadPool,
}

trait Asset: Send + Sync {
  fn get_type(&self) -> &str;
  fn get_id(&self) -> AssetId;
}

trait AssetImporter {
  fn supported_extensions(&self) -> Vec<&str>;
  fn import(&self, path: &Path) -> Result<Box<dyn Asset>>;
}

impl AssetManager {
  fn load<T: Asset>(&mut self, path: &str) -> Result<AssetHandle<T>> {
    # Check cache
    let asset_id = AssetId::from_path(path);
    if let Some(cached) = self.cache.get(&asset_id) {
      return Ok(AssetHandle::new(asset_id, cached.clone()));
    }
    
    # Determine importer from extension
    let extension = get_extension(path);
    let importer = self.importers.get(extension)
      .ok_or("No importer for extension")?;
    
    # Import asset
    let asset = importer.import(Path::new(path))?;
    
    # Cache and return
    let asset_arc = Arc::new(asset);
    self.cache.insert(asset_id, asset_arc.clone());
    Ok(AssetHandle::new(asset_id, asset_arc))
  }
  
  fn unload(&mut self, asset_id: AssetId) {
    self.cache.remove(&asset_id);
  }
  
  fn reload(&mut self, asset_id: AssetId) -> Result<()> {
    if let Some(path) = self.get_path(asset_id) {
      self.unload(asset_id);
      self.load(&path)?;
    }
    Ok(())
  }
}
```

### Component System (ECS Pattern)

```
# Entity is just an ID
type Entity = u64;

# Components are data
struct Transform {
  position: Vec3,
  rotation: Quat,
  scale: Vec3,
}

struct MeshRenderer {
  mesh: AssetHandle<Mesh>,
  material: AssetHandle<Material>,
}

struct RigidBody {
  mass: f32,
  velocity: Vec3,
  is_kinematic: bool,
}

# World stores entities and components
struct World {
  entities: Vec<Entity>,
  transforms: HashMap<Entity, Transform>,
  mesh_renderers: HashMap<Entity, MeshRenderer>,
  rigid_bodies: HashMap<Entity, RigidBody>,
  # ... more component types
}

# Systems operate on components
trait System {
  fn update(&mut self, world: &mut World, delta_time: f32);
}

struct PhysicsSystem;
impl System for PhysicsSystem {
  fn update(&mut self, world: &mut World, delta_time: f32) {
    # Query entities with both Transform and RigidBody
    for entity in world.entities.iter() {
      if let (Some(transform), Some(body)) = 
        (world.transforms.get_mut(entity), world.rigid_bodies.get(entity)) {
        
        # Apply physics
        transform.position += body.velocity * delta_time;
        body.velocity += GRAVITY * delta_time;
      }
    }
  }
}
```

### Scripting Integration Example (Lua)

```
# Lua script: player_controller.lua
function on_update(entity, delta_time)
  local input = get_input()
  local transform = get_component(entity, "Transform")
  
  local move_speed = 5.0
  local movement = vec3(0, 0, 0)
  
  if input:is_key_down("W") then
    movement.z = movement.z - 1
  end
  if input:is_key_down("S") then
    movement.z = movement.z + 1
  end
  
  transform.position = transform.position + movement * move_speed * delta_time
  set_component(entity, "Transform", transform)
end

# Engine bindings (Rust side)
impl ScriptEngine {
  fn register_bindings(&mut self) {
    # Register functions callable from Lua
    self.lua.globals().set("get_input", self.lua.create_function(|lua, ()| {
      let input = lua.globals().get::<_, InputManager>("__input_manager")?;
      Ok(input)
    }))?;
    
    self.lua.globals().set("get_component", self.lua.create_function(
      |lua, (entity, component_type): (Entity, String)| {
        # Fetch component from ECS world
        let world = lua.globals().get::<_, &World>("__world")?;
        match component_type.as_str() {
          "Transform" => {
            let transform = world.transforms.get(&entity)?;
            Ok(transform.clone())
          }
          _ => Err("Unknown component type")
        }
      }
    ))?;
    
    # ... more bindings
  }
}
```

## Milestone Plan

### Phase 1: Engine Foundation (Weeks 1-2)

**Milestone 1.1: Core Engine Loop**
- Set up project structure
- Implement Engine struct with lifecycle
- Main loop with delta time calculation
- Module/subsystem trait
- Basic configuration system
- Window creation (using SDL, GLFW, or winit)

**Milestone 1.2: Event System**
- Implement event bus (pub-sub)
- Window events (close, resize)
- Input events (keyboard, mouse)
- Subscribe/emit pattern
- Event queue (if needed for threading)

**Milestone 1.3: Resource Management**
- AssetManager skeleton
- Asset handle/reference counting
- Basic asset loading (textures, meshes)
- Asset ID system (paths or UUIDs)
- Cache management

### Phase 2: Rendering Subsystem (Weeks 2-3)

**Milestone 2.1: Renderer Integration**
- Choose graphics API (Vulkan, OpenGL, etc.)
- Render context initialization
- Clear color, basic rendering
- Camera system
- Shader compilation

**Milestone 2.2: Mesh Rendering**
- Mesh asset importer (OBJ or GLTF)
- Vertex/index buffer management
- Material system (textures, colors)
- Draw mesh with transforms
- Basic lighting (directional light)

**Milestone 2.3: Scene Rendering**
- Render multiple objects
- Frustum culling
- Depth testing
- Transparency sorting (if needed)

### Phase 3: Scene & ECS (Weeks 3-4)

**Milestone 3.1: Entity-Component System**
- Entity creation/destruction
- Component registration
- Component storage (HashMap or ECS library)
- Query system (iterate entities with components)

**Milestone 3.2: Core Components**
- Transform component (position, rotation, scale)
- MeshRenderer component
- Camera component
- Hierarchy (parent-child relationships)

**Milestone 3.3: Scene Management**
- Scene struct (collection of entities)
- Scene serialization (JSON or binary)
- Scene loading/unloading
- Scene switching

### Phase 4: Physics & Input (Weeks 4-5)

**Milestone 4.1: Physics Integration**
- Integrate physics library (Rapier, Bullet, etc.)
- RigidBody component
- Collider component
- Physics step in main loop
- Sync transforms with physics

**Milestone 4.2: Input System**
- InputManager module
- Keyboard/mouse state tracking
- Input mapping (actions/axes)
- Input polling API
- Gamepad support (optional)

### Phase 5: Audio & Animation (Week 5)

**Milestone 5.1: Audio System**
- Integrate audio library (Kira, rodio, etc.)
- Background music playback
- Sound effect playback
- 3D spatial audio (optional)
- Volume control

**Milestone 5.2: Animation System** (optional)
- Skeletal animation playback
- AnimationPlayer component
- Animation blending basics
- Or skip if time-constrained

### Phase 6: Scripting (Week 6)

**Milestone 6.1: Script Integration**
- Integrate scripting language (Lua via mlua, rhai, etc.)
- ScriptComponent (attaches script to entity)
- Lifecycle hooks (on_init, on_update, on_destroy)
- Hot-reload scripts

**Milestone 6.2: Script Bindings**
- Expose input system to scripts
- Expose entity/component API
- Expose math utilities (vectors, etc.)
- Example gameplay scripts

### Phase 7: Editor (Weeks 6-7)

**Milestone 7.1: Basic Editor** (reuse Project 08)
- Editor application separate from runtime
- Scene viewport
- Object spawning
- Transform gizmos
- Play mode (run game in editor)

**Milestone 7.2: Property Inspector**
- Display selected entity components
- Edit component properties
- Add/remove components
- Material editor

**Milestone 7.3: Asset Browser**
- Display available assets
- Drag-drop to scene
- Asset import settings
- Thumbnail previews (optional)

### Phase 8: Demo Game & Polish (Weeks 7-8)

**Milestone 8.1: Create Demo Game**
- Design simple game (e.g., collect coins, avoid enemies)
- Implement gameplay using engine features
- Multiple levels/scenes
- Win/lose conditions
- UI (score, health, etc.)

**Milestone 8.2: Optimization & Profiling**
- Profile engine performance
- Optimize hot paths
- Reduce draw calls (batching)
- Optimize asset loading
- Memory leak checking

**Milestone 8.3: Documentation & Deployment**
- Write engine documentation
- Example projects/tutorials
- Build standalone game executable
- Package assets with game
- Celebrate completion! 🎉

## Technical Challenges

### Challenge 1: Subsystem Dependencies

**Problem**: Subsystems depend on each other (circular dependencies)

**Approach**:
- Define clear initialization order
- Use dependency injection (pass dependencies to modules)
- Event-based communication (decouple via events)
- Service locator pattern (central registry)

**Example**:
```
init_order = [
  Platform,      # First: windowing, filesystem
  Renderer,      # Needs window
  Physics,       # Independent
  Audio,         # Independent
  Input,         # Needs window events
  Scene,         # Needs all subsystems
  Scripting,     # Needs scene, input, etc.
  Editor,        # Last: depends on everything
]
```

### Challenge 2: Resource Lifetime Management

**Problem**: Assets referenced by multiple systems, need proper cleanup

**Approach**:
- Reference counting (Arc/Rc)
- Asset handles (weak references)
- Explicit unload when scene changes
- Garbage collection pass (periodically remove unreferenced)

### Challenge 3: Editor-Runtime Separation

**Problem**: Editor code shouldn't ship with game

**Approach**:
- Separate crates/modules (engine-runtime, engine-editor)
- Conditional compilation (#[cfg(feature = "editor")])
- Plugin architecture (editor is a plugin)
- Build profiles (release builds exclude editor)

### Challenge 4: Save/Load Compatibility

**Problem**: Changing component structure breaks old save files

**Approach**:
- Version scene file format
- Migration system (upgrade old versions)
- Use flexible serialization (JSON, MessagePack)
- Component versioning
- Schema validation

### Challenge 5: Performance with Many Entities

**Problem**: Iterating all entities becomes slow

**Approach**:
- Use ECS library (bevy_ecs, hecs, specs)
- Cache queries (don't rebuild each frame)
- Spatial partitioning (only process nearby entities)
- Entity culling (don't update off-screen entities)
- Multi-threading (parallel systems)

## Reference Implementations

### Praxis Engine (Rust)
- **Entire codebase**: This project is a complete reference!
- Study overall architecture, subsystem integration
- See `praxis_core`, `praxis_assets`, `praxis_scene`, etc.

### Other Engines

**Bevy (Rust)**
- Modern ECS-based engine
- Plugin architecture
- Study: modularity, ECS patterns, asset pipeline

**Godot (C++/GDScript)**
- Open-source engine
- Scene system, node hierarchy
- Study: editor integration, scripting, export pipeline

**Unity (C#)**
- Commercial engine (reference architecture)
- Component-based workflow
- Study: editor design, asset import, scripting API

**Unreal Engine (C++)**
- AAA engine
- Study: advanced rendering, blueprint system, build pipeline

**Amethyst (Rust)**
- Data-driven ECS engine (archived, but good reference)
- Study: specs ECS usage, system scheduling

**Mini Engines (Educational)**
- Hazel Engine (Cherno, C++)
- Sparky Engine (Cherno, C++)
- Study: from-scratch development, subsystem design

## Extension Ideas

### Beginner Extensions
- More asset formats (FBX, DAE)
- UI system (buttons, text, layouts)
- Particle system integration
- More example games

### Intermediate Extensions
- Build pipeline (asset packing, compression)
- Console/debug overlay
- Reflection system (introspect components)
- Networked multiplayer

### Advanced Extensions
- Data-driven design (define entities in config files)
- Visual scripting (node-based logic)
- Advanced rendering (PBR, deferred, shadows)
- Cross-platform deployment (WebAssembly, mobile)

## Success Criteria

Your mini game engine should:

1. ✅ Run a complete game from start to finish
2. ✅ Support all core subsystems (render, physics, audio, input, scripting)
3. ✅ Provide editor for scene creation
4. ✅ Load and save scenes reliably
5. ✅ Run at 60 FPS with moderate scene complexity
6. ✅ Have clear API for game development
7. ✅ Be extensible (easy to add new components/systems)
8. ✅ Ship standalone game without editor dependencies

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Architecture** | Basic modular structure | Clear subsystems, ECS | Plugin system, hot-reload, scalable |
| **Features** | Render, input, basic gameplay | + Physics, audio, scripting | + Animation, networking, advanced |
| **Editor** | Basic scene manipulation | Property editing, asset browser | Play mode, profiling, full pipeline |
| **Game** | Simple prototype | Complete small game | Polished game, multiple levels |

## Common Pitfalls

1. **Premature Optimization**: Focus on functionality first, optimize later
2. **Over-Engineering**: Start simple, add complexity as needed
3. **Monolithic Design**: Keep subsystems loosely coupled
4. **Ignoring Data Orientation**: ECS/DOD patterns improve performance
5. **No Version Control**: Use Git from day one
6. **Poor Error Handling**: Engine should never crash, report errors gracefully
7. **Hardcoded Paths**: Use asset system, not absolute paths
8. **Not Dogfooding**: Use your engine to build a game (find issues)

## Lessons Learned

After completing this project, you will understand:

- **Engine Architecture**: How major subsystems fit together
- **Trade-offs**: Performance vs flexibility, simplicity vs features
- **Iteration**: Engines evolve, initial design won't be perfect
- **Documentation**: Critical for usability
- **Testing**: Important for stability (unit tests, integration tests)
- **User Needs**: Engine features driven by actual game development

## Next Steps

After completing your mini game engine:

1. **Build More Games**: Stress-test your engine with different game types
2. **Open Source**: Share your engine, get feedback, iterate
3. **Study AAA Engines**: Dive deeper into Unreal/Unity/Godot source
4. **Specialize**: Focus on one subsystem (rendering, physics, networking)
5. **Join Engine Community**: Contribute to open-source engines (Bevy, Godot, etc.)
6. **Professional Development**: Apply skills to game studio or engine company

## Celebrate Your Achievement! 🎉

Building a game engine from scratch is a monumental accomplishment. You've learned:
- Core engine architecture patterns
- Subsystem design and integration
- Asset pipeline development
- Editor tool creation
- Performance optimization
- Complete game development workflow

You now have the knowledge to work on professional game engines or build advanced graphics/simulation applications. Well done!
