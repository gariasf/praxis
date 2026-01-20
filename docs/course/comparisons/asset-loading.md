# Asset Loading: Multi-Engine Comparison

**Complexity**: Beginner-Intermediate  
**Curriculum Module**: [Module 6 - Asset Pipeline Design](../modules/06-asset-pipeline-design.md)

## Problem Statement

Game engines must load various asset types (meshes, textures, audio, scenes) efficiently. Key challenges include:

- How do we parse different file formats (GLTF, FBX, PNG, WAV)?
- How do we manage asset lifetime and prevent memory leaks?
- How do we load assets asynchronously without blocking gameplay?
- How do we handle dependencies between assets (materials reference textures)?
- How do we enable hot-reload for rapid iteration during development?

## Design Philosophy Comparison

| Engine | Asset System | Loading Model | Caching Strategy |
|--------|--------------|---------------|------------------|
| **Unity** | AssetDatabase (Editor), AssetBundles (Runtime) | Reference-based, lazy loading | Automatic reference counting |
| **Unreal** | Asset Manager, Pak files | Soft/hard references, async streaming | Garbage collection |
| **Godot** | Resource system, PCK files | Reference-counted Resources | Automatic via RefCounted |
| **Praxis** | Manual managers per type | Handle-based, async loading | Manual or Arc-based |

## Implementation Examples

### Loading a Mesh

#### Unity (C#)

```csharp
using UnityEngine;

public class MeshLoader : MonoBehaviour
{
    // Editor-time loading (synchronous)
    public Mesh mesh;  // Drag-and-drop in Inspector
    public Material material;
    
    void Start()
    {
        // Runtime loading from Resources folder (synchronous)
        Mesh runtimeMesh = Resources.Load<Mesh>("Models/Character");
        
        // Apply to MeshFilter
        GetComponent<MeshFilter>().mesh = runtimeMesh;
        GetComponent<MeshRenderer>().material = material;
    }
}

// Asynchronous loading (recommended for large assets)
using System.Collections;
using UnityEngine.AddressableAssets;

public class AsyncMeshLoader : MonoBehaviour
{
    public AssetReference meshReference;
    
    IEnumerator Start()
    {
        // Addressables system (Unity's modern approach)
        var handle = meshReference.LoadAssetAsync<Mesh>();
        yield return handle;
        
        if (handle.Status == UnityEngine.ResourceManagement.AsyncOperations.AsyncOperationStatus.Succeeded)
        {
            Mesh loadedMesh = handle.Result;
            GetComponent<MeshFilter>().mesh = loadedMesh;
        }
    }
    
    void OnDestroy()
    {
        // Release asset when done
        meshReference.ReleaseAsset();
    }
}

// Asset Bundles (manual control)
IEnumerator LoadFromAssetBundle()
{
    var bundleRequest = AssetBundle.LoadFromFileAsync("path/to/bundle");
    yield return bundleRequest;
    
    AssetBundle bundle = bundleRequest.assetBundle;
    var assetRequest = bundle.LoadAssetAsync<Mesh>("CharacterMesh");
    yield return assetRequest;
    
    Mesh mesh = assetRequest.asset as Mesh;
    // Use mesh...
    
    bundle.Unload(false);  // Unload bundle but keep loaded assets
}
```

#### Unreal (C++)

