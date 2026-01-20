# Matrices

Matrices are the foundation of 3D transformations in game engines. They allow us to represent translation, rotation, scaling, projection, and combinations thereof in a unified framework.

## What is a Matrix?

A **matrix** is a rectangular array of numbers arranged in rows and columns. In 3D graphics, we primarily work with:

- **3×3 matrices**: Rotation and scale (no translation)
- **4×4 matrices**: Full 3D transformations (rotation, scale, translation)

### Mathematical Notation

A 4×4 matrix $\mathbf{M}$ is written as:

$$\mathbf{M} = \begin{bmatrix}
m_{00} & m_{01} & m_{02} & m_{03} \\
m_{10} & m_{11} & m_{12} & m_{13} \\
m_{20} & m_{21} & m_{22} & m_{23} \\
m_{30} & m_{31} & m_{32} & m_{33}
\end{bmatrix}$$

Convention: $m_{row,col}$ (row index first, column index second)

### Why 4×4 for 3D?

We use **homogeneous coordinates** to represent 3D points as 4D vectors $(x, y, z, w)$ where $w = 1$ for points. This allows us to represent translation as matrix multiplication (impossible with 3×3 matrices).

## Identity Matrix

The **identity matrix** $\mathbf{I}$ is the multiplicative identity:

$$\mathbf{I} = \begin{bmatrix}
1 & 0 & 0 & 0 \\
0 & 1 & 0 & 0 \\
0 & 0 & 1 & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}$$

**Property**: $\mathbf{I}\mathbf{M} = \mathbf{M}\mathbf{I} = \mathbf{M}$ (no transformation)

=== "Pseudocode"
    ```
    function identity_matrix():
        return [
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [0, 0, 0, 1]
        ]
    ```

=== "Rust (glam)"
    ```rust
    use glam::Mat4;
    
    let identity = Mat4::IDENTITY;
    ```

=== "C++ (glm)"
    ```cpp
    #include <glm/glm.hpp>
    
    glm::mat4 identity = glm::mat4(1.0f);  // Identity matrix
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    using Unity.Mathematics;
    
    float4x4 identity = float4x4.identity;
    ```

## Matrix-Vector Multiplication

Multiplying a matrix $\mathbf{M}$ by a vector $\mathbf{v}$ transforms the vector:

$$\mathbf{M}\mathbf{v} = \begin{bmatrix}
m_{00} & m_{01} & m_{02} & m_{03} \\
m_{10} & m_{11} & m_{12} & m_{13} \\
m_{20} & m_{21} & m_{22} & m_{23} \\
m_{30} & m_{31} & m_{32} & m_{33}
\end{bmatrix} \begin{bmatrix} x \\ y \\ z \\ w \end{bmatrix} = \begin{bmatrix}
m_{00}x + m_{01}y + m_{02}z + m_{03}w \\
m_{10}x + m_{11}y + m_{12}z + m_{13}w \\
m_{20}x + m_{21}y + m_{22}z + m_{23}w \\
m_{30}x + m_{31}y + m_{32}z + m_{33}w
\end{bmatrix}$$

### Points vs. Vectors

- **Points** (positions): Use $w = 1$ → affected by translation
- **Vectors** (directions): Use $w = 0$ → not affected by translation

=== "Pseudocode"
    ```
    function transform_point(matrix, point):
        // Treat point as (x, y, z, 1)
        x = m[0][0]*p.x + m[0][1]*p.y + m[0][2]*p.z + m[0][3]
        y = m[1][0]*p.x + m[1][1]*p.y + m[1][2]*p.z + m[1][3]
        z = m[2][0]*p.x + m[2][1]*p.y + m[2][2]*p.z + m[2][3]
        w = m[3][0]*p.x + m[3][1]*p.y + m[3][2]*p.z + m[3][3]
        return Vector(x/w, y/w, z/w)  // Perspective divide
    
    function transform_vector(matrix, vector):
        // Treat vector as (x, y, z, 0)
        x = m[0][0]*v.x + m[0][1]*v.y + m[0][2]*v.z
        y = m[1][0]*v.x + m[1][1]*v.y + m[1][2]*v.z
        z = m[2][0]*v.x + m[2][1]*v.y + m[2][2]*v.z
        return Vector(x, y, z)
    ```

