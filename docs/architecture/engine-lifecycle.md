# Engine Lifecycle

This document provides a detailed overview of the Praxis engine's lifecycle, from initialization through frame execution to shutdown. Understanding the engine lifecycle is essential for integrating systems, managing resources, and optimizing performance.

## Overview

The Praxis engine follows a structured initialization and execution pattern that sets up all core systems before entering the main event loop. The lifecycle consists of three main phases:

1. **Initialization**: System setup and resource allocation
2. **Main Loop**: Event processing and frame rendering
3. **Shutdown**: Resource cleanup and graceful termination

## Initialization Phase

### Entry Point

The engine entry point is `praxis_core::run()`, which orchestrates the initialization of all subsystems:

```rust
pub fn run() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;
    praxis_window::run()?;
    
    Ok(())
}
```

### Initialization Sequence

The initialization follows a specific order to ensure dependencies are satisfied:

#### 1. Utilities Initialization (`praxis_utils::init()`)

- Configures the logging system using `tracing` and `tracing-subscriber`
- Sets up error reporting with `color-eyre`
- Initializes timing utilities and frame counters
- Establishes console output formatting

**Purpose**: Provides foundational infrastructure for logging, error handling, and diagnostics that all other systems depend on.

#### 2. ECS Initialization (`praxis_ecs::init()`)

- Prepares the Entity-Component-System runtime
- Currently performs minimal setup as `bevy_ecs` is largely self-contained
- Reserves space for future global ECS configuration

**Purpose**: While `bevy_ecs` doesn't require explicit initialization, this entry point allows for future engine-specific ECS setup such as component registration or default resources.

#### 3. Input System Initialization (`praxis_input::init()`)

- Initializes input state tracking structures
- Prepares keyboard, mouse, and gamepad input handlers
- Sets up input event buffering

**Purpose**: Ensures input systems are ready to receive events from the windowing system.

#### 4. Audio System Initialization (`praxis_audio::init()`)

- Initializes the Kira audio backend
- Creates the audio manager and main audio track
- Prepares 3D spatial audio systems

**Purpose**: Sets up audio playback infrastructure before window creation to avoid audio device conflicts.

#### 5. Window and Graphics Initialization (`praxis_window::run()`)

This is the most complex initialization step, creating the window and graphics context:

```rust
pub fn run() -> Result<()> {
    info!("Starting Praxis application");
    
    let mut app = App::default();
    let event_loop = EventLoop::new()?;
    
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
```

The window initialization triggers graphics context creation in the `resumed` event:

```rust
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                Window::default_attributes()
                    .with_inner_size(PhysicalSize::new(1920, 1080))
                    .with_title("In Praxis")
                    .with_resizable(true)
            )?
        );
        
        let state = State::new(window).await?;
        self.state = Some(state);
    }
}
```

### Graphics Context Initialization

The `RenderContext::new()` call performs the Vulkan initialization sequence:

```rust
pub async fn new(window: Arc<Window>) -> Result<Self> {
    // 1. Vulkan Instance and Device
    let (vulkan_device, surface) = VulkanDevice::new(&window)?;
    
    // 2. Swapchain Creation
    let (swapchain, swapchain_images) = 
        Self::create_swapchain(&device, &physical_device, &surface, &window)?;
    
    // 3. Image Views
    let swapchain_image_views = swapchain_images
        .iter()
        .map(|image| ImageView::new_default(image.clone()))
        .collect()?;
    
    // 4. Render Pass
    let render_pass = Self::create_render_pass(&device, swapchain.image_format())?;
    
    // 5. Framebuffers
    let framebuffers = Self::create_framebuffers(&swapchain_image_views, &render_pass)?;
    
    // 6. Command Buffer Allocator
    let command_buffer_allocator = Arc::new(
        StandardCommandBufferAllocator::new(device.clone(), Default::default())
    );
    
    // 7. Memory Allocator
    let memory_allocator = Arc::new(
        StandardMemoryAllocator::new_default(device.clone())
    );
    
    // 8. Graphics Pipeline
    let graphics_pipeline = create_simple_pipeline_3d(&device, &render_pass, extent)?;
    
    // 9. Descriptor Set Allocator
    let descriptor_set_allocator = Arc::new(
        StandardDescriptorSetAllocator::new(device.clone(), Default::default())
    );
    
    // 10. Resource Managers
    let mesh_manager = MeshAssetManager::new(memory_allocator.clone());
    let texture_manager = TextureManager::new(
        memory_allocator.clone(),
        command_buffer_allocator.clone(),
        graphics_queue.clone()
    );
    let material_manager = MaterialManager::new();
    
    // 11. Default Resources
    texture_manager.create_default_white_texture()?;
    texture_manager.create_default_flat_normal()?;
    
    // 12. Uniform Buffers
    let lighting_buffer = LightingUniformBuffer::new(memory_allocator.clone())?;
    let dynamic_uniform_buffer = DynamicUniformBuffer::new(&device, memory_allocator.clone(), 3, 1024)?;
    let view_proj_buffer = Buffer::from_data(...)?;
    
    Ok(Self { /* ... */ })
}
```

