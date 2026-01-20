# Interpolation

Interpolation is the process of computing intermediate values between two known values. In game engines, smooth interpolation is essential for animations, camera movements, transitions, and gameplay feel.

## What is Interpolation?

**Interpolation** finds a value between two endpoints based on a parameter $t \in [0, 1]$:

- At $t = 0$: Return the start value
- At $t = 1$: Return the end value
- At $0 < t < 1$: Return a blended value

```text
Start ●━━━━━━━━━━━━━━━━━━━━━━━━━● End
      t=0    t=0.25  t=0.5  t=0.75  t=1
             ●       ●       ●
```

## Linear Interpolation (LERP)

The simplest and most common interpolation method.

### Definition

$$\text{lerp}(a, b, t) = (1 - t)a + tb = a + t(b - a)$$

Where:
- $a$ = start value
- $b$ = end value
- $t \in [0, 1]$ = interpolation parameter

### Properties

- **Linear**: Constant rate of change
- **Symmetric**: $\text{lerp}(a, b, 1-t) = \text{lerp}(b, a, t)$
- **Exact endpoints**: $\text{lerp}(a, b, 0) = a$, $\text{lerp}(a, b, 1) = b$

### Scalar LERP

=== "Pseudocode"
    ```
    function lerp(a, b, t):
        return a + t * (b - a)
    
    // Alternative (more stable for large values)
    function lerp_stable(a, b, t):
        return (1 - t) * a + t * b
    ```

=== "Rust (glam)"
    ```rust
    // Built-in for floats
    let a = 0.0_f32;
    let b = 10.0_f32;
    let t = 0.5;
    let result = a.lerp(b, t);  // 5.0
    
    // Manual
    let manual = a + t * (b - a);
    ```

=== "C++ (glm)"
    ```cpp
    float a = 0.0f;
    float b = 10.0f;
    float t = 0.5f;
    float result = glm::mix(a, b, t);  // 5.0
    
    // Manual
    float manual = a + t * (b - a);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float a = 0.0f;
    float b = 10.0f;
    float t = 0.5f;
    float result = math.lerp(a, b, t);  // 5.0
    ```

### Vector LERP

Apply LERP component-wise to vectors:

$$\text{lerp}(\mathbf{v}_0, \mathbf{v}_1, t) = ((1-t)v_{0x} + tv_{1x},\ (1-t)v_{0y} + tv_{1y},\ (1-t)v_{0z} + tv_{1z})$$

=== "Pseudocode"
    ```
    function lerp_vector(v0, v1, t):
        return Vector(
            lerp(v0.x, v1.x, t),
            lerp(v0.y, v1.y, t),
            lerp(v0.z, v1.z, t)
        )
    ```

=== "Rust (glam)"
    ```rust
    let v0 = Vec3::new(0.0, 0.0, 0.0);
    let v1 = Vec3::new(10.0, 20.0, 30.0);
    let t = 0.5;
    let interpolated = v0.lerp(v1, t);  // Vec3(5.0, 10.0, 15.0)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v0(0.0f, 0.0f, 0.0f);
    glm::vec3 v1(10.0f, 20.0f, 30.0f);
    float t = 0.5f;
    glm::vec3 interpolated = glm::mix(v0, v1, t);  // vec3(5.0, 10.0, 15.0)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v0 = new float3(0.0f, 0.0f, 0.0f);
    float3 v1 = new float3(10.0f, 20.0f, 30.0f);
    float t = 0.5f;
    float3 interpolated = math.lerp(v0, v1, t);  // float3(5.0, 10.0, 15.0)
    ```

### Color LERP

Interpolate colors for smooth transitions:

=== "Rust (glam)"
    ```rust
    let red = Vec4::new(1.0, 0.0, 0.0, 1.0);    // RGBA
    let blue = Vec4::new(0.0, 0.0, 1.0, 1.0);
    let purple = red.lerp(blue, 0.5);            // Vec4(0.5, 0.0, 0.5, 1.0)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec4 red(1.0f, 0.0f, 0.0f, 1.0f);
    glm::vec4 blue(0.0f, 0.0f, 1.0f, 1.0f);
    glm::vec4 purple = glm::mix(red, blue, 0.5f);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float4 red = new float4(1.0f, 0.0f, 0.0f, 1.0f);
    float4 blue = new float4(0.0f, 0.0f, 1.0f, 1.0f);
    float4 purple = math.lerp(red, blue, 0.5f);
    ```