=== "Rust (glam)"
    ```rust
    let matrix = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
    
    // Transform point (affected by translation)
    let point = Vec3::new(1.0, 2.0, 3.0);
    let transformed_point = matrix.transform_point3(point);  // (6, 2, 3)
    
    // Transform vector (not affected by translation)
    let vector = Vec3::new(1.0, 0.0, 0.0);
    let transformed_vector = matrix.transform_vector3(vector);  // (1, 0, 0)
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 matrix = glm::translate(glm::mat4(1.0f), glm::vec3(5.0f, 0.0f, 0.0f));
    
    // Transform point (w = 1)
    glm::vec3 point(1.0f, 2.0f, 3.0f);
    glm::vec3 transformedPoint = glm::vec3(matrix * glm::vec4(point, 1.0f));  // (6, 2, 3)
    
    // Transform vector (w = 0)
    glm::vec3 vector(1.0f, 0.0f, 0.0f);
    glm::vec3 transformedVector = glm::vec3(matrix * glm::vec4(vector, 0.0f));  // (1, 0, 0)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 matrix = float4x4.Translate(new float3(5.0f, 0.0f, 0.0f));
    
    // Transform point
    float3 point = new float3(1.0f, 2.0f, 3.0f);
    float3 transformedPoint = math.transform(matrix, point);  // (6, 2, 3)
    
    // Transform vector (direction)
    float3 vector = new float3(1.0f, 0.0f, 0.0f);
    float3 transformedVector = math.rotate(matrix, vector);  // (1, 0, 0)
    ```

## Matrix-Matrix Multiplication

Multiplying two matrices $\mathbf{A}$ and $\mathbf{B}$ combines their transformations:

$$(\mathbf{A}\mathbf{B})_{ij} = \sum_{k=0}^{3} a_{ik} b_{kj}$$

**Important**: Matrix multiplication is **not commutative**: $\mathbf{AB} \neq \mathbf{BA}$

**Order matters**: $\mathbf{TRS}$ (Translate-Rotate-Scale) applies scale first, then rotation, then translation.

=== "Pseudocode"
    ```
    function multiply_matrices(A, B):
        result = zero_matrix(4, 4)
        for i in 0..4:
            for j in 0..4:
                for k in 0..4:
                    result[i][j] += A[i][k] * B[k][j]
        return result
    ```

=== "Rust (glam)"
    ```rust
    let scale = Mat4::from_scale(Vec3::splat(2.0));
    let rotation = Mat4::from_rotation_y(90_f32.to_radians());
    let translation = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
    
    // Combine: TRS (right-to-left application)
    let combined = translation * rotation * scale;
    
    // Applying to a point: first scales, then rotates, then translates
    let point = Vec3::new(1.0, 0.0, 0.0);
    let transformed = combined.transform_point3(point);
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 scale = glm::scale(glm::mat4(1.0f), glm::vec3(2.0f));
    glm::mat4 rotation = glm::rotate(glm::mat4(1.0f), glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::mat4 translation = glm::translate(glm::mat4(1.0f), glm::vec3(5.0f, 0.0f, 0.0f));
    
    // Combine: TRS
    glm::mat4 combined = translation * rotation * scale;
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 scale = float4x4.Scale(2.0f);
    float4x4 rotation = float4x4.RotateY(math.radians(90.0f));
    float4x4 translation = float4x4.Translate(new float3(5.0f, 0.0f, 0.0f));
    
    // Combine: TRS
    float4x4 combined = math.mul(translation, math.mul(rotation, scale));
    ```

