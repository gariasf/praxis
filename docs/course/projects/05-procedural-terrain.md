# Project 05: Procedural Terrain Generator

**Difficulty**: Intermediate  
**Estimated Time**: 3-4 weeks  
**Core Learning**: Terrain generation, noise algorithms, LOD systems, procedural textures

## Overview

Build a procedurally generated terrain system with level-of-detail (LOD), texture splatting, and dynamic generation. This project teaches noise-based generation, spatial partitioning, LOD techniques, and large-scale world rendering optimization.

### Learning Objectives

- Implement noise-based heightmap generation (Perlin, Simplex)
- Build chunk-based terrain streaming system
- Create LOD system for large terrains
- Apply procedural texture splatting
- Optimize rendering for massive landscapes
- Implement terrain physics colliders

## Feature Requirements

### Core Features (Minimum Viable)

1. **Heightmap Generation**
   - Perlin or Simplex noise-based terrain
   - Adjustable parameters (frequency, amplitude, octaves)
   - Multiple noise layers (base terrain + details)
   - Seed-based generation (reproducible)

2. **Terrain Mesh Generation**
   - Heightmap to 3D mesh conversion
   - Normal calculation
   - Efficient vertex/index buffers
   - Chunk-based subdivision (e.g., 64x64 per chunk)

3. **Basic Rendering**
   - Textured terrain (at least one texture)
   - Directional lighting
   - Simple camera controls (fly-through)
   - Fog for distant terrain

4. **Parameter Tweaking**
   - UI for height scale, frequency, octaves
   - Real-time regeneration
   - Seed input field
   - Save/load heightmap

### Extended Features (Recommended)

5. **Level of Detail (LOD)**
   - Multiple LOD levels (e.g., 4 levels)
   - Distance-based LOD selection
   - Smooth LOD transitions (geomorphing or seamless stitching)
   - Optimized far terrain (low poly)

6. **Texture Splatting**
   - Multi-texture blending (grass, rock, snow, sand)
   - Height-based splatting (snow on peaks, grass in valleys)
   - Slope-based splatting (rock on cliffs)
   - Normal map support

7. **Infinite Terrain Streaming**
   - Generate chunks on-demand as camera moves
   - Unload distant chunks
   - Seamless chunk boundaries
   - Background generation (threading)

### Stretch Goals

8. **Advanced Features**
   - Water plane with reflections
   - Vegetation placement (trees, grass via instancing)
   - Erosion simulation (hydraulic or thermal)
   - Caves/overhangs (3D noise, marching cubes)

9. **Biomes**
   - Multiple biome types (desert, forest, tundra)
   - Biome blending at boundaries
   - Biome-specific vegetation and textures
   - Temperature/moisture maps

## Architecture Guidance

### System Components

```
ProceduralTerrain
├── NoiseGenerator
│   ├── PerlinNoise
│   ├── SimplexNoise
│   └── FractalNoise (multi-octave)
├── TerrainGenerator
│   ├── HeightmapGenerator
│   ├── MeshBuilder
│   └── NormalCalculator
├── ChunkManager
│   ├── ChunkLoader
│   ├── ChunkUnloader
│   └── ChunkCache
├── LODSystem
│   ├── LODSelector
│   ├── LODMeshGenerator
│   └── LODTransitioner
├── TerrainRenderer
│   ├── MaterialSystem (texture splatting)
│   ├── BatchRenderer
│   └── CullingSystem
└── PhysicsCollider
    ├── ColliderGenerator
    └── ColliderCache
```

### Data Structures

**Terrain Chunk**
```
TerrainChunk:
  - chunk_x: int (grid coordinates)
  - chunk_z: int
  - size: int (vertices per side, e.g., 65 for 64x64 quads)
  - heightmap: 2D array of floats
  - mesh_lod: array of Mesh (one per LOD level)
  - current_lod: int
  - position: vec3 (world space)
  - is_loaded: bool
  - is_visible: bool

Methods:
  - generate(noise_params, seed)
  - build_mesh(lod_level)
  - get_height_at(local_x, local_z) -> float
```

**Noise Parameters**
```
NoiseParams:
  - seed: int
  - frequency: float (base frequency)
  - amplitude: float (height multiplier)
  - octaves: int (detail layers, e.g., 4-8)
  - lacunarity: float (frequency multiplier per octave, e.g., 2.0)
  - persistence: float (amplitude multiplier per octave, e.g., 0.5)
  - noise_type: Perlin | Simplex | Worley

Methods:
  - sample(x, z) -> float (0-1 or -1 to 1)
  - fractal_sample(x, z) -> float (multi-octave)
```