!!! warning "Color Space"
    LERP in RGB space can produce muddy colors. For better results, use HSV or LAB color spaces for interpolation, then convert back to RGB.

### Common Use Cases

1. **Smooth Movement**: Move object toward target
2. **Camera Follow**: Smoothly track player
3. **Fade Effects**: Alpha blending, dissolves
4. **Damping**: Spring-like behavior

## Spherical Linear Interpolation (SLERP)

SLERP interpolates along the **arc** of a sphere, maintaining constant angular velocity. Essential for rotating quaternions and directions.

### Definition

For unit quaternions or normalized vectors on a unit sphere:

$$\text{slerp}(\mathbf{q}_0, \mathbf{q}_1, t) = \frac{\sin((1-t)\theta)}{\sin\theta}\mathbf{q}_0 + \frac{\sin(t\theta)}{\sin\theta}\mathbf{q}_1$$

Where $\theta = \arccos(\mathbf{q}_0 \cdot \mathbf{q}_1)$ is the angle between them.

### Quaternion SLERP

See [Quaternions - SLERP](quaternions.md#spherical-linear-interpolation-slerp) for detailed explanation.

=== "Rust (glam)"
    ```rust
    let q0 = Quat::from_rotation_x(0_f32.to_radians());
    let q1 = Quat::from_rotation_x(90_f32.to_radians());
    let interpolated = q0.slerp(q1, 0.5);  // 45° rotation
    ```

=== "C++ (glm)"
    ```cpp
    glm::quat q0 = glm::angleAxis(0.0f, glm::vec3(1.0f, 0.0f, 0.0f));
    glm::quat q1 = glm::angleAxis(glm::radians(90.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    glm::quat interpolated = glm::slerp(q0, q1, 0.5f);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    quaternion q0 = quaternion.RotateX(0.0f);
    quaternion q1 = quaternion.RotateX(math.radians(90.0f));
    quaternion interpolated = math.slerp(q0, q1, 0.5f);
    ```

### Direction Vector SLERP

Interpolate directions while maintaining constant angle:

=== "Pseudocode"
    ```
    function slerp_direction(v0, v1, t):
        // Normalize inputs
        v0 = normalize(v0)
        v1 = normalize(v1)
        
        // Compute angle
        dot = clamp(dot_product(v0, v1), -1, 1)
        theta = acos(dot)
        
        // Handle nearly parallel vectors
        if theta < 0.001:
            return normalize(lerp(v0, v1, t))
        
        // SLERP formula
        sin_theta = sin(theta)
        w0 = sin((1 - t) * theta) / sin_theta
        w1 = sin(t * theta) / sin_theta
        
        return w0 * v0 + w1 * v1
    ```

=== "Rust (glam)"
    ```rust
    fn slerp_direction(v0: Vec3, v1: Vec3, t: f32) -> Vec3 {
        let v0 = v0.normalize();
        let v1 = v1.normalize();
        
        let dot = v0.dot(v1).clamp(-1.0, 1.0);
        let theta = dot.acos();
        
        if theta < 0.001 {
            return v0.lerp(v1, t).normalize();
        }
        
        let sin_theta = theta.sin();
        let w0 = ((1.0 - t) * theta).sin() / sin_theta;
        let w1 = (t * theta).sin() / sin_theta;
        
        w0 * v0 + w1 * v1
    }
    ```

## Normalized Linear Interpolation (NLERP)

**NLERP** is a faster approximation of SLERP: LERP followed by normalization.

$$\text{nlerp}(\mathbf{v}_0, \mathbf{v}_1, t) = \frac{\text{lerp}(\mathbf{v}_0, \mathbf{v}_1, t)}{\|\text{lerp}(\mathbf{v}_0, \mathbf{v}_1, t)\|}$$

**Pros**: Faster than SLERP (no trigonometry)  
**Cons**: Non-constant angular velocity (speeds up in the middle)

See [Quaternions - NLERP](quaternions.md#normalized-linear-interpolation-nlerp) for details.

## Easing Functions

**Easing functions** modify the interpolation parameter $t$ to create non-linear motion.

### Ease In (Slow Start)

Accelerate from rest:

$$t_{\text{ease-in}} = t^2$$

Or cubic: $t^3$

=== "Pseudocode"
    ```
    function ease_in_quad(t):
        return t * t
    
    function ease_in_cubic(t):
        return t * t * t
    ```

=== "Rust (glam)"
    ```rust
    fn ease_in_quad(t: f32) -> f32 {
        t * t
    }
    
    // Apply to LERP
    let t = 0.5;
    let eased_t = ease_in_quad(t);
    let result = start.lerp(end, eased_t);
    ```

### Ease Out (Slow End)

Decelerate to rest:

$$t_{\text{ease-out}} = 1 - (1 - t)^2$$

=== "Pseudocode"
    ```
    function ease_out_quad(t):
        return 1 - (1 - t) * (1 - t)
    ```

### Ease In-Out (Slow Start and End)

Smooth acceleration and deceleration:

$$t_{\text{ease-in-out}} = \begin{cases}
2t^2 & \text{if } t < 0.5 \\
1 - 2(1-t)^2 & \text{if } t \geq 0.5
\end{cases}$$

=== "Pseudocode"
    ```
    function ease_in_out_quad(t):
        if t < 0.5:
            return 2 * t * t
        else:
            return 1 - 2 * (1 - t) * (1 - t)
    ```

=== "Rust (glam)"
    ```rust
    fn ease_in_out_quad(t: f32) -> f32 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - 2.0 * (1.0 - t).powi(2)
        }
    }
    ```

### Smoothstep

A classic smooth interpolation function with zero derivatives at endpoints:

$$\text{smoothstep}(t) = 3t^2 - 2t^3$$

**Smoother variant** (zero first and second derivatives at endpoints):

$$\text{smootherstep}(t) = 6t^5 - 15t^4 + 10t^3$$

=== "Pseudocode"
    ```
    function smoothstep(t):
        return t * t * (3 - 2 * t)
    
    function smootherstep(t):
        return t * t * t * (t * (t * 6 - 15) + 10)
    ```

=== "Rust (glam)"
    ```rust
    fn smoothstep(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
    
    fn smootherstep(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }
    ```

=== "C++ (glm)"
    ```cpp
    float smoothstep(float t) {
        return glm::smoothstep(0.0f, 1.0f, t);
    }
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float Smoothstep(float t) {
        return math.smoothstep(0.0f, 1.0f, t);
    }
    ```

### Common Easing Curves

| Curve | Formula | Use Case |
|-------|---------|----------|
| **Linear** | $t$ | Constant speed, mechanical |
| **Quadratic** | $t^2$ | Subtle acceleration |
| **Cubic** | $t^3$ | More pronounced |
| **Sine** | $\sin(\frac{\pi}{2}t)$ | Smooth, natural |
| **Exponential** | $2^{10(t-1)}$ | Fast acceleration |
| **Elastic** | Oscillates | Bouncy, spring-like |
| **Bounce** | Multiple bounces | Impact effects |

!!! tip "Easing Library"
    Use established easing libraries (e.g., Robert Penner's easing functions) for production:
    - [easings.net](https://easings.net/) - Visual reference
    - Implementations in most game engines

## Damped Interpolation

**Damped interpolation** (also called "exponential decay" or "smooth damp") approaches the target asymptotically, never quite reaching it but getting arbitrarily close.

### Exponential Decay

$$\text{value}_{\text{new}} = \text{lerp}(\text{value}_{\text{current}}, \text{target}, 1 - e^{-\lambda \Delta t})$$

Where:
- $\lambda$ = damping factor (higher = faster convergence)
- $\Delta t$ = time step

**Simplified** (good approximation for small $\lambda \Delta t$):

$$\text{value}_{\text{new}} = \text{lerp}(\text{value}_{\text{current}}, \text{target}, \lambda \Delta t)$$

=== "Pseudocode"
    ```
    function exponential_decay(current, target, lambda, delta_time):
        // Exact
        t = 1 - exp(-lambda * delta_time)
        return lerp(current, target, t)
    
    function exponential_decay_approx(current, target, lambda, delta_time):
        // Approximation (faster)
        t = lambda * delta_time
        return lerp(current, target, t)
    ```

=== "Rust (glam)"
    ```rust
    fn exponential_decay(current: Vec3, target: Vec3, lambda: f32, dt: f32) -> Vec3 {
        let t = 1.0 - (-lambda * dt).exp();
        current.lerp(target, t)
    }
    
    // Approximate (faster, good for small dt)
    fn exponential_decay_approx(current: Vec3, target: Vec3, lambda: f32, dt: f32) -> Vec3 {
        let t = (lambda * dt).min(1.0);  // Clamp to prevent overshoot
        current.lerp(target, t)
    }
    ```

### Spring Damping

More sophisticated damping with velocity:

=== "Pseudocode"
    ```
    function smooth_damp(current, target, velocity, smooth_time, delta_time):
        omega = 2.0 / smooth_time
        x = omega * delta_time
        exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x)
        
        change = current - target
        max_change = max_speed * smooth_time
        change = clamp(change, -max_change, max_change)
        
        target_adjusted = current - change
        temp = (velocity + omega * change) * delta_time
        new_velocity = (velocity - omega * temp) * exp
        
        output = target_adjusted + (change + temp) * exp
        
        // Prevent overshooting
        if (target - current > 0) == (output > target):
            output = target
            new_velocity = 0
        
        return (output, new_velocity)
    ```

### Use Cases for Damping

1. **Camera Following**: Smooth camera tracking without jitter
2. **UI Animations**: Smooth panel sliding
3. **Physics**: Velocity damping, drag
4. **Value Smoothing**: Smooth framerate, input smoothing

## Cubic Bezier Curves

**Bezier curves** provide flexible interpolation with control points.

### Quadratic Bezier (3 control points)

$$\mathbf{B}(t) = (1-t)^2\mathbf{P}_0 + 2(1-t)t\mathbf{P}_1 + t^2\mathbf{P}_2$$

### Cubic Bezier (4 control points)

$$\mathbf{B}(t) = (1-t)^3\mathbf{P}_0 + 3(1-t)^2t\mathbf{P}_1 + 3(1-t)t^2\mathbf{P}_2 + t^3\mathbf{P}_3$$

=== "Pseudocode"
    ```
    function cubic_bezier(p0, p1, p2, p3, t):
        u = 1 - t
        tt = t * t
        uu = u * u
        uuu = uu * u
        ttt = tt * t
        
        p = uuu * p0
        p += 3 * uu * t * p1
        p += 3 * u * tt * p2
        p += ttt * p3
        
        return p
    ```

=== "Rust (glam)"
    ```rust
    fn cubic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;
        
        uuu * p0 + 3.0 * uu * t * p1 + 3.0 * u * tt * p2 + ttt * p3
    }
    ```

**Use cases**: 
- Animation curves (Unity's Animation Curve uses Bezier)
- Camera paths
- Procedural motion

## Catmull-Rom Splines

**Catmull-Rom splines** interpolate through all control points (unlike Bezier).

$$\mathbf{P}(t) = 0.5 \begin{bmatrix}
1 & t & t^2 & t^3
\end{bmatrix} \begin{bmatrix}
0 & 2 & 0 & 0 \\
-1 & 0 & 1 & 0 \\
2 & -5 & 4 & -1 \\
-1 & 3 & -3 & 1
\end{bmatrix} \begin{bmatrix}
\mathbf{P}_{i-1} \\ \mathbf{P}_i \\ \mathbf{P}_{i+1} \\ \mathbf{P}_{i+2}
\end{bmatrix}$$

=== "Pseudocode"
    ```
    function catmull_rom(p0, p1, p2, p3, t):
        tt = t * t
        ttt = tt * t
        
        return 0.5 * (
            2 * p1 +
            (-p0 + p2) * t +
            (2 * p0 - 5 * p1 + 4 * p2 - p3) * tt +
            (-p0 + 3 * p1 - 3 * p2 + p3) * ttt
        )
    ```

=== "Rust (glam)"
    ```rust
    fn catmull_rom(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
        let tt = t * t;
        let ttt = tt * t;
        
        0.5 * (
            2.0 * p1 +
            (-p0 + p2) * t +
            (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * tt +
            (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * ttt
        )
    }
    ```

**Use cases**: Camera paths, smooth motion through waypoints

## Practical Examples

### Example 1: Smooth Camera Follow

=== "Rust (glam)"
    ```rust
    struct SmoothCamera {
        position: Vec3,
        velocity: Vec3,
    }
    
    impl SmoothCamera {
        fn update(&mut self, target: Vec3, smooth_time: f32, dt: f32) {
            let lambda = 2.0 / smooth_time;
            let t = 1.0 - (-lambda * dt).exp();
            
            // Smooth position
            self.position = self.position.lerp(target, t);
            
            // Update velocity for next frame
            self.velocity = (target - self.position) * lambda;
        }
    }
    ```

### Example 2: Fade In/Out

=== "Rust (glam)"
    ```rust
    struct FadeEffect {
        alpha: f32,
        duration: f32,
        elapsed: f32,
    }
    
    impl FadeEffect {
        fn update(&mut self, dt: f32) -> f32 {
            self.elapsed += dt;
            let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
            
            // Smooth fade
            let eased_t = smoothstep(t);
            self.alpha = eased_t
        }
    }
    
    fn smoothstep(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }
    ```

### Example 3: Springy UI Element

=== "Rust (glam)"
    ```rust
    struct SpringyUI {
        position: f32,
        velocity: f32,
        target: f32,
    }
    
    impl SpringyUI {
        fn update(&mut self, dt: f32) {
            let stiffness = 100.0;
            let damping = 10.0;
            
            let force = stiffness * (self.target - self.position);
            let damping_force = -damping * self.velocity;
            
            let acceleration = force + damping_force;
            self.velocity += acceleration * dt;
            self.position += self.velocity * dt;
        }
    }
    ```

## Performance Considerations

### LERP vs. SLERP

| Method | Cost | Use When |
|--------|------|----------|
| **LERP** | ~5 ops | Vectors, colors, scalars |
| **NLERP** | ~10 ops | Approximate rotation |
| **SLERP** | ~30 ops (sin, cos, acos) | Precise rotation, cinematics |

**Rule of thumb**: Use SLERP for important rotations, NLERP for real-time gameplay.

### Caching Easing Values

For repeated easing curves, pre-compute a lookup table:

=== "Rust (glam)"
    ```rust
    struct EasingLUT {
        values: Vec<f32>,
    }
    
    impl EasingLUT {
        fn new(samples: usize) -> Self {
            let values = (0..samples)
                .map(|i| {
                    let t = i as f32 / (samples - 1) as f32;
                    smoothstep(t)
                })
                .collect();
            
            Self { values }
        }
        
        fn sample(&self, t: f32) -> f32 {
            let index = (t * (self.values.len() - 1) as f32).floor() as usize;
            let index = index.min(self.values.len() - 1);
            self.values[index]
        }
    }
    ```

## Common Pitfalls

### 1. Frame-Rate Dependent Interpolation

```rust
// Bad: speed varies with frame rate
let t = 0.1;  // Fixed value
position = position.lerp(target, t);

// Good: time-based
let t = 5.0 * delta_time;  // Speed in units/second
position = position.lerp(target, t.min(1.0));
```

### 2. Oscillation with High Damping

```rust
// Bad: can overshoot and oscillate
let t = damping * delta_time;  // No clamping
position = position.lerp(target, t);

// Good: clamp to prevent overshoot
let t = (damping * delta_time).min(1.0);
position = position.lerp(target, t);
```

### 3. Lerping Angles Incorrectly

```rust
// Bad: lerp raw angles (wraps around 0/360)
let angle1 = 350.0;  // degrees
let angle2 = 10.0;
let lerped = lerp(angle1, angle2, 0.5);  // 180° (wrong!)

// Good: use shortest path
fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let delta = ((b - a + 180.0) % 360.0) - 180.0;
    a + delta * t
}
```

## Summary

| Method | Formula | Speed | Quality | Use Case |
|--------|---------|-------|---------|----------|
| **LERP** | $(1-t)a + tb$ | Fast | Linear | General interpolation |
| **SLERP** | Spherical arc | Slow | Perfect | Rotation (quaternions) |
| **NLERP** | Normalize(LERP) | Medium | Good | Fast rotation |
| **Smoothstep** | $3t^2 - 2t^3$ | Fast | Smooth | Smooth transitions |
| **Bezier** | Polynomial | Medium | Flexible | Animation curves |
| **Catmull-Rom** | Polynomial | Medium | Smooth path | Camera paths |
| **Exponential** | $1 - e^{-\lambda t}$ | Fast | Asymptotic | Damping, following |

## Next Steps

- **[Vectors](vectors.md)** - Vector interpolation in detail
- **[Quaternions](quaternions.md)** - Rotation interpolation (SLERP/NLERP)
- **Animation systems** - Apply interpolation to skeletal animation

## Further Reading

- **Easings.net**: [Easing Functions Cheat Sheet](https://easings.net/)
- **Red Blob Games**: [Bezier Curves and Splines](https://www.redblobgames.com/articles/curved-paths/)
- **Freya Holmér**: [The Continuity of Splines](https://www.youtube.com/watch?v=jvPPXbo87ds)
- **Game Programming Patterns**: [Smoothing Input](https://gameprogrammingpatterns.com/update-method.html)
