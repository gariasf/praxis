# Vectors

Vectors are the foundation of 3D mathematics and game engine development. They represent positions, directions, velocities, forces, and countless other quantities in games.

## What is a Vector?

A **vector** is an ordered list of numbers. In 3D graphics, we primarily work with:

- **2D Vectors**: $(x, y)$ - Screen positions, UV coordinates
- **3D Vectors**: $(x, y, z)$ - Positions, directions, colors
- **4D Vectors**: $(x, y, z, w)$ - Homogeneous coordinates (for transformations)

### Geometric Interpretation

A 3D vector can be thought of in two ways:

1. **Position**: A point in 3D space relative to the origin
2. **Direction & Magnitude**: An arrow pointing from one location to another

```text
     y
     ↑
     |    v = (2, 3, 1)
     |   ●───→ 
     |  /
     | /
     |/________→ x
    /
   /z
```

### Mathematical Notation

We write vectors using:

- **Tuple notation**: $\mathbf{v} = (x, y, z)$
- **Column vector**: $\mathbf{v} = \begin{bmatrix} x \\ y \\ z \end{bmatrix}$
- **Component notation**: $v_x, v_y, v_z$

Throughout this guide, bold lowercase letters (e.g., $\mathbf{v}$, $\mathbf{w}$) denote vectors.

## Basic Vector Operations

### Vector Addition

Add corresponding components:

$$\mathbf{v} + \mathbf{w} = (v_x + w_x,\ v_y + w_y,\ v_z + w_z)$$

**Geometric meaning**: Place the tail of $\mathbf{w}$ at the head of $\mathbf{v}$

**Example**: $(1, 2, 3) + (4, 5, 6) = (5, 7, 9)$

**Uses**: 
- Moving an object: `new_position = old_position + velocity * delta_time`
- Combining forces in physics

=== "Pseudocode"
    ```
    function add(v, w):
        return Vector(v.x + w.x, v.y + w.y, v.z + w.z)
    ```

=== "Rust (glam)"
    ```rust
    use glam::Vec3;
    
    let v = Vec3::new(1.0, 2.0, 3.0);
    let w = Vec3::new(4.0, 5.0, 6.0);
    let sum = v + w;  // Vec3(5.0, 7.0, 9.0)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v(1.0f, 2.0f, 3.0f);
    glm::vec3 w(4.0f, 5.0f, 6.0f);
    glm::vec3 sum = v + w;  // vec3(5.0, 7.0, 9.0)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v = new float3(1.0f, 2.0f, 3.0f);
    float3 w = new float3(4.0f, 5.0f, 6.0f);
    float3 sum = v + w;  // float3(5.0, 7.0, 9.0)
    ```

### Vector Subtraction

Subtract corresponding components:

$$\mathbf{v} - \mathbf{w} = (v_x - w_x,\ v_y - w_y,\ v_z - w_z)$$

**Geometric meaning**: Vector from $\mathbf{w}$ to $\mathbf{v}$

**Example**: $(5, 7, 9) - (1, 2, 3) = (4, 5, 6)$

**Uses**:
- Direction from point A to point B: `direction = target_position - current_position`
- Relative velocity: `relative_vel = object_vel - player_vel`

=== "Pseudocode"
    ```
    function subtract(v, w):
        return Vector(v.x - w.x, v.y - w.y, v.z - w.z)
    ```

=== "Rust (glam)"
    ```rust
    let target = Vec3::new(10.0, 5.0, 0.0);
    let current = Vec3::new(2.0, 1.0, 0.0);
    let direction = target - current;  // Vec3(8.0, 4.0, 0.0)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 target(10.0f, 5.0f, 0.0f);
    glm::vec3 current(2.0f, 1.0f, 0.0f);
    glm::vec3 direction = target - current;  // vec3(8.0, 4.0, 0.0)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 target = new float3(10.0f, 5.0f, 0.0f);
    float3 current = new float3(2.0f, 1.0f, 0.0f);
    float3 direction = target - current;  // float3(8.0, 4.0, 0.0)
    ```

