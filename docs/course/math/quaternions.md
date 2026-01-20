# Quaternions

Quaternions provide a robust, gimbal-lock-free way to represent rotations in 3D space. While initially less intuitive than Euler angles, they are essential for smooth animations, stable physics, and efficient rotation interpolation.

## What is a Quaternion?

A **quaternion** is a 4D complex number used to represent 3D rotations:

$$\mathbf{q} = w + xi + yj + zk$$

Or as a 4-tuple: $\mathbf{q} = (w, x, y, z)$ or $(x, y, z, w)$ (convention varies)

Where:
- $w$ is the **scalar** (real) part
- $(x, y, z)$ is the **vector** (imaginary) part
- $i^2 = j^2 = k^2 = ijk = -1$ (quaternion units)

!!! info "Component Order Convention"
    - **glam (Rust)**: `(x, y, z, w)` - vector part first
    - **glm (C++)**: `(w, x, y, z)` - scalar first
    - **Unity.Mathematics (C#)**: `(x, y, z, w)` - vector part first
    
    Always check your library's convention!

## Why Quaternions?

### Euler Angles: The Problem

**Euler angles** represent rotation as three angles around X, Y, Z axes (e.g., pitch, yaw, roll).

**Problems**:
1. **Gimbal Lock**: Lose a degree of freedom when two axes align
2. **Discontinuities**: Multiple representations for same rotation (0° = 360°)
3. **Difficult Interpolation**: Linear interpolation produces non-smooth rotation
4. **Order-Dependent**: XYZ vs. ZYX produces different results

**Example of Gimbal Lock**:
```text
Pitch = 90° → X and Z axes become aligned
Now you can't distinguish between yaw and roll!
```

### Quaternions: The Solution

**Advantages**:
1. **No Gimbal Lock**: Always four degrees of freedom
2. **Smooth Interpolation**: Spherical linear interpolation (SLERP) is well-defined
3. **Efficient**: Faster than matrix composition for multiple rotations
4. **Compact**: 4 numbers vs. 9 for a 3×3 rotation matrix
5. **Numerically Stable**: Less susceptible to floating-point errors

**Disadvantage**: Less intuitive than Euler angles (requires practice to visualize)

## Unit Quaternions for Rotation

A **unit quaternion** (length = 1) represents a rotation:

$$\|\mathbf{q}\| = \sqrt{w^2 + x^2 + y^2 + z^2} = 1$$

### Axis-Angle Representation

A rotation by angle $\theta$ around unit axis $\mathbf{v} = (v_x, v_y, v_z)$ is represented as:

$$\mathbf{q} = \left(\cos\frac{\theta}{2},\ v_x\sin\frac{\theta}{2},\ v_y\sin\frac{\theta}{2},\ v_z\sin\frac{\theta}{2}\right)$$

Or using vector notation: $\mathbf{q} = (w, x, y, z)$ where:
- $w = \cos(\theta/2)$
- $(x, y, z) = \mathbf{v} \sin(\theta/2)$

**Key insight**: Half-angle formulation eliminates ambiguity ($\mathbf{q}$ and $-\mathbf{q}$ represent the same rotation).

=== "Pseudocode"
    ```
    function quat_from_axis_angle(axis, angle):
        // axis must be normalized
        half_angle = angle / 2
        s = sin(half_angle)
        c = cos(half_angle)
        return Quaternion(
            w = c,
            x = axis.x * s,
            y = axis.y * s,
            z = axis.z * s
        )
    ```

=== "Rust (glam)"
    ```rust
    use glam::Quat;
    
    let axis = Vec3::Y;  // Rotate around Y-axis
    let angle = 90_f32.to_radians();
    let q = Quat::from_axis_angle(axis, angle);
    
    // Common axes
    let q_x = Quat::from_rotation_x(angle);
    let q_y = Quat::from_rotation_y(angle);
    let q_z = Quat::from_rotation_z(angle);
    ```

=== "C++ (glm)"
    ```cpp
    #include <glm/gtc/quaternion.hpp>
    
    glm::vec3 axis(0.0f, 1.0f, 0.0f);  // Y-axis
    float angle = glm::radians(90.0f);
    glm::quat q = glm::angleAxis(angle, axis);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    using Unity.Mathematics;
    
    float3 axis = new float3(0.0f, 1.0f, 0.0f);
    float angle = math.radians(90.0f);
    quaternion q = quaternion.AxisAngle(axis, angle);
    
    // Common axes
    quaternion qY = quaternion.RotateY(angle);
    ```

## Identity Quaternion

The **identity quaternion** represents no rotation:

$$\mathbf{q}_{\text{identity}} = (1, 0, 0, 0)$$

(Scalar part = 1, vector part = zero)

=== "Rust (glam)"
    ```rust
    let identity = Quat::IDENTITY;
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat identity = glm::quat(1.0f, 0.0f, 0.0f, 0.0f);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion identity = quaternion.identity;
    ```

## Quaternion Operations

### Quaternion Multiplication (Composition)

Combining rotations: $\mathbf{q}_1$ followed by $\mathbf{q}_2$ is $\mathbf{q}_2 \mathbf{q}_1$

$$\mathbf{q}_1 \mathbf{q}_2 = (w_1 w_2 - \mathbf{v}_1 \cdot \mathbf{v}_2,\ w_1\mathbf{v}_2 + w_2\mathbf{v}_1 + \mathbf{v}_1 \times \mathbf{v}_2)$$

Where $\mathbf{v}_1 = (x_1, y_1, z_1)$ and $\mathbf{v}_2 = (x_2, y_2, z_2)$

**Explicit form**:
```math
w = w_1 w_2 - x_1 x_2 - y_1 y_2 - z_1 z_2
x = w_1 x_2 + x_1 w_2 + y_1 z_2 - z_1 y_2
y = w_1 y_2 - x_1 z_2 + y_1 w_2 + z_1 x_2
z = w_1 z_2 + x_1 y_2 - y_1 x_2 + z_1 w_2
```

**Important**: Quaternion multiplication is **not commutative**: $\mathbf{q}_1 \mathbf{q}_2 \neq \mathbf{q}_2 \mathbf{q}_1$

=== "Pseudocode"
    ```
    function multiply_quaternions(q1, q2):
        w = q1.w*q2.w - q1.x*q2.x - q1.y*q2.y - q1.z*q2.z
        x = q1.w*q2.x + q1.x*q2.w + q1.y*q2.z - q1.z*q2.y
        y = q1.w*q2.y - q1.x*q2.z + q1.y*q2.w + q1.z*q2.x
        z = q1.w*q2.z + q1.x*q2.y - q1.y*q2.x + q1.z*q2.w
        return Quaternion(w, x, y, z)
    ```

=== "Rust (glam)"
    ```rust
    let q1 = Quat::from_rotation_x(45_f32.to_radians());
    let q2 = Quat::from_rotation_y(90_f32.to_radians());
    
    // Combine rotations: first q1, then q2
    let combined = q2 * q1;
    
    // Order matters!
    let different = q1 * q2;  // Different result
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q1 = glm::angleAxis(glm::radians(45.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    glm::quat q2 = glm::angleAxis(glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    
    glm::quat combined = q2 * q1;
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q1 = quaternion.RotateX(math.radians(45.0f));
    quaternion q2 = quaternion.RotateY(math.radians(90.0f));
    
    quaternion combined = math.mul(q2, q1);
    ```

### Rotating Vectors

To rotate a vector $\mathbf{v}$ by quaternion $\mathbf{q}$:

$$\mathbf{v}' = \mathbf{q} \mathbf{v} \mathbf{q}^*$$

Where $\mathbf{q}^*$ is the conjugate (explained below).

Libraries provide convenient functions for this:

=== "Rust (glam)"
    ```rust
    let q = Quat::from_rotation_y(90_f32.to_radians());
    let v = Vec3::X;  // (1, 0, 0)
    let rotated = q * v;  // Rotates 90° around Y
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q = glm::angleAxis(glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::vec3 v(1.0f, 0.0f, 0.0f);
    glm::vec3 rotated = q * v;
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q = quaternion.RotateY(math.radians(90.0f));
    float3 v = new float3(1.0f, 0.0f, 0.0f);
    float3 rotated = math.rotate(q, v);
    ```

### Quaternion Conjugate

The **conjugate** negates the vector part:

$$\mathbf{q}^* = (w, -x, -y, -z)$$

For unit quaternions, the conjugate is the **inverse** (reverse rotation):

$$\mathbf{q}^{-1} = \mathbf{q}^* \quad \text{(when } \|\mathbf{q}\| = 1\text{)}$$

=== "Rust (glam)"
    ```rust
    let q = Quat::from_rotation_y(90_f32.to_radians());
    let q_conjugate = q.conjugate();
    
    // For unit quaternions, conjugate = inverse
    let q_inverse = q.inverse();
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q = glm::angleAxis(glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::quat qConjugate = glm::conjugate(q);
    glm::quat qInverse = glm::inverse(q);  // Same for unit quaternions
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q = quaternion.RotateY(math.radians(90.0f));
    quaternion qConjugate = math.conjugate(q);
    quaternion qInverse = math.inverse(q);
    ```

### Normalization

Ensure quaternion is unit length (required for valid rotations):

$$\hat{\mathbf{q}} = \frac{\mathbf{q}}{\|\mathbf{q}\|} = \frac{(w, x, y, z)}{\sqrt{w^2 + x^2 + y^2 + z^2}}$$

=== "Rust (glam)"
    ```rust
    let q = Quat::from_xyzw(1.0, 2.0, 3.0, 4.0);  // Not unit length
    let normalized = q.normalize();
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q(4.0f, 1.0f, 2.0f, 3.0f);  // Not unit length
    glm::quat normalized = glm::normalize(q);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q = new quaternion(1.0f, 2.0f, 3.0f, 4.0f);
    quaternion normalized = math.normalize(q);
    ```

## Conversion Between Representations

### Quaternion ↔ Euler Angles

**Euler to Quaternion**:

=== "Rust (glam)"
    ```rust
    let euler = Vec3::new(
        45_f32.to_radians(),  // Pitch (X)
        90_f32.to_radians(),  // Yaw (Y)
        0_f32.to_radians(),   // Roll (Z)
    );
    let q = Quat::from_euler(glam::EulerRot::XYZ, euler.x, euler.y, euler.z);
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 euler(glm::radians(45.0f), glm::radians(90.0f), 0.0f);
    glm::quat q = glm::quat(euler);  // Assumes XYZ order
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 euler = math.radians(new float3(45.0f, 90.0f, 0.0f));
    quaternion q = quaternion.Euler(euler.x, euler.y, euler.z);
    ```

**Quaternion to Euler**:

=== "Rust (glam)"
    ```rust
    let q = Quat::from_rotation_y(90_f32.to_radians());
    let (x, y, z) = q.to_euler(glam::EulerRot::XYZ);
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q = glm::angleAxis(glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::vec3 euler = glm::eulerAngles(q);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    // Unity doesn't provide built-in quat-to-euler
    // Use Unity.Mathematics.Euler() or manual extraction
    ```

!!! warning "Euler Angle Ambiguity"
    Multiple Euler angle representations can produce the same rotation. Conversion may not return the original Euler angles.

### Quaternion ↔ Matrix

**Quaternion to Matrix**:

=== "Rust (glam)"
    ```rust
    let q = Quat::from_rotation_y(90_f32.to_radians());
    let matrix = Mat4::from_quat(q);
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q = glm::angleAxis(glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::mat4 matrix = glm::mat4_cast(q);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q = quaternion.RotateY(math.radians(90.0f));
    float3x3 matrix3x3 = new float3x3(q);
    float4x4 matrix4x4 = new float4x4(q, float3.zero);
    ```

**Matrix to Quaternion**:

=== "Rust (glam)"
    ```rust
    let matrix = Mat4::from_rotation_y(90_f32.to_radians());
    let q = Quat::from_mat4(&matrix);
    ```

=== "C++ (glm)"
    ```cpp
    glm::mat4 matrix = glm::rotate(glm::mat4(1.0f), glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::quat q = glm::quat_cast(matrix);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3x3 matrix = float3x3.RotateY(math.radians(90.0f));
    quaternion q = new quaternion(matrix);
    ```

## Interpolation: SLERP and NLERP

### Spherical Linear Interpolation (SLERP)

**SLERP** provides smooth interpolation between two quaternions along the shortest arc on the 4D unit sphere:

$$\text{slerp}(\mathbf{q}_0, \mathbf{q}_1, t) = \frac{\sin((1-t)\theta)}{\sin\theta}\mathbf{q}_0 + \frac{\sin(t\theta)}{\sin\theta}\mathbf{q}_1$$

Where $\theta = \arccos(\mathbf{q}_0 \cdot \mathbf{q}_1)$

**Properties**:
- Constant angular velocity
- Shortest path on sphere
- More expensive (trigonometry)

=== "Pseudocode"
    ```
    function slerp(q0, q1, t):
        // Compute angle between quaternions
        dot = dot_product(q0, q1)
        
        // If dot < 0, slerp will take the long path
        // Negate q1 to take shorter path
        if dot < 0:
            q1 = -q1
            dot = -dot
        
        // Close to identical? Use lerp to avoid division by zero
        if dot > 0.9995:
            return normalize(lerp(q0, q1, t))
        
        // SLERP formula
        theta = acos(clamp(dot, -1, 1))
        sin_theta = sin(theta)
        
        w0 = sin((1 - t) * theta) / sin_theta
        w1 = sin(t * theta) / sin_theta
        
        return w0 * q0 + w1 * q1
    ```

=== "Rust (glam)"
    ```rust
    let q0 = Quat::from_rotation_x(0_f32.to_radians());
    let q1 = Quat::from_rotation_x(90_f32.to_radians());
    
    let interpolated = q0.slerp(q1, 0.5);  // Halfway between
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q0 = glm::angleAxis(glm::radians(0.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    glm::quat q1 = glm::angleAxis(glm::radians(90.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    
    glm::quat interpolated = glm::slerp(q0, q1, 0.5f);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q0 = quaternion.RotateX(0.0f);
    quaternion q1 = quaternion.RotateX(math.radians(90.0f));
    
    quaternion interpolated = math.slerp(q0, q1, 0.5f);
    ```

### Normalized Linear Interpolation (NLERP)

**NLERP** is a faster approximation: linear interpolation followed by normalization:

$$\text{nlerp}(\mathbf{q}_0, \mathbf{q}_1, t) = \text{normalize}((1-t)\mathbf{q}_0 + t\mathbf{q}_1)$$

**Properties**:
- Faster (no trigonometry)
- Non-constant angular velocity (speed varies)
- Visually acceptable for small rotations
- Often used in games for performance

=== "Pseudocode"
    ```
    function nlerp(q0, q1, t):
        // Handle quaternion double-cover
        if dot_product(q0, q1) < 0:
            q1 = -q1
        
        return normalize((1 - t) * q0 + t * q1)
    ```

=== "Rust (glam)"
    ```rust
    let q0 = Quat::from_rotation_x(0_f32.to_radians());
    let q1 = Quat::from_rotation_x(90_f32.to_radians());
    
    // Manual NLERP
    let t = 0.5;
    let interpolated = q0.lerp(q1, t).normalize();
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q0 = glm::angleAxis(glm::radians(0.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    glm::quat q1 = glm::angleAxis(glm::radians(90.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    
    float t = 0.5f;
    glm::quat interpolated = glm::normalize(glm::mix(q0, q1, t));
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q0 = quaternion.RotateX(0.0f);
    quaternion q1 = quaternion.RotateX(math.radians(90.0f));
    
    float t = 0.5f;
    quaternion interpolated = math.normalize(math.lerp(q0, q1, t));
    ```

### SLERP vs. NLERP

| Feature | SLERP | NLERP |
|---------|-------|-------|
| **Speed** | Slower (sin, cos, acos) | Faster (just add + normalize) |
| **Angular Velocity** | Constant | Variable (faster in middle) |
| **Quality** | Perfect arc | Approximation |
| **Use Case** | Cinematics, important animations | Gameplay, real-time animations |

**Rule of thumb**: Use NLERP unless you need perfectly constant speed.

## Practical Applications

### Character Rotation Toward Target

Smoothly rotate a character to face a target:

=== "Pseudocode"
    ```
    function rotate_toward(current_rotation, target_direction, turn_speed, delta_time):
        // Current forward direction
        current_forward = current_rotation * Vector(0, 0, 1)
        
        // Desired rotation
        target_rotation = look_rotation(target_direction, Vector(0, 1, 0))
        
        // Interpolate
        t = turn_speed * delta_time
        new_rotation = slerp(current_rotation, target_rotation, t)
        
        return new_rotation
    ```

=== "Rust (glam)"
    ```rust
    fn rotate_toward(
        current: Quat,
        target_dir: Vec3,
        turn_speed: f32,
        delta_time: f32,
    ) -> Quat {
        // Target rotation to face direction
        let target = Quat::from_rotation_arc(Vec3::Z, target_dir.normalize());
        
        // Interpolate
        let t = (turn_speed * delta_time).min(1.0);
        current.slerp(target, t)
    }
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat rotateToward(
        glm::quat current,
        glm::vec3 targetDir,
        float turnSpeed,
        float deltaTime
    ) {
        glm::vec3 forward(0.0f, 0.0f, 1.0f);
        glm::quat target = glm::rotation(forward, glm::normalize(targetDir));
        
        float t = glm::min(turnSpeed * deltaTime, 1.0f);
        return glm::slerp(current, target, t);
    }
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion RotateToward(
        quaternion current,
        float3 targetDir,
        float turnSpeed,
        float deltaTime
    ) {
        quaternion target = quaternion.LookRotation(targetDir, new float3(0, 1, 0));
        float t = math.min(turnSpeed * deltaTime, 1.0f);
        return math.slerp(current, target, t);
    }
    ```

### Look-At Rotation

Create a quaternion that makes an object look at a target:

=== "Rust (glam)"
    ```rust
    fn look_at_rotation(from: Vec3, to: Vec3, up: Vec3) -> Quat {
        let forward = (to - from).normalize();
        let right = up.cross(forward).normalize();
        let up_actual = forward.cross(right);
        
        Quat::from_mat3(&Mat3::from_cols(right, up_actual, forward))
    }
    
    // Or use built-in
    let forward = (to - from).normalize();
    let rotation = Quat::from_rotation_arc(Vec3::Z, forward);
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat lookAtRotation(glm::vec3 from, glm::vec3 to, glm::vec3 up) {
        glm::mat4 lookMat = glm::lookAt(from, to, up);
        return glm::quat_cast(lookMat);
    }
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion LookAtRotation(float3 from, float3 to, float3 up) {
        float3 forward = math.normalize(to - from);
        return quaternion.LookRotation(forward, up);
    }
    ```

### Camera Orbit Controller

Orbit camera around a target using quaternions:

=== "Rust (glam)"
    ```rust
    struct OrbitCamera {
        target: Vec3,
        distance: f32,
        rotation: Quat,
    }
    
    impl OrbitCamera {
        fn update(&mut self, yaw_delta: f32, pitch_delta: f32) {
            let yaw = Quat::from_rotation_y(yaw_delta);
            let pitch = Quat::from_rotation_x(pitch_delta);
            self.rotation = yaw * self.rotation * pitch;
        }
        
        fn position(&self) -> Vec3 {
            let offset = self.rotation * Vec3::new(0.0, 0.0, self.distance);
            self.target + offset
        }
    }
    ```

## Common Pitfalls

### 1. Quaternion Double Cover

$\mathbf{q}$ and $-\mathbf{q}$ represent the same rotation. Always take the shorter path when interpolating:

```rust
// Bad: might take the long way
let interpolated = q0.slerp(q1, t);

// Good: check dot product first
let q1_adjusted = if q0.dot(q1) < 0.0 { -q1 } else { q1 };
let interpolated = q0.slerp(q1_adjusted, t);
```

Most libraries handle this automatically in their `slerp` implementation.

### 2. Not Normalizing After Multiple Operations

Floating-point errors accumulate. Normalize quaternions periodically:

```rust
// After many multiplications
q = q.normalize();
```

### 3. Gimbal Lock When Converting to Euler

Converting quaternion → Euler → quaternion can introduce gimbal lock. Avoid round-tripping.

### 4. Wrong Multiplication Order

Like matrices, quaternion multiplication order matters:

```rust
let combined = q2 * q1;  // Apply q1, then q2
let wrong = q1 * q2;     // Apply q2, then q1 (different!)
```

## Summary

| Operation | Formula / Function | Use Case |
|-----------|-------------------|----------|
| **From Axis-Angle** | `Quat::from_axis_angle(axis, angle)` | Rotate around arbitrary axis |
| **Multiply** | `q2 * q1` | Combine rotations |
| **Rotate Vector** | `q * v` | Apply rotation to vector |
| **Inverse** | `q.conjugate()` (for unit quats) | Reverse rotation |
| **SLERP** | `q0.slerp(q1, t)` | Smooth interpolation (constant speed) |
| **NLERP** | `q0.lerp(q1, t).normalize()` | Fast interpolation (variable speed) |
| **To Matrix** | `Mat4::from_quat(q)` | Use in rendering pipeline |
| **From Euler** | `Quat::from_euler(...)` | Artist-friendly input |

## Why Use Quaternions in Games?

1. **Skeletal Animation**: Blend bone rotations smoothly with SLERP/NLERP
2. **Camera Systems**: Smooth camera rotation without gimbal lock
3. **Physics**: Represent rigid body orientation
4. **Networking**: Compact representation (4 floats vs. 9 for matrix)
5. **Procedural Animation**: Inverse kinematics, look-at constraints

## Next Steps

- **[Interpolation](interpolation.md)** - Detailed coverage of LERP, SLERP, and other blending techniques
- **[Coordinate Spaces](coordinate-spaces.md)** - Apply rotations in different reference frames
- **[Matrices](matrices.md)** - Convert quaternions to matrices for rendering

## Further Reading

- **Visualizer**: [Quaternion Visualizer](https://eater.net/quaternions) - Interactive 3D explanation
- **In-Depth**: *3D Math Primer for Graphics and Game Development*, Chapter 8
- **Tutorial**: [Understanding Quaternions](https://www.3dgep.com/understanding-quaternions/)