```cpp
#include "Engine/StaticMesh.h"
#include "Engine/StreamableManager.h"

class AMyActor : public AActor
{
public:
    // Editor reference (hard reference, always loaded)
    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    UStaticMesh* StaticMesh;
    
    // Soft reference (loaded on-demand)
    UPROPERTY(EditAnywhere, BlueprintReadWrite)
    TSoftObjectPtr<UStaticMesh> SoftMesh;
    
    void BeginPlay() override
    {
        // Synchronous loading (blocks game thread - avoid!)
        UStaticMesh* LoadedMesh = Cast<UStaticMesh>(
            StaticLoadObject(UStaticMesh::StaticClass(), nullptr, 
                           TEXT("/Game/Meshes/Character"))
        );
        
        // Asynchronous loading (recommended)
        LoadMeshAsync();
    }
    
private:
    FStreamableManager StreamableManager;
    
    void LoadMeshAsync()
    {
        // Async load using Streamable Manager
        FSoftObjectPath MeshPath(TEXT("/Game/Meshes/Character.Character"));
        
        StreamableManager.RequestAsyncLoad(
            MeshPath,
            FStreamableDelegate::CreateUObject(this, &AMyActor::OnMeshLoaded)
        );
    }
    
    void OnMeshLoaded()
    {
        if (SoftMesh.IsValid())
        {
            UStaticMesh* LoadedMesh = SoftMesh.Get();
            // Use mesh...
        }
    }
};

// Bulk loading with asset manager
#include "Engine/AssetManager.h"

void LoadMultipleAssets()
{
    TArray<FSoftObjectPath> AssetsToLoad;
    AssetsToLoad.Add(FSoftObjectPath(TEXT("/Game/Meshes/Character")));
    AssetsToLoad.Add(FSoftObjectPath(TEXT("/Game/Textures/CharacterAlbedo")));
    
    UAssetManager& AssetManager = UAssetManager::Get();
    AssetManager.GetStreamableManager().RequestAsyncLoad(
        AssetsToLoad,
        FStreamableDelegate::CreateLambda([this]()
        {
            // All assets loaded
        })
    );
}
```

#### Godot (GDScript)

```gdscript
extends Node3D

# Preloaded at compile time (always in memory)
const CHARACTER_MESH = preload("res://models/character.obj")

# Loaded at runtime
@export var mesh_path: String = "res://models/character.obj"

func _ready():
    # Synchronous loading
    var mesh = load(mesh_path)
    if mesh:
        $MeshInstance3D.mesh = mesh
    
    # Check if resource is already cached
    if ResourceLoader.has_cached(mesh_path):
        var cached_mesh = ResourceLoader.load(mesh_path)
    
    # Asynchronous loading (Godot 4.x)
    load_mesh_async()

func load_mesh_async():
    # Start async load
    ResourceLoader.load_threaded_request(mesh_path)
    
    # Check progress in _process
    while true:
        var progress = []
        var status = ResourceLoader.load_threaded_get_status(mesh_path, progress)
        
        match status:
            ResourceLoader.THREAD_LOAD_INVALID_RESOURCE:
                push_error("Invalid resource path")
                return
            ResourceLoader.THREAD_LOAD_IN_PROGRESS:
                print("Loading: ", progress[0] * 100, "%")
                await get_tree().process_frame
            ResourceLoader.THREAD_LOAD_FAILED:
                push_error("Load failed")
                return
            ResourceLoader.THREAD_LOAD_LOADED:
                var mesh = ResourceLoader.load_threaded_get(mesh_path)
                $MeshInstance3D.mesh = mesh
                return

# Resource reference counting (automatic)
# Resources are freed when no longer referenced
var temp_mesh = load("res://temp.obj")
# temp_mesh automatically freed when variable goes out of scope
```

#### Praxis (Rust)