### Scalar Multiplication

Multiply each component by a scalar $k$:

$$k\mathbf{v} = (kv_x,\ kv_y,\ kv_z)$$

**Geometric meaning**: Scale the vector's length by $|k|$, reverse direction if $k < 0$

**Example**: $2 \times (1, 2, 3) = (2, 4, 6)$

**Uses**:
- Scaling velocity: `velocity = direction * speed`
- Applying forces: `acceleration = force / mass`

=== "Pseudocode"
    ```
    function scale(v, k):
        return Vector(v.x * k, v.y * k, v.z * k)
    ```

=== "Rust (glam)"
    ```rust
    let v = Vec3::new(1.0, 2.0, 3.0);
    let scaled = v * 2.0;  // Vec3(2.0, 4.0, 6.0)
    let reversed = v * -1.0;  // Vec3(-1.0, -2.0, -3.0)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v(1.0f, 2.0f, 3.0f);
    glm::vec3 scaled = v * 2.0f;  // vec3(2.0, 4.0, 6.0)
    glm::vec3 reversed = v * -1.0f;  // vec3(-1.0, -2.0, -3.0)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v = new float3(1.0f, 2.0f, 3.0f);
    float3 scaled = v * 2.0f;  // float3(2.0, 4.0, 6.0)
    float3 reversed = v * -1.0f;  // float3(-1.0, -2.0, -3.0)
    ```

## Vector Length (Magnitude)

The **length** or **magnitude** of a vector is its distance from the origin:

$$\|\mathbf{v}\| = \sqrt{v_x^2 + v_y^2 + v_z^2}$$

This comes from the Pythagorean theorem extended to 3D.

**Example**: $\|(3, 4, 0)\| = \sqrt{9 + 16 + 0} = 5$

**Uses**:
- Distance between points: `distance = length(target - current)`
- Speed: `speed = length(velocity)`
- Range checks: `if length(enemy_pos - player_pos) < attack_range`

=== "Pseudocode"
    ```
    function length(v):
        return sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
    
    function length_squared(v):
        // Faster, avoids sqrt
        return v.x * v.x + v.y * v.y + v.z * v.z
    ```

=== "Rust (glam)"
    ```rust
    let v = Vec3::new(3.0, 4.0, 0.0);
    let len = v.length();  // 5.0
    let len_sq = v.length_squared();  // 25.0 (faster, no sqrt)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v(3.0f, 4.0f, 0.0f);
    float len = glm::length(v);  // 5.0
    float len_sq = glm::dot(v, v);  // 25.0 (faster)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v = new float3(3.0f, 4.0f, 0.0f);
    float len = math.length(v);  // 5.0
    float lenSq = math.lengthsq(v);  // 25.0 (faster)
    ```

!!! tip "Performance: Length Squared"
    When comparing distances, use `length_squared()` to avoid the expensive `sqrt()`:
    ```
    // Bad: sqrt is slow
    if length(v) < 10.0:
        ...
    
    // Good: compare squared values
    if length_squared(v) < 100.0:
        ...
    ```

## Normalization

A **unit vector** (or **normalized vector**) has length 1. We denote unit vectors with a hat: $\hat{\mathbf{v}}$

To normalize a vector, divide by its length:

$$\hat{\mathbf{v}} = \frac{\mathbf{v}}{\|\mathbf{v}\|}$$

**Example**: $(3, 4, 0) \to \left(\frac{3}{5}, \frac{4}{5}, 0\right) = (0.6, 0.8, 0)$

**Uses**:
- Pure direction (discarding magnitude)
- Consistent velocity: `velocity = normalize(direction) * speed`
- Surface normals in lighting

=== "Pseudocode"
    ```
    function normalize(v):
        len = length(v)
        if len == 0:
            return Vector(0, 0, 0)  // Handle zero vector
        return Vector(v.x / len, v.y / len, v.z / len)
    ```