**LOD Configuration**
```
LODConfig:
  - lod_levels: array of LODLevel
  
LODLevel:
  - lod_index: int (0 = highest detail)
  - distance: float (switch distance from camera)
  - resolution: int (vertex stride, e.g., 1, 2, 4, 8)
  
Example:
  LOD 0: full resolution (64x64), distance 0-100m
  LOD 1: half resolution (32x32), distance 100-250m
  LOD 2: quarter resolution (16x16), distance 250-500m
  LOD 3: eighth resolution (8x8), distance 500-1000m
```

**Texture Splatting**
```
TerrainMaterial:
  - textures: array of Texture (e.g., [grass, rock, snow, sand])
  - splat_map: texture (RGBA, each channel = weight)
  - tiling_scale: float (UV repeat)
  
Splatting Rules (in shader or CPU):
  - If height > snow_threshold: snow
  - Else if slope > cliff_threshold: rock
  - Else if height < sand_threshold: sand
  - Else: grass
  - Blend between using smooth transitions
```

### Terrain Generation Pipeline

```
generate_chunk(chunk_x, chunk_z, params):
  1. Create heightmap array (size x size)
  2. For each vertex (x, z):
     - world_x = chunk_x * chunk_size + x
     - world_z = chunk_z * chunk_size + z
     - height = fractal_noise(world_x, world_z, params)
     - heightmap[x][z] = height * height_scale
  3. Generate mesh:
     - For each quad (x, z):
       - Create 4 vertices (positions from heightmap)
       - Calculate normals (cross product of edges or average)
       - Calculate UVs (based on world position)
       - Add 2 triangles (indices)
  4. Generate LOD meshes (skip vertices for lower LODs)
  5. Upload to GPU
  6. Optionally: generate physics collider
```

### LOD Selection

```
update_lod(camera_position):
  for chunk in loaded_chunks:
    distance = distance(camera_position, chunk.center)
    
    new_lod = 0
    for lod_level in lod_config.levels:
      if distance > lod_level.distance:
        new_lod = lod_level.index
    
    if new_lod != chunk.current_lod:
      chunk.set_lod(new_lod)
```

### Chunk Streaming

```
update_chunks(camera_position):
  camera_chunk_x = floor(camera_position.x / chunk_size)
  camera_chunk_z = floor(camera_position.z / chunk_size)
  
  # Load chunks in view radius
  for x in range(camera_chunk_x - view_radius, camera_chunk_x + view_radius):
    for z in range(camera_chunk_z - view_radius, camera_chunk_z + view_radius):
      if chunk(x, z) not in loaded_chunks:
        load_chunk(x, z)
  
  # Unload distant chunks
  for chunk in loaded_chunks:
    if distance(chunk, camera_chunk) > unload_distance:
      unload_chunk(chunk)
```

## Milestone Plan

### Milestone 1: Basic Heightmap Generation (Week 1, Days 1-3)

**Goal**: Generate and display flat terrain with noise-based heights

**Tasks**:
- Implement Perlin or Simplex noise function
- Generate 2D heightmap array (e.g., 128x128)
- Create flat mesh grid
- Apply heightmap values to vertex Y coordinates
- Render with simple lighting
- Add UI for noise parameters

**Deliverable**: Single terrain patch with adjustable height

### Milestone 2: Normal Calculation and Texturing (Week 1, Days 4-5)

**Goal**: Proper lighting and texture

**Tasks**:
- Calculate vertex normals from heightmap
- Apply single terrain texture with UV mapping
- Implement Phong or PBR lighting
- Add directional light
- Tune appearance (colors, texture scale)

**Deliverable**: Textured, lit terrain

### Milestone 3: Chunk System (Week 1, Days 6-7)

**Goal**: Multiple terrain chunks with seamless boundaries

**Tasks**:
- Split terrain into chunks (e.g., 64x64 each)
- Generate multiple chunks in grid
- Ensure seamless boundaries (share edge vertices)
- Implement chunk culling (frustum or distance)
- Add chunk debug visualization (borders)

**Deliverable**: Multi-chunk terrain with seamless joins

