# Project 01: 3D Model Viewer

**Difficulty**: Beginner  
**Estimated Time**: 1-2 weeks  
**Core Learning**: Asset loading, camera controls, basic lighting, transformation hierarchies

## Overview

Build a 3D model viewer that loads and displays mesh files with camera navigation and basic lighting. This project teaches fundamental 3D graphics concepts including coordinate spaces, camera systems, mesh data structures, and the rendering pipeline.

### Learning Objectives

- Understand 3D asset file formats (OBJ, GLTF)
- Implement camera controls (orbit, pan, zoom)
- Apply basic lighting models (Phong/Blinn-Phong)
- Manage transformation matrices
- Structure a basic rendering pipeline

## Feature Requirements

### Core Features (Minimum Viable)

1. **Mesh Loading**
   - Load OBJ or GLTF format files
   - Parse vertex positions, normals, and texture coordinates
   - Support multiple meshes in a single file
   - Display mesh statistics (vertex/triangle count)

2. **Camera System**
   - Orbit camera (rotate around target point)
   - Pan (translate view horizontally/vertically)
   - Zoom (dolly camera in/out)
   - Frame model (auto-fit to viewport)
   - Perspective projection

3. **Basic Lighting**
   - Single directional light
   - Diffuse and specular components
   - Ambient lighting
   - Normal visualization mode (optional debug view)

4. **Display Controls**
   - Wireframe/solid toggle
   - Reset camera button
   - Background color selection
   - FPS counter

### Extended Features (Recommended)

5. **Material Support**
   - Load material definitions from model files
   - Apply diffuse textures
   - Display untextured models with flat colors
   - Material property visualization

6. **Multi-Model Support**
   - Load multiple models simultaneously
   - Switch between loaded models
   - Simple scene hierarchy display
   - Model transformation controls (translate, rotate, scale)

7. **Enhanced Lighting**
   - Multiple light sources
   - Point and spot lights
   - Interactive light positioning
   - Shadow mapping (basic)

### Stretch Goals

8. **Advanced Camera**
   - First-person camera mode
   - Fly-through mode
   - Camera position bookmarks
   - Smooth camera transitions

9. **Rendering Options**
   - Vertex/normal visualization
   - UV coordinate display
   - Bounding box/sphere display
   - LOD level preview

## Architecture Guidance

### System Components

```
ModelViewer
├── AssetLoader
│   ├── MeshParser (OBJ/GLTF)
│   ├── TextureLoader
│   └── MaterialParser
├── Renderer
│   ├── MeshRenderer
│   ├── ShaderManager
│   └── LightingSystem
├── CameraController
│   ├── OrbitCamera
│   └── InputHandler
└── UI
    ├── FileDialog
    ├── ControlPanel
    └── StatsDisplay
```

### Data Structures

**Mesh Representation**
```
Mesh:
  - vertices: array of positions (vec3)
  - normals: array of normals (vec3)
  - texcoords: array of UVs (vec2)
  - indices: array of triangle indices
  - material_id: reference to material

Material:
  - diffuse_color: vec3
  - specular_color: vec3
  - shininess: float
  - diffuse_texture: texture reference (optional)

Light:
  - type: directional/point/spot
  - position/direction: vec3
  - color: vec3
  - intensity: float
```

**Camera State**
```
OrbitCamera:
  - target: vec3 (look-at point)
  - distance: float (zoom level)
  - azimuth: float (horizontal rotation)
  - elevation: float (vertical rotation)
  - fov: float (field of view)
  - aspect_ratio: float

Methods:
  - get_view_matrix() -> mat4
  - get_projection_matrix() -> mat4
  - update_from_input(delta_mouse, scroll)
```

### Rendering Pipeline

1. **Initialization**
   - Create graphics context
   - Compile shaders
   - Set up render targets

2. **Per-Frame**
   - Process input
   - Update camera matrices
   - Clear framebuffer
   - For each mesh:
     - Bind vertex/index buffers
     - Set shader uniforms (MVP, lighting)
     - Draw triangles
   - Render UI overlay
   - Present frame

3. **Shader Structure**