!!! warning "Matrix Multiplication Order"
    - **Column-major** (glam, glm): Read right-to-left: $\mathbf{TRS}$ applies S, then R, then T
    - **Row-major** (DirectXMath): Read left-to-right: $\mathbf{SRT}$ applies S, then R, then T
    
    Most math libraries use column-major. Always check your library's convention!

## Basic Transformation Matrices

### Translation Matrix

Translates (moves) points by $(t_x, t_y, t_z)$:

$$\mathbf{T} = \begin{bmatrix}
1 & 0 & 0 & t_x \\
0 & 1 & 0 & t_y \\
0 & 0 & 1 & t_z \\
0 & 0 & 0 & 1
\end{bmatrix}$$

=== "Pseudocode"
    ```
    function translation_matrix(tx, ty, tz):
        return [
            [1, 0, 0, tx],
            [0, 1, 0, ty],
            [0, 0, 1, tz],
            [0, 0, 0, 1]
        ]
    ```

=== "Rust (glam)"
    ```rust
    let translation = Mat4::from_translation(Vec3::new(5.0, 10.0, 15.0));
    
    let point = Vec3::new(1.0, 2.0, 3.0);
    let moved = translation.transform_point3(point);  // (6, 12, 18)
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 translation = glm::translate(glm::mat4(1.0f), glm::vec3(5.0f, 10.0f, 15.0f));
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 translation = float4x4.Translate(new float3(5.0f, 10.0f, 15.0f));
    ```

### Scale Matrix

Scales points by factors $(s_x, s_y, s_z)$:

$$\mathbf{S} = \begin{bmatrix}
s_x & 0 & 0 & 0 \\
0 & s_y & 0 & 0 \\
0 & 0 & s_z & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}$$

**Uniform scale**: $s_x = s_y = s_z$ (preserves proportions)  
**Non-uniform scale**: Different values (stretches/squashes)

=== "Pseudocode"
    ```
    function scale_matrix(sx, sy, sz):
        return [
            [sx,  0,  0, 0],
            [ 0, sy,  0, 0],
            [ 0,  0, sz, 0],
            [ 0,  0,  0, 1]
        ]
    ```

=== "Rust (glam)"
    ```rust
    let uniform_scale = Mat4::from_scale(Vec3::splat(2.0));  // 2x in all directions
    let non_uniform = Mat4::from_scale(Vec3::new(2.0, 1.0, 0.5));  // Stretch X, squash Z
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 uniformScale = glm::scale(glm::mat4(1.0f), glm::vec3(2.0f));
    glm::mat4 nonUniform = glm::scale(glm::mat4(1.0f), glm::vec3(2.0f, 1.0f, 0.5f));
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 uniformScale = float4x4.Scale(2.0f);
    float4x4 nonUniform = float4x4.Scale(new float3(2.0f, 1.0f, 0.5f));
    ```

### Rotation Matrices

Rotation around each cardinal axis by angle $\theta$ (in radians):

**Rotation around X-axis**:
$$\mathbf{R}_x(\theta) = \begin{bmatrix}
1 & 0 & 0 & 0 \\
0 & \cos\theta & -\sin\theta & 0 \\
0 & \sin\theta & \cos\theta & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}$$

**Rotation around Y-axis**:
$$\mathbf{R}_y(\theta) = \begin{bmatrix}
\cos\theta & 0 & \sin\theta & 0 \\
0 & 1 & 0 & 0 \\
-\sin\theta & 0 & \cos\theta & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}$$

**Rotation around Z-axis**:
$$\mathbf{R}_z(\theta) = \begin{bmatrix}
\cos\theta & -\sin\theta & 0 & 0 \\
\sin\theta & \cos\theta & 0 & 0 \\
0 & 0 & 1 & 0 \\
0 & 0 & 0 & 1
\end{bmatrix}$$