**Key Points**:
- Initialization is **async** to support future asynchronous asset loading
- Uses **Arc<T>** for shared ownership of Vulkan resources
- Creates **default resources** (white texture, flat normal map) during initialization
- Allocates **pools and buffers** with capacity for 1024 objects per frame

### Initialization Timing

The engine logs initialization timings for performance analysis:

```
[INFO] Initializing ECS system
[DEBUG] ECS system initialized successfully
[INFO] Initializing graphics context...
[DEBUG] Creating Vulkan device and surface
[DEBUG] Vulkan device created in 45ms
[DEBUG] Creating swapchain
[INFO] Created swapchain with 3 images at 1920x1080 in 12ms
[INFO] Graphics context initialization complete in 89ms
[INFO] Starting event loop (initialized in 156ms)
```

## Main Loop Phase

### Event Loop Architecture

The engine uses `winit`'s `ApplicationHandler` trait for the main event loop:

```rust
impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                // Frame execution
                self.render_frame();
            }
            WindowEvent::Resized(size) => {
                // Handle window resize
                self.handle_resize(size);
            }
            WindowEvent::CloseRequested => {
                // Cleanup and exit
                event_loop.exit();
            }
            _ => {}
        }
    }
}
```

### Frame Execution Cycle

Each frame follows a consistent execution pattern:

```
┌─────────────────────────────────────────┐
│         Frame Timing & Stats             │
│  - Calculate delta time                  │
│  - Update FPS counter                    │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         Resize Debouncing                │
│  - Check for pending resize              │
│  - Wait for debounce period (16ms)       │
│  - Trigger swapchain recreation          │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         ECS Systems Update               │
│  - Update transforms                     │
│  - Update cameras                        │
│  - Gather lighting                       │
│  - Update physics                        │
│  - Update animations                     │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         Render Preparation               │
│  - Sort draw commands by material        │
│  - Update uniform buffers                │
│  - Update lighting data                  │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         Rendering Execution              │
│  - Acquire swapchain image               │
│  - Record command buffer                 │
│  - Submit to GPU                         │
│  - Present frame                         │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│         Cleanup & Next Frame             │
│  - Cleanup finished GPU work             │
│  - Advance dynamic buffer frame          │
│  - Request next redraw                   │
└─────────────────────────────────────────┘
```

### Frame Timing

The engine tracks frame timing for performance monitoring:

```rust
pub struct FrameTimer {
    last_frame: Instant,
    delta: Duration,
    fps_samples: VecDeque<Duration>,
}

impl FrameTimer {
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        self.delta = now - self.last_frame;
        self.last_frame = now;
        
        self.fps_samples.push_back(self.delta);
        if self.fps_samples.len() > 60 {
            self.fps_samples.pop_front();
        }
        
        self.delta
    }
    
    pub fn fps(&self) -> f64 {
        let avg = self.fps_samples.iter().sum::<Duration>() / self.fps_samples.len() as u32;
        1.0 / avg.as_secs_f64()
    }
}
```

### Resize Handling

Window resizing is debounced to prevent excessive swapchain recreation:

```rust
fn resize(&mut self, new_size: PhysicalSize<u32>) {
    // Debounce resizes with 16ms delay (~1 frame at 60fps)
    const DEBOUNCE_DURATION: Duration = Duration::from_millis(16);
    
    if let Some((pending_size, resize_time)) = self.pending_resize {
        if resize_time.elapsed() >= DEBOUNCE_DURATION {
            // Execute the resize
            self.render_context.configure_surface(new_size.width, new_size.height);
            self.pending_resize = None;
        }
    } else {
        // Start debounce timer
        self.pending_resize = Some((new_size, Instant::now()));
    }
}
```

**Why Debouncing?**
- Windows can generate many resize events during a drag operation
- Swapchain recreation is expensive (~50-100ms)
- Debouncing coalesces rapid resizes into a single recreation
- Improves responsiveness during window manipulation

### Rendering Pipeline

The `RenderContext::render()` method orchestrates frame rendering:

```rust
pub fn render(&mut self, cmds: &RenderCommands) -> Result<()> {
    // 1. Frame timing
    let delta = self.frame_timer.tick();
    
    // 2. Handle swapchain recreation if needed
    if self.recreate_swapchain {
        self.recreate_swapchain_and_framebuffers()?;
        self.recreate_swapchain = false;
    }
    
    // 3. Cleanup previous frame
    previous_frame_end.cleanup_finished();
    
    // 4. Advance dynamic uniform buffer
    self.dynamic_uniform_buffer.next_frame();
    
    // 5. Update lighting if provided
    if let Some(lighting) = cmds.lighting {
        self.lighting_buffer.update(lighting)?;
    }
    
    // 6. Update view/projection uniforms
    let view_proj_uniforms = ViewProjectionUniforms::new(cmds.view, cmds.proj);
    *self.view_proj_buffer.write()? = view_proj_uniforms;
    
    // 7. Sort draw commands by material for batching
    let mut indexed_commands: Vec<_> = cmds.draw_commands.iter().enumerate().collect();
    indexed_commands.sort_by(|(_, a), (_, b)| {
        // Sort by texture, then material properties
        // ...
    });
    
    // 8. Update model matrices
    let model_matrices: Vec<Mat4> = indexed_commands
        .iter()
        .map(|(_, cmd)| cmd.model)
        .collect();
    self.dynamic_uniform_buffer.write_models(&model_matrices)?;
    
    // 9. Acquire swapchain image
    let (image_index, suboptimal, acquire_future) = 
        acquire_next_image(self.swapchain.clone(), None)?;
    
    // 10. Build command buffer
    let mut builder = AutoCommandBufferBuilder::primary(...)?;
    
    builder.begin_render_pass(...)?
           .bind_pipeline_graphics(self.graphics_pipeline.clone())?
           .set_viewport(0, [self.viewport.clone()])?;
    
    // 11. Record draw commands with descriptor set reuse
    for (transform_set, material_set, mesh, object_index) in draw_list {
        builder.bind_vertex_buffers(0, mesh.vertex_buffer.clone())?
               .bind_index_buffer(mesh.index_buffer.clone())?;
        
        let dynamic_offset = self.dynamic_uniform_buffer.get_dynamic_offset(object_index);
        
        builder.bind_descriptor_sets_unchecked(
            PipelineBindPoint::Graphics,
            self.graphics_pipeline.layout().clone(),
            0,
            DescriptorSetWithOffsets::new(transform_set, [dynamic_offset])
        );
        
        // Only bind material set if it changed
        if material_changed {
            builder.bind_descriptor_sets(..., material_set)?;
        }
        
        builder.draw_indexed(mesh.index_count, 1, 0, 0, 0)?;
    }
    
    builder.end_render_pass()?;
    
    let command_buffer = builder.build()?;
    
    // 12. Submit and present
    let execution = previous_frame_end
        .join(acquire_future)
        .then_execute(self.graphics_queue.clone(), command_buffer)?;
    
    let future = execution
        .then_swapchain_present(self.present_queue.clone(), ...)
        .then_signal_fence_and_flush()?;
    
    self.previous_frame_end = Some(future.boxed());
    
    Ok(())
}
```

### Frame Synchronization

The engine uses Vulkan fences and semaphores for GPU synchronization:

- **Acquire Semaphore**: Signals when swapchain image is available
- **Render Semaphore**: Signals when rendering is complete
- **Fence**: CPU waits for GPU work to finish before reusing resources

```rust
// Chain futures for proper synchronization
previous_frame_end      // Wait for previous frame
    .join(acquire_future)  // Wait for image acquisition
    .then_execute(queue, cmd_buffer)  // Submit rendering work
    .then_swapchain_present(queue, ...)  // Present to screen
    .then_signal_fence_and_flush()  // Signal completion
```

### Resource Lifecycle

**Per-Frame Resources**:
- Command buffers (allocated per frame)
- Dynamic uniform buffer offsets (rotates every 3 frames)
- Descriptor sets (created per draw command, auto-freed)

**Persistent Resources**:
- Swapchain images (recreated on resize)
- Pipelines (recreated on swapchain format change)
- Meshes (lifetime managed by asset manager)
- Textures (lifetime managed by texture manager)
- Materials (lifetime managed by material manager)

## Shutdown Phase

### Graceful Shutdown

Shutdown is triggered by window close or Escape key:

```rust
WindowEvent::CloseRequested => {
    info!("Close requested, exiting event loop...");
    event_loop.exit();
}
```

### Cleanup Sequence

When the event loop exits, Rust's RAII ensures proper cleanup:

1. **Drop State**: Window state is dropped, triggering cleanup
2. **Flush GPU**: `previous_frame_end.flush()` waits for GPU completion
3. **Drop RenderContext**: Graphics resources are released in reverse order
4. **Drop Allocators**: Memory and descriptor allocators are freed
5. **Drop Device**: Logical device is destroyed
6. **Drop Instance**: Vulkan instance is destroyed

```rust
impl Drop for RenderContext {
    fn drop(&mut self) {
        // Explicit cleanup if needed
        if let Some(mut previous) = self.previous_frame_end.take() {
            let _ = previous.flush();
        }
        
        // Automatic cleanup via Drop impls:
        // - Command buffers freed
        // - Descriptor sets freed
        // - Buffers unmapped and freed
        // - Images destroyed
        // - Pipeline destroyed
        // - Swapchain destroyed
        // - Surface destroyed
        // - Device destroyed
        // - Instance destroyed
    }
}
```

### Resource Cleanup Order

**Correct order** (enforced by Rust's drop order):
1. Command buffers
2. Descriptor sets
3. Framebuffers
4. Render pass
5. Pipeline
6. Image views
7. Images
8. Buffers
9. Swapchain
10. Allocators
11. Surface
12. Device
13. Instance

**Why Order Matters**:
- Child resources must be destroyed before parents
- Vulkan validates destruction order
- Incorrect order causes validation layer errors
- Memory leaks can occur if order is wrong

## Performance Considerations

### Initialization Optimization

**Parallel Initialization** (future enhancement):
```rust
// Current: Sequential
praxis_utils::init()?;
praxis_ecs::init()?;
praxis_input::init()?;
praxis_audio::init()?;

// Future: Parallel where possible
let (utils_result, ecs_result, input_result) = join!(
    praxis_utils::init(),
    praxis_ecs::init(),
    praxis_input::init(),
);
```

### Frame Pacing

The engine uses `ControlFlow::Poll` for maximum frame rate:

```rust
event_loop.set_control_flow(ControlFlow::Poll);
```

**Alternative modes**:
- `ControlFlow::Wait`: Wait for events (lower CPU usage)
- `ControlFlow::WaitUntil(time)`: V-Sync or fixed timestep

### GPU Utilization

**Triple Buffering** (default: 3 frames in flight):
- CPU prepares frame N+1 while GPU renders frame N
- Maximizes GPU utilization
- Reduces input latency compared to more buffering

**Dynamic Uniform Buffers**:
- Pre-allocate buffer for 1024 objects
- Rotate between 3 frame buffers
- Avoids per-frame allocation overhead

### Memory Management

**Vulkan Memory Allocator**:
- Pools allocations to reduce fragmentation
- Aligns allocations for optimal GPU access
- Automatically selects appropriate memory types

**Resource Pools**:
- Command buffer pool (per thread)
- Descriptor set pool (shared)
- Render target pool (for post-processing)

## Common Patterns

### Extending Initialization

To add a new system to initialization:

```rust
pub fn run() -> Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;
    
    // Add your system here
    my_system::init()?;
    
    praxis_window::run()?;
    Ok(())
}
```

**Considerations**:
- Initialize after dependencies
- Log initialization status
- Return errors via `Result<()>`
- Keep initialization fast (<100ms per system)

### Custom Application Loop

For advanced use cases, implement `ApplicationHandler` directly:

```rust
struct MyApp {
    state: Option<MyState>,
}

impl ApplicationHandler for MyApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window and initialize
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                // Custom frame logic
                self.update();
                self.render();
            }
            _ => {}
        }
    }
}
```

### Resource Pre-loading

Load assets during initialization to avoid frame hitches:

```rust
impl State {
    async fn new(window: Arc<Window>) -> Result<Self> {
        let mut render_context = RenderContext::new(window.clone()).await?;
        
        // Pre-load common assets
        render_context.mesh_manager_mut().load_mesh("cube", colored_cube_mesh())?;
        render_context.mesh_manager_mut().load_mesh("sphere", sphere_mesh(32, 32))?;
        
        render_context.texture_manager_mut().load_texture_from_file("logo", "assets/logo.png")?;
        
        Ok(State { render_context, /* ... */ })
    }
}
```

## Summary

The Praxis engine lifecycle is designed for:

- **Predictable Initialization**: Clear dependency order
- **High Performance**: Optimized main loop with minimal overhead
- **Graceful Shutdown**: Proper resource cleanup via RAII
- **Flexibility**: Easy to extend with new systems
- **Debuggability**: Comprehensive logging at every stage

Understanding the lifecycle is key to building efficient games and tools with Praxis. The structured approach ensures resources are properly managed, synchronization is correct, and performance is optimal throughout the application's lifetime.
