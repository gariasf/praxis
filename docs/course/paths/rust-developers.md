# For Rust Developers Using Praxis

A comprehensive guide for Rust developers to master Praxis engine architecture, leverage Rust's unique features, and build games efficiently with modern ECS patterns.

## Overview

This guide assumes Rust proficiency and focuses on game engine-specific patterns, Praxis subsystems, and idiomatic Rust usage in real-time systems.

**Target Audience**: Rust developers wanting to build games, contribute to Praxis, or understand game engine architecture through a modern Rust lens.

**Prerequisites**:
- Proficiency with Rust (ownership, traits, lifetimes, async)
- Understanding of `cargo` and the Rust ecosystem
- Basic 3D math concepts (vectors, matrices, quaternions)

**Learning Approach**: Explore Praxis subsystems, understand design decisions, optimize performance, and extend functionality using Rust's strengths.

---

## Rust in Game Development: Unique Advantages

### Fearless Concurrency
```rust
// Safe parallel iteration with rayon
use rayon::prelude::*;

fn parallel_physics_update(velocities: &mut [Vec3], forces: &[Vec3], dt: f32) {
    velocities.par_iter_mut()
        .zip(forces.par_iter())
        .for_each(|(vel, force)| {
            *vel += *force * dt;
        });
}

// No data races possible - compiler enforces!
```

### Zero-Cost Abstractions
```rust
// Generic without runtime cost
fn apply_velocity<T: Component + HasVelocity>(
    query: Query<(&mut Transform, &T)>,
    time: Res<Time>,
) {
    for (mut transform, component) in query.iter_mut() {
        transform.translation += component.velocity() * time.delta_seconds();
    }
}
// Monomorphized at compile-time, no virtual function overhead
```

### Compile-Time Safety
```rust
// ECS query guarantees no aliasing
fn safe_system(
    mut query: Query<&mut Transform>,
    other: Query<&Velocity>,
) {
    // Compile error if queries overlap
}

// Type-safe component access
fn typed_system(query: Query<&Health>) {
    for health in query.iter() {
        // health is guaranteed to be Health component
        println!("HP: {}", health.current);
    }
}
```

---

## Praxis Architecture Deep Dive

### Crate Organization

Praxis uses a 19-crate workspace for modularity:

```
praxis/
├── praxis_core          # Engine lifecycle, main loop
├── praxis_window        # Window management (winit)
├── praxis_graphics      # Vulkan rendering (vulkano)
├── praxis_ecs           # ECS wrappers (bevy_ecs)
├── praxis_math          # Math utilities (glam)
├── praxis_scene         # Transform hierarchy, animation
├── praxis_spatial       # Spatial data structures
├── praxis_assets        # Asset loading (OBJ, GLTF)
├── praxis_input         # Input handling
├── praxis_gui           # Editor GUI (egui)
├── praxis_physics       # Physics integration (rapier3d)
├── praxis_audio         # Audio system (kira)
├── praxis_procedural    # Procedural generation
├── praxis_terrain       # Terrain rendering
├── praxis_profiling     # Performance profiling
├── praxis_scripting     # Lua integration (mlua)
├── praxis_networking    # Multiplayer networking
├── praxis_editor        # Editor tools
└── praxis_utils         # Logging, errors, timing
```

**Design Principle**: Each crate has minimal dependencies, enabling:
- Independent compilation
- Optional feature flags
- Clear API boundaries

**Read**: `AGENTS.md` (project root) for detailed crate purposes.

---

## Learning Path by Subsystem

### Phase 1: ECS Mastery with bevy_ecs (Week 1-2)

**Goal**: Master bevy_ecs patterns used throughout Praxis.

#### Understanding Archetypes

**bevy_ecs** uses archetype-based storage for cache efficiency:

```rust
// Archetype: (Transform, Velocity, Health)
world.spawn((
    Transform::default(),
    Velocity { value: Vec3::ZERO },
    Health { current: 100.0, max: 100.0 },
));

// Different archetype: (Transform, Velocity)
world.spawn((
    Transform::default(),
    Velocity { value: Vec3::X },
));

// Entities with same component types stored contiguously in memory
```