=== "Pseudocode"
    ```
    function rotation_x(angle_radians):
        c = cos(angle_radians)
        s = sin(angle_radians)
        return [
            [1,  0,  0, 0],
            [0,  c, -s, 0],
            [0,  s,  c, 0],
            [0,  0,  0, 1]
        ]
    
    function rotation_y(angle_radians):
        c = cos(angle_radians)
        s = sin(angle_radians)
        return [
            [ c, 0,  s, 0],
            [ 0, 1,  0, 0],
            [-s, 0,  c, 0],
            [ 0, 0,  0, 1]
        ]
    
    function rotation_z(angle_radians):
        c = cos(angle_radians)
        s = sin(angle_radians)
        return [
            [ c, -s, 0, 0],
            [ s,  c, 0, 0],
            [ 0,  0, 1, 0],
            [ 0,  0, 0, 1]
        ]
    ```

=== "Rust (glam)"
    ```rust
    let angle = 90_f32.to_radians();
    
    let rot_x = Mat4::from_rotation_x(angle);
    let rot_y = Mat4::from_rotation_y(angle);
    let rot_z = Mat4::from_rotation_z(angle);
    
    // Arbitrary axis rotation
    let axis = Vec3::new(1.0, 1.0, 0.0).normalize();
    let rot_axis = Mat4::from_axis_angle(axis, angle);
    ```

=== "C++ (glm)"
    ```cpp
    float angle = glm::radians(90.0f);
    
    glm::mat4 rotX = glm::rotate(glm::mat4(1.0f), angle, glm::vec3(1.0f, 0.0f, 0.0f));
    glm::mat4 rotY = glm::rotate(glm::mat4(1.0f), angle, glm::vec3(0.0f, 1.0f, 0.0f));
    glm::mat4 rotZ = glm::rotate(glm::mat4(1.0f), angle, glm::vec3(0.0f, 0.0f, 1.0f));
    
    // Arbitrary axis
    glm::vec3 axis = glm::normalize(glm::vec3(1.0f, 1.0f, 0.0f));
    glm::mat4 rotAxis = glm::rotate(glm::mat4(1.0f), angle, axis);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float angle = math.radians(90.0f);
    
    float4x4 rotX = float4x4.RotateX(angle);
    float4x4 rotY = float4x4.RotateY(angle);
    float4x4 rotZ = float4x4.RotateZ(angle);
    
    // Arbitrary axis
    float3 axis = math.normalize(new float3(1.0f, 1.0f, 0.0f));
    float4x4 rotAxis = float4x4.AxisAngle(axis, angle);
    ```

## Transform Composition

Combine transformations by multiplying matrices. **Order is crucial**!

### Standard Transform Order: TRS

$$\mathbf{M} = \mathbf{T} \cdot \mathbf{R} \cdot \mathbf{S}$$

Applied right-to-left:
1. Scale the object
2. Rotate the scaled object
3. Translate to final position

=== "Pseudocode"
    ```
    function create_transform(translation, rotation, scale):
        T = translation_matrix(translation.x, translation.y, translation.z)
        R = rotation_matrix(rotation)  // From quaternion or Euler angles
        S = scale_matrix(scale.x, scale.y, scale.z)
        return T * R * S  // Multiply right-to-left
    ```

=== "Rust (glam)"
    ```rust
    let translation = Vec3::new(5.0, 0.0, 0.0);
    let rotation = Quat::from_rotation_y(45_f32.to_radians());
    let scale = Vec3::splat(2.0);
    
    // Create TRS matrix
    let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
    
    // Or manually
    let t = Mat4::from_translation(translation);
    let r = Mat4::from_quat(rotation);
    let s = Mat4::from_scale(scale);
    let transform_manual = t * r * s;
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 translation(5.0f, 0.0f, 0.0f);
    glm::quat rotation = glm::angleAxis(glm::radians(45.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::vec3 scale(2.0f);
    
    glm::mat4 t = glm::translate(glm::mat4(1.0f), translation);
    glm::mat4 r = glm::mat4_cast(rotation);
    glm::mat4 s = glm::scale(glm::mat4(1.0f), scale);
    
    glm::mat4 transform = t * r * s;
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 translation = new float3(5.0f, 0.0f, 0.0f);
    quaternion rotation = quaternion.RotateY(math.radians(45.0f));
    float3 scale = new float3(2.0f);
    
    float4x4 transform = float4x4.TRS(translation, rotation, scale);
    ```

