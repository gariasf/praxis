# Coordinate Spaces

Understanding coordinate spaces (also called "reference frames" or "coordinate systems") is essential for 3D graphics and game development. The same point in 3D can have different coordinates depending on which space it's measured from.

## What is a Coordinate Space?

A **coordinate space** defines:
- An **origin**: The (0, 0, 0) point
- **Basis vectors**: The directions of the X, Y, and Z axes
- **Scale**: The unit of measurement

The same physical point can have different coordinates in different spaces.

### Example: A Coffee Cup on a Table

Consider a coffee cup:
- **Room space**: Cup is at (2m, 1m, 3m) from the room's corner
- **Table space**: Cup is at (0.5m, 0m, 0.2m) from the table's center
- **Cup space**: The cup's handle is at (0.08m, 0m, 0.03m) from the cup's center

Same object, different coordinates—different coordinate spaces!

## Common Coordinate Spaces in Game Engines

### 1. Local Space (Object Space, Model Space)

**Definition**: Coordinates relative to an object's own origin and orientation.

**Origin**: The object's pivot point (often its center)

**Axes**: Aligned with the object's natural orientation

**Example**: A character model's vertex positions in the 3D modeling software

**Uses**:
- Mesh vertex data
- Physics colliders
- Animation offsets

```text
Character Model (local space):
  Head: (0, 1.8, 0)
  Left Hand: (-0.5, 1.0, 0.2)
  Right Foot: (0.15, 0, 0)
```

### 2. World Space (Global Space)

**Definition**: The "absolute" coordinate system for the entire scene.

**Origin**: Arbitrary fixed point (often the scene center)

**Axes**: Fixed directions (typically Y-up or Z-up)

**Example**: Positions of all objects in the game level

**Uses**:
- Object positions in the scene
- Physics simulation
- Spatial partitioning (octrees, grids)

```text
World Space:
  Player: (10.5, 0, 5.3)
  Enemy: (15.2, 0, 8.7)
  Camera: (12.0, 5.0, 3.0)
```

### 3. View Space (Camera Space, Eye Space)

**Definition**: Coordinates relative to the camera's position and orientation.

**Origin**: Camera position

**Axes**: 
- X: Right
- Y: Up
- Z: Forward (or backward, depending on convention)

**Example**: Object positions as "seen" by the camera

**Uses**:
- Lighting calculations
- Culling (frustum, occlusion)
- Effects (fog, depth of field)

```text
View Space (from camera's perspective):
  Object in front: (0, 0, 5)     // 5 units ahead
  Object to right: (3, 0, 2)     // 3 units right, 2 units ahead
  Object behind:   (0, 0, -1)    // 1 unit behind camera
```

### 4. Clip Space

**Definition**: Normalized coordinates after perspective projection.

**Range**: Typically [-1, 1] for X, Y and [0, 1] or [-1, 1] for Z (API-dependent)

**Purpose**: Prepares geometry for clipping and rasterization

**Transform**: View space → Clip space via projection matrix

### 5. Screen Space (Viewport Space)

**Definition**: 2D pixel coordinates on the screen.

**Origin**: Top-left or bottom-left corner (convention varies)

**Units**: Pixels

**Range**: X: [0, width], Y: [0, height]

**Uses**:
- UI rendering
- Mouse picking
- Post-processing effects

## Coordinate Space Transformations

### Transformation Pipeline

The standard rendering pipeline transforms vertices through multiple spaces:

```text
Local Space (Mesh)
    ↓ Model Matrix (M)
World Space
    ↓ View Matrix (V)
View Space
    ↓ Projection Matrix (P)
Clip Space
    ↓ Perspective Divide
NDC (Normalized Device Coordinates)
    ↓ Viewport Transform
Screen Space
```

### Model Matrix (Local → World)

Transforms from object's local coordinates to world coordinates:

$$\mathbf{M} = \mathbf{T} \cdot \mathbf{R} \cdot \mathbf{S}$$