=== "Rust (glam)"
    ```rust
    let v = Vec3::new(3.0, 4.0, 0.0);
    let normalized = v.normalize();  // Vec3(0.6, 0.8, 0.0)
    
    // Safe version (returns zero vector if length is zero)
    let safe_norm = v.normalize_or_zero();
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v(3.0f, 4.0f, 0.0f);
    glm::vec3 normalized = glm::normalize(v);  // vec3(0.6, 0.8, 0.0)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v = new float3(3.0f, 4.0f, 0.0f);
    float3 normalized = math.normalize(v);  // float3(0.6, 0.8, 0.0)
    
    // Safe version
    float3 safeNorm = math.normalizesafe(v);
    ```

!!! warning "Zero Vector"
    Cannot normalize a zero vector (length = 0). Always check or use safe variants.

## Dot Product

The **dot product** (or **scalar product**) of two vectors produces a scalar:

$$\mathbf{v} \cdot \mathbf{w} = v_x w_x + v_y w_y + v_z w_z$$

**Alternate form** (using angle between vectors):

$$\mathbf{v} \cdot \mathbf{w} = \|\mathbf{v}\| \|\mathbf{w}\| \cos\theta$$

where $\theta$ is the angle between the vectors.

### Properties

- **Commutative**: $\mathbf{v} \cdot \mathbf{w} = \mathbf{w} \cdot \mathbf{v}$
- **If perpendicular** ($\theta = 90°$): $\mathbf{v} \cdot \mathbf{w} = 0$
- **If parallel** ($\theta = 0°$): $\mathbf{v} \cdot \mathbf{w} = \|\mathbf{v}\| \|\mathbf{w}\|$
- **If opposite** ($\theta = 180°$): $\mathbf{v} \cdot \mathbf{w} = -\|\mathbf{v}\| \|\mathbf{w}\|$

### Uses

1. **Angle between vectors**:
   $$\theta = \arccos\left(\frac{\mathbf{v} \cdot \mathbf{w}}{\|\mathbf{v}\| \|\mathbf{w}\|}\right)$$

2. **Check if vectors point in same/opposite directions**:
   - Dot product > 0: Same general direction
   - Dot product < 0: Opposite directions
   - Dot product = 0: Perpendicular

3. **Projection**: Component of $\mathbf{v}$ along $\mathbf{w}$

4. **Lighting**: Lambertian diffuse lighting uses $\mathbf{N} \cdot \mathbf{L}$ (normal · light direction)

=== "Pseudocode"
    ```
    function dot(v, w):
        return v.x * w.x + v.y * w.y + v.z * w.z
    
    function angle_between(v, w):
        // Assumes v and w are normalized
        cos_theta = dot(v, w)
        return acos(clamp(cos_theta, -1, 1))
    ```

=== "Rust (glam)"
    ```rust
    let v = Vec3::new(1.0, 0.0, 0.0);
    let w = Vec3::new(0.0, 1.0, 0.0);
    let dot = v.dot(w);  // 0.0 (perpendicular)
    
    // Angle between vectors (in radians)
    let angle = v.angle_between(w);  // π/2 (90 degrees)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v(1.0f, 0.0f, 0.0f);
    glm::vec3 w(0.0f, 1.0f, 0.0f);
    float dot = glm::dot(v, w);  // 0.0 (perpendicular)
    
    // Angle between normalized vectors
    float angle = glm::acos(glm::clamp(glm::dot(v, w), -1.0f, 1.0f));
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v = new float3(1.0f, 0.0f, 0.0f);
    float3 w = new float3(0.0f, 1.0f, 0.0f);
    float dot = math.dot(v, w);  // 0.0 (perpendicular)
    
    // Angle between normalized vectors
    float angle = math.acos(math.clamp(dot, -1.0f, 1.0f));
    ```

### Example: Field of View Check

Check if an enemy is within the player's field of view:

=== "Pseudocode"
    ```
    function is_in_fov(player_pos, player_forward, enemy_pos, fov_angle):
        to_enemy = normalize(enemy_pos - player_pos)
        dot_product = dot(player_forward, to_enemy)
        angle = acos(dot_product)
        return angle < fov_angle / 2
    ```