**Implications**:
- Adding/removing components moves entity to different archetype
- Structural changes (add/remove) expensive, use `Commands`
- Queries iterate archetypes with matching components (cache-friendly)

#### Query Patterns

**Basic Query**:
```rust
fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.value;
    }
}
```

**Filtered Query**:
```rust
fn player_only_system(
    query: Query<&Transform, With<Player>>
) {
    // Only entities with Player tag
}

fn non_static_system(
    query: Query<&mut Transform, Without<Static>>
) {
    // Exclude static entities
}
```

**Changed Detection**:
```rust
fn react_to_damage(
    query: Query<&Health, Changed<Health>>
) {
    for health in query.iter() {
        println!("Health changed: {}", health.current);
    }
}
```

**Mutable Access Guarantees**:
```rust
// ERROR: Cannot have overlapping mutable queries
fn invalid_system(
    mut query1: Query<&mut Transform>,
    mut query2: Query<&mut Transform>, // Compile error!
) {}

// OK: Disjoint queries with filters
fn valid_system(
    mut players: Query<&mut Transform, With<Player>>,
    mut enemies: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
) {}
```

#### Resources and Commands

**Resources** (global singleton data):
```rust
#[derive(Resource)]
struct Time {
    delta: f32,
    elapsed: f32,
}

fn time_system(time: Res<Time>) {
    println!("Delta: {}", time.delta);
}

fn modify_time(mut time: ResMut<Time>) {
    time.elapsed += time.delta;
}
```

**Commands** (deferred structural changes):
```rust
fn spawn_system(mut commands: Commands) {
    // Deferred spawn (applied at end of stage)
    commands.spawn((
        Transform::default(),
        Health { current: 100.0, max: 100.0 },
    ));
    
    // Deferred despawn
    commands.entity(some_entity).despawn();
    
    // Insert component
    commands.entity(some_entity).insert(Velocity::default());
}
```

**Why Commands?** Structural changes during iteration would invalidate iterators. Commands batch changes for application between stages.

#### System Ordering

```rust
let mut schedule = Schedule::default();

// Sequential
schedule.add_systems(
    (input_system, physics_system, render_system).chain()
);

// Parallel (default if no dependencies)
schedule.add_systems((
    system_a,
    system_b,
    system_c, // Run in parallel if queries don't conflict
));

// Explicit ordering
schedule.add_systems(
    physics_system
        .before(transform_propagation)
        .after(input_system)
);
```

**Exercise**:
1. Read `crates/praxis_ecs/src/lib.rs`
2. Run `cargo run --example ecs_integration`
3. Implement custom system with queries, resources, commands
4. Profile with `--features profiling`

---

### Phase 2: Rendering with Vulkano (Week 3-4)

**Goal**: Understand Praxis's Vulkan abstraction and rendering pipeline.

#### Vulkano Fundamentals

Praxis uses `vulkano` for type-safe Vulkan bindings:

```rust
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage};

// Type-safe buffer creation
#[derive(BufferContents, Vertex)]
#[repr(C)]
struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32_SFLOAT)]
    normal: [f32; 3],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
}

let vertices = vec![
    Vertex { position: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
    // ...
];

let vertex_buffer = Buffer::from_iter(
    allocator.clone(),
    BufferCreateInfo {
        usage: BufferUsage::VERTEX_BUFFER,
        ..Default::default()
    },
    AllocationCreateInfo {
        usage: MemoryUsage::Upload,
        ..Default::default()
    },
    vertices,
)?;
```

**Type Safety**: Vulkano checks buffer usage, memory types, synchronization at compile-time.

#### Shader Compilation

**GLSL → SPIR-V** at compile-time:
```rust
mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450
            
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;
            layout(location = 2) in vec2 uv;
            
            layout(location = 0) out vec3 frag_normal;
            layout(location = 1) out vec2 frag_uv;
            
            layout(push_constant) uniform PushConstants {
                mat4 model;
            } push;
            
            layout(set = 0, binding = 0) uniform UniformBufferObject {
                mat4 view;
                mat4 proj;
            } ubo;
            
            void main() {
                gl_Position = ubo.proj * ubo.view * push.model * vec4(position, 1.0);
                frag_normal = mat3(push.model) * normal;
                frag_uv = uv;
            }
        ",
    }
}
```