### Milestone 4: LOD System (Week 2, Days 1-4)

**Goal**: Distance-based level of detail

**Tasks**:
- Generate multiple LOD meshes per chunk (full, half, quarter res)
- Calculate chunk-to-camera distance
- Select appropriate LOD per chunk
- Handle LOD transitions (popping vs. smooth)
- Optimize far terrain rendering
- Profile performance improvement

**Deliverable**: LOD-enabled terrain running faster

### Milestone 5: Texture Splatting (Week 2-3, Days 5-7)

**Goal**: Multi-texture blending based on height/slope

**Tasks**:
- Add multiple textures (grass, rock, snow, sand)
- Calculate splat weights per vertex (height/slope rules)
- Implement texture splatting in shader
- Add normal maps for each texture
- Tune blending thresholds
- UI for splat parameters

**Deliverable**: Realistic multi-textured terrain

### Milestone 6: Infinite Streaming (Week 3-4, Days 1-7+)

**Goal**: Dynamically generate terrain as camera moves

**Tasks**:
- Implement chunk loading based on camera position
- Unload chunks outside view radius
- Background thread for chunk generation
- Prevent stuttering (load ahead of camera)
- Add loading indicators (optional)
- Implement chunk caching (save generated chunks)

**Deliverable**: Infinite explorable terrain

### Optional Milestone 7: Advanced Features

**Goal**: Water, vegetation, erosion

**Tasks**:
- Add water plane at specific height
- Implement vegetation instancing (trees, grass)
- Vegetation placement based on terrain properties
- Simple erosion pass (smooth or hydraulic)
- Performance optimization (batching, instancing)

**Deliverable**: Rich, detailed terrain environment

## Technical Challenges

### Challenge 1: Seamless Chunk Boundaries

**Problem**: Cracks appear between chunks at different LOD levels

**Approach**:
- Share vertices along chunk boundaries
- Use skirts (extra geometry hanging down from edges)
- Match neighbor LOD at boundaries (force same LOD for adjacent)
- Geomorphing (blend between LOD levels smoothly)
- T-junctions: triangulate edges carefully

**Skirt Technique**:
```
for each edge vertex:
  create duplicate vertex below terrain
  connect edge vertices to skirt vertices with triangles
  hides gaps caused by LOD mismatch
```

### Challenge 2: Normal Calculation

**Problem**: Flat or incorrect lighting on terrain

**Approach**:
- Calculate normals from heightmap, not mesh
- Use cross product of tangent vectors
- Average normals at vertices (sum normals of adjacent faces)
- Smooth normals across chunk boundaries

**Algorithm**:
```
calculate_normal(x, z, heightmap):
  # Sample neighbors
  h_left = heightmap[x-1][z]
  h_right = heightmap[x+1][z]
  h_down = heightmap[x][z-1]
  h_up = heightmap[x][z+1]
  
  # Tangent vectors
  tangent_x = vec3(2.0, h_right - h_left, 0.0)
  tangent_z = vec3(0.0, h_up - h_down, 2.0)
  
  # Cross product
  normal = normalize(cross(tangent_z, tangent_x))
  return normal
```

### Challenge 3: LOD Popping

**Problem**: Visible "pop" when terrain switches LOD

**Approach**:
- Geomorphing: interpolate vertex positions over time
- Increase switch distance to reduce frequency
- Fade alpha during transition (less noticeable)
- Use continuous LOD (adjust vertex positions based on distance)

**Geomorphing Example**:
```
vertex_shader:
  float lod_blend = compute_lod_blend(distance_to_camera);
  vec3 high_lod_pos = ...
  vec3 low_lod_pos = ...
  vec3 final_pos = mix(high_lod_pos, low_lod_pos, lod_blend);
```

### Challenge 4: Performance with Large View Distance

**Problem**: Too many chunks, low framerate