**Vertex Shader**
```glsl
// Inputs: position, normal, texcoord
// Uniforms: model, view, projection matrices
// Outputs: world position, world normal, texcoord
```

**Fragment Shader**
```glsl
// Inputs: world position, world normal, texcoord
// Uniforms: light direction/position, material properties
// Output: final color (Phong lighting)
```

## Milestone Plan

### Milestone 1: Basic Rendering (Week 1, Days 1-3)

**Goal**: Display a hardcoded triangle/cube

**Tasks**:
- Set up rendering context (window, graphics API initialization)
- Create simple vertex and fragment shaders
- Define vertex buffer with hardcoded geometry
- Implement basic render loop
- Display a colored 3D shape

**Deliverable**: Spinning cube or triangle with solid color

### Milestone 2: Camera System (Week 1, Days 4-5)

**Goal**: Implement orbit camera controls

**Tasks**:
- Implement OrbitCamera class/struct
- Calculate view and projection matrices
- Handle mouse input (drag for orbit, scroll for zoom)
- Add pan controls (middle mouse or Shift+drag)
- Implement "frame model" feature

**Deliverable**: Interactive camera that orbits around origin

### Milestone 3: Mesh Loading (Week 1, Days 6-7)

**Goal**: Load and display OBJ files

**Tasks**:
- Implement OBJ file parser (or use existing library)
- Parse vertex positions, normals, texture coordinates
- Build vertex/index buffers from parsed data
- Calculate mesh bounding box for auto-framing
- Load and display a test model

**Deliverable**: Load and display external OBJ models

### Milestone 4: Basic Lighting (Week 2, Days 1-3)

**Goal**: Implement Phong lighting model

**Tasks**:
- Extend shaders with lighting calculations
- Add directional light uniform
- Implement diffuse lighting (N·L)
- Implement specular highlights
- Add ambient term
- Make light direction controllable

**Deliverable**: Properly lit 3D models with shading

### Milestone 5: Material Support (Week 2, Days 4-5)

**Goal**: Support materials and textures

**Tasks**:
- Parse material definitions from OBJ/MTL files
- Implement texture loading
- Update shaders to sample textures
- Handle models without textures (use flat color)
- Display material properties in UI

**Deliverable**: Textured models with material properties

### Milestone 6: UI and Polish (Week 2, Days 6-7)

**Goal**: Add user interface and polish features

**Tasks**:
- Add file picker dialog
- Create control panel (wireframe toggle, reset camera, etc.)
- Display mesh statistics (vertex/tri count)
- Add FPS counter
- Implement background color picker
- Error handling for invalid files

**Deliverable**: Complete, polished model viewer with UI

## Technical Challenges

### Challenge 1: Coordinate Space Transformations

**Problem**: Understanding model, world, view, and clip spaces

**Approach**:
- Study transformation matrix pipeline
- Visualize each space transformation
- Implement matrix debugging (print view/projection matrices)
- Test with simple shapes before complex models

**Key Concepts**:
- Model matrix: local → world space
- View matrix: world → camera space
- Projection matrix: camera → clip space
- MVP = Projection × View × Model

### Challenge 2: Camera Controls Feel

**Problem**: Making orbit camera feel natural and intuitive

**Approach**:
- Use spherical coordinates (azimuth, elevation, distance)
- Clamp elevation to prevent gimbal lock (e.g., -89° to +89°)
- Add damping/smoothing for professional feel
- Implement proper "frame model" using bounding boxes

**Tips**:
- Store camera as target point + spherical offset
- Convert to Cartesian for view matrix calculation
- Handle input in screen space, convert to camera-relative rotations

### Challenge 3: Normal Transformation

**Problem**: Normals don't transform the same way as positions

**Approach**:
- Use transpose of inverse model matrix for normals
- Understand why: normals are perpendicular vectors
- For uniform scaling, model matrix works; otherwise, use proper transform
- Normalize normals in shader after transformation

### Challenge 4: Asset Loading Performance

**Problem**: Large models take time to parse and upload to GPU