**Benefits**:
- Compile-time shader errors
- Type-safe descriptor sets
- No runtime shader compilation

#### Rendering Flow

**Read**: `crates/praxis_graphics/src/render_context.rs`

```rust
// 1. Begin frame
let (image_index, suboptimal, acquire_future) = 
    swapchain.acquire_next_image(None)?;

// 2. Build command buffer
let mut builder = AutoCommandBufferBuilder::primary(...)?;

// 3. Begin render pass
builder.begin_render_pass(
    RenderPassBeginInfo {
        clear_values: vec![
            Some([0.0, 0.0, 0.0, 1.0].into()),
            Some(1.0.into()),
        ],
        ..RenderPassBeginInfo::framebuffer(framebuffers[image_index].clone())
    },
    SubpassContents::Inline,
)?;

// 4. Bind pipeline
builder.bind_pipeline_graphics(pipeline.clone());

// 5. Bind descriptor sets (uniforms, textures)
builder.bind_descriptor_sets(
    PipelineBindPoint::Graphics,
    pipeline_layout.clone(),
    0,
    descriptor_set.clone(),
);

// 6. Push constants (per-object data)
builder.push_constants(
    pipeline_layout.clone(),
    0,
    model_matrix,
);

// 7. Bind vertex/index buffers
builder.bind_vertex_buffers(0, vertex_buffer.clone());
builder.bind_index_buffer(index_buffer.clone());

// 8. Draw
builder.draw_indexed(index_count, 1, 0, 0, 0)?;

// 9. End render pass
builder.end_render_pass()?;

// 10. Submit and present
let command_buffer = builder.build()?;
let future = acquire_future
    .then_execute(queue.clone(), command_buffer)?
    .then_swapchain_present(...)
    .then_signal_fence_and_flush()?;
```

**Optimization**: Batch draw calls with same material/pipeline to minimize state changes.

#### Deferred Rendering

**Read**: `crates/praxis_graphics/src/deferred_renderer.rs`

```rust
// G-buffer pass
deferred_renderer.begin_geometry_pass(&mut builder, clear_values)?;

for draw_cmd in draw_commands {
    // Bind pipeline, descriptors, vertex/index buffers
    // Write to G-buffer (position, normal, albedo, etc.)
}

deferred_renderer.end_geometry_pass(&mut builder)?;

// Lighting pass
deferred_renderer.begin_lighting_pass(&mut builder)?;

// Fullscreen quad, read G-buffer, calculate lighting
deferred_renderer.render_lights(&mut builder, lights)?;

deferred_renderer.end_lighting_pass(&mut builder)?;
```

**Exercise**:
1. Run `cargo run --example scene_demo`
2. Enable deferred rendering in code
3. Add custom shader effect (e.g., outline, toon shading)
4. Profile GPU with RenderDoc

---

### Phase 3: Transform Hierarchy and Animation (Week 5-6)

**Goal**: Master hierarchical transforms and skeletal animation systems.

#### Transform Propagation

**Read**: `crates/praxis_scene/src/transform.rs`

```rust
#[derive(Component)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.translation,
        )
    }
}

#[derive(Component)]
pub struct GlobalTransform {
    pub matrix: Mat4,
}

#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);
```

**Propagation System**:
```rust
pub fn propagate_transforms(
    mut root_query: Query<
        (&Transform, &mut GlobalTransform, Option<&Children>),
        (Changed<Transform>, Without<Parent>)
    >,
    mut child_query: Query<(&Transform, &mut GlobalTransform, Option<&Children>)>,
) {
    for (transform, mut global, children) in root_query.iter_mut() {
        // Root: global = local
        global.matrix = transform.to_matrix();
        
        if let Some(children) = children {
            propagate_recursive(&mut child_query, &children.0, &global);
        }
    }
}

fn propagate_recursive(
    query: &mut Query<(&Transform, &mut GlobalTransform, Option<&Children>)>,
    children: &[Entity],
    parent_global: &GlobalTransform,
) {
    for &child in children {
        if let Ok((transform, mut global, grandchildren)) = query.get_mut(child) {
            global.matrix = parent_global.matrix * transform.to_matrix();
            
            if let Some(grandchildren) = grandchildren {
                propagate_recursive(query, &grandchildren.0, &global);
            }
        }
    }
}
```