**Approach**:
- Frustum culling (don't render out-of-view chunks)
- Occlusion culling (don't render hidden chunks)
- Aggressive LOD (lower detail sooner)
- GPU instancing for similar chunks (rare)
- Limit draw calls (batch chunks, use multi-draw)

**Optimization Checklist**:
- Profile: measure time spent in generation vs rendering
- Use spatial data structures (quadtree, grid) for chunk lookup
- Minimize state changes (batch by material/LOD)
- Use index buffers efficiently

### Challenge 5: Texture Splatting Performance

**Problem**: Too many texture samples in fragment shader

**Approach**:
- Limit to 4 textures maximum
- Use texture arrays instead of separate samplers
- Bake splat map to texture (offline or runtime once)
- Use triplanar mapping for vertical surfaces (avoid UV distortion)

**Shader Optimization**:
```glsl
// Use splat map (RGBA texture)
vec4 splat_weights = texture(splat_map, uv);
vec3 color = 
  texture(texture_array, vec3(uv, 0)).rgb * splat_weights.r +
  texture(texture_array, vec3(uv, 1)).rgb * splat_weights.g +
  texture(texture_array, vec3(uv, 2)).rgb * splat_weights.b +
  texture(texture_array, vec3(uv, 3)).rgb * splat_weights.a;
```

## Reference Implementations

### Praxis Engine (Rust)
- **File**: `examples/terrain_demo.rs`
- **Crates**: `praxis_terrain` (LOD, generation)
- **Concepts**: Noise-based generation, chunking, LOD

### Other Engines/Frameworks

**Unity (C#)**
- Tutorial: "Procedural Landmass Generation" (Sebastian Lague, YouTube)
- Asset: Unity Terrain system
- Key APIs: `Terrain`, `TerrainData`, `TerrainLayer`

**Unreal Engine (C++)**
- Tutorial: "Landscape" system documentation
- Key APIs: `ALandscape`, `ULandscapeComponent`, landscape materials

**Godot (GDScript)**
- Plugin: HeightMap Terrain (Zylann)
- Example: Procedural terrain generation tutorial

**Three.js (JavaScript)**
- Example: Procedural terrain with noise
- Libraries: `simplex-noise`, Three.js PlaneGeometry

**Minecraft-style (Voxel)**
- Tutorial: Various voxel engine tutorials (C++, Rust)
- Pattern: Chunk system, greedy meshing, infinite generation

**GPU Gems / ShaderToy**
- Reference: GPU Gems 3, Chapter on terrain rendering
- Examples: ShaderToy procedural terrain shaders

## Extension Ideas

### Beginner Extensions
- Export heightmap as image
- Import heightmap from image
- Multiple terrain presets (mountains, plains, islands)
- Time-of-day lighting

### Intermediate Extensions
- Terrain painting tool (modify heights, textures)
- Roads and paths (flatten terrain, custom texture)
- Detail textures (macro + micro variation)
- GPU-based generation (compute shaders)

### Advanced Extensions
- Volumetric terrain (caves, overhangs)
- Hydraulic erosion simulation
- Real-time deformable terrain (digging, explosions)
- Planetary terrain (spherical, infinite LOD)

## Success Criteria

Your procedural terrain system should:

1. ✅ Generate diverse, interesting terrain from noise
2. ✅ Handle large view distances (1000+ units) smoothly
3. ✅ Maintain 60 FPS with LOD system active
4. ✅ Provide seamless chunk boundaries (no cracks)
5. ✅ Support infinite or very large terrain sizes
6. ✅ Render with realistic texturing and lighting
7. ✅ Respond to parameter changes quickly (< 1 second for regen)

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Generation** | Single heightmap, basic noise | Multi-chunk, fractal noise | Infinite streaming, biomes |
| **LOD** | No LOD or basic distance culling | 2-3 LOD levels, smooth transitions | 4+ levels, geomorphing, optimized |
| **Texturing** | Single texture | Height-based splatting, 2-3 textures | Slope + height, 4+ textures, normals |
| **Performance** | 30 FPS, small terrain | 60 FPS, medium terrain | 60 FPS, infinite terrain, optimized |

## Common Pitfalls

1. **No Delta Time**: Use consistent time/distance units for noise sampling
2. **Ignoring Normals**: Always recalculate normals after heightmap changes
3. **Chunk Edge Misalignment**: Ensure edge vertices use exact same heightmap values
4. **Too Many Draw Calls**: Batch chunks, minimize state changes
5. **Synchronous Generation**: Use background threads for chunk generation
6. **Memory Leaks**: Unload old chunks, free GPU buffers
7. **Z-Fighting**: Use appropriate depth buffer precision, avoid overlapping geometry

## Next Steps

After completing this project, you're ready for:
- **Project 06**: Particle Effects System (place effects on terrain)
- **Project 08**: Scene Editor (terrain editing tools)
- **Project 10**: Mini Game Engine (integrate terrain as subsystem)