=== "Rust (glam)"
    ```rust
    fn is_in_fov(
        player_pos: Vec3,
        player_forward: Vec3,
        enemy_pos: Vec3,
        fov_angle: f32,
    ) -> bool {
        let to_enemy = (enemy_pos - player_pos).normalize();
        let dot = player_forward.dot(to_enemy);
        let angle = dot.acos();
        angle < fov_angle / 2.0
    }
    ```

=== "C++ (glm)"
    ```cpp
    bool isInFOV(
        glm::vec3 playerPos,
        glm::vec3 playerForward,
        glm::vec3 enemyPos,
        float fovAngle
    ) {
        glm::vec3 toEnemy = glm::normalize(enemyPos - playerPos);
        float dot = glm::dot(playerForward, toEnemy);
        float angle = glm::acos(dot);
        return angle < fovAngle / 2.0f;
    }
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    bool IsInFOV(
        float3 playerPos,
        float3 playerForward,
        float3 enemyPos,
        float fovAngle
    ) {
        float3 toEnemy = math.normalize(enemyPos - playerPos);
        float dot = math.dot(playerForward, toEnemy);
        float angle = math.acos(dot);
        return angle < fovAngle / 2.0f;
    }
    ```

## Cross Product

The **cross product** of two 3D vectors produces another vector perpendicular to both:

$$\mathbf{v} \times \mathbf{w} = \begin{pmatrix} v_y w_z - v_z w_y \\ v_z w_x - v_x w_z \\ v_x w_y - v_y w_x \end{pmatrix}$$

### Properties

- **Not commutative**: $\mathbf{v} \times \mathbf{w} = -(\mathbf{w} \times \mathbf{v})$
- **Result is perpendicular**: $(\mathbf{v} \times \mathbf{w}) \cdot \mathbf{v} = 0$ and $(\mathbf{v} \times \mathbf{w}) \cdot \mathbf{w} = 0$
- **Magnitude**: $\|\mathbf{v} \times \mathbf{w}\| = \|\mathbf{v}\| \|\mathbf{w}\| \sin\theta$ (area of parallelogram)
- **If parallel**: $\mathbf{v} \times \mathbf{w} = \mathbf{0}$

### Uses

1. **Find perpendicular vector**: Construct coordinate systems, surface normals
2. **Determine handedness**: Check if vectors follow right-hand rule
3. **Calculate surface normals**: From two edges of a triangle
4. **Torque and angular momentum**: Physics calculations

=== "Pseudocode"
    ```
    function cross(v, w):
        return Vector(
            v.y * w.z - v.z * w.y,
            v.z * w.x - v.x * w.z,
            v.x * w.y - v.y * w.x
        )
    ```

=== "Rust (glam)"
    ```rust
    let v = Vec3::new(1.0, 0.0, 0.0);  // X-axis
    let w = Vec3::new(0.0, 1.0, 0.0);  // Y-axis
    let cross = v.cross(w);  // Vec3(0.0, 0.0, 1.0) - Z-axis
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 v(1.0f, 0.0f, 0.0f);  // X-axis
    glm::vec3 w(0.0f, 1.0f, 0.0f);  // Y-axis
    glm::vec3 cross = glm::cross(v, w);  // vec3(0.0, 0.0, 1.0) - Z-axis
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 v = new float3(1.0f, 0.0f, 0.0f);  // X-axis
    float3 w = new float3(0.0f, 1.0f, 0.0f);  // Y-axis
    float3 cross = math.cross(v, w);  // float3(0.0, 0.0, 1.0) - Z-axis
    ```

### Example: Calculate Triangle Normal

Given triangle vertices A, B, C, compute the surface normal:

=== "Pseudocode"
    ```
    function calculate_triangle_normal(a, b, c):
        edge1 = b - a
        edge2 = c - a
        normal = cross(edge1, edge2)
        return normalize(normal)
    ```

