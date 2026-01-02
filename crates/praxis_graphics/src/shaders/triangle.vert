#version 450

// ============================================================================
// Vertex Shader for 3D Textured Rendering with Lighting
// ============================================================================
//
// This shader transforms vertices from model space to clip space through a
// series of coordinate system transformations. It also prepares data needed
// by the fragment shader for lighting calculations and texture sampling.
//
// # Coordinate System Transformations
//
// Vertices undergo several transformations to go from 3D model space to
// 2D screen coordinates:
//
// 1. **Model Space → World Space** (Model Matrix)
//    - Model space: Coordinates relative to the object's local origin
//    - World space: Coordinates in the global scene
//    - Example: A cube at (0,0,0) in model space might be at (10,5,0) in world
//
// 2. **World Space → View/Camera Space** (View Matrix)
//    - World space: Global scene coordinates
//    - View space: Coordinates relative to the camera (camera at origin)
//    - The view matrix represents the camera's position and orientation
//
// 3. **View Space → Clip Space** (Projection Matrix)
//    - View space: Camera-relative coordinates
//    - Clip space: Homogeneous coordinates for perspective division
//    - Projection matrix creates perspective (far objects smaller)
//    - Coordinates outside [-w, w] for x,y,z are clipped (not drawn)
//
// 4. **Clip Space → NDC → Screen Space** (Automatic)
//    - GPU automatically divides by w (perspective division)
//    - Normalized Device Coordinates (NDC): [-1, 1] for x,y,z
//    - Viewport transform maps NDC to screen pixels
//
// Complete transformation: P * V * M * position
//   where P = projection, V = view, M = model
//
// # Data Flow: CPU to GPU
//
// Data flows from the application (Rust) to this shader through:
//
// ## 1. Vertex Attributes (per-vertex data)
//    - Source: Vertex buffers uploaded by application
//    - Contains: position, normal, color, UV coordinates
//    - Type: `Vertex3D` struct in Rust (vertex.rs)
//    - Upload: Once when mesh is loaded, stored in device-local GPU memory
//    - Frequency: Data is static unless mesh is modified
//    - Access: Each vertex shader invocation gets ONE vertex's attributes
//
//    Memory layout (Vertex3D):
//      Location 0: vec3 position (12 bytes) - vertex position in model space
//      Location 1: vec3 normal   (12 bytes) - vertex normal in model space
//      Location 2: vec3 color    (12 bytes) - vertex color (RGB)
//      Location 3: vec2 uv       (8 bytes)  - texture coordinates
//      Total: 44 bytes per vertex (may be padded to 48 for alignment)
//
// ## 2. Uniform Buffer (per-object data)
//    - Source: `Uniforms` struct in Rust (lib.rs)
//    - Contains: model, view, projection matrices, and camera position
//    - Descriptor: Set 0, Binding 0
//    - Upload: Every frame, per object drawn
//    - Frequency: Updated each frame for dynamic objects, less often for static
//    - Access: Same data for all vertices of an object
//    - Memory: Host-visible buffer, 208 bytes (3 matrices + camera position + padding)
//
//    Memory layout (Uniforms):
//      Offset 0:   mat4 model (64 bytes) - model-to-world transform
//      Offset 64:  mat4 view  (64 bytes) - world-to-view transform  
//      Offset 128: mat4 proj  (64 bytes) - view-to-clip transform
//      Offset 192: vec3 camera_position (12 bytes) - camera position in world space
//      Offset 204: float _padding (4 bytes) - std140 alignment padding
//      Total: 208 bytes
//
// ## 3. Output to Fragment Shader (interpolated per-fragment)
//    - World position: For lighting distance calculations
//    - World normal: For lighting angle calculations
//    - Vertex color: For color tinting
//    - UV coordinates: For texture sampling
//
//    The GPU automatically interpolates these values across the triangle:
//    - If a triangle has vertices at (0,0,0), (1,0,0), (0.5,1,0)
//    - A fragment at the center gets interpolated values from all 3 vertices
//    - This is why normals and UVs smoothly vary across surfaces
//
// # Normal Transformation
//
// Normals require special handling when transforming to world space:
// - Position: Transform with full model matrix (model * position)
// - Normal: Transform with 3x3 upper-left of model matrix (mat3(model) * normal)
//
// Why only the 3x3 part?
// - Normals are directions, not positions (no translation needed)
// - We only need rotation and scale, not translation
//
// For non-uniform scaling (different scale on x,y,z), normals should use:
//   transpose(inverse(mat3(model))) * normal
// But for uniform scaling or rotation-only, mat3(model) is sufficient.
//
// # Coordinate Handedness
//
// - Model/World/View space: Right-handed (x right, y up, z backward)
// - Clip/NDC space: Right-handed in Vulkan (z forward is positive)
// - Vulkan uses: x right, y down, z forward in NDC
//
// The projection matrix handles the y-flip convention difference.

// ============================================================================
// Input Vertex Attributes
// ============================================================================
// These come from the vertex buffer and vary per vertex

layout(location = 0) in vec3 position;  // Vertex position in model space
layout(location = 1) in vec3 normal;    // Vertex normal in model space (unit vector)
layout(location = 2) in vec3 color;     // Vertex color (RGB, range [0,1])
layout(location = 3) in vec2 uv;        // Texture coordinates (range typically [0,1])