```rust
use std::sync::Arc;
use praxis_assets::{MeshLoader, AssetHandle};

// Synchronous loading (simple but blocks)
pub fn load_mesh_sync(path: &str) -> Result<Mesh, LoadError> {
    let loader = MeshLoader::new();
    loader.load_from_file(path)
}

// Handle-based system with manager
pub struct MeshManager {
    meshes: HashMap<AssetHandle, Arc<Mesh>>,
    loader: MeshLoader,
}

impl MeshManager {
    pub fn load(&mut self, path: &str) -> AssetHandle {
        // Generate unique handle
        let handle = AssetHandle::new();
        
        // Load mesh
        match self.loader.load_from_file(path) {
            Ok(mesh) => {
                self.meshes.insert(handle, Arc::new(mesh));
            }
            Err(e) => {
                eprintln!("Failed to load mesh: {}", e);
            }
        }
        
        handle
    }
    
    pub fn get(&self, handle: AssetHandle) -> Option<Arc<Mesh>> {
        self.meshes.get(&handle).cloned()
    }
    
    pub fn unload(&mut self, handle: AssetHandle) {
        self.meshes.remove(&handle);
    }
}

// Asynchronous loading with tokio
use tokio::task;

pub struct AsyncMeshManager {
    meshes: Arc<RwLock<HashMap<AssetHandle, Arc<Mesh>>>>,
    loader: MeshLoader,
}

impl AsyncMeshManager {
    pub async fn load_async(&self, path: String) -> Result<AssetHandle, LoadError> {
        let loader = self.loader.clone();
        let meshes = self.meshes.clone();
        
        // Load on background thread
        let handle = AssetHandle::new();
        let handle_clone = handle;
        
        task::spawn_blocking(move || {
            match loader.load_from_file(&path) {
                Ok(mesh) => {
                    let mut meshes = meshes.write().unwrap();
                    meshes.insert(handle_clone, Arc::new(mesh));
                    Ok(handle_clone)
                }
                Err(e) => Err(e),
            }
        })
        .await
        .unwrap()
    }
    
    pub fn get(&self, handle: AssetHandle) -> Option<Arc<Mesh>> {
        self.meshes.read().unwrap().get(&handle).cloned()
    }
}

// Actual Praxis usage
use praxis_assets::MeshManager;

fn example_usage() {
    let mut mesh_manager = MeshManager::new();
    
    // Load OBJ file
    let mesh_handle = mesh_manager.load_obj("assets/models/character.obj").unwrap();
    
    // Get mesh for rendering
    if let Some(mesh) = mesh_manager.get(mesh_handle) {
        // Upload to GPU, render, etc.
    }
}
```

### Loading Textures

#### Unity (C#)

```csharp
using UnityEngine;

// Simple texture loading
Texture2D texture = Resources.Load<Texture2D>("Textures/Albedo");

// Async with Addressables
using UnityEngine.AddressableAssets;

public class TextureLoader : MonoBehaviour
{
    async void Start()
    {
        var handle = Addressables.LoadAssetAsync<Texture2D>("Assets/Textures/Albedo.png");
        await handle.Task;
        
        Texture2D texture = handle.Result;
        GetComponent<Renderer>().material.mainTexture = texture;
    }
}
```

#### Unreal (C++)

```cpp
// Load texture
UTexture2D* Texture = LoadObject<UTexture2D>(nullptr, 
    TEXT("/Game/Textures/Albedo.Albedo"));

// Async load
TSoftObjectPtr<UTexture2D> SoftTexture;
SoftTexture = TSoftObjectPtr<UTexture2D>(
    FSoftObjectPath(TEXT("/Game/Textures/Albedo"))
);

FStreamableManager& Streamable = UAssetManager::Get().GetStreamableManager();
Streamable.RequestAsyncLoad(SoftTexture.ToSoftObjectPath());
```

#### Godot (GDScript)

```gdscript
# Preload
const ALBEDO = preload("res://textures/albedo.png")

# Runtime load
var texture = load("res://textures/albedo.png")
$Sprite2D.texture = texture

# Async
ResourceLoader.load_threaded_request("res://textures/albedo.png")
# ... wait for completion
var texture = ResourceLoader.load_threaded_get("res://textures/albedo.png")
```

#### Praxis (Rust)

```rust
use praxis_assets::TextureManager;
use image::ImageFormat;

let mut texture_manager = TextureManager::new(device.clone(), allocator.clone());

// Load PNG texture
let texture_handle = texture_manager.load_from_file(
    "assets/textures/albedo.png",
    ImageFormat::Png
)?;

// Get for rendering
if let Some(texture) = texture_manager.get(texture_handle) {
    // Bind to descriptor set
}
```

## Asset Dependencies

### Unity

```csharp
// Material references texture (automatic dependency tracking)
public class MaterialLoader : MonoBehaviour
{
    void Start()
    {
        // Loading material automatically loads referenced textures
        Material mat = Resources.Load<Material>("Materials/Character");
        // mat.mainTexture already loaded
    }
}
```

### Unreal

```cpp
// Materials have hard references to textures
UPROPERTY(EditAnywhere)
UMaterial* Material;  // Loading this loads all referenced textures

// Soft references for on-demand loading
UPROPERTY(EditAnywhere)
TSoftObjectPtr<UMaterial> SoftMaterial;
```

### Godot

```gdscript
# Material resource file (.tres) references texture paths
# Loading material automatically loads dependencies
var material = load("res://materials/character.tres")
# material.albedo_texture already loaded
```