**Approach**:
- Load assets asynchronously (separate thread)
- Show loading indicator during parse/upload
- Stream large meshes in chunks (advanced)
- Cache parsed data for reload
- Use binary formats (GLTF binary) for faster loading

### Challenge 5: Lighting Artifacts

**Problem**: Specular highlights look wrong or faces appear flat

**Approach**:
- Verify normals are unit length after transformation
- Ensure normals point outward (check winding order)
- Use high polygon models for smooth specular highlights
- Visualize normals as colors to debug (R = normal.x, etc.)
- Check light direction is normalized

## Reference Implementations

### Praxis Engine (Rust)
- **File**: `examples/comprehensive_scene_demo.rs`
- **Concepts**: Mesh loading, camera, lighting, material system
- **Crates**: `praxis_assets`, `praxis_graphics`, `praxis_scene`

### Other Engines/Frameworks

**Unity (C#)**
- Tutorial: "3D Model Viewer" (Unity Learn)
- Key APIs: `AssetDatabase`, `Camera`, `Material`, `Light`

**Unreal Engine (C++)**
- Tutorial: "Model Viewer Template"
- Key APIs: `UStaticMeshComponent`, `UCameraComponent`, `ULightComponent`

**Three.js (JavaScript)**
- Example: [three.js model viewer](https://threejs.org/examples/#webgl_loader_gltf)
- Key APIs: `GLTFLoader`, `OrbitControls`, `DirectionalLight`

**OpenGL (C++)**
- Learn OpenGL: [Model Loading](https://learnopengl.com/Model-Loading/Model)
- Libraries: Assimp (asset loading), GLM (math), GLFW (window)

**raylib (C)**
- Example: `models/models_loading.c`
- Key APIs: `LoadModel()`, `UpdateCamera()`, `DrawModel()`

**Godot (GDScript)**
- Tutorial: "3D Model Viewer Scene"
- Key Nodes: `MeshInstance`, `Camera`, `DirectionalLight`

## Extension Ideas

### Beginner Extensions
- Screenshot capture
- Multiple background colors/gradients
- Model rotation animation
- Grid/ground plane display

### Intermediate Extensions
- GLTF animation playback
- PBR material support
- Environment maps for reflections
- HDR lighting

### Advanced Extensions
- Real-time mesh editing (vertex manipulation)
- Model comparison mode (side-by-side)
- VR/AR model viewing
- Shader hot-reloading for experimentation

## Success Criteria

Your model viewer should:

1. ✅ Load standard 3D model formats without crashing
2. ✅ Render models with correct geometry and normals
3. ✅ Provide smooth, intuitive camera controls
4. ✅ Apply realistic lighting (no pure-black or blown-out areas)
5. ✅ Display textures correctly (if present in model)
6. ✅ Run at 60+ FPS for models with <100K triangles
7. ✅ Handle errors gracefully (invalid files, missing textures)
8. ✅ Provide clear UI for all controls

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Functionality** | Loads OBJ, basic camera, directional light | + Textures, materials, multiple lights | + GLTF, animations, advanced rendering |
| **Code Quality** | Works, some organization | Clear structure, reusable components | Modular, extensible, well-documented |
| **User Experience** | Usable controls, basic UI | Intuitive controls, polished UI | Professional feel, keyboard shortcuts |
| **Performance** | 30+ FPS for simple models | 60 FPS for medium models | 60 FPS for complex models, optimized |

## Common Pitfalls

1. **Incorrect Matrix Order**: Remember matrix multiplication order matters (often right-to-left)
2. **Missing Normalization**: Always normalize vectors in lighting calculations
3. **Wrong Winding Order**: CCW vs CW affects backface culling
4. **Coordinate System Confusion**: Know your API's handedness (right vs left)
5. **Ignoring Aspect Ratio**: Always match projection to viewport dimensions
6. **Hardcoded Paths**: Make asset loading flexible (file dialog, relative paths)

## Next Steps

After completing this project, you're ready for:
- **Project 02**: First-Person Explorer (input handling, collision)
- **Project 04**: Animation Showcase (skeletal animation, blending)
- **Project 08**: Scene Editor (selection, manipulation, serialization)
