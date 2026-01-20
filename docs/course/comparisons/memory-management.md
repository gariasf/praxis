# Memory Management: Multi-Engine Comparison

**Complexity**: Advanced  
**Curriculum Module**: [Module 7 - Memory Management Patterns](../modules/07-memory-management-patterns.md)

## Problem Statement

Game engines must efficiently manage both CPU and GPU memory. Key challenges:

- How do we allocate GPU memory (buffers, textures, render targets)?
- How do we minimize allocation overhead and fragmentation?
- How do we handle resource lifetime and prevent leaks?
- How do we optimize for cache performance?
- How do we manage memory across different platforms (PC, console, mobile)?

## Design Philosophy Comparison

| Engine | CPU Memory | GPU Memory | Lifetime Management |
|--------|------------|-----------|---------------------|
| **Unity** | Managed (C# GC) | Automatic allocation | Reference counting + GC |
| **Unreal** | Manual (C++) + GC | Automatic pooling | Garbage collection (UObject) |
| **Godot** | Reference counting | Automatic (RenderingDevice) | RefCounted base class |
| **Praxis** | Manual (Rust) | Explicit (Vulkan allocators) | Arc/ownership system |

## Implementation Examples

### GPU Buffer Allocation

#### Unity (C#)

```csharp
using UnityEngine;
using UnityEngine.Rendering;

public class BufferExample : MonoBehaviour
{
    private ComputeBuffer vertexBuffer;
    private ComputeBuffer indexBuffer;
    
    void Start()
    {
        // Allocate GPU buffer (automatic memory management)
        int vertexCount = 1000;
        int stride = sizeof(float) * 8;  // Position (3) + Normal (3) + UV (2)
        
        vertexBuffer = new ComputeBuffer(vertexCount, stride, ComputeBufferType.Default);
        
        // Upload data
        Vector3[] vertices = new Vector3[vertexCount];
        // ... fill vertices
        vertexBuffer.SetData(vertices);
        
        // Index buffer
        indexBuffer = new ComputeBuffer(3000, sizeof(int), ComputeBufferType.Index);
    }
    
    void OnDestroy()
    {
        // Manual release (important! GC doesn't know about GPU memory)
        vertexBuffer?.Release();
        indexBuffer?.Release();
    }
    
    // Texture allocation
    void CreateTexture()
    {
        // RenderTexture (GPU only)
        RenderTexture rt = new RenderTexture(1024, 1024, 24, RenderTextureFormat.ARGB32);
        rt.Create();
        
        // Texture2D (CPU+GPU, managed)
        Texture2D tex = new Texture2D(512, 512, TextureFormat.RGBA32, false);
        byte[] pixels = new byte[512 * 512 * 4];
        tex.LoadRawTextureData(pixels);
        tex.Apply();  // Upload to GPU
        
        // Release
        rt.Release();
        Destroy(tex);  // GC will eventually collect
    }
}

// Memory profiler usage
void AnalyzeMemory()
{
    // Unity Profiler shows:
    // - GC allocations per frame
    // - Managed heap size
    // - GPU memory usage (textures, buffers, render targets)
}
```

#### Unreal (C++)

```cpp
#include "RenderResource.h"
#include "RHI.h"

// Vertex buffer
class FMyVertexBuffer : public FVertexBuffer
{
public:
    virtual void InitRHI() override
    {
        // Allocate GPU memory
        FRHIResourceCreateInfo CreateInfo(TEXT("MyVertexBuffer"));
        VertexBufferRHI = RHICreateVertexBuffer(
            NumVertices * sizeof(FVertex),
            BUF_Static,  // Usage hint
            CreateInfo
        );
        
        // Upload data
        void* BufferData = RHILockBuffer(VertexBufferRHI, 0, NumVertices * sizeof(FVertex), RLM_WriteOnly);
        FMemory::Memcpy(BufferData, Vertices.GetData(), NumVertices * sizeof(FVertex));
        RHIUnlockBuffer(VertexBufferRHI);
    }
    
    virtual void ReleaseRHI() override
    {
        VertexBufferRHI.SafeRelease();
        FVertexBuffer::ReleaseRHI();
    }
    
private:
    TArray<FVertex> Vertices;
    int32 NumVertices;
};

// Texture allocation
UTexture2D* CreateTexture()
{
    UTexture2D* Texture = UTexture2D::CreateTransient(512, 512, PF_R8G8B8A8);
    
    // Lock mip for writing
    FTexture2DMipMap& Mip = Texture->GetPlatformData()->Mips[0];
    void* Data = Mip.BulkData.Lock(LOCK_READ_WRITE);
    
    // Write pixels
    FMemory::Memset(Data, 0, 512 * 512 * 4);
    
    Mip.BulkData.Unlock();
    Texture->UpdateResource();
    
    return Texture;  // UObject, garbage collected
}

// Memory pools (Unreal internal)
// - FMemory::Malloc() uses platform allocator
// - GMalloc is global allocator
// - Binned allocator for small objects
// - GPU resources use RHI allocator
```

#### Godot (GDScript)

```gdscript
# Godot 4.x RenderingDevice for low-level GPU access
var rd = RenderingServer.get_rendering_device()

func create_vertex_buffer():
    # Vertex data
    var vertices = PackedVector3Array([
        Vector3(0, 0, 0),
        Vector3(1, 0, 0),
        Vector3(0, 1, 0)
    ])
    
    # Convert to bytes
    var vertex_bytes = vertices.to_byte_array()
    
    # Create GPU buffer
    var buffer = rd.storage_buffer_create(vertex_bytes.size(), vertex_bytes)
    
    return buffer

func create_texture():
    # Texture data
    var img = Image.create(512, 512, false, Image.FORMAT_RGBA8)
    img.fill(Color.RED)
    
    # Create GPU texture
    var texture_format = RDTextureFormat.new()
    texture_format.width = 512
    texture_format.height = 512
    texture_format.format = RenderingDevice.DATA_FORMAT_R8G8B8A8_UNORM
    texture_format.usage_bits = RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT | \
                                 RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
    
    var texture = rd.texture_create(texture_format, RDTextureView.new(), [img.get_data()])
    
    return texture

func cleanup():
    # Free GPU resources
    rd.free_rid(buffer)
    rd.free_rid(texture)

# High-level (automatic memory management)
func simple_texture():
    var texture = ImageTexture.create_from_image(Image.create(512, 512, false, Image.FORMAT_RGBA8))
    # RefCounted, automatically freed when no references
```

#### Praxis (Rust)

```rust
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryAllocator, StandardMemoryAllocator};
use vulkano::image::{Image, ImageCreateInfo, ImageUsage};

// Vertex buffer allocation
fn create_vertex_buffer(
    allocator: &StandardMemoryAllocator,
) -> Arc<Buffer<[Vertex]>> {
    // Allocate GPU buffer
    Buffer::from_iter(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,  // VRAM
            ..Default::default()
        },
        vertices.iter().cloned(),
    ).unwrap()
}

// Staging buffer (CPU visible, GPU accessible)
fn create_staging_buffer(
    allocator: &StandardMemoryAllocator,
    data: &[u8],
) -> Arc<Buffer<[u8]>> {
    Buffer::from_iter(
        allocator,
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST 
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,  // CPU RAM
            ..Default::default()
        },
        data.iter().cloned(),
    ).unwrap()
}

// Texture allocation
fn create_texture(
    allocator: &StandardMemoryAllocator,
) -> Arc<Image> {
    Image::new(
        allocator,
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_SRGB,
            extent: [512, 512, 1],
            usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
    ).unwrap()
}

// Memory usage tracking
fn get_memory_usage(allocator: &StandardMemoryAllocator) {
    let stats = allocator.memory_usage();
    println!("GPU memory: {} MB", stats.used_device_memory / 1_000_000);
    println!("CPU memory: {} MB", stats.used_host_memory / 1_000_000);
}

// Lifetime management via Arc (reference counting)
let buffer = create_vertex_buffer(&allocator);
let buffer_clone = buffer.clone();  // Increment ref count
drop(buffer);  // Decrement ref count
// Buffer freed when last Arc dropped
```

## Memory Allocation Strategies

### Unity (C# GC)

```csharp
// Garbage collection model
public class GCExample : MonoBehaviour
{
    void Update()
    {
        // ❌ BAD: Allocates every frame (GC pressure)
        string[] names = new string[100];
        Vector3 temp = new Vector3(1, 2, 3);  // Struct, stack allocated (OK)
        
        // ✅ GOOD: Reuse objects
        names = cachedArray;  // Pre-allocated
    }
    
    // Object pooling to reduce GC
    private Queue<GameObject> pool = new Queue<GameObject>();
    
    GameObject GetFromPool()
    {
        if (pool.Count > 0)
            return pool.Dequeue();
        else
            return Instantiate(prefab);
    }
    
    void ReturnToPool(GameObject obj)
    {
        obj.SetActive(false);
        pool.Enqueue(obj);
    }
}

// GC.Collect() - Force garbage collection (avoid in gameplay)
// Use structs for value types (avoid heap allocation)
```

### Unreal (Manual + GC for UObjects)

```cpp
// Manual memory management
void* Memory = FMemory::Malloc(1024);  // Allocate
FMemory::Free(Memory);  // Deallocate

// RAII with smart pointers
TSharedPtr<FMyData> SharedData = MakeShared<FMyData>();  // Reference counted
TUniquePtr<FMyData> UniqueData = MakeUnique<FMyData>();  // Single owner

// UObject garbage collection
UCLASS()
class UMyObject : public UObject
{
    UPROPERTY()  // Prevents GC
    UTexture2D* Texture;
};

// Preventing GC with AddToRoot
UObject* ImportantObject = NewObject<UMyObject>();
ImportantObject->AddToRoot();  // Never GC'd
// Later:
ImportantObject->RemoveFromRoot();  // Allow GC

// Object pooling
class FObjectPool
{
public:
    TArray<UMyObject*> Pool;
    
    UMyObject* Acquire()
    {
        if (Pool.Num() > 0)
            return Pool.Pop();
        return NewObject<UMyObject>();
    }
    
    void Release(UMyObject* Obj)
    {
        Pool.Add(Obj);
    }
};
```

### Godot (Reference Counting)

```gdscript
# RefCounted base class (automatic reference counting)
class MyResource extends RefCounted:
    var data: PackedByteArray
    
    func _init():
        data = PackedByteArray()
        data.resize(1024)
    
    # _notification(NOTIFICATION_PREDELETE) called before deletion

# Usage
var resource = MyResource.new()  # Ref count = 1
var ref2 = resource              # Ref count = 2
resource = null                  # Ref count = 1
ref2 = null                      # Ref count = 0, deleted

# Manual memory for unmanaged resources
var buffer = PackedByteArray()
buffer.resize(1024)
# Automatically freed when out of scope

# Object pooling
var pool: Array[Node] = []

func get_from_pool() -> Node:
    if pool.size() > 0:
        return pool.pop_back()
    return Node.new()

func return_to_pool(node: Node):
    node.queue_free()  # Deferred deletion
    pool.append(node)
```

### Praxis (Ownership + Arc)

```rust
// Ownership system (compile-time memory safety)
fn ownership_example() {
    let buffer = vec![0u8; 1024];  // Owned
    process_buffer(buffer);  // Ownership transferred
    // buffer is invalid here (compile error if used)
}

fn process_buffer(buffer: Vec<u8>) {
    // buffer is valid here
}  // buffer dropped (freed) here

// Borrowing (no transfer)
fn borrow_example() {
    let buffer = vec![0u8; 1024];
    read_buffer(&buffer);  // Borrow (read-only)
    modify_buffer(&mut buffer);  // Mutable borrow
    // buffer still valid
}  // buffer dropped here

// Reference counting for shared ownership
use std::sync::Arc;

fn shared_ownership() {
    let texture = Arc::new(load_texture("albedo.png"));
    
    let material_1 = Material {
        albedo: texture.clone(),  // Increment ref count
    };
    
    let material_2 = Material {
        albedo: texture.clone(),  // Increment ref count
    };
    
    // texture freed when all Arc dropped
}

// Object pooling
pub struct Pool<T> {
    objects: Vec<T>,
}

impl<T> Pool<T> {
    pub fn acquire(&mut self) -> Option<T> {
        self.objects.pop()
    }
    
    pub fn release(&mut self, obj: T) {
        self.objects.push(obj);
    }
}
```

## GPU Memory Types

### Unity

```csharp
// Unity abstracts memory types
// ComputeBufferType:
// - Default: GPU read/write
// - Structured: Structured buffer
// - Append/Consume: Append/consume buffers
// - Counter: With counter

// RenderTexture memory hints (automatic)
RenderTexture rt = new RenderTexture(1024, 1024, 24);
rt.useMipMap = true;  // Allocates mip chain
rt.autoGenerateMips = true;
```

### Unreal

```cpp
// Buffer usage flags
BUF_Static    // Rarely updated, optimized for GPU read
BUF_Dynamic   // Frequently updated from CPU
BUF_Volatile  // Updated every frame
BUF_UnorderedAccess  // GPU write (compute shaders)
BUF_ShaderResource   // GPU read (shaders)

// Texture pool (automatic)
// Unreal manages texture streaming and pool
```

### Godot

```gdscript
# RenderingDevice memory types (Godot 4.x)
# - TEXTURE_USAGE_SAMPLING_BIT: Read in shaders
# - TEXTURE_USAGE_COLOR_ATTACHMENT_BIT: Render target
# - TEXTURE_USAGE_STORAGE_BIT: Compute shader read/write
# - TEXTURE_USAGE_CAN_UPDATE_BIT: CPU update
```

### Praxis (Vulkan)

```rust
// Explicit memory type selection
use vulkano::memory::allocator::MemoryTypeFilter;

// Device-local (VRAM, fastest for GPU)
AllocationCreateInfo {
    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
    ..Default::default()
}

// Host-visible (RAM, CPU accessible)
AllocationCreateInfo {
    memory_type_filter: MemoryTypeFilter::PREFER_HOST 
        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
    ..Default::default()
}

// Host-cached (optimized for CPU reads)
AllocationCreateInfo {
    memory_type_filter: MemoryTypeFilter::PREFER_HOST
        | MemoryTypeFilter::HOST_RANDOM_ACCESS,
    ..Default::default()
}

// Buffer usage flags
BufferUsage::VERTEX_BUFFER
BufferUsage::INDEX_BUFFER
BufferUsage::UNIFORM_BUFFER
BufferUsage::STORAGE_BUFFER  // Compute shader read/write
BufferUsage::TRANSFER_SRC | BufferUsage::TRANSFER_DST  // Staging
```

## Cache Optimization

### Structure of Arrays (SoA) vs. Array of Structures (AoS)

```csharp
// ❌ Array of Structures (cache unfriendly)
struct Entity {
    Vector3 position;
    Vector3 velocity;
    float health;
    int teamID;
}
Entity[] entities = new Entity[1000];

// Iterating positions causes cache misses (loading unused data)
for (int i = 0; i < entities.Length; i++) {
    entities[i].position += entities[i].velocity * dt;  // Also loads health, teamID
}

// ✅ Structure of Arrays (cache friendly)
Vector3[] positions = new Vector3[1000];
Vector3[] velocities = new Vector3[1000];
float[] healths = new float[1000];
int[] teamIDs = new int[1000];

// Only loads position and velocity (better cache utilization)
for (int i = 0; i < 1000; i++) {
    positions[i] += velocities[i] * dt;
}
```

### ECS Archetype Storage (Unity DOTS, Praxis)

```rust
// Archetype: entities with same components grouped
// Cache-friendly iteration
Archetype: [Position, Velocity, Health]
┌──────────────────────────────────────┐
│ Positions:  [P0, P1, P2, ..., P99]  │  ← Contiguous, cached together
│ Velocities: [V0, V1, V2, ..., V99]  │
│ Healths:    [H0, H1, H2, ..., H99]  │
└──────────────────────────────────────┘

// Iteration only loads needed components
for (pos, vel) in query.iter() {
    pos += vel * dt;  // Only loads Position, Velocity arrays
}
```

## Trade-Off Analysis

### Unity

**Pros**:
- Managed memory (C# GC) prevents manual errors
- Automatic GPU resource management
- Profiler shows GC and GPU usage
- Structs avoid heap allocation

**Cons**:
- GC pauses can cause hitches
- GPU memory not freed immediately (GC delay)
- Less control over allocation strategy
- Object pooling manual

**Memory Model**: Garbage collection with manual GPU resource release

### Unreal

**Pros**:
- Smart pointers prevent most leaks
- UObject GC for managed objects
- Fine control with manual allocation
- Excellent memory profiler

**Cons**:
- Mix of manual and GC (complexity)
- Easy to leak non-UObject memory
- GC pauses (mitigated with incremental GC)
- C++ memory bugs possible

**Memory Model**: Hybrid manual + garbage collection

### Godot

**Pros**:
- RefCounted prevents leaks automatically
- Lightweight runtime
- Simple mental model
- queue_free() for deferred deletion

**Cons**:
- Reference cycles can leak (rare)
- Less control than manual management
- Godot 3.x had more GC pauses (improved in 4.x)

**Memory Model**: Reference counting

### Praxis

**Pros**:
- Compile-time memory safety (Rust)
- No garbage collection pauses
- Explicit control (Vulkan allocators)
- Arc for shared ownership
- Zero-cost abstractions

**Cons**:
- Learning curve (ownership, lifetimes)
- Verbose allocation code (Vulkan)
- Manual resource management
- Rust compiler errors can be cryptic

**Memory Model**: Ownership + reference counting (Arc)

## Performance Comparison

### Memory Allocation Overhead

| Engine | CPU Allocation | GPU Allocation | Release Overhead |
|--------|---------------|----------------|------------------|
| Unity | Fast (GC) | Medium (driver) | Deferred (GC) |
| Unreal | Fast (binned) | Medium (pooled) | Deferred (GC) or immediate |
| Godot | Fast (pooled) | Medium (driver) | Immediate (ref counting) |
| Praxis | Fast (system) | Slow (Vulkan verbose) | Immediate (RAII) |

### Memory Footprint (10,000 entities)

| Engine | ECS Overhead | GC/Runtime Overhead |
|--------|-------------|---------------------|
| Unity (Classic) | ~200 bytes/entity | ~50 MB baseline |
| Unity (DOTS) | ~48 bytes/entity | ~50 MB baseline |
| Unreal | ~200 bytes/entity | ~80 MB baseline |
| Godot | ~100 bytes/entity | ~20 MB baseline |
| Praxis | ~48 bytes/entity | ~5 MB baseline |

## Key Takeaways

### Universal Principles

1. **Pool Frequently Allocated Objects**: Reduce allocator pressure
2. **Minimize GC Allocations**: Per-frame allocations cause GC pauses
3. **Structure of Arrays**: Better cache performance for iteration
4. **Use Appropriate Memory Types**: Device-local for GPU, host-visible for CPU
5. **Track Lifetime**: Ensure resources are freed when done

### Design Patterns to Steal

- **Object Pooling**: Reuse objects instead of allocating
- **Ring Buffers**: Per-frame data with fixed memory
- **Staging Buffers**: CPU-writable for GPU upload
- **Smart Pointers**: Automatic lifetime management (Arc, shared_ptr)
- **RAII**: Resource acquisition is initialization (C++, Rust)

### Common Pitfalls

- **Forgetting GPU Release**: GPU memory leaked despite CPU GC
- **Per-Frame Allocation**: GC pressure from temporary objects
- **Fragmentation**: Many small allocations cause fragmentation
- **Ignoring Cache**: AoS layout causes cache misses
- **Reference Cycles**: Circular references prevent GC (use weak references)

## Further Reading

### Unity
- [Memory Management](https://docs.unity3d.com/Manual/performance-memory-overview.html)
- [Profiler](https://docs.unity3d.com/Manual/Profiler.html)
- [Object Pooling](https://learn.unity.com/tutorial/object-pooling)

### Unreal
- [Memory Management](https://docs.unrealengine.com/5.0/en-US/unreal-engine-memory-management/)
- [Garbage Collection](https://docs.unrealengine.com/5.0/en-US/garbage-collection-in-unreal-engine/)
- [Memory Profiler](https://docs.unrealengine.com/5.0/en-US/memory-profiling-in-unreal-engine/)

### Godot
- [Memory Management](https://docs.godotengine.org/en/stable/tutorials/best_practices/memory_management.html)
- [Object Class](https://docs.godotengine.org/en/stable/classes/class_object.html)

### Praxis
- [Vulkan Memory Management](https://gpuopen.com/learn/vulkan-memory-management/)
- [Vulkano Allocators](https://docs.rs/vulkano/latest/vulkano/memory/allocator/)

### General
- [Data-Oriented Design](https://www.dataorienteddesign.com/dodbook/)
- [Cache-Friendly Code](https://gameprogrammingpatterns.com/data-locality.html)