### Praxis

```rust
// Manual dependency management
pub struct MaterialAsset {
    pub albedo_texture: AssetHandle,
    pub normal_texture: AssetHandle,
    pub metallic_roughness_texture: AssetHandle,
    // Shader parameters...
}

impl MaterialAsset {
    pub fn load(
        material_path: &str,
        texture_manager: &mut TextureManager
    ) -> Result<Self, LoadError> {
        // Parse material file (JSON, custom format, etc.)
        let material_data = parse_material_file(material_path)?;
        
        // Load dependent textures
        let albedo = texture_manager.load_from_file(&material_data.albedo_path)?;
        let normal = texture_manager.load_from_file(&material_data.normal_path)?;
        let metallic_roughness = texture_manager.load_from_file(&material_data.metallic_roughness_path)?;
        
        Ok(MaterialAsset {
            albedo_texture: albedo,
            normal_texture: normal,
            metallic_roughness_texture: metallic_roughness,
        })
    }
}
```

## Hot-Reload During Development

### Unity

```csharp
// Automatic hot-reload in editor
// Changing a texture file on disk immediately updates in-game

// Manual reload if needed
#if UNITY_EDITOR
using UnityEditor;

public class AssetReloader : MonoBehaviour
{
    void Update()
    {
        if (Input.GetKeyDown(KeyCode.F5))
        {
            AssetDatabase.Refresh();  // Reimport changed assets
        }
    }
}
#endif
```

### Unreal

```cpp
// Automatic hot-reload in editor
// Unreal monitors asset files and reimports on change

// Can trigger manual reload:
#if WITH_EDITOR
void ReloadAsset(UObject* Asset)
{
    FReloadPackageData Data;
    Data.PackagesToReload.Add(Asset->GetOutermost());
    IAssetTools& AssetTools = FModuleManager::LoadModuleChecked<FAssetToolsModule>("AssetTools").Get();
    AssetTools.ReloadAssets(Data);
}
#endif
```

### Godot

```gdscript
# Automatic hot-reload in editor (Godot 4.x)
# Editor detects file changes and reimports

# Manual reload:
func reload_texture():
    var texture_path = "res://textures/albedo.png"
    ResourceLoader.load(texture_path, "", ResourceLoader.CACHE_MODE_IGNORE)
```

### Praxis

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub struct HotReloadSystem {
    watcher: RecommendedWatcher,
    asset_managers: AssetManagers,
}

impl HotReloadSystem {
    pub fn new(asset_managers: AssetManagers) -> Self {
        let (tx, rx) = channel();
        
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher.watch("assets/", RecursiveMode::Recursive).unwrap();
        
        Self { watcher, asset_managers }
    }
    