**Optimization**: Use `Changed<Transform>` to only propagate dirty subtrees.

#### Skeletal Animation

**Read**: `crates/praxis_scene/src/animation.rs`

```rust
#[derive(Component)]
pub struct Skeleton {
    pub joints: Vec<Joint>,
}

#[derive(Component)]
pub struct AnimationPlayer {
    pub current_clip: Handle<AnimationClip>,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
}

pub struct AnimationClip {
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
}

pub struct AnimationChannel {
    pub joint_index: usize,
    pub keyframes: Vec<Keyframe>,
}

pub struct Keyframe {
    pub time: f32,
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

**Animation System**:
```rust
fn animate_skeletons(
    mut query: Query<(&mut Skeleton, &AnimationPlayer)>,
    clips: Res<Assets<AnimationClip>>,
    time: Res<Time>,
) {
    for (mut skeleton, player) in query.iter_mut() {
        let Some(clip) = clips.get(&player.current_clip) else { continue };
        
        let playback_time = (player.time * player.speed) % clip.duration;
        
        for channel in &clip.channels {
            let transform = interpolate_keyframes(&channel.keyframes, playback_time);
            skeleton.joints[channel.joint_index].local_transform = transform;
        }
    }
}

fn interpolate_keyframes(keyframes: &[Keyframe], time: f32) -> Transform {
    // Find surrounding keyframes
    let (prev, next) = find_keyframes(keyframes, time);
    
    // Interpolate
    let t = (time - prev.time) / (next.time - prev.time);
    
    Transform {
        translation: prev.translation.lerp(next.translation, t),
        rotation: prev.rotation.slerp(next.rotation, t),
        scale: prev.scale.lerp(next.scale, t),
    }
}
```

**Exercise**:
1. Run `cargo run --example skeletal_animation_demo`
2. Load GLTF with animations
3. Implement blend tree (walk → run transition)
4. Add inverse kinematics (IK) for foot placement

---

### Phase 4: Physics Integration with Rapier (Week 7)

**Goal**: Integrate Rapier3D physics engine with ECS.

**Read**: `crates/praxis_physics/README.md`

#### Physics Components

```rust
#[derive(Component)]
pub enum RigidBody {
    Dynamic,
    Static,
    Kinematic,
}

#[derive(Component)]
pub enum Collider {
    Box { half_extents: Vec3 },
    Sphere { radius: f32 },
    Capsule { half_height: f32, radius: f32 },
    Mesh { vertices: Vec<Vec3>, indices: Vec<u32> },
}

#[derive(Component)]
pub struct PhysicsVelocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

#[derive(Resource)]
pub struct PhysicsWorld {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
}
```

#### Sync Systems

```rust
// 1. ECS → Physics (kinematic bodies)
fn sync_transforms_to_physics(
    query: Query<(Entity, &Transform, &RigidBody), With<Kinematic>>,
    mut physics_world: ResMut<PhysicsWorld>,
    entity_to_handle: Res<EntityToHandle>,
) {
    for (entity, transform, _) in query.iter() {
        if let Some(&handle) = entity_to_handle.get(&entity) {
            if let Some(rb) = physics_world.rigid_body_set.get_mut(handle) {
                let pos = Isometry3::new(
                    transform.translation.into(),
                    transform.rotation.into(),
                );
                rb.set_next_kinematic_position(pos);
            }
        }
    }
}

// 2. Step physics (fixed timestep)
fn physics_step(
    mut physics_world: ResMut<PhysicsWorld>,
    time: Res<Time>,
) {
    let dt = 1.0 / 60.0; // Fixed 60 Hz
    physics_world.integration_parameters.dt = dt;
    
    physics_world.physics_pipeline.step(
        &Vector3::new(0.0, -9.81, 0.0), // Gravity
        &physics_world.integration_parameters,
        &mut physics_world.island_manager,
        &mut physics_world.broad_phase,
        &mut physics_world.narrow_phase,
        &mut physics_world.rigid_body_set,
        &mut physics_world.collider_set,
        &mut physics_world.impulse_joint_set,
        &mut physics_world.multibody_joint_set,
        &mut physics_world.ccd_solver,
        None, // query_pipeline
        &(), // hooks
        &(), // events
    );
}