Where:
- $\mathbf{T}$: Translation (object's world position)
- $\mathbf{R}$: Rotation (object's world orientation)
- $\mathbf{S}$: Scale (object's size)

=== "Pseudocode"
    ```
    function create_model_matrix(position, rotation, scale):
        T = translation_matrix(position)
        R = rotation_matrix(rotation)
        S = scale_matrix(scale)
        return T * R * S
    ```

=== "Rust (glam)"
    ```rust
    let position = Vec3::new(5.0, 0.0, 10.0);
    let rotation = Quat::from_rotation_y(45_f32.to_radians());
    let scale = Vec3::splat(2.0);
    
    let model = Mat4::from_scale_rotation_translation(scale, rotation, position);
    
    // Transform local vertex to world
    let local_vertex = Vec3::new(1.0, 0.0, 0.0);
    let world_vertex = model.transform_point3(local_vertex);
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 position(5.0f, 0.0f, 10.0f);
    glm::quat rotation = glm::angleAxis(glm::radians(45.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::vec3 scale(2.0f);
    
    glm::mat4 T = glm::translate(glm::mat4(1.0f), position);
    glm::mat4 R = glm::mat4_cast(rotation);
    glm::mat4 S = glm::scale(glm::mat4(1.0f), scale);
    glm::mat4 model = T * R * S;
    
    glm::vec3 localVertex(1.0f, 0.0f, 0.0f);
    glm::vec3 worldVertex = glm::vec3(model * glm::vec4(localVertex, 1.0f));
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 position = new float3(5.0f, 0.0f, 10.0f);
    quaternion rotation = quaternion.RotateY(math.radians(45.0f));
    float3 scale = new float3(2.0f);
    
    float4x4 model = float4x4.TRS(position, rotation, scale);
    
    float3 localVertex = new float3(1.0f, 0.0f, 0.0f);
    float3 worldVertex = math.transform(model, localVertex);
    ```

### View Matrix (World → View)

Transforms from world coordinates to camera's perspective:

$$\mathbf{V} = \text{inverse}(\mathbf{M}_{\text{camera}})$$

Or equivalently, using look-at:

$$\mathbf{V} = \text{lookAt}(\text{eye}, \text{target}, \text{up})$$

=== "Pseudocode"
    ```
    function create_view_matrix(camera_pos, camera_target, up):
        // Forward direction (camera looks "into" the scene)
        forward = normalize(camera_target - camera_pos)
        
        // Right vector
        right = normalize(cross(forward, up))
        
        // Recompute up (ensure orthogonal)
        up_actual = cross(right, forward)
        
        // Build rotation (camera axes)
        rotation = [
            [right.x,      up_actual.x,      -forward.x,      0],
            [right.y,      up_actual.y,      -forward.y,      0],
            [right.z,      up_actual.z,      -forward.z,      0],
            [0,            0,                 0,              1]
        ]
        
        // Translation (move world opposite to camera)
        translation = translation_matrix(-camera_pos)
        
        return rotation * translation
    ```

=== "Rust (glam)"
    ```rust
    let camera_pos = Vec3::new(0.0, 5.0, 10.0);
    let target = Vec3::ZERO;
    let up = Vec3::Y;
    
    let view = Mat4::look_at_rh(camera_pos, target, up);
    
    // Transform world point to view space
    let world_point = Vec3::new(5.0, 0.0, 0.0);
    let view_point = view.transform_point3(world_point);
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 cameraPos(0.0f, 5.0f, 10.0f);
    glm::vec3 target(0.0f, 0.0f, 0.0f);
    glm::vec3 up(0.0f, 1.0f, 0.0f);
    
    glm::mat4 view = glm::lookAt(cameraPos, target, up);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 cameraPos = new float3(0.0f, 5.0f, 10.0f);
    float3 target = new float3(0.0f, 0.0f, 0.0f);
    float3 up = new float3(0.0f, 1.0f, 0.0f);
    
    float4x4 view = float4x4.LookAt(cameraPos, target, up);
    ```

### Projection Matrix (View → Clip)

Transforms view space to clip space, applying perspective or orthographic projection.

See [Matrices - Projection Matrix](matrices.md#projection-matrix) for details.

### Complete MVP Transform

In shaders, vertices are typically transformed by the **Model-View-Projection (MVP)** matrix:

$$\text{MVP} = \mathbf{P} \cdot \mathbf{V} \cdot \mathbf{M}$$

=== "Rust (glam)"
    ```rust
    // Per-object model matrix
    let model = Mat4::from_scale_rotation_translation(scale, rotation, position);
    
    // Camera view matrix
    let view = Mat4::look_at_rh(camera_pos, target, up);
    
    // Projection matrix
    let projection = Mat4::perspective_rh(
        60_f32.to_radians(),  // FOV
        16.0 / 9.0,           // Aspect ratio
        0.1,                  // Near
        100.0,                // Far
    );
    
    // Combined transform
    let mvp = projection * view * model;
    
    // Transform vertex from local to clip space
    let local_vertex = Vec3::new(1.0, 0.0, 0.0);
    let clip_pos = mvp.transform_point3(local_vertex);
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 model = /* ... */;
    glm::mat4 view = /* ... */;
    glm::mat4 projection = /* ... */;
    
    glm::mat4 mvp = projection * view * model;
    
    glm::vec4 localVertex(1.0f, 0.0f, 0.0f, 1.0f);
    glm::vec4 clipPos = mvp * localVertex;
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 model = /* ... */;
    float4x4 view = /* ... */;
    float4x4 projection = /* ... */;
    
    float4x4 mvp = math.mul(projection, math.mul(view, model));
    
    float3 localVertex = new float3(1.0f, 0.0f, 0.0f);
    float4 clipPos = math.mul(mvp, new float4(localVertex, 1.0f));
    ```

## Left-Handed vs. Right-Handed Coordinates

Different engines use different handedness conventions:

### Right-Handed (OpenGL, glm, Vulkan)

- **X**: Right
- **Y**: Up
- **Z**: Out of screen (toward viewer)

```text
     Y (up)
     |
     |_____ X (right)
    /
   Z (toward you)
```

### Left-Handed (DirectX, Unity)

- **X**: Right
- **Y**: Up
- **Z**: Into screen (away from viewer)

```text
     Y (up)
     |
     |_____ X (right)
   /
  Z (away from you)
```

**Practical Impact**:
- Different view/projection matrices
- Cross product results flipped
- Winding order for culling reversed

!!! tip "Consistency"
    Stick to your engine's convention. Most math libraries provide both variants:
    - `look_at_rh()` vs. `look_at_lh()`
    - `perspective_rh()` vs. `perspective_lh()`

## Transforming Directions vs. Points

### Points (Positions)

**Affected by translation**: Use $w = 1$ in homogeneous coordinates

$$\mathbf{p}_{\text{transformed}} = \mathbf{M} \begin{bmatrix} x \\ y \\ z \\ 1 \end{bmatrix}$$

### Directions (Vectors)

**Not affected by translation**: Use $w = 0$ in homogeneous coordinates

$$\mathbf{d}_{\text{transformed}} = \mathbf{M} \begin{bmatrix} x \\ y \\ z \\ 0 \end{bmatrix}$$

### Normals

**Require special handling** when non-uniform scaling is present.

Use the **inverse-transpose** of the model matrix:

$$\mathbf{n}_{\text{transformed}} = (\mathbf{M}^{-1})^T \mathbf{n}$$

=== "Rust (glam)"
    ```rust
    let model = Mat4::from_scale(Vec3::new(2.0, 1.0, 1.0));  // Non-uniform scale
    
    // Transform normal correctly
    let normal = Vec3::Y;
    let normal_matrix = model.inverse().transpose();
    let transformed_normal = (normal_matrix * normal.extend(0.0)).truncate().normalize();
    
    // Or use 3x3 submatrix
    let normal_matrix_3x3 = Mat3::from_mat4(model).inverse().transpose();
    let transformed_normal = (normal_matrix_3x3 * normal).normalize();
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 model = glm::scale(glm::mat4(1.0f), glm::vec3(2.0f, 1.0f, 1.0f));
    
    // Normal matrix
    glm::mat3 normalMatrix = glm::transpose(glm::inverse(glm::mat3(model)));
    
    glm::vec3 normal(0.0f, 1.0f, 0.0f);
    glm::vec3 transformedNormal = glm::normalize(normalMatrix * normal);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 model = float4x4.Scale(new float3(2.0f, 1.0f, 1.0f));
    
    // Extract 3x3 rotation/scale part
    float3x3 model3x3 = new float3x3(model);
    float3x3 normalMatrix = math.transpose(math.inverse(model3x3));
    
    float3 normal = new float3(0.0f, 1.0f, 0.0f);
    float3 transformedNormal = math.normalize(math.mul(normalMatrix, normal));
    ```

## Practical Examples

### Example 1: Attach Object to Character's Hand

Attach a sword to a character's right hand bone:

=== "Pseudocode"
    ```
    function attach_to_bone(object, character, bone_name):
        // Get bone's world transform
        bone_world_matrix = character.model_matrix * character.skeleton.bone_matrix(bone_name)
        
        // Object's local offset (relative to bone)
        object_local_offset = translation_matrix(0.1, 0, 0)  // 0.1 units to the right
        
        // Compute object's world transform
        object.model_matrix = bone_world_matrix * object_local_offset
    ```

=== "Rust (glam)"
    ```rust
    fn attach_to_bone(
        character_model: Mat4,
        bone_local: Mat4,
        object_offset: Mat4,
    ) -> Mat4 {
        // Bone in world space
        let bone_world = character_model * bone_local;
        
        // Object in world space
        bone_world * object_offset
    }
    ```

### Example 2: Convert World Position to Screen Position

Project a 3D world point to 2D screen coordinates (for UI markers):

=== "Pseudocode"
    ```
    function world_to_screen(world_pos, mvp, screen_width, screen_height):
        // Transform to clip space
        clip_pos = mvp * Vector4(world_pos.x, world_pos.y, world_pos.z, 1.0)
        
        // Perspective divide
        if clip_pos.w == 0:
            return None  // Point at infinity
        
        ndc = Vector3(
            clip_pos.x / clip_pos.w,
            clip_pos.y / clip_pos.w,
            clip_pos.z / clip_pos.w
        )
        
        // Check if point is behind camera
        if ndc.z < 0 or ndc.z > 1:
            return None
        
        // NDC to screen space
        screen_x = (ndc.x + 1.0) * 0.5 * screen_width
        screen_y = (1.0 - ndc.y) * 0.5 * screen_height  // Flip Y
        
        return Vector2(screen_x, screen_y)
    ```

=== "Rust (glam)"
    ```rust
    fn world_to_screen(
        world_pos: Vec3,
        mvp: Mat4,
        screen_size: Vec2,
    ) -> Option<Vec2> {
        // To clip space
        let clip_pos = mvp * world_pos.extend(1.0);
        
        if clip_pos.w == 0.0 {
            return None;
        }
        
        // Perspective divide
        let ndc = clip_pos.truncate() / clip_pos.w;
        
        // Check if behind camera or outside frustum
        if ndc.z < 0.0 || ndc.z > 1.0 {
            return None;
        }
        
        // NDC [-1,1] to screen [0, width/height]
        let screen_x = (ndc.x + 1.0) * 0.5 * screen_size.x;
        let screen_y = (1.0 - ndc.y) * 0.5 * screen_size.y;
        
        Some(Vec2::new(screen_x, screen_y))
    }
    ```

### Example 3: Mouse Picking (Screen to World Ray)

Cast a ray from screen coordinates into the 3D world:

=== "Pseudocode"
    ```
    function screen_to_world_ray(screen_pos, inv_mvp, screen_width, screen_height):
        // Screen to NDC
        ndc_x = (2.0 * screen_pos.x / screen_width) - 1.0
        ndc_y = 1.0 - (2.0 * screen_pos.y / screen_height)
        
        // NDC to clip space (near and far plane)
        clip_near = Vector4(ndc_x, ndc_y, 0.0, 1.0)  // Near plane
        clip_far = Vector4(ndc_x, ndc_y, 1.0, 1.0)   // Far plane
        
        // Clip to world
        world_near = inv_mvp * clip_near
        world_far = inv_mvp * clip_far
        
        // Perspective divide
        world_near = world_near.xyz / world_near.w
        world_far = world_far.xyz / world_far.w
        
        // Create ray
        ray_origin = world_near
        ray_direction = normalize(world_far - world_near)
        
        return Ray(ray_origin, ray_direction)
    ```

=== "Rust (glam)"
    ```rust
    struct Ray {
        origin: Vec3,
        direction: Vec3,
    }
    
    fn screen_to_world_ray(
        screen_pos: Vec2,
        inv_mvp: Mat4,
        screen_size: Vec2,
    ) -> Ray {
        // Screen to NDC
        let ndc_x = (2.0 * screen_pos.x / screen_size.x) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y / screen_size.y);
        
        // NDC to clip space
        let clip_near = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        let clip_far = Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
        
        // Clip to world
        let world_near_h = inv_mvp * clip_near;
        let world_far_h = inv_mvp * clip_far;
        
        // Perspective divide
        let world_near = world_near_h.truncate() / world_near_h.w;
        let world_far = world_far_h.truncate() / world_far_h.w;
        
        Ray {
            origin: world_near,
            direction: (world_far - world_near).normalize(),
        }
    }
    ```

## Common Pitfalls

### 1. Mixing Coordinate Spaces

```rust
// Bad: Mixing world and local coordinates
let world_pos = character.world_position;
let local_offset = Vec3::new(1.0, 0.0, 0.0);
let bad_position = world_pos + local_offset;  // Wrong space!

// Good: Transform offset to world space first
let world_offset = character.world_rotation * local_offset;
let correct_position = world_pos + world_offset;
```

### 2. Forgetting Perspective Divide

After projection, divide by $w$ to get NDC:

```rust
let clip_pos = projection * view_pos;
let ndc = clip_pos.truncate() / clip_pos.w;  // Must divide by w!
```

### 3. Wrong Inverse for View Matrix

```rust
// Bad: General inverse is expensive
let view = camera_model.inverse();  // Slow!

// Good: Use look-at directly
let view = Mat4::look_at_rh(eye, target, up);  // Fast!
```

### 4. Not Normalizing Directions After Transform

```rust
let direction = Vec3::X;
let transformed = (rotation_matrix * direction.extend(0.0)).truncate();
// Bad: transformed might not be unit length anymore

let normalized = transformed.normalize();  // Good!
```

## Summary

| Space | Description | Transform From Previous |
|-------|-------------|------------------------|
| **Local** | Object's own coordinates | - |
| **World** | Scene's global coordinates | Model matrix (TRS) |
| **View** | Camera's perspective | View matrix (inverse of camera) |
| **Clip** | After projection | Projection matrix |
| **NDC** | Normalized [-1,1] or [0,1] | Perspective divide (/ w) |
| **Screen** | Pixel coordinates | Viewport transform |

**Key Transforms**:
- **MVP**: $\mathbf{P} \cdot \mathbf{V} \cdot \mathbf{M}$ (Local → Clip)
- **Points**: $w = 1$ (affected by translation)
- **Directions**: $w = 0$ (not affected by translation)
- **Normals**: Use $(\mathbf{M}^{-1})^T$ for non-uniform scale

## Next Steps

- **[Matrices](matrices.md)** - Deep dive into transformation matrices
- **[Vectors](vectors.md)** - Vector operations in different spaces
- **[Interpolation](interpolation.md)** - Blending between transforms

## Further Reading

- **Scratchapixel**: [Geometry - Transforming Points and Vectors](https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/geometry/transforming-points-and-vectors)
- **learnopengl.com**: [Coordinate Systems](https://learnopengl.com/Getting-started/Coordinate-Systems)
- **Real-Time Rendering**: Chapter 4 - Transforms