### Why TRS Order?

Consider a character model:
1. **Scale** (S): Resize model to appropriate size
2. **Rotate** (R): Orient character direction
3. **Translate** (T): Move to world position

Changing order produces different results:

```text
TRS: Scale → Rotate → Translate (standard)
RST: Rotate → Scale → Translate (stretches rotation)
STR: Scale → Translate → Rotate (rotates around origin, not object center)
```

## Matrix Transpose

The **transpose** of a matrix $\mathbf{M}$ swaps rows and columns:

$$\mathbf{M}^T_{ij} = \mathbf{M}_{ji}$$

$$\begin{bmatrix}
a & b & c \\
d & e & f \\
g & h & i
\end{bmatrix}^T = \begin{bmatrix}
a & d & g \\
b & e & h \\
c & f & i
\end{bmatrix}$$

**Use case**: Transform normals when non-uniform scaling is present (use inverse-transpose).

=== "Rust (glam)"
    ```rust
    let matrix = Mat4::from_rotation_y(45_f32.to_radians());
    let transposed = matrix.transpose();
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 matrix = glm::rotate(glm::mat4(1.0f), glm::radians(45.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::mat4 transposed = glm::transpose(matrix);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 matrix = float4x4.RotateY(math.radians(45.0f));
    float4x4 transposed = math.transpose(matrix);
    ```

## Matrix Inverse

The **inverse** of a matrix $\mathbf{M}^{-1}$ undoes the transformation:

$$\mathbf{M}\mathbf{M}^{-1} = \mathbf{M}^{-1}\mathbf{M} = \mathbf{I}$$

**Computing the inverse** is expensive (for general matrices). However, for common transforms:

- **Translation**: $\mathbf{T}^{-1}(t_x, t_y, t_z) = \mathbf{T}(-t_x, -t_y, -t_z)$
- **Rotation**: $\mathbf{R}^{-1} = \mathbf{R}^T$ (for rotation matrices, inverse = transpose)
- **Scale**: $\mathbf{S}^{-1}(s_x, s_y, s_z) = \mathbf{S}(1/s_x, 1/s_y, 1/s_z)$

**Use cases**:
- Transform from world space to local space
- Compute view matrix (inverse of camera transform)
- Undo transformations

=== "Pseudocode"
    ```
    function inverse_transform(T, R, S):
        S_inv = scale_matrix(1/S.x, 1/S.y, 1/S.z)
        R_inv = transpose(R)  // For rotation matrices
        T_inv = translation_matrix(-T.x, -T.y, -T.z)
        return S_inv * R_inv * T_inv  // Reverse order!
    ```

=== "Rust (glam)"
    ```rust
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::splat(2.0),
        Quat::from_rotation_y(45_f32.to_radians()),
        Vec3::new(5.0, 0.0, 0.0),
    );
    
    let inverse = transform.inverse();
    
    // Verify: transform * inverse ≈ identity
    let identity = transform * inverse;
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 transform = /* ... */;
    glm::mat4 inverse = glm::inverse(transform);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 transform = /* ... */;
    float4x4 inverse = math.inverse(transform);
    ```

!!! warning "Matrix Inversion Performance"
    General 4×4 matrix inversion is expensive (~50-100 operations). For transform matrices (TRS), use specialized fast inverse or decompose and invert components separately.

## View and Projection Matrices

### View Matrix (Camera Transform)

The **view matrix** transforms world space to camera space. It's the inverse of the camera's world transform.

**Look-At Matrix**: Create a view matrix from camera position, target, and up vector:

$$\text{view} = \text{lookAt}(\text{eye}, \text{target}, \text{up})$$

=== "Pseudocode"
    ```
    function look_at(eye, target, up):
        // Build camera coordinate system
        forward = normalize(target - eye)  // Camera looks toward target
        right = normalize(cross(forward, up))
        up_actual = cross(right, forward)
        
        // Rotation part (inverse = transpose for orthonormal)
        rotation = [
            [right.x,      up_actual.x,      -forward.x,      0],
            [right.y,      up_actual.y,      -forward.y,      0],
            [right.z,      up_actual.z,      -forward.z,      0],
            [0,            0,                0,               1]
        ]
        
        // Translation part (move world opposite to camera position)
        translation = translation_matrix(-eye.x, -eye.y, -eye.z)
        
        return rotation * translation
    ```

=== "Rust (glam)"
    ```rust
    let eye = Vec3::new(0.0, 5.0, 10.0);
    let target = Vec3::ZERO;
    let up = Vec3::Y;
    
    let view = Mat4::look_at_rh(eye, target, up);  // Right-handed
    // Or left-handed
    let view_lh = Mat4::look_at_lh(eye, target, up);
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 eye(0.0f, 5.0f, 10.0f);
    glm::vec3 target(0.0f, 0.0f, 0.0f);
    glm::vec3 up(0.0f, 1.0f, 0.0f);
    
    glm::mat4 view = glm::lookAt(eye, target, up);  // Right-handed
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 eye = new float3(0.0f, 5.0f, 10.0f);
    float3 target = new float3(0.0f, 0.0f, 0.0f);
    float3 up = new float3(0.0f, 1.0f, 0.0f);
    
    float4x4 view = float4x4.LookAt(eye, target, up);
    ```

### Projection Matrix

The **projection matrix** transforms camera space to clip space (for rasterization).

**Perspective Projection**: Objects farther away appear smaller (realistic 3D)

$$\text{projection} = \text{perspective}(\text{fov}, \text{aspect}, \text{near}, \text{far})$$

- **fov**: Field of view angle (vertical, in radians)
- **aspect**: Width / height ratio
- **near**: Near clipping plane distance
- **far**: Far clipping plane distance

=== "Pseudocode"
    ```
    function perspective(fov_y, aspect, near, far):
        f = 1.0 / tan(fov_y / 2.0)
        return [
            [f/aspect,  0,  0,                            0                        ],
            [0,         f,  0,                            0                        ],
            [0,         0,  (far+near)/(near-far),        (2*far*near)/(near-far)  ],
            [0,         0, -1,                            0                        ]
        ]
    ```

=== "Rust (glam)"
    ```rust
    let fov = 60_f32.to_radians();
    let aspect = 16.0 / 9.0;
    let near = 0.1;
    let far = 100.0;
    
    let projection = Mat4::perspective_rh(fov, aspect, near, far);  // Right-handed
    // Infinite far plane (for better precision)
    let projection_inf = Mat4::perspective_infinite_rh(fov, aspect, near);
    ```

=== "C++ (glm)"
    ```cpp
    float fov = glm::radians(60.0f);
    float aspect = 16.0f / 9.0f;
    float near = 0.1f;
    float far = 100.0f;
    
    glm::mat4 projection = glm::perspective(fov, aspect, near, far);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float fov = math.radians(60.0f);
    float aspect = 16.0f / 9.0f;
    float near = 0.1f;
    float far = 100.0f;
    
    float4x4 projection = float4x4.PerspectiveFov(fov, aspect, near, far);
    ```

**Orthographic Projection**: No perspective, parallel lines stay parallel (used for UI, shadow maps, 2D games)

=== "Rust (glam)"
    ```rust
    let left = -10.0;
    let right = 10.0;
    let bottom = -10.0;
    let top = 10.0;
    let near = 0.1;
    let far = 100.0;
    
    let ortho = Mat4::orthographic_rh(left, right, bottom, top, near, far);
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 ortho = glm::ortho(-10.0f, 10.0f, -10.0f, 10.0f, 0.1f, 100.0f);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4x4 ortho = float4x4.Ortho(10.0f, 10.0f, 0.1f, 100.0f);
    ```