=== "Rust (glam)"
    ```rust
    fn calculate_triangle_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
        let edge1 = b - a;
        let edge2 = c - a;
        edge1.cross(edge2).normalize()
    }
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 calculateTriangleNormal(
        glm::vec3 a, glm::vec3 b, glm::vec3 c
    ) {
        glm::vec3 edge1 = b - a;
        glm::vec3 edge2 = c - a;
        return glm::normalize(glm::cross(edge1, edge2));
    }
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 CalculateTriangleNormal(float3 a, float3 b, float3 c) {
        float3 edge1 = b - a;
        float3 edge2 = c - a;
        return math.normalize(math.cross(edge1, edge2));
    }
    ```

## Distance Between Points

The distance between two points $\mathbf{p}$ and $\mathbf{q}$ is the length of the vector between them:

$$\text{distance}(\mathbf{p}, \mathbf{q}) = \|\mathbf{q} - \mathbf{p}\|$$

=== "Pseudocode"
    ```
    function distance(p, q):
        return length(q - p)
    
    function distance_squared(p, q):
        diff = q - p
        return length_squared(diff)
    ```

=== "Rust (glam)"
    ```rust
    let p = Vec3::new(0.0, 0.0, 0.0);
    let q = Vec3::new(3.0, 4.0, 0.0);
    let dist = p.distance(q);  // 5.0
    let dist_sq = p.distance_squared(q);  // 25.0 (faster)
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 p(0.0f, 0.0f, 0.0f);
    glm::vec3 q(3.0f, 4.0f, 0.0f);
    float dist = glm::distance(p, q);  // 5.0
    float distSq = glm::dot(q - p, q - p);  // 25.0 (faster)
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 p = new float3(0.0f, 0.0f, 0.0f);
    float3 q = new float3(3.0f, 4.0f, 0.0f);
    float dist = math.distance(p, q);  // 5.0
    float distSq = math.lengthsq(q - p);  // 25.0 (faster)
    ```

## Practical Applications

### Movement Toward Target

Move an object toward a target at a fixed speed:

=== "Pseudocode"
    ```
    function move_toward(current_pos, target_pos, speed, delta_time):
        direction = normalize(target_pos - current_pos)
        movement = direction * speed * delta_time
        new_pos = current_pos + movement
        
        // Don't overshoot
        if distance(new_pos, target_pos) < distance(current_pos, target_pos):
            return new_pos
        else:
            return target_pos
    ```

=== "Rust (glam)"
    ```rust
    fn move_toward(
        current_pos: Vec3,
        target_pos: Vec3,
        speed: f32,
        delta_time: f32,
    ) -> Vec3 {
        let direction = (target_pos - current_pos).normalize();
        let movement = direction * speed * delta_time;
        let new_pos = current_pos + movement;
        
        // Don't overshoot
        if new_pos.distance(target_pos) < current_pos.distance(target_pos) {
            new_pos
        } else {
            target_pos
        }
    }
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 moveToward(
        glm::vec3 currentPos,
        glm::vec3 targetPos,
        float speed,
        float deltaTime
    ) {
        glm::vec3 direction = glm::normalize(targetPos - currentPos);
        glm::vec3 movement = direction * speed * deltaTime;
        glm::vec3 newPos = currentPos + movement;
        
        if (glm::distance(newPos, targetPos) < glm::distance(currentPos, targetPos)) {
            return newPos;
        } else {
            return targetPos;
        }
    }
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 MoveToward(
        float3 currentPos,
        float3 targetPos,
        float speed,
        float deltaTime
    ) {
        float3 direction = math.normalize(targetPos - currentPos);
        float3 movement = direction * speed * deltaTime;
        float3 newPos = currentPos + movement;
        
        if (math.distance(newPos, targetPos) < math.distance(currentPos, targetPos)) {
            return newPos;
        } else {
            return targetPos;
        }
    }
    ```

### Reflection Vector

Reflect a vector $\mathbf{v}$ across a surface with normal $\mathbf{n}$:

$$\mathbf{r} = \mathbf{v} - 2(\mathbf{v} \cdot \mathbf{n})\mathbf{n}$$

Used for: Mirror reflections, bouncing projectiles, specular lighting.

=== "Pseudocode"
    ```
    function reflect(v, n):
        // n must be normalized
        return v - 2 * dot(v, n) * n
    ```

=== "Rust (glam)"
    ```rust
    let incident = Vec3::new(1.0, -1.0, 0.0).normalize();
    let normal = Vec3::Y;  // (0, 1, 0)
    let reflected = incident - 2.0 * incident.dot(normal) * normal;
    
    // Or use built-in
    let reflected = incident.reflect(normal);
    ```

=== "C++ (glm)"
    ```cpp
    glm::vec3 incident = glm::normalize(glm::vec3(1.0f, -1.0f, 0.0f));
    glm::vec3 normal(0.0f, 1.0f, 0.0f);
    glm::vec3 reflected = glm::reflect(incident, normal);
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    float3 incident = math.normalize(new float3(1.0f, -1.0f, 0.0f));
    float3 normal = new float3(0.0f, 1.0f, 0.0f);
    float3 reflected = math.reflect(incident, normal);
    ```

## Common Pitfalls

### 1. Not Normalizing Direction Vectors

```rust
// Bad: direction has arbitrary length
let direction = target_pos - current_pos;
let velocity = direction * speed;  // Wrong! Speed varies with distance

// Good: normalize to get unit direction
let direction = (target_pos - current_pos).normalize();
let velocity = direction * speed;  // Correct! Consistent speed
```

### 2. Normalizing Zero Vectors

```rust
// Bad: crashes if positions are identical
let direction = (target_pos - current_pos).normalize();

// Good: check for zero or use safe variant
let direction = (target_pos - current_pos).normalize_or_zero();
```

### 3. Unnecessary Square Roots

```rust
// Bad: expensive sqrt for comparison
if (target_pos - current_pos).length() < 5.0 {
    // ...
}

// Good: compare squared values
if (target_pos - current_pos).length_squared() < 25.0 {
    // ...
}
```

### 4. Confusing Position and Direction

Vectors represent both positions and directions, but they're conceptually different:

- **Position**: A point in space (relative to origin)
- **Direction**: An arrow/offset (often normalized)

Keep them semantically distinct in your code.

## Summary

| Operation | Formula | Use Case |
|-----------|---------|----------|
| **Addition** | $\mathbf{v} + \mathbf{w}$ | Movement, combining forces |
| **Subtraction** | $\mathbf{v} - \mathbf{w}$ | Direction from w to v |
| **Scaling** | $k\mathbf{v}$ | Apply speed, scale forces |
| **Length** | $\|\mathbf{v}\| = \sqrt{v_x^2 + v_y^2 + v_z^2}$ | Distance, speed |
| **Normalize** | $\hat{\mathbf{v}} = \mathbf{v} / \|\mathbf{v}\|$ | Pure direction |
| **Dot Product** | $\mathbf{v} \cdot \mathbf{w} = v_x w_x + v_y w_y + v_z w_z$ | Angle, projection, lighting |
| **Cross Product** | $\mathbf{v} \times \mathbf{w}$ | Perpendicular vector, normals |
| **Distance** | $\|\mathbf{q} - \mathbf{p}\|$ | Distance between points |

## Next Steps

Now that you understand vectors, you're ready to explore:

- **[Matrices](matrices.md)** - Transform vectors with matrices
- **[Coordinate Spaces](coordinate-spaces.md)** - Different reference frames
- **[Interpolation](interpolation.md)** - Smooth vector transitions

## Further Reading

- **Interactive Visualization**: [3Blue1Brown - Vectors](https://www.youtube.com/watch?v=fNk_zzaMoSs)
- **In-Depth**: *3D Math Primer for Graphics and Game Development*, Chapter 2
- **glam Documentation**: [Vector Types](https://docs.rs/glam/latest/glam/f32/struct.Vec3.html)