// ============================================================================
// Output Variables (to Fragment Shader)
// ============================================================================
// These are interpolated across the triangle by the GPU

layout(location = 0) out vec3 v_world_pos;  // Fragment position in world space
layout(location = 1) out vec3 v_normal;     // Fragment normal in world space
layout(location = 2) out vec3 v_color;      // Fragment color (interpolated)
layout(location = 3) out vec2 v_uv;         // Fragment UV coordinates (interpolated)

// ============================================================================
// Uniform Buffer (Transformation Matrices)
// ============================================================================
// This uniform buffer contains the matrices needed for coordinate transformations
//
// Data Flow:
//   1. CPU: Application computes matrices based on object position and camera
//   2. CPU: Matrices packed into `Uniforms` struct (lib.rs)
//   3. CPU: Written to host-visible uniform buffer
//   4. GPU: Bound to descriptor set at binding 0
//   5. GPU: Read by vertex shader for each vertex
//
// The std140 layout ensures consistent memory layout between CPU and GPU:
// - mat4 is 64 bytes (16 floats * 4 bytes)
// - Each column is 16-byte aligned (vec4)
// - vec3 + float padding is 16 bytes
// - Total buffer size: 208 bytes (3 * 64 + 16)

layout(set = 0, binding = 0, std140) uniform Uniforms {
    mat4 model;  // Model matrix: transforms model space → world space
    mat4 view;   // View matrix: transforms world space → view/camera space  
    mat4 proj;   // Projection matrix: transforms view space → clip space
    vec3 camera_position;  // Camera position in world space
    float _padding;  // Padding for std140 alignment
} ubo;  // UBO = Uniform Buffer Object

// ============================================================================
// Main Vertex Shader
// ============================================================================
void main() {
    // ========================================================================
    // Step 1: Transform position to world space
    // ========================================================================
    
    // Apply model matrix to transform from model space to world space
    // We need world-space position for lighting calculations in fragment shader
    // (point lights need distance from fragment to light position)
    //
    // The position is a vec3, but we convert to vec4 with w=1.0 for the transform
    // - w=1.0 indicates this is a POSITION (affected by translation)
    // - w=0.0 would indicate a DIRECTION (not affected by translation)
    //
    // Matrix-vector multiplication: [4x4] * [4x1] = [4x1]
    vec4 world_pos = ubo.model * vec4(position, 1.0);
    
    // ========================================================================
    // Step 2: Transform position to clip space
    // ========================================================================
    
    // Apply view and projection matrices in sequence
    // Order matters: first view (world→view), then projection (view→clip)
    //
    // This is the standard MVP (Model-View-Projection) transformation:
    //   clip_pos = proj * view * model * position
    //
    // The result is stored in gl_Position, a built-in output variable
    // The GPU uses gl_Position for:
    //   - Clipping (discarding vertices outside view frustum)
    //   - Perspective division (dividing x,y,z by w)
    //   - Viewport transformation (NDC to screen coordinates)
    gl_Position = ubo.proj * ubo.view * world_pos;
    
    // ========================================================================
    // Step 3: Transform normal to world space
    // ========================================================================
    
    // Normals need special handling when transforming to world space
    //
    // We use only the 3x3 upper-left part of the model matrix:
    //   - Normals are directions (vectors), not positions
    //   - Directions aren't affected by translation (only by rotation/scale)
    //   - The 4th row/column of the matrix contains translation
    //   - mat3(ubo.model) extracts just the rotation/scale portion
    //
    // For non-uniform scaling (different scale on x,y,z axes), we should use:
    //   v_normal = mat3(transpose(inverse(ubo.model))) * normal;
    // This is the "normal matrix" that handles non-uniform scaling correctly.
    //
    // However, for uniform scaling and rotations, mat3(ubo.model) works fine
    // and is more efficient (no inverse/transpose calculation needed).
    //
    // The fragment shader will re-normalize this vector since interpolation
    // can change its length.
    v_normal = mat3(ubo.model) * normal;
    
    // ========================================================================
    // Step 4: Pass world position to fragment shader
    // ========================================================================
    
    // Extract xyz components of world position (drop w component)
    // Fragment shader needs this for:
    //   - Point light distance calculations
    //   - View direction calculations (camera_pos - world_pos)
    //
    // This value will be interpolated across the triangle, giving each
    // fragment its precise world-space position.
    v_world_pos = world_pos.xyz;
    
    // ========================================================================
    // Step 5: Pass color to fragment shader
    // ========================================================================
    
    // Vertex colors are passed through unchanged
    // These will be linearly interpolated across the triangle
    //
    // Uses:
    //   - Vertex-based color gradients
    //   - Tinting/modulating textures
    //   - Debug visualization (e.g., color-coding normals)
    v_color = color;
    
    // ========================================================================
    // Step 6: Pass UV coordinates to fragment shader
    // ========================================================================
    
    // Texture coordinates are passed through unchanged
    // These will be linearly interpolated across the triangle
    //
    // The fragment shader uses these to sample the texture:
    //   - UV (0,0) typically represents top-left or bottom-left of texture
    //   - UV (1,1) typically represents bottom-right or top-right
    //   - Values outside [0,1] may wrap or clamp depending on sampler settings
    //
    // Interpolation ensures smooth texture mapping across the triangle
    v_uv = uv;
}
