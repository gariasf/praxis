# For C++ Programmers Building Custom Engines

A comprehensive guide for C++ programmers learning modern game engine architecture through Praxis, with focus on translating patterns to C++ implementations.

## Overview

This guide bridges C++ and Rust for engine development. While syntax differs, architectural patterns are universal. You'll learn modern engine design through Praxis, then apply concepts in C++.

**Target Audience**: C++ programmers building custom engines, Unreal developers exploring engine internals, or experienced programmers transitioning to Rust.

**Prerequisites**:
- Strong C++ knowledge (C++11 or later)
- Understanding of pointers, memory management, templates
- Basic graphics programming (OpenGL, DirectX, or Vulkan)
- 3D math fundamentals

**Learning Approach**: See modern patterns in Rust (simpler syntax), understand the concepts, then apply in C++ with appropriate idioms.

---

## Key Language Mappings

### Memory Management

| C++ | Rust (Praxis) | Universal Pattern |
|-----|---------------|-------------------|
| `new` / `delete` | Ownership + Drop | Manual allocation |
| `std::unique_ptr<T>` | `Box<T>` | Unique ownership |
| `std::shared_ptr<T>` | `Arc<T>` | Shared ownership |
| `T&` (reference) | `&T` (borrow) | Immutable reference |
| `T&` (mutable) | `&mut T` | Mutable reference |
| `std::optional<T>` | `Option<T>` | Nullable values |
| `std::variant<T, E>` | `Result<T, E>` | Error handling |

### Type System

| C++ | Rust | Concept |
|-----|------|---------|
| Template `template<typename T>` | Generic `<T>` | Compile-time polymorphism |
| Virtual functions | Trait objects `dyn Trait` | Runtime polymorphism |
| Concepts (C++20) | Traits | Type constraints |
| `constexpr` | `const fn` | Compile-time evaluation |
| `static_assert` | `compile_error!` | Compile-time checks |

### Concurrency

| C++ | Rust | Safety |
|-----|------|--------|
| `std::mutex` + manual locking | `Mutex<T>` (RAII) | Lock tied to data |
| `std::atomic` | `AtomicT` | Lock-free primitives |
| `std::thread` | `std::thread` | Thread spawning |
| Manual sync (error-prone) | `Send` + `Sync` traits | Compile-time safety |

---

## C++ to Rust: Core Patterns

### RAII (Resource Acquisition Is Initialization)

**C++ (Manual)**:
```cpp
class Mesh {
    GLuint vbo_;
public:
    Mesh() { glGenBuffers(1, &vbo_); }
    ~Mesh() { glDeleteBuffers(1, &vbo_); }
    
    // Rule of Five
    Mesh(const Mesh&) = delete;
    Mesh& operator=(const Mesh&) = delete;
    Mesh(Mesh&& other) noexcept : vbo_(other.vbo_) {
        other.vbo_ = 0;
    }
    Mesh& operator=(Mesh&& other) noexcept {
        if (this != &other) {
            glDeleteBuffers(1, &vbo_);
            vbo_ = other.vbo_;
            other.vbo_ = 0;
        }
        return *this;
    }
};
```

**Rust (Automatic)**:
```rust
struct Mesh {
    vbo: VulkanBuffer, // Implements Drop automatically
}

impl Drop for Mesh {
    fn drop(&mut self) {
        // Cleanup happens automatically, no move semantics needed
    }
}

// Move is default, copy requires explicit opt-in
// No copy/move constructors needed
```

**Key Insight**: Rust enforces move-by-default and automatic cleanup. C++ requires manual Rule of Five/Zero.

### Ownership and Borrowing

**C++ (Pointers)**:
```cpp
void process_mesh(const Mesh& mesh) {
    // Can read, cannot modify
}

void modify_mesh(Mesh& mesh) {
    // Can modify
}

void consume_mesh(std::unique_ptr<Mesh> mesh) {
    // Takes ownership, will delete
}

// Multiple mutable references possible (dangerous!)
Mesh* mesh = new Mesh();
Mesh& ref1 = *mesh;
Mesh& ref2 = *mesh;
ref1.modify();
ref2.modify(); // Data race potential
```

**Rust (Borrow Checker)**:
```rust
fn process_mesh(mesh: &Mesh) {
    // Immutable borrow
}

fn modify_mesh(mesh: &mut Mesh) {
    // Mutable borrow (exclusive)
}

fn consume_mesh(mesh: Mesh) {
    // Takes ownership
} // Dropped here

// Only one mutable OR many immutable borrows
let mut mesh = Mesh::new();
let ref1 = &mesh;
let ref2 = &mesh; // OK: multiple immutable
// let ref3 = &mut mesh; // ERROR: cannot borrow as mutable while immutable borrows exist
```

**Universal Pattern**: Single writer OR multiple readers, never both simultaneously.