## Matrix Decomposition

Extract translation, rotation, and scale from a transformation matrix:

=== "Rust (glam)"
    ```rust
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 3.0, 1.0),
        Quat::from_rotation_y(45_f32.to_radians()),
        Vec3::new(5.0, 10.0, 15.0),
    );
    
    // Decompose
    let (scale, rotation, translation) = transform.to_scale_rotation_translation();
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 scale;
    glm::quat rotation;
    glm::vec3 translation;
    glm::vec3 skew;
    glm::vec4 perspective;
    
    glm::decompose(transform, scale, rotation, translation, skew, perspective);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    // Unity doesn't have built-in decompose, extract manually
    float3 translation = transform.c3.xyz;
    float3 scaleX = new float3(transform.c0.x, transform.c0.y, transform.c0.z);
    float scaleXLen = math.length(scaleX);
    // ... (extract rotation from normalized basis vectors)
    ```

## Common Pitfalls

### 1. Wrong Multiplication Order

```rust
// Bad: applies in wrong order
let wrong = scale * rotation * translation;  // SRT instead of TRS

// Good: standard TRS order
let correct = translation * rotation * scale;
```

### 2. Transforming Normals Incorrectly

When non-uniform scaling is present, normals must be transformed by the **inverse-transpose** of the model matrix:

```rust
let normal_matrix = model_matrix.inverse().transpose();
let transformed_normal = normal_matrix.transform_vector3(normal).normalize();
```

### 3. Mixing Column-Major and Row-Major

Different libraries use different conventions. Always check documentation!

### 4. Forgetting Perspective Divide

After projection, divide by $w$ to get normalized device coordinates (NDC).

## Performance Considerations

### Matrix Multiplication Cost

- **4×4 matrix multiply**: ~64 multiplications, ~48 additions
- **Matrix-vector multiply**: ~16 multiplications, ~12 additions

**Optimization tips**:
- Pre-compute combined matrices (model-view-projection)
- Use specialized functions for known matrix types (rotation, translation)
- Leverage SIMD instructions (modern math libraries do this automatically)

### Cache Matrices When Possible

```rust
// Bad: recompute every frame for static objects
let mvp = projection * view * model;  // Every frame

// Good: cache for static objects
struct CachedTransform {
    mvp: Mat4,
    dirty: bool,
}
// Only recompute when dirty
```

## Summary

| Matrix Type | Purpose | Construction |
|-------------|---------|--------------|
| **Identity** | No transformation | `Mat4::IDENTITY` |
| **Translation** | Move points | `Mat4::from_translation(v)` |
| **Rotation** | Rotate around axis | `Mat4::from_rotation_*(angle)` |
| **Scale** | Resize | `Mat4::from_scale(v)` |
| **TRS** | Combined transform | `translation * rotation * scale` |
| **View** | World → Camera space | `Mat4::look_at_rh(eye, target, up)` |
| **Projection** | Camera → Clip space | `Mat4::perspective_rh(fov, aspect, near, far)` |
| **Inverse** | Undo transformation | `matrix.inverse()` |
| **Transpose** | Swap rows/columns | `matrix.transpose()` |

## Next Steps

- **[Quaternions](quaternions.md)** - Better rotation representation
- **[Coordinate Spaces](coordinate-spaces.md)** - Understanding space transformations
- **[Interpolation](interpolation.md)** - Smooth transitions

## Further Reading

- **In-Depth**: *3D Math Primer for Graphics and Game Development*, Chapters 4-6
- **GLM Manual**: [Matrix Transformations](https://glm.g-truc.net/0.9.9/api/a00247.html)
- **learnopengl.com**: [Transformations](https://learnopengl.com/Getting-started/Transformations)