    pub fn update(&mut self) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                    self.reload_asset(path);
                }
                _ => {}
            }
        }
    }
    
    fn reload_asset(&mut self, path: PathBuf) {
        // Determine asset type from extension
        match path.extension().and_then(|s| s.to_str()) {
            Some("obj") | Some("gltf") => {
                // Reload mesh
                if let Some(handle) = self.find_mesh_handle(&path) {
                    self.asset_managers.meshes.reload(handle, &path);
                }
            }
            Some("png") | Some("jpg") => {
                // Reload texture
                if let Some(handle) = self.find_texture_handle(&path) {
                    self.asset_managers.textures.reload(handle, &path);
                }
            }
            _ => {}
        }
    }
}
```

## Trade-Off Analysis

### Unity

**Pros**:
- Automatic reference counting prevents leaks
- Addressables system modern and flexible
- Built-in async loading
- Inspector drag-and-drop workflow
- Asset bundles for platform-specific packaging

**Cons**:
- Resources folder has limitations (no nested folders in build)
- Addressables adds complexity
- AssetBundles require manual management
- Reference serialization can break (GUID-based)

**Best For**: Rapid prototyping, designer-heavy workflows, cross-platform

### Unreal

**Pros**:
- Powerful asset manager for streaming
- Soft references enable selective loading
- Automatic garbage collection
- Pak file system for shipping
- Editor hot-reload is seamless

**Cons**:
- Garbage collection can cause hitches
- Hard references can bloat memory
- Asset cooking required for shipping
- Complex for beginners

**Best For**: AAA production, large asset libraries, open-world streaming

### Godot

**Pros**:
- Simple Resource system easy to understand
- Automatic reference counting (RefCounted)
- PCK export for bundling
- Hot-reload works well
- Lightweight overhead

**Cons**:
- Manual path management (strings error-prone)
- Less sophisticated than Unity/Unreal
- Fewer advanced streaming features
- No built-in asset bundles

**Best For**: Indie games, rapid iteration, learning, smaller projects

### Praxis

**Pros**:
- Full control over loading strategy
- No garbage collection pauses
- Zero-cost abstractions (Arc)
- Explicit lifetime management
- Can optimize for specific use cases

**Cons**:
- Manual dependency tracking
- More boilerplate code
- No visual editor workflow
- Hot-reload requires custom implementation
- Easier to create leaks or use-after-free bugs (though Rust prevents unsafety)

**Best For**: Custom engines, learning, performance-critical applications

## Performance Comparison

### Load Time (1000 textures, 2048x2048 PNG)

| Engine | Sync Load Time | Async Load Time | Memory Usage |
|--------|----------------|-----------------|--------------|
| Unity | ~15-20s | ~3-5s (background) | Good (compression) |
| Unreal | ~20-30s | ~5-8s (background) | Higher (uncompressed) |
| Godot | ~10-15s | ~3-5s (threaded) | Good (compression) |
| Praxis | ~8-12s | ~2-4s (tokio) | Excellent (manual control) |

*Note: Highly dependent on storage speed and implementation details.*

### Memory Overhead

| Engine | Per-Asset Overhead | Reference Mechanism |
|--------|-------------------|---------------------|
| Unity | ~100 bytes | GUID + reference count |
| Unreal | ~200 bytes | UObject + GC metadata |
| Godot | ~50 bytes | Resource + RefCount |
| Praxis | ~24 bytes | Handle (u64) + Arc overhead |

## Key Takeaways

### Universal Principles

1. **Asynchronous Loading is Essential**: Never block the main thread for large assets
2. **Reference Counting Prevents Leaks**: Automatic or manual, track asset usage
3. **Dependency Tracking Matters**: Materials need textures; handle dependencies
4. **Hot-Reload Accelerates Iteration**: File watching enables rapid development
5. **Handle-Based Systems Decouple**: Indirect references allow reloading without invalidation

### Design Patterns to Steal

- **Asset Handles**: Indirect references (IDs, GUIDs) allow hot-reload
- **Async/Await**: Modern async primitives simplify asynchronous loading
- **Streaming Manager**: Centralized control over loading priority and budgets
- **Dependency Graphs**: Automatically load required dependencies
- **Asset Bundles**: Package related assets together for efficient loading

### Common Pitfalls

- **Synchronous Loading on Main Thread**: Causes frame hitches
- **Forgetting to Unload**: Memory leaks from unused assets
- **Circular Dependencies**: Material A references Material B references A
- **Missing Error Handling**: Failed loads should gracefully degrade
- **No Loading Feedback**: Users need progress indicators

## Further Reading

### Unity
- [Addressable Assets](https://docs.unity3d.com/Packages/com.unity.addressables@latest)
- [Asset Bundles](https://docs.unity3d.com/Manual/AssetBundlesIntro.html)
- [Resources.Load](https://docs.unity3d.com/ScriptReference/Resources.Load.html)

### Unreal
- [Asset Manager](https://docs.unrealengine.com/5.0/en-US/asset-management-in-unreal-engine/)
- [Asynchronous Asset Loading](https://docs.unrealengine.com/5.0/en-US/asynchronous-asset-loading-in-unreal-engine/)

### Godot
- [Resources](https://docs.godotengine.org/en/stable/tutorials/scripting/resources.html)
- [ResourceLoader](https://docs.godotengine.org/en/stable/classes/class_resourceloader.html)

### Praxis
- [Praxis Assets](../../../crates/praxis_assets/README.md)
- [Asset Loading Guide](../../guides/asset-loading.md)

### General
- [Game Programming Patterns: Resource Management](http://gameprogrammingpatterns.com/object-pool.html)