**C++ Implementation**:
```cpp
// Manual enforcement via coding standards
// Use const& by default
// Document ownership in comments
// Use smart pointers to clarify ownership
```

---

## Learning Path by Architecture Subsystem

Refer to [Course Curriculum](../CURRICULUM.md) for universal concepts. This guide shows C++ implementation patterns.

### Module 1: Game Loop Implementation (Week 1)

**See**: [Game Loop Patterns](../../course/patterns/game-loop-patterns.md)

#### Fixed Timestep in C++

**Praxis Pattern** (Rust):
```rust
let mut accumulator = 0.0;
let fixed_dt = 1.0 / 60.0;

loop {
    let frame_time = timer.delta();
    accumulator += frame_time;
    
    while accumulator >= fixed_dt {
        physics_step(fixed_dt);
        accumulator -= fixed_dt;
    }
    
    render();
}
```

**C++ Translation**:
```cpp
class GameLoop {
    float accumulator_ = 0.0f;
    const float kFixedDt = 1.0f / 60.0f;
    
public:
    void Run() {
        auto last_time = std::chrono::high_resolution_clock::now();
        
        while (running_) {
            auto current_time = std::chrono::high_resolution_clock::now();
            float frame_time = std::chrono::duration<float>(current_time - last_time).count();
            last_time = current_time;
            
            accumulator_ += frame_time;
            
            while (accumulator_ >= kFixedDt) {
                physics_world_->Step(kFixedDt);
                accumulator_ -= kFixedDt;
            }
            
            Render();
        }
    }
};
```

**Best Practices**:
- Use `std::chrono` for time (portable, type-safe)
- Clamp max frame time to prevent spiral of death
- Consider frame pacing for VSync

---

### Module 2: Vulkan Rendering Architecture (Week 2-3)

**See**: [Vulkan Rendering](../../concepts/vulkan-rendering.md)

#### Rendering Abstraction

**Praxis Approach** (Vulkano wrapper):
```rust
let render_context = RenderContext::new(device, queue, swapchain)?;

render_context.render(&RenderCommands {
    view: camera.view_matrix(),
    proj: camera.projection_matrix(),
    draw_commands: &[
        DrawCommand {
            mesh_id: "cube".into(),
            model: Mat4::IDENTITY,
            texture_name: Some("wall".into()),
            material_properties: None,
        },
    ],
    lighting: Some(&lights),
})?;
```

**C++ Equivalent** (Direct Vulkan):
```cpp
class RenderContext {
    VkDevice device_;
    VkQueue graphics_queue_;
    VkSwapchainKHR swapchain_;
    VkCommandPool command_pool_;
    
public:
    struct DrawCommand {
        std::string mesh_id;
        glm::mat4 model;
        std::optional<std::string> texture_name;
        std::optional<MaterialProperties> material;
    };
    
    struct RenderCommands {
        glm::mat4 view;
        glm::mat4 proj;
        std::span<const DrawCommand> draw_commands;
        const LightingUniforms* lighting;
    };
    
    void Render(const RenderCommands& commands) {
        VkCommandBuffer cmd_buffer = BeginFrame();
        
        // Begin render pass
        VkRenderPassBeginInfo pass_info = {};
        // ... setup
        vkCmdBeginRenderPass(cmd_buffer, &pass_info, VK_SUBPASS_CONTENTS_INLINE);
        
        // Bind pipeline
        vkCmdBindPipeline(cmd_buffer, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline_);
        
        // Upload uniforms
        UpdateUniformBuffer(commands.view, commands.proj, commands.lighting);
        
        // Draw commands
        for (const auto& draw : commands.draw_commands) {
            const Mesh& mesh = mesh_manager_->Get(draw.mesh_id);
            
            // Bind vertex/index buffers
            VkBuffer vertex_buffers[] = {mesh.vertex_buffer};
            VkDeviceSize offsets[] = {0};
            vkCmdBindVertexBuffers(cmd_buffer, 0, 1, vertex_buffers, offsets);
            vkCmdBindIndexBuffer(cmd_buffer, mesh.index_buffer, 0, VK_INDEX_TYPE_UINT32);
            
            // Bind descriptors
            vkCmdBindDescriptorSets(cmd_buffer, VK_PIPELINE_BIND_POINT_GRAPHICS,
                                   pipeline_layout_, 0, 1, &descriptor_set_, 0, nullptr);
            
            // Push constants (model matrix)
            vkCmdPushConstants(cmd_buffer, pipeline_layout_, VK_SHADER_STAGE_VERTEX_BIT,
                              0, sizeof(glm::mat4), &draw.model);
            
            // Draw
            vkCmdDrawIndexed(cmd_buffer, mesh.index_count, 1, 0, 0, 0);
        }
        
        vkCmdEndRenderPass(cmd_buffer);
        EndFrame(cmd_buffer);
    }
};
```