// 3. Physics → ECS (dynamic bodies)
fn sync_transforms_from_physics(
    mut query: Query<(Entity, &mut Transform, &RigidBody), With<Dynamic>>,
    physics_world: Res<PhysicsWorld>,
    entity_to_handle: Res<EntityToHandle>,
) {
    for (entity, mut transform, _) in query.iter_mut() {
        if let Some(&handle) = entity_to_handle.get(&entity) {
            if let Some(rb) = physics_world.rigid_body_set.get(handle) {
                let pos = rb.translation();
                let rot = rb.rotation();
                
                transform.translation = Vec3::new(pos.x, pos.y, pos.z);
                transform.rotation = Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);
            }
        }
    }
}
```

**System Ordering**:
```rust
schedule.add_systems(
    (
        sync_transforms_to_physics,
        physics_step,
        sync_transforms_from_physics,
        handle_collision_events,
    ).chain()
);
```

**Exercise**:
1. Run `cargo run --example ecs_integration`
2. Create stacks of boxes (dynamic bodies)
3. Add kinematic platform (moving obstacle)
4. Implement raycasting for hit detection

---

### Phase 5: Scripting with Lua (Week 8)

**Goal**: Integrate Lua scripting for gameplay logic and hot-reload.

**Read**: `crates/praxis_scripting/README.md`

#### Lua Context Setup

```rust
use mlua::{Lua, Table, Function, UserData, UserDataMethods};

#[derive(Clone)]
pub struct ScriptingContext {
    lua: Lua,
    hot_reload: Option<HotReloadWatcher>,
}

impl ScriptingContext {
    pub fn new(config: ScriptingConfig) -> Result<Self> {
        let lua = Lua::new();
        
        // Sandbox environment
        if config.sandboxing == SandboxLevel::Strict {
            disable_dangerous_functions(&lua)?;
        }
        
        Ok(Self {
            lua,
            hot_reload: None,
        })
    }
    
    pub fn load_script(&self, name: &str, path: &Path) -> Result<()> {
        let source = std::fs::read_to_string(path)?;
        self.lua.load(&source).set_name(name)?.exec()?;
        Ok(())
    }
    
    pub fn call_function<'lua, A, R>(&'lua self, name: &str, args: A) -> Result<R>
    where
        A: mlua::IntoLuaMulti<'lua>,
        R: mlua::FromLuaMulti<'lua>,
    {
        let func: Function = self.lua.globals().get(name)?;
        func.call(args)
    }
}
```

#### ECS Bindings

```rust
// Expose World to Lua
impl UserData for LuaWorldHandle {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("spawn", |_, this, ()| {
            let entity = this.world.spawn_empty().id();
            Ok(LuaEntity(entity))
        });
        
        methods.add_method("get_component", |_, this, (entity, component_name): (LuaEntity, String)| {
            match component_name.as_str() {
                "Transform" => {
                    let transform = this.world.get::<Transform>(entity.0)?;
                    Ok(LuaTransform::from(transform))
                }
                "Health" => {
                    let health = this.world.get::<Health>(entity.0)?;
                    Ok(LuaHealth::from(health))
                }
                _ => Err(mlua::Error::external("Unknown component")),
            }
        });
        
        methods.add_method_mut("set_component", |_, this, (entity, component): (LuaEntity, LuaValue)| {
            // Parse component from Lua table and insert
            Ok(())
        });
    }
}

// Lua script
/*
local player = world:spawn()
world:set_component(player, {
    type = "Transform",
    position = { x = 0, y = 10, z = 0 }
})
world:set_component(player, {
    type = "Health",
    current = 100,
    max = 100
})
*/
```

#### Hot-Reload

```rust
use notify::{Watcher, RecursiveMode, Event};

pub struct HotReloadWatcher {
    watcher: RecommendedWatcher,
    scripts: HashMap<String, PathBuf>,
}

