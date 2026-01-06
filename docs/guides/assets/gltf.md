# GLTF Loading Guide

Comprehensive guide to loading and using GLTF 2.0 (GL Transmission Format) assets in the Praxis engine, including meshes, materials, textures, and scene hierarchies.

## Table of Contents

1. [Overview](#overview)
2. [GLTF Format Basics](#gltf-format-basics)
3. [Supported Features](#supported-features)
4. [Quick Start](#quick-start)
5. [API Reference](#api-reference)
6. [Scene Hierarchy](#scene-hierarchy)
7. [Materials and Textures](#materials-and-textures)
8. [Integration with Praxis](#integration-with-praxis)
9. [Performance Considerations](#performance-considerations)
10. [Troubleshooting](#troubleshooting)

---

## Overview

GLTF (GL Transmission Format) is an open standard file format for 3D scenes and models. It's designed for efficient transmission and loading of 3D content, making it ideal for real-time applications and games.

### Why GLTF?

- **Industry Standard:** Supported by major 3D tools (Blender, Maya, 3ds Max, etc.)
- **Efficient:** Binary GLB format for fast loading
- **Feature-Rich:** Meshes, materials, textures, animations, and more
- **PBR Materials:** Built-in support for physically-based rendering
- **Extensible:** Support for custom extensions

### Praxis GLTF Support

The `praxis_assets` crate provides comprehensive GLTF loading with:
- Full mesh geometry support (positions, normals, UVs, tangents)
- PBR material loading
- Texture loading (embedded and external)
- Scene hierarchy with transforms
- Asset caching to avoid redundant loading

---

## GLTF Format Basics

### File Formats

GLTF comes in two variants:

#### .gltf (JSON + Separate Files)

```
my_model/
├── my_model.gltf        # JSON scene description
├── my_model.bin         # Binary geometry data
├── texture_0.png        # External texture
└── texture_1.png        # External texture
```

**Pros:**
- Human-readable JSON
- Easy to inspect and debug
- Can reference existing textures

**Cons:**
- Multiple files to manage
- Slower to load (multiple I/O operations)

#### .glb (Binary Container)

```
my_model.glb             # Single binary file containing everything
```

**Pros:**
- Single file (easy distribution)
- Faster to load
- Smaller file size (compressed)

**Cons:**
- Binary format (not human-readable)
- Requires tools to inspect

**Recommendation:** Use GLB for production, GLTF for development/debugging.

### GLTF Structure

A GLTF file contains several key concepts:

```
GLTF Document
├── Scenes (one or more scene graphs)
│   └── Nodes (tree of transforms)
│       ├── Mesh (reference to geometry)
│       ├── Transform (position, rotation, scale)
│       └── Children (other nodes)
├── Meshes (geometry data)
│   └── Primitives (drawable parts)
│       ├── Attributes (positions, normals, UVs, etc.)
│       ├── Indices (triangle connectivity)
│       └── Material (reference to material)
├── Materials (PBR properties)
│   ├── PBR Metallic-Roughness
│   ├── Base Color Factor
│   ├── Base Color Texture
│   └── Normal Map Texture
├── Textures (image references)
│   ├── Image (source data)
│   └── Sampler (filtering settings)
├── Images (actual pixel data)
│   ├── Embedded (base64 in JSON or buffer)
│   └── External (file path)
└── Buffers (binary data)
    └── Buffer Views (slices of buffer data)
```

---

## Supported Features

### ✅ Fully Supported

#### Meshes

- **Vertex Positions** (`POSITION` attribute)
- **Vertex Normals** (`NORMAL` attribute)
- **Texture Coordinates** (`TEXCOORD_0` attribute)
- **Tangent Vectors** (`TANGENT` attribute, for normal mapping)
- **Triangle Primitives** (most common, fully supported)
- **Multiple Primitives per Mesh** (different materials on same object)

#### Materials

- **PBR Metallic-Roughness Workflow**
  - Base color factor (RGBA)
  - Metallic factor (0-1)
  - Roughness factor (0-1)
  - Base color texture
  - Normal map texture
- **Double-Sided Materials** (metadata available)
- **Alpha Mode** (OPAQUE, MASK, BLEND metadata)

#### Textures

- **Image Formats:** PNG, JPEG
- **Color Spaces:** RGBA, RGB
- **Embedded Images:** Base64-encoded or binary buffers
- **External Images:** File path references
- **Texture Coordinates:** UV mapping

#### Scene Hierarchy

- **Node Transforms:**
  - Matrix form (4×4 transformation)
  - TRS form (Translation, Rotation, Scale)
- **Node Hierarchy:** Parent-child relationships
- **Multiple Root Nodes:** Multiple objects at scene root
- **Named Nodes:** Optional names for debugging

### ❌ Not Yet Supported

- **Animations:** Keyframe animations, skinning
- **Skins:** Skeletal animation, bone weights
- **Morph Targets:** Blend shapes
- **Cameras:** Camera definitions (data loaded but not used)
- **Lights:** Light definitions (use Praxis ECS lights instead)
- **Point/Line Primitives:** Only triangle primitives supported
- **Sparse Accessors:** Dense accessors only
- **Extensions:** No custom extension support

---

## Quick Start

### Loading a GLTF File

```rust
use praxis_assets::{GltfLoader, GltfAssetManager};

// Method 1: Direct loading (no caching)
let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/my_model.gltf")?;

// Method 2: Cached loading (recommended)
let mut manager = GltfAssetManager::new();
let asset = manager.load("assets/models/my_model.gltf")?;

// Inspect what was loaded
println!("Loaded {} meshes", asset.meshes.len());
println!("Loaded {} materials", asset.materials.len());
println!("Loaded {} textures", asset.textures.len());
println!("Loaded {} nodes", asset.nodes.len());
```

### Uploading Meshes to GPU

```rust
use praxis_graphics::RenderContext;

// Upload all meshes to GPU
for (i, mesh) in asset.meshes.iter().enumerate() {
    render_context
        .mesh_manager_mut()
        .load_mesh(
            format!("gltf_mesh_{}", i),  // Unique ID
            mesh.clone(),
        )?;
}
```

### Loading Textures

```rust
// Upload all textures to GPU
for (i, texture) in asset.textures.iter().enumerate() {
    // Convert GltfTexture to image data
    let image_data = &texture.image_data;
    let width = texture.width;
    let height = texture.height;
    
    render_context
        .texture_manager_mut()
        .load_texture_from_memory(
            format!("gltf_texture_{}", i),
            image_data,
            width,
            height,
        )?;
}
```

### Creating Scene Entities

```rust
use bevy_ecs::world::World;
use praxis_scene::{Transform, GlobalTransform};
use praxis_graphics::MeshHandle;

// Spawn entities for each node with a mesh
for (node_index, node) in asset.nodes_with_meshes() {
    let mesh_index = node.mesh_index.unwrap();
    
    // Decompose node transform
    let (translation, rotation, scale) = node.decompose_transform();
    
    // Spawn entity in ECS
    world.spawn((
        Transform::new(translation, rotation, scale),
        GlobalTransform::default(),
        MeshHandle::new(format!("gltf_mesh_{}", mesh_index)),
        // Add material, texture, etc.
    ));
}
```

---

## API Reference

### GltfLoader

The main loader for GLTF files:

```rust
pub struct GltfLoader;

impl GltfLoader {
    /// Create a new GLTF loader
    pub fn new() -> Self;
    
    /// Load a GLTF file from disk
    pub fn load_gltf(&self, path: impl AsRef<Path>) -> Result<GltfAsset>;
}
```

### GltfAssetManager

Cached asset manager for GLTF files:

```rust
pub struct GltfAssetManager {
    cache: HashMap<PathBuf, Arc<GltfAsset>>,
}

impl GltfAssetManager {
    /// Create a new asset manager
    pub fn new() -> Self;
    
    /// Load a GLTF file (cached)
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<Arc<GltfAsset>>;
    
    /// Check if asset is already loaded
    pub fn is_loaded(&self, path: impl AsRef<Path>) -> bool;
    
    /// Unload an asset from cache
    pub fn unload(&mut self, path: impl AsRef<Path>);
    
    /// Clear all cached assets
    pub fn clear(&mut self);
}
```

### GltfAsset

The loaded GLTF data:

```rust
pub struct GltfAsset {
    /// All meshes in the file
    pub meshes: Vec<MeshData>,
    
    /// All materials in the file
    pub materials: Vec<GltfMaterial>,
    
    /// All textures in the file
    pub textures: Vec<GltfTexture>,
    
    /// Scene graph nodes
    pub nodes: Vec<GltfNode>,
    
    /// Root node indices (no parent)
    pub root_nodes: Vec<usize>,
    
    /// Default scene index
    pub default_scene: Option<usize>,
}

impl GltfAsset {
    /// Iterator over nodes that have meshes
    pub fn nodes_with_meshes(&self) -> impl Iterator<Item = (usize, &GltfNode)>;
    
    /// Traverse scene graph depth-first
    pub fn traverse_depth_first<F>(&self, visitor: F)
    where
        F: FnMut(usize, &GltfNode, usize);
        
    /// Get all nodes in a subtree
    pub fn get_subtree(&self, root_index: usize) -> Vec<usize>;
}
```

### GltfNode

A node in the scene hierarchy:

```rust
pub struct GltfNode {
    /// Node name (optional)
    pub name: Option<String>,
    
    /// Local transform matrix (4×4)
    pub transform: Mat4,
    
    /// Reference to mesh (if any)
    pub mesh_index: Option<usize>,
    
    /// Child node indices
    pub children: Vec<usize>,
    
    /// Parent node index (None for root nodes)
    pub parent: Option<usize>,
}

impl GltfNode {
    /// Decompose transform matrix into TRS components
    pub fn decompose_transform(&self) -> (Vec3, Quat, Vec3);
    
    /// Check if this is a root node
    pub fn is_root(&self) -> bool;
    
    /// Check if this node has a mesh
    pub fn has_mesh(&self) -> bool;
}
```

### GltfMaterial

PBR material properties:

```rust
pub struct GltfMaterial {
    /// Material name (optional)
    pub name: Option<String>,
    
    /// Base color factor (RGBA)
    pub base_color_factor: [f32; 4],
    
    /// Metallic factor (0-1)
    pub metallic_factor: f32,
    
    /// Roughness factor (0-1)
    pub roughness_factor: f32,
    
    /// Base color texture index (optional)
    pub base_color_texture_index: Option<usize>,
    
    /// Normal map texture index (optional)
    pub normal_texture_index: Option<usize>,
    
    /// Double-sided rendering
    pub double_sided: bool,
    
    /// Alpha mode (OPAQUE, MASK, BLEND)
    pub alpha_mode: AlphaMode,
}

impl GltfMaterial {
    /// Convert to Praxis MaterialProperties
    pub fn to_material_properties(&self) -> MaterialProperties;
}
```

### GltfTexture

Texture image data:

```rust
pub struct GltfTexture {
    /// Image pixel data (RGBA or RGB)
    pub image_data: Vec<u8>,
    
    /// Image width in pixels
    pub width: u32,
    
    /// Image height in pixels
    pub height: u32,
    
    /// Color format (RGBA or RGB)
    pub format: TextureFormat,
    
    /// Source file path (if external)
    pub source_path: Option<PathBuf>,
}
```

---

## Scene Hierarchy

### Understanding Node Transforms

GLTF nodes can specify transforms in two ways:

#### 1. Matrix Form (Direct)

```rust
// 4×4 transformation matrix
let transform = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    10.0, 5.0, 2.0, 1.0,  // Translation
]);
```

#### 2. TRS Form (Decomposed)

```rust
// Translation, Rotation, Scale
let translation = Vec3::new(10.0, 5.0, 2.0);
let rotation = Quat::from_rotation_y(PI / 4.0);
let scale = Vec3::new(1.0, 1.0, 1.0);

// Reconstruct matrix
let transform = Mat4::from_scale_rotation_translation(
    scale,
    rotation,
    translation,
);
```

### Traversing the Scene Graph

#### Depth-First Traversal

```rust
asset.traverse_depth_first(|node_index, node, depth| {
    let indent = "  ".repeat(depth);
    println!("{}Node {}: {:?}", indent, node_index, node.name);
    
    if let Some(mesh_idx) = node.mesh_index {
        println!("{}  Mesh: {}", indent, mesh_idx);
    }
});
```

**Output Example:**
```
Node 0: "Scene Root"
  Node 1: "Character"
    Mesh: 0
    Node 3: "Left Arm"
      Mesh: 1
    Node 4: "Right Arm"
      Mesh: 2
  Node 2: "Floor"
    Mesh: 3
```

#### Finding All Meshes

```rust
for (node_index, node) in asset.nodes_with_meshes() {
    let mesh_index = node.mesh_index.unwrap();
    let mesh = &asset.meshes[mesh_index];
    
    println!("Node {} has mesh {} with {} vertices",
        node_index,
        mesh_index,
        mesh.vertices.len()
    );
}
```

#### Building Parent-Child Relationships

```rust
// Parent → Children
for (i, node) in asset.nodes.iter().enumerate() {
    if !node.children.is_empty() {
        println!("Node {} has children: {:?}", i, node.children);
    }
}

// Child → Parent
for (i, node) in asset.nodes.iter().enumerate() {
    if let Some(parent_idx) = node.parent {
        println!("Node {} has parent {}", i, parent_idx);
    }
}
```

### Transform Propagation

Node transforms are local to their parent. To get world-space positions:

```rust
fn compute_world_transform(
    asset: &GltfAsset,
    node_index: usize,
) -> Mat4 {
    let node = &asset.nodes[node_index];
    
    // Start with local transform
    let mut world_transform = node.transform;
    
    // Walk up parent chain
    let mut current_parent = node.parent;
    while let Some(parent_idx) = current_parent {
        let parent = &asset.nodes[parent_idx];
        world_transform = parent.transform * world_transform;
        current_parent = parent.parent;
    }
    
    world_transform
}
```

---

## Materials and Textures

### PBR Material Workflow

GLTF uses the PBR metallic-roughness workflow:

```rust
let material = &asset.materials[0];

// Base color: Tint applied to texture
let base_color = material.base_color_factor;  // [R, G, B, A]

// Metallic: 0.0 = dielectric, 1.0 = metal
let metallic = material.metallic_factor;

// Roughness: 0.0 = smooth, 1.0 = rough
let roughness = material.roughness_factor;
```

### Loading Materials

```rust
// Convert GLTF material to Praxis material
let material_props = material.to_material_properties();

// Load base color texture (if present)
let texture_name = if let Some(tex_idx) = material.base_color_texture_index {
    let texture = &asset.textures[tex_idx];
    let texture_id = format!("gltf_texture_{}", tex_idx);
    
    // Upload to GPU
    render_context.texture_manager_mut().load_texture_from_memory(
        &texture_id,
        &texture.image_data,
        texture.width,
        texture.height,
    )?;
    
    Some(texture_id)
} else {
    None
};

// Load normal map (if present)
let normal_map_name = if let Some(tex_idx) = material.normal_texture_index {
    // Similar process as base color texture
    Some(format!("gltf_normal_{}", tex_idx))
} else {
    None
};
```

### Texture Sampling

GLTF textures include sampler settings:

```rust
pub struct GltfTexture {
    // Magnification filter (NEAREST, LINEAR)
    pub mag_filter: MagFilter,
    
    // Minification filter (NEAREST, LINEAR, MIPMAP variants)
    pub min_filter: MinFilter,
    
    // Wrap mode S (REPEAT, CLAMP_TO_EDGE, MIRRORED_REPEAT)
    pub wrap_s: WrapMode,
    
    // Wrap mode T
    pub wrap_t: WrapMode,
}
```

**Note:** Current Praxis implementation uses default sampler settings. Custom sampler configuration is a future enhancement.

---

## Integration with Praxis

### Complete Loading Pipeline

```rust
use praxis_assets::GltfAssetManager;
use praxis_graphics::RenderContext;
use praxis_ecs::World;

pub fn load_gltf_scene(
    path: &str,
    gltf_manager: &mut GltfAssetManager,
    render_context: &mut RenderContext,
    world: &mut World,
) -> Result<()> {
    // 1. Load GLTF file
    let asset = gltf_manager.load(path)?;
    
    // 2. Upload meshes
    for (i, mesh) in asset.meshes.iter().enumerate() {
        render_context
            .mesh_manager_mut()
            .load_mesh(format!("{}_mesh_{}", path, i), mesh.clone())?;
    }
    
    // 3. Upload textures
    for (i, texture) in asset.textures.iter().enumerate() {
        render_context
            .texture_manager_mut()
            .load_texture_from_memory(
                format!("{}_texture_{}", path, i),
                &texture.image_data,
                texture.width,
                texture.height,
            )?;
    }
    
    // 4. Create entities
    for (node_index, node) in asset.nodes_with_meshes() {
        let mesh_index = node.mesh_index.unwrap();
        let (translation, rotation, scale) = node.decompose_transform();
        
        // Get material if available
        let material_index = asset.meshes[mesh_index].material_index;
        let material_props = material_index
            .map(|idx| asset.materials[idx].to_material_properties())
            .unwrap_or_default();
        
        // Spawn entity
        world.spawn((
            Transform::new(translation, rotation, scale),
            GlobalTransform::default(),
            MeshHandle::new(format!("{}_mesh_{}", path, mesh_index)),
            MaterialPropertiesComponent(material_props),
            // Add texture handle if material has texture
        ));
    }
    
    Ok(())
}
```

### Handling Multiple Primitives

GLTF meshes can have multiple primitives (sub-meshes with different materials):

```rust
// Praxis flattens primitives into separate MeshData objects
// If a GLTF mesh has 3 primitives:
//   mesh.primitives[0] → asset.meshes[mesh_index + 0]
//   mesh.primitives[1] → asset.meshes[mesh_index + 1]
//   mesh.primitives[2] → asset.meshes[mesh_index + 2]

// When spawning entities, spawn one per primitive:
for (node_index, node) in asset.nodes_with_meshes() {
    let base_mesh_index = node.mesh_index.unwrap();
    let primitive_count = /* stored in metadata */;
    
    for prim_offset in 0..primitive_count {
        let mesh_index = base_mesh_index + prim_offset;
        // Spawn entity for this primitive
    }
}
```

---

## Performance Considerations

### Loading Performance

**Factors affecting load time:**
1. **File size:** Larger files take longer to read
2. **Texture count:** More textures = more decoding time
3. **Compression:** PNG decompression is CPU-intensive
4. **Mesh complexity:** More vertices = more processing

**Optimization strategies:**
1. **Use GLB format:** Single file, faster I/O
2. **Compress textures:** Use appropriate quality settings
3. **Asset caching:** Reuse loaded assets via `GltfAssetManager`
4. **Background loading:** Load assets on separate thread (future enhancement)
5. **Texture atlasing:** Combine multiple textures (manual process)

### Memory Usage

**Memory breakdown for a typical model:**
```
Model: "Character"
- Mesh data (CPU):    2 MB  (vertices, indices, normals, UVs)
- Mesh data (GPU):    2 MB  (uploaded to VRAM)
- Textures (CPU):     8 MB  (4 textures @ 2K resolution)
- Textures (GPU):     8 MB  (uploaded to VRAM)
- Scene metadata:   <1 MB  (nodes, materials, hierarchy)
─────────────────────────
Total CPU:           11 MB
Total GPU:           10 MB
Total:               21 MB
```

**Memory optimization:**
1. **Drop CPU data:** After GPU upload, release CPU copies
2. **Texture compression:** Use DXT/BC formats (not yet implemented)
3. **LOD system:** Load lower detail at distance
4. **Streaming:** Load/unload assets based on visibility

### Runtime Performance

**GLTF has no runtime overhead:**
- Data is loaded once at startup/level load
- GPU meshes and textures are native formats
- No per-frame parsing or processing
- Scene hierarchy is converted to ECS entities

**Rendering performance depends on:**
- Mesh complexity (triangle count)
- Texture resolution
- Material complexity
- Draw call count

---

## Troubleshooting

### Common Issues

#### "Failed to parse GLTF file"

**Cause:** Corrupted file or unsupported GLTF version

**Solutions:**
1. Verify file is valid GLTF 2.0
2. Try re-exporting from 3D tool
3. Use GLTF validator: https://github.khronos.org/glTF-Validator/

#### "Missing mesh data"

**Cause:** GLTF mesh has no primitives or no position attribute

**Solutions:**
1. Ensure mesh has geometry in 3D tool
2. Verify export settings include geometry
3. Check for empty/hidden objects

#### "Texture failed to load"

**Cause:** External texture file not found, or unsupported format

**Solutions:**
1. Verify external texture files exist
2. Use absolute paths or correct relative paths
3. Convert textures to PNG or JPEG
4. Use embedded textures (GLB format)

#### "Wrong colors/appearance"

**Cause:** Material properties not correctly exported or loaded

**Solutions:**
1. Check material settings in 3D tool
2. Verify PBR workflow is used (not legacy materials)
3. Ensure textures are in correct color space (sRGB for color, linear for normals)

#### "Normals look wrong"

**Cause:** Missing tangent vectors for normal mapping

**Solutions:**
1. Enable tangent export in 3D tool
2. Recalculate normals/tangents in tool
3. Check normal map format (Y-up vs Y-down)

#### "Transforms are incorrect"

**Cause:** Coordinate system mismatch (Y-up vs Z-up)

**Solutions:**
1. Praxis uses Y-up coordinate system
2. Configure export to Y-up in 3D tool
3. Apply transform correction in code if needed

### Validation Tools

**Online GLTF Validator:**
https://github.khronos.org/glTF-Validator/

**Drag and drop your GLTF/GLB file to check for:**
- Format errors
- Missing data
- Invalid references
- Compliance issues

**3D Viewer:**
https://gltf-viewer.donmccurdy.com/

**View your GLTF/GLB file to verify:**
- Geometry appears correct
- Textures are present
- Materials look right
- Hierarchy is correct

---

## Examples

### Blender Export Settings

```
File → Export → glTF 2.0 (.gltf/.glb)

Include:
✓ Selected Objects (or Visible Objects)
✓ Apply Modifiers
✓ UVs
✓ Normals
✓ Tangents  (important for normal maps!)
✓ Materials

Transform:
Y-Up

Geometry:
✓ Compress

Compression: None (or Draco if supported)
```

### Complete Example

See `examples/gltf_demo.rs` for a complete working example that loads a GLTF file and displays it in the engine.

```bash
cargo run --example gltf_demo
```

---

## Future Enhancements

### Planned Features

1. **Animation Support**
   - Keyframe animation playback
   - Skeletal animation (skinning)
   - Morph target animation

2. **Sparse Accessors**
   - Efficient storage for sparse data

3. **Custom Extensions**
   - KHR_materials_unlit
   - KHR_materials_pbrSpecularGlossiness
   - KHR_draco_mesh_compression

4. **Async Loading**
   - Background loading on separate thread
   - Progressive loading (stream geometry)

5. **Instancing**
   - Detect and optimize instanced objects

6. **Custom Sampler Support**
   - Respect GLTF sampler settings
   - Custom filtering and wrapping

---

## References

- [GLTF 2.0 Specification](https://www.khronos.org/registry/glTF/specs/2.0/glTF-2.0.html)
- [GLTF Tutorial Series](https://github.com/KhronosGroup/glTF-Tutorials)
- [Khronos GLTF Overview](https://www.khronos.org/gltf/)
- [GLTF Sample Models](https://github.com/KhronosGroup/glTF-Sample-Models)

---

## See Also

- [Praxis Assets README](../crates/praxis_assets/README.md) - Full API documentation
- [Mesh System](mesh_system.md) - Mesh data format and GPU upload
- [OBJ Loading](obj_loading.md) - Alternative mesh format
- [Material System](BEGINNERS_GUIDE.md#material-system) - Material properties and PBR