**Key Differences**:
- Rust: `vulkano` provides safety wrappers
- C++: Direct Vulkan API (verbose but explicit)
- Both: Same conceptual flow (command recording → submission → presentation)

**Best Practices**:
- Use RAII for Vulkan resources (`VkDevice`, `VkBuffer`, etc.)
- Wrap raw handles in `std::unique_ptr` with custom deleters
- Consider [Vulkan-HPP](https://github.com/KhronosGroup/Vulkan-Hpp) for C++ bindings
- Profile with RenderDoc/Nsight

**Exercise**:
1. Run `cargo run --example scene_demo` in Praxis
2. Study `praxis_graphics/src/render_context.rs`
3. Implement equivalent in C++ (use validation layers!)
4. Compare performance and code complexity

---

### Module 3: ECS Implementation (Week 4-5)

**See**: [ECS Architecture](../../concepts/ecs-architecture.md)

#### Archetype-Based ECS

**Praxis** (bevy_ecs):
```rust
#[derive(Component)]
struct Transform {
    position: Vec3,
    rotation: Quat,
}

#[derive(Component)]
struct Velocity {
    value: Vec3,
}

fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.position += velocity.value;
    }
}
```

**C++ Implementation** (EnTT-style):
```cpp
#include <entt/entt.hpp>

struct Transform {
    glm::vec3 position;
    glm::quat rotation;
};

struct Velocity {
    glm::vec3 value;
};

void movement_system(entt::registry& registry) {
    auto view = registry.view<Transform, Velocity>();
    
    for (auto entity : view) {
        auto& transform = view.get<Transform>(entity);
        const auto& velocity = view.get<Velocity>(entity);
        
        transform.position += velocity.value;
    }
}

// Usage
entt::registry registry;

// Spawn entity
auto entity = registry.create();
registry.emplace<Transform>(entity, glm::vec3(0.0f), glm::quat(1.0f, 0.0f, 0.0f, 0.0f));
registry.emplace<Velocity>(entity, glm::vec3(1.0f, 0.0f, 0.0f));

// Run system
movement_system(registry);
```

**Alternative: Custom ECS**:
```cpp
// Simplified archetype storage
template<typename... Components>
class Archetype {
    std::vector<std::tuple<Components...>> components_;
    std::vector<EntityId> entities_;
    
public:
    void Add(EntityId entity, Components... components) {
        entities_.push_back(entity);
        components_.emplace_back(std::move(components)...);
    }
    
    template<typename Func>
    void ForEach(Func&& func) {
        for (auto& tuple : components_) {
            std::apply(func, tuple);
        }
    }
};

// Usage
Archetype<Transform, Velocity> moving_entities;
moving_entities.Add(entity_id, Transform{}, Velocity{});

moving_entities.ForEach([](Transform& t, Velocity& v) {
    t.position += v.value;
});
```

**Key Considerations**:
- **EnTT**: Production-ready, header-only, fast
- **Custom**: Educational, full control, integrate with engine
- **Sparse Set** (EnTT default) vs **Archetype** (Flecs, bevy_ecs)

**Best Practices**:
- Use EnTT for rapid development
- Study EnTT source for custom implementations
- Benchmark with realistic entity counts (10k+)
- Profile cache misses (perf on Linux, VTune on Windows)

**Exercise**:
1. Install EnTT: `git clone https://github.com/skypjack/entt.git`
2. Port Praxis `movement_system` to C++ with EnTT
3. Spawn 10,000 entities, measure iteration time
4. Compare to Praxis performance

---

### Module 4: Transform Hierarchy Optimization (Week 6)

**See**: [Transform Hierarchy](../../concepts/transform-hierarchy.md)

#### Dirty Flag Propagation

**Praxis Approach**:
```rust
// Bevy's change detection
fn propagate_transforms(
    mut root_query: Query<
        (&Transform, &mut GlobalTransform, Option<&Children>),
        (Changed<Transform>, Without<Parent>) // Only changed roots
    >,
    // ...
) {
    // Process only dirty transforms
}
```

**C++ Pattern**:
```cpp
class TransformSystem {
    struct TransformNode {
        glm::mat4 local;
        glm::mat4 global;
        EntityId entity;
        std::vector<EntityId> children;
        bool dirty = true;
    };
    
    std::unordered_map<EntityId, TransformNode> nodes_;
    std::vector<EntityId> dirty_roots_;
    
public:
    void SetLocalTransform(EntityId entity, const glm::mat4& local) {
        auto& node = nodes_[entity];
        node.local = local;
        MarkDirty(entity);
    }
    
    void Update() {
        for (EntityId root : dirty_roots_) {
            UpdateRecursive(root, glm::mat4(1.0f));
        }
        dirty_roots_.clear();
    }
    
private:
    void MarkDirty(EntityId entity) {
        auto& node = nodes_[entity];
        if (!node.dirty) {
            node.dirty = true;
            
            // If root, add to dirty list
            if (IsRoot(entity)) {
                dirty_roots_.push_back(entity);
            }
            
            // Mark children dirty
            for (EntityId child : node.children) {
                MarkDirty(child);
            }
        }
    }
    
    void UpdateRecursive(EntityId entity, const glm::mat4& parent_global) {
        auto& node = nodes_[entity];
        if (!node.dirty) return;
        
        node.global = parent_global * node.local;
        node.dirty = false;
        
        for (EntityId child : node.children) {
            UpdateRecursive(child, node.global);
        }
    }
    
    bool IsRoot(EntityId entity) const {
        // Check if entity has Parent component
        // Implementation depends on ECS
        return true; // Placeholder
    }
};
```

**Optimization: Breadth-First Update**:
```cpp
void UpdateBreadthFirst() {
    std::queue<std::pair<EntityId, glm::mat4>> queue;
    
    // Enqueue roots
    for (EntityId root : dirty_roots_) {
        queue.push({root, glm::mat4(1.0f)});
    }
    
    while (!queue.empty()) {
        auto [entity, parent_global] = queue.front();
        queue.pop();
        
        auto& node = nodes_[entity];
        node.global = parent_global * node.local;
        node.dirty = false;
        
        for (EntityId child : node.children) {
            queue.push({child, node.global});
        }
    }
    
    dirty_roots_.clear();
}
```

**Best Practices**:
- Use dirty flags to avoid unnecessary updates
- Consider breadth-first for better cache locality
- Parallelize independent subtrees (job system)
- Profile: Is transform propagation a bottleneck?

---

### Module 5: Physics Integration (Week 7)

**See**: [Physics Guide](../../guides/physics.md)

#### Bidirectional Sync with Rapier/PhysX

**Praxis Pattern**:
```rust
// 1. Sync ECS → Physics (kinematic)
fn sync_to_physics(
    query: Query<(&Transform, &RigidBody), With<Kinematic>>,
    mut physics: ResMut<PhysicsWorld>,
) {
    for (transform, _) in query.iter() {
        physics.set_body_position(entity, transform.translation);
    }
}

// 2. Step physics
fn physics_step(mut physics: ResMut<PhysicsWorld>, time: Res<Time>) {
    physics.step(1.0 / 60.0);
}

// 3. Sync Physics → ECS (dynamic)
fn sync_from_physics(
    mut query: Query<(&mut Transform, &RigidBody), With<Dynamic>>,
    physics: Res<PhysicsWorld>,
) {
    for (mut transform, _) in query.iter_mut() {
        transform.translation = physics.get_body_position(entity);
    }
}
```

**C++ with PhysX**:
```cpp
#include <PxPhysicsAPI.h>

class PhysicsSystem {
    physx::PxPhysics* physics_;
    physx::PxScene* scene_;
    float accumulator_ = 0.0f;
    
public:
    void Update(float dt, entt::registry& registry) {
        // 1. Sync kinematic bodies
        auto kinematic_view = registry.view<Transform, RigidBody, KinematicTag>();
        for (auto entity : kinematic_view) {
            auto& transform = kinematic_view.get<Transform>(entity);
            auto& rb = kinematic_view.get<RigidBody>(entity);
            
            physx::PxRigidDynamic* actor = static_cast<physx::PxRigidDynamic*>(rb.actor);
            actor->setKinematicTarget(ToPxTransform(transform));
        }
        
        // 2. Fixed timestep physics
        const float kFixedDt = 1.0f / 60.0f;
        accumulator_ += dt;
        
        while (accumulator_ >= kFixedDt) {
            scene_->simulate(kFixedDt);
            scene_->fetchResults(true);
            accumulator_ -= kFixedDt;
        }
        
        // 3. Sync dynamic bodies
        auto dynamic_view = registry.view<Transform, RigidBody, DynamicTag>();
        for (auto entity : dynamic_view) {
            auto& transform = dynamic_view.get<Transform>(entity);
            auto& rb = dynamic_view.get<RigidBody>(entity);
            
            physx::PxRigidDynamic* actor = static_cast<physx::PxRigidDynamic*>(rb.actor);
            physx::PxTransform px_transform = actor->getGlobalPose();
            transform.position = ToGlmVec3(px_transform.p);
            transform.rotation = ToGlmQuat(px_transform.q);
        }
    }
    
private:
    physx::PxTransform ToPxTransform(const Transform& t) {
        return physx::PxTransform(
            physx::PxVec3(t.position.x, t.position.y, t.position.z),
            physx::PxQuat(t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w)
        );
    }
};
```

**Best Practices**:
- Always use fixed timestep for physics
- Sync kinematic TO physics, dynamic FROM physics
- Handle collision events via callbacks
- Consider continuous collision detection (CCD) for fast objects

---

### Module 7: Memory Allocators (Week 8-9)

**See**: [Memory Management Approaches](../../course/patterns/memory-management-approaches.md)

#### GPU Memory Management

**Praxis** (Vulkano allocator):
```rust
// Automatic allocation
let buffer = CpuAccessibleBuffer::from_iter(
    allocator.clone(),
    BufferUsage::vertex_buffer(),
    false,
    vertices.iter().cloned(),
)?;
```

**C++ Vulkan Memory Allocator (VMA)**:
```cpp
#include <vk_mem_alloc.h>

class GPUAllocator {
    VmaAllocator allocator_;
    
public:
    GPUAllocator(VkInstance instance, VkDevice device, VkPhysicalDevice physical_device) {
        VmaAllocatorCreateInfo info = {};
        info.vulkanApiVersion = VK_API_VERSION_1_2;
        info.instance = instance;
        info.device = device;
        info.physicalDevice = physical_device;
        
        vmaCreateAllocator(&info, &allocator_);
    }
    
    ~GPUAllocator() {
        vmaDestroyAllocator(allocator_);
    }
    
    struct Buffer {
        VkBuffer buffer;
        VmaAllocation allocation;
        
        void Destroy(VmaAllocator allocator) {
            vmaDestroyBuffer(allocator, buffer, allocation);
        }
    };
    
    Buffer CreateBuffer(VkDeviceSize size, VkBufferUsageFlags usage, VmaMemoryUsage memory_usage) {
        VkBufferCreateInfo buffer_info = {};
        buffer_info.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
        buffer_info.size = size;
        buffer_info.usage = usage;
        
        VmaAllocationCreateInfo alloc_info = {};
        alloc_info.usage = memory_usage;
        
        Buffer result;
        vmaCreateBuffer(allocator_, &buffer_info, &alloc_info, &result.buffer, &result.allocation, nullptr);
        return result;
    }
    
    void* Map(const Buffer& buffer) {
        void* data;
        vmaMapMemory(allocator_, buffer.allocation, &data);
        return data;
    }
    
    void Unmap(const Buffer& buffer) {
        vmaUnmapMemory(allocator_, buffer.allocation);
    }
};

// Usage
GPUAllocator allocator(instance, device, physical_device);

// Vertex buffer (device-local for performance)
auto vertex_buffer = allocator.CreateBuffer(
    vertices.size() * sizeof(Vertex),
    VK_BUFFER_USAGE_VERTEX_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT,
    VMA_MEMORY_USAGE_GPU_ONLY
);

// Staging buffer (host-visible for uploads)
auto staging_buffer = allocator.CreateBuffer(
    vertices.size() * sizeof(Vertex),
    VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
    VMA_MEMORY_USAGE_CPU_ONLY
);

// Upload data
void* data = allocator.Map(staging_buffer);
memcpy(data, vertices.data(), vertices.size() * sizeof(Vertex));
allocator.Unmap(staging_buffer);

// Copy staging → device
// ... vkCmdCopyBuffer ...

// Cleanup (use RAII wrapper in production)
vertex_buffer.Destroy(allocator.allocator_);
staging_buffer.Destroy(allocator.allocator_);
```

**Custom Allocators for CPU**:
```cpp
// Stack allocator (linear allocator)
class StackAllocator {
    std::byte* buffer_;
    size_t capacity_;
    size_t offset_ = 0;
    
public:
    StackAllocator(size_t capacity)
        : capacity_(capacity)
        , buffer_(new std::byte[capacity]) {}
    
    ~StackAllocator() {
        delete[] buffer_;
    }
    
    void* Allocate(size_t size, size_t alignment = alignof(std::max_align_t)) {
        size_t padding = (alignment - (offset_ % alignment)) % alignment;
        size_t aligned_offset = offset_ + padding;
        
        if (aligned_offset + size > capacity_) {
            return nullptr; // Out of memory
        }
        
        void* ptr = buffer_ + aligned_offset;
        offset_ = aligned_offset + size;
        return ptr;
    }
    
    void Reset() {
        offset_ = 0;
    }
};

// Per-frame allocator pattern
class FrameAllocator {
    static constexpr size_t kFrameCount = 3; // Triple buffering
    StackAllocator allocators_[kFrameCount];
    size_t current_frame_ = 0;
    
public:
    FrameAllocator(size_t per_frame_size)
        : allocators_{StackAllocator(per_frame_size),
                      StackAllocator(per_frame_size),
                      StackAllocator(per_frame_size)} {}
    
    void* Allocate(size_t size, size_t alignment = alignof(std::max_align_t)) {
        return allocators_[current_frame_].Allocate(size, alignment);
    }
    
    void NextFrame() {
        current_frame_ = (current_frame_ + 1) % kFrameCount;
        allocators_[current_frame_].Reset();
    }
};

// Usage
FrameAllocator frame_allocator(1024 * 1024); // 1 MB per frame

void Render() {
    // Allocate per-frame uniform data
    UniformData* uniforms = static_cast<UniformData*>(
        frame_allocator.Allocate(sizeof(UniformData))
    );
    // ... populate uniforms ...
    
    // Render...
}

void EndFrame() {
    frame_allocator.NextFrame(); // Reset previous frame's allocator
}
```

**Best Practices**:
- Use VMA for Vulkan GPU memory (handles fragmentation, defragmentation)
- Custom allocators for per-frame CPU data
- Profile with tools (Valgrind, Tracy, RADTelemetry)
- Consider memory budgets for console platforms

---

## Advanced Patterns

### Module 10: Multithreading and Job Systems (Week 10-11)

#### Task-Based Parallelism

**Praxis** (bevy_ecs parallelism):
```rust
// Automatic parallelization
fn parallel_system(query: Query<&mut Transform>) {
    query.par_iter_mut().for_each(|mut transform| {
        // Runs in parallel automatically
        transform.position += Vec3::X;
    });
}
```

**C++ Task System**:
```cpp
#include <taskflow/taskflow.hpp>

class JobSystem {
    tf::Executor executor_;
    
public:
    JobSystem(size_t num_threads = std::thread::hardware_concurrency())
        : executor_(num_threads) {}
    
    // Parallel for
    template<typename Func>
    void ParallelFor(size_t count, Func&& func) {
        tf::Taskflow taskflow;
        
        taskflow.for_each_index(size_t(0), count, size_t(1),
            [&](size_t i) { func(i); }
        );
        
        executor_.run(taskflow).wait();
    }
    
    // Task graph
    tf::Taskflow CreateTaskflow() {
        return tf::Taskflow();
    }
    
    void Run(tf::Taskflow& taskflow) {
        executor_.run(taskflow).wait();
    }
};

// Usage with ECS
void ParallelMovementSystem(entt::registry& registry, JobSystem& jobs) {
    auto view = registry.view<Transform, Velocity>();
    std::vector<entt::entity> entities(view.begin(), view.end());
    
    jobs.ParallelFor(entities.size(), [&](size_t i) {
        auto entity = entities[i];
        auto& transform = view.get<Transform>(entity);
        const auto& velocity = view.get<Velocity>(entity);
        
        transform.position += velocity.value;
    });
}

// Task graph
void TaskGraphExample(JobSystem& jobs) {
    tf::Taskflow taskflow = jobs.CreateTaskflow();
    
    auto A = taskflow.emplace([]{ /* Input system */ });
    auto B = taskflow.emplace([]{ /* Physics system */ });
    auto C = taskflow.emplace([]{ /* Animation system */ });
    auto D = taskflow.emplace([]{ /* Rendering system */ });
    
    A.precede(B, C);  // B and C depend on A (can run in parallel)
    D.succeed(B, C);  // D depends on B and C
    
    jobs.Run(taskflow);
}
```

**Alternative: EnTT Groups for Cache-Friendly Iteration**:
```cpp
// EnTT group for optimal iteration
auto group = registry.group<Transform>(entt::get<Velocity>);

// Parallel iteration with std::execution (C++17)
std::for_each(std::execution::par_unseq, group.begin(), group.end(),
    [&](auto entity) {
        auto& transform = group.get<Transform>(entity);
        const auto& velocity = group.get<Velocity>(entity);
        transform.position += velocity.value;
    }
);
```

**Best Practices**:
- Use Taskflow or similar for task graphs
- EnTT groups for optimal data layout
- Avoid false sharing (align data to cache lines)
- Profile with thread profilers (Tracy, Optick)

---

### Module 12: Networking Architecture (Week 12-13)

**See**: [Networking Guide](../../guides/systems/networking.md)

#### Entity Replication

**Praxis Networking**:
```rust
// Register replicated components
let mut registry = ReplicationRegistry::new();
registry.register_component::<Transform>();
registry.register_component::<Health>();

// Server: replicate entity
server.replicate_entity(entity_id, &components)?;

// Client: receive updates
fn client_replication_system(
    mut events: EventReader<ReplicationEvent>,
    mut query: Query<(&mut Transform, &mut Health)>,
) {
    // Handle incoming updates
}
```

**C++ Networking** (Custom):
```cpp
#include <enet/enet.h>

class NetworkReplicator {
public:
    // Component serialization
    template<typename T>
    void RegisterComponent() {
        component_serializers_[typeid(T)] = [](const void* component, std::vector<uint8_t>& buffer) {
            const T& comp = *static_cast<const T*>(component);
            size_t offset = buffer.size();
            buffer.resize(offset + sizeof(T));
            std::memcpy(buffer.data() + offset, &comp, sizeof(T));
        };
        
        component_deserializers_[typeid(T)] = [](const uint8_t* data, void* component) {
            T& comp = *static_cast<T*>(component);
            std::memcpy(&comp, data, sizeof(T));
        };
    }
    
    // Replicate entity (server → clients)
    void ReplicateEntity(entt::registry& registry, entt::entity entity, ENetPeer* peer) {
        std::vector<uint8_t> packet;
        
        // Packet header
        uint32_t entity_id = static_cast<uint32_t>(entity);
        packet.insert(packet.end(), reinterpret_cast<uint8_t*>(&entity_id),
                     reinterpret_cast<uint8_t*>(&entity_id) + sizeof(entity_id));
        
        // Serialize components
        if (registry.all_of<Transform>(entity)) {
            auto& transform = registry.get<Transform>(entity);
            SerializeComponent(typeid(Transform), &transform, packet);
        }
        
        if (registry.all_of<Health>(entity)) {
            auto& health = registry.get<Health>(entity);
            SerializeComponent(typeid(Health), &health, packet);
        }
        
        // Send packet
        ENetPacket* enet_packet = enet_packet_create(packet.data(), packet.size(),
                                                      ENET_PACKET_FLAG_RELIABLE);
        enet_peer_send(peer, 0, enet_packet);
    }
    
    // Client receives update
    void OnReplicationPacket(entt::registry& registry, const uint8_t* data, size_t size) {
        // Parse entity ID
        uint32_t entity_id;
        std::memcpy(&entity_id, data, sizeof(entity_id));
        data += sizeof(entity_id);
        size -= sizeof(entity_id);
        
        entt::entity entity = static_cast<entt::entity>(entity_id);
        
        // Ensure entity exists
        if (!registry.valid(entity)) {
            entity = registry.create(entity);
        }
        
        // Deserialize components
        // (Simplified: would need component type info in packet)
        if (size >= sizeof(Transform)) {
            Transform transform;
            DeserializeComponent(typeid(Transform), data, &transform);
            registry.emplace_or_replace<Transform>(entity, transform);
            data += sizeof(Transform);
            size -= sizeof(Transform);
        }
    }
    
private:
    std::unordered_map<std::type_index, std::function<void(const void*, std::vector<uint8_t>&)>> component_serializers_;
    std::unordered_map<std::type_index, std::function<void(const uint8_t*, void*)>> component_deserializers_;
    
    void SerializeComponent(std::type_index type, const void* component, std::vector<uint8_t>& buffer) {
        component_serializers_[type](component, buffer);
    }
    
    void DeserializeComponent(std::type_index type, const uint8_t* data, void* component) {
        component_deserializers_[type](data, component);
    }
};

// Usage
NetworkReplicator replicator;
replicator.RegisterComponent<Transform>();
replicator.RegisterComponent<Health>();

// Server
replicator.ReplicateEntity(registry, player_entity, client_peer);

// Client
replicator.OnReplicationPacket(registry, packet_data, packet_size);
```

**Best Practices**:
- Use reliable ordered for critical data (health, inventory)
- Use unreliable for frequent updates (position, rotation)
- Delta compression for bandwidth savings
- Client prediction + server reconciliation for responsiveness

---

## Practical Projects

### Project 1: Minimal Vulkan Renderer
**Time**: 2-3 weeks  
**Difficulty**: Intermediate

**Goals**:
1. Initialize Vulkan (instance, device, swapchain)
2. Load vertex/index buffers
3. Compile shaders (GLSL → SPIR-V)
4. Render triangle, then textured cube

**C++ Libraries**:
- Vulkan SDK
- GLFW (windowing)
- GLM (math)
- stb_image (texture loading)

**Compare to**: `cargo run --example hello_triangle`

### Project 2: ECS with Transform Hierarchy
**Time**: 1-2 weeks  
**Difficulty**: Intermediate

**Goals**:
1. Integrate EnTT
2. Implement Transform + Parent + Children components
3. Propagate transforms recursively
4. Test with 1,000+ entity hierarchy

**Compare to**: Praxis transform system performance

### Project 3: Physics Integration
**Time**: 2-3 weeks  
**Difficulty**: Advanced

**Goals**:
1. Integrate PhysX or Jolt Physics
2. Implement bidirectional sync (ECS ↔ Physics)
3. Fixed timestep physics loop
4. Collision events

**Compare to**: Praxis `cargo run --example ecs_integration`

### Project 4: Scripting with Lua
**Time**: 1-2 weeks  
**Difficulty**: Intermediate

**Goals**:
1. Embed Lua (sol2 or LuaBridge)
2. Expose ECS to Lua
3. Hot-reload script files
4. Call Lua functions from C++

**Compare to**: `cargo run --example scripting_demo`

---

## C++ vs Rust: When to Use Each

### Use C++ When:
- ✅ Platform requires it (consoles, legacy codebases)
- ✅ Need specific libraries (PhysX, Wwise, Havok)
- ✅ Team expertise in C++
- ✅ Maximum control and no runtime overhead

### Use Rust When:
- ✅ Greenfield project (new engine)
- ✅ Safety is critical (modding, server-side)
- ✅ Fearless concurrency needed
- ✅ Faster iteration with compile-time guarantees

### Hybrid Approach:
- Performance-critical: C++ (physics, rendering)
- Tools and editor: Rust (safer, faster development)
- Scripting: Lua/Python (rapid iteration)

---

## Common Pitfalls

### 1. Memory Leaks
**Problem**: Forget to delete or mismatched new/delete  
**Solution**: Use smart pointers exclusively, RAII for all resources

### 2. Data Races
**Problem**: Multiple threads modify same data  
**Solution**: Mutex for shared data, prefer immutable patterns

### 3. Undefined Behavior
**Problem**: Dangling pointers, use-after-free, buffer overruns  
**Solution**: Sanitizers (ASan, UBSan), static analysis (clang-tidy), Valgrind

### 4. Template Bloat
**Problem**: Binary size explosion from templates  
**Solution**: Use type erasure, minimize template instantiations

### 5. Build Times
**Problem**: Slow compilation with heavy templates  
**Solution**: Precompiled headers, unity builds, distributed compilation (ccache, distcc)

---

## Recommended Study Order

### 4-Week C++ Engine Bootcamp
```
Week 1: Vulkan rendering + GLFW integration
Week 2: ECS with EnTT + transform hierarchy
Week 3: Physics (PhysX) + fixed timestep
Week 4: Asset loading + scripting (Lua)
```

### 12-Week Engine Development
```
Weeks 1-3: Rendering (Vulkan, deferred, shadows)
Weeks 4-5: ECS architecture + optimization
Weeks 6-7: Transform + physics + animation
Weeks 8-9: Memory management + multithreading
Weeks 10-11: Asset pipeline + scripting
Week 12: Networking basics
```

**Parallel Study**: Follow [Course Curriculum](../CURRICULUM.md) modules alongside C++ implementations.

---

## Resources

### C++ Libraries
- **ECS**: [EnTT](https://github.com/skypjack/entt), [Flecs](https://github.com/SanderMertens/flecs)
- **Rendering**: Vulkan SDK, [Vulkan-HPP](https://github.com/KhronosGroup/Vulkan-Hpp)
- **Physics**: [PhysX](https://github.com/NVIDIAGameWorks/PhysX), [Jolt](https://github.com/jrouwe/JoltPhysics)
- **Math**: [GLM](https://github.com/g-truc/glm), [DirectXMath](https://github.com/microsoft/DirectXMath)
- **Multithreading**: [Taskflow](https://github.com/taskflow/taskflow), [enkiTS](https://github.com/dougbinks/enkiTS)
- **Scripting**: [sol2](https://github.com/ThePhD/sol2), [LuaBridge](https://github.com/vinniefalco/LuaBridge)
- **Memory**: [VMA](https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator), [mimalloc](https://github.com/microsoft/mimalloc)

### Learning Resources
- [Praxis Curriculum](../CURRICULUM.md) - Universal concepts
- [Vulkan Tutorial](https://vulkan-tutorial.com/) - Step-by-step Vulkan
- [Game Engine Architecture](https://www.gameenginebook.com/) by Jason Gregory
- [Foundations of Game Engine Development](https://foundationsofgameenginedev.com/) series

### Tooling
- **Profilers**: Tracy, Optick, Intel VTune, AMD μProf
- **Debuggers**: RenderDoc (graphics), Nsight (NVIDIA), RAD Debugger
- **Static Analysis**: clang-tidy, cppcheck, PVS-Studio
- **Sanitizers**: AddressSanitizer, UndefinedBehaviorSanitizer, ThreadSanitizer

---

## Next Steps

After this path:
- ✅ Understand modern C++ engine architecture
- ✅ Can integrate Vulkan/DX12 rendering
- ✅ Know ECS implementation patterns
- ✅ Understand physics/scripting/networking integration
- ✅ Can optimize for performance and memory

**Where next?**
1. Build custom engine in C++ with learned patterns
2. Contribute to open-source engines (Godot, O3DE)
3. Explore Rust for safer engine development
4. Specialize in rendering, physics, or tools
5. Join [For Rust Developers](rust-developers.md) path for deep Praxis dive

**Related Paths**:
- [For Unity Developers](unity-developers.md) - C# to engine concepts
- [For Rust Developers](rust-developers.md) - Praxis-specific patterns