impl HotReloadWatcher {
    pub fn watch(&mut self, path: impl AsRef<Path>) -> Result<()> {
        self.watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
        Ok(())
    }
    
    pub fn poll(&mut self, scripting: &ScriptingContext) -> Result<()> {
        while let Ok(event) = self.receiver.try_recv() {
            if let Event::Modify(_) = event {
                for (name, path) in &self.scripts {
                    scripting.load_script(name, path)?;
                    println!("Hot-reloaded: {}", name);
                }
            }
        }
        Ok(())
    }
}
```

**Exercise**:
1. Run `cargo run --example scripting_demo`
2. Modify `scripts/game.lua` while running
3. Implement custom Lua bindings for your components
4. Create AI behavior in Lua

---

### Phase 6: Networking with Tokio (Week 9-10)

**Goal**: Build client-server architecture with entity replication.

**Read**: `crates/praxis_networking/README.md`

#### Network Server

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

pub struct NetworkServer {
    listener: TcpListener,
    clients: HashMap<ClientId, ClientConnection>,
    replication_registry: ReplicationRegistry,
}

impl NetworkServer {
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        let listener = TcpListener::bind(&config.bind_address).await?;
        
        Ok(Self {
            listener,
            clients: HashMap::new(),
            replication_registry: ReplicationRegistry::new(),
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            println!("Client connected: {}", addr);
            
            let client_id = ClientId::new();
            let conn = ClientConnection::new(socket).await?;
            self.clients.insert(client_id, conn);
        }
    }
    
    pub fn replicate_entity(&self, entity: Entity, components: &World) -> Result<()> {
        let mut packet = Vec::new();
        
        // Serialize entity ID
        packet.extend_from_slice(&entity.to_bits().to_le_bytes());
        
        // Serialize registered components
        for (type_id, serializer) in &self.replication_registry.serializers {
            if let Some(component_data) = components.get_by_id(entity, *type_id) {
                serializer(component_data, &mut packet)?;
            }
        }
        
        // Send to all clients
        for client in self.clients.values() {
            client.send(&packet).await?;
        }
        
        Ok(())
    }
}
```

#### Component Replication

```rust
pub struct ReplicationRegistry {
    serializers: HashMap<TypeId, Box<dyn Fn(&dyn Any, &mut Vec<u8>) -> Result<()>>>,
    deserializers: HashMap<TypeId, Box<dyn Fn(&[u8], &mut World, Entity) -> Result<()>>>,
}

impl ReplicationRegistry {
    pub fn register_component<T: Component + Serialize + DeserializeOwned>(&mut self) {
        let type_id = TypeId::of::<T>();
        
        // Serializer
        self.serializers.insert(type_id, Box::new(|component, buffer| {
            let comp = component.downcast_ref::<T>().unwrap();
            let bytes = bincode::serialize(comp)?;
            buffer.extend_from_slice(&bytes);
            Ok(())
        }));
        
        // Deserializer
        self.deserializers.insert(type_id, Box::new(|bytes, world, entity| {
            let comp: T = bincode::deserialize(bytes)?;
            world.entity_mut(entity).insert(comp);
            Ok(())
        }));
    }
}
```

**Exercise**:
1. Run `cargo run --example networking_demo` (server + client)
2. Implement client prediction for movement
3. Add lag compensation for hit detection
4. Profile network bandwidth usage

---

## Advanced Optimization

### Profiling with Tracy

```rust
// Add to Cargo.toml
[dependencies]
tracy-client = "0.15"

// In code
use tracy_client::span;

fn expensive_system() {
    let _span = span!("expensive_system");
    
    // System logic
    for i in 0..1000 {
        let _inner_span = span!("inner_loop");
        // ...
    }
}
```

Run Tracy server, profile Praxis, identify bottlenecks.

### SIMD with packed_simd

```rust
use std::simd::{f32x4, SimdFloat};

fn simd_transform_vertices(vertices: &mut [[f32; 3]], matrix: &Mat4) {
    let m = matrix.to_cols_array_2d();
    
    for chunk in vertices.chunks_exact_mut(4) {
        let x = f32x4::from_array([chunk[0][0], chunk[1][0], chunk[2][0], chunk[3][0]]);
        let y = f32x4::from_array([chunk[0][1], chunk[1][1], chunk[2][1], chunk[3][1]]);
        let z = f32x4::from_array([chunk[0][2], chunk[1][2], chunk[2][2], chunk[3][2]]);
        
        // Matrix multiplication (simplified)
        let out_x = x * f32x4::splat(m[0][0]) + y * f32x4::splat(m[1][0]) + z * f32x4::splat(m[2][0]);
        let out_y = x * f32x4::splat(m[0][1]) + y * f32x4::splat(m[1][1]) + z * f32x4::splat(m[2][1]);
        let out_z = x * f32x4::splat(m[0][2]) + y * f32x4::splat(m[1][2]) + z * f32x4::splat(m[2][2]);
        
        let out_x_arr = out_x.to_array();
        let out_y_arr = out_y.to_array();
        let out_z_arr = out_z.to_array();
        
        for i in 0..4 {
            chunk[i] = [out_x_arr[i], out_y_arr[i], out_z_arr[i]];
        }
    }
}
```

---

## Best Practices

### Error Handling
```rust
// Use thiserror for custom errors
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Vulkan error: {0}")]
    VulkanError(#[from] vulkano::VulkanError),
    
    #[error("Shader compilation failed: {0}")]
    ShaderError(String),
    
    #[error("Mesh not found: {0}")]
    MeshNotFound(String),
}

// Return Result, use ? operator
fn render_mesh(id: &str) -> Result<(), RenderError> {
    let mesh = mesh_manager.get(id)
        .ok_or_else(|| RenderError::MeshNotFound(id.to_string()))?;
    
    // Vulkan calls that can fail
    command_buffer.bind_vertex_buffers(0, mesh.vertex_buffer.clone())?;
    
    Ok(())
}
```

### Logging with tracing
```rust
use tracing::{info, warn, error, debug, trace};

fn load_asset(path: &Path) -> Result<Asset> {
    info!("Loading asset: {:?}", path);
    
    let data = std::fs::read(path)
        .map_err(|e| {
            error!("Failed to read file: {}", e);
            e
        })?;
    
    debug!("Read {} bytes", data.len());
    
    Ok(Asset::from_bytes(data))
}
```

---

## Recommended Study Order

### 4-Week Praxis Mastery
```
Week 1: ECS (bevy_ecs deep dive)
Week 2: Rendering (Vulkano + shaders)
Week 3: Transform + Animation
Week 4: Physics + Scripting
```

### 12-Week Complete Engine Developer
```
Weeks 1-2: ECS + Rendering
Weeks 3-4: Transform hierarchy + Animation
Weeks 5-6: Physics + Audio
Weeks 7-8: Scripting + Hot-reload
Weeks 9-10: Networking
Weeks 11-12: Custom feature + optimization
```

---

## Resources

### Praxis-Specific
- [Architecture](../../architecture.md)
- [Beginner's Guide](../../beginners-guide.md)
- [Curriculum](../CURRICULUM.md)
- Examples: `cargo run --example <name>`

### Rust Game Development
- [Bevy Engine](https://bevyengine.org/)  - Modern Rust ECS engine
- [Amethyst](https://github.com/amethyst/amethyst) - Data-driven engine
- [Are We Game Yet?](https://arewegameyet.rs/) - Rust gamedev ecosystem

### Libraries
- [bevy_ecs](https://docs.rs/bevy_ecs/) - ECS documentation
- [vulkano](https://docs.rs/vulkano/) - Vulkan wrapper
- [rapier3d](https://rapier.rs/) - Physics engine
- [mlua](https://docs.rs/mlua/) - Lua bindings

---

## Next Steps

After this path:
- ✅ Master Praxis architecture
- ✅ Proficient with bevy_ecs patterns
- ✅ Understand Vulkan rendering
- ✅ Can optimize performance
- ✅ Ready to contribute or build games

**Continue to**:
- Build complete game with Praxis
- Contribute to Praxis (add features, optimize, fix bugs)
- Explore [Bevy](https://bevyengine.org/) for comparison
- Write custom engine with learned patterns
