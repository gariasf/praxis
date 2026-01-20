# Mathematical Foundations for Game Engines

Welcome to the mathematical foundations course for 3D game engine development. This section covers the essential mathematical concepts required to understand and implement modern game engines.

## Prerequisites

- **Basic Algebra**: Understanding of equations, variables, and functions
- **High School Geometry**: Triangles, angles, basic trigonometry (sin, cos, tan)
- **Programming Fundamentals**: Variables, functions, basic data structures

No advanced mathematics background is required—we'll build concepts from the ground up.

## Course Structure

This course is organized into self-contained modules that can be studied independently or sequentially:

### Core Topics

1. **[Vectors](vectors.md)** - Positions, directions, and vector operations
2. **[Matrices](matrices.md)** - Transformations, projections, and matrix operations
3. **[Quaternions](quaternions.md)** - Rotation representation without gimbal lock
4. **[Coordinate Spaces](coordinate-spaces.md)** - Local, world, view, and clip spaces
5. **[Interpolation](interpolation.md)** - Smooth transitions and blending

### Recommended Learning Order

For beginners, we recommend this sequence:

```text
1. Vectors (foundation)
   ↓
2. Coordinate Spaces (understanding different reference frames)
   ↓
3. Matrices (transformations)
   ↓
4. Interpolation (smooth motion)
   ↓
5. Quaternions (advanced rotations)
```

For experienced developers, jump directly to topics of interest.

## Teaching Approach

### Language-Agnostic Mathematics

All mathematical concepts are presented using:

- **Mathematical Notation**: Standard notation used in textbooks and papers
- **Geometric Intuition**: Visual explanations and diagrams
- **Pseudocode Algorithms**: Language-neutral implementations
- **Library References**: How to use existing math libraries

### Library-Specific Examples

Where appropriate, we provide examples using popular math libraries:

| Language | Library | Description |
|----------|---------|-------------|
| **Rust** | `glam` | High-performance, SIMD-optimized, used in Praxis |
| **C++** | `glm` | Header-only, OpenGL-style API |
| **C#** | `Unity.Mathematics` | Burst-compatible, Unity's official library |
| **C++** | DirectXMath | Microsoft's SIMD library for DirectX |

!!! info "Library Independence"
    You can implement these concepts using any math library or from scratch. The mathematics remains the same regardless of the library.

### Why Learn the Math?

Understanding the mathematics behind game engines allows you to:

- **Debug effectively**: Understand why transforms behave unexpectedly
- **Optimize intelligently**: Know which operations are expensive
- **Implement features**: Build custom systems (IK, constraints, procedural animation)
- **Read documentation**: Understand graphics papers and API docs
- **Communicate clearly**: Discuss problems with other developers

## Mathematical Notation Guide

Throughout this course, we use standard mathematical notation:

### Common Symbols

| Symbol | Meaning | Example |
|--------|---------|---------|
| $\mathbf{v}$ | Vector (bold lowercase) | $\mathbf{v} = (x, y, z)$ |
| $\mathbf{M}$ | Matrix (bold uppercase) | $\mathbf{M} = 4 \times 4$ matrix |
| $\|\mathbf{v}\|$ | Magnitude/length | $\|\mathbf{v}\| = \sqrt{x^2 + y^2 + z^2}$ |
| $\mathbf{v} \cdot \mathbf{w}$ | Dot product | $\mathbf{v} \cdot \mathbf{w} = v_x w_x + v_y w_y + v_z w_z$ |
| $\mathbf{v} \times \mathbf{w}$ | Cross product | $\mathbf{v} \times \mathbf{w}$ = perpendicular vector |
| $\mathbf{M}\mathbf{v}$ | Matrix-vector product | Transform vector by matrix |
| $\theta$ | Angle | $\theta = 45°$ or $\theta = \frac{\pi}{4}$ rad |
| $\hat{\mathbf{v}}$ | Normalized vector | $\hat{\mathbf{v}} = \frac{\mathbf{v}}{\|\mathbf{v}\|}$ |

### Coordinate Notation

We use multiple notations for clarity:

- **Tuple**: $(x, y, z)$ - Explicit coordinates
- **Subscript**: $v_x, v_y, v_z$ - Individual components
- **Bold**: $\mathbf{v}$ - The vector as a whole

## Quick Reference

### Essential Formulas

#### Vector Operations
```math
\mathbf{v} + \mathbf{w} = (v_x + w_x, v_y + w_y, v_z + w_z)
```
```math
k\mathbf{v} = (kv_x, kv_y, kv_z)
```
```math
\|\mathbf{v}\| = \sqrt{v_x^2 + v_y^2 + v_z^2}
```

#### Dot Product
```math
\mathbf{v} \cdot \mathbf{w} = v_x w_x + v_y w_y + v_z w_z = \|\mathbf{v}\| \|\mathbf{w}\| \cos\theta
```

#### Cross Product
```math
\mathbf{v} \times \mathbf{w} = (v_y w_z - v_z w_y,\ v_z w_x - v_x w_z,\ v_x w_y - v_y w_x)
```

#### Linear Interpolation
```math
\text{lerp}(\mathbf{a}, \mathbf{b}, t) = (1 - t)\mathbf{a} + t\mathbf{b}, \quad t \in [0, 1]
```

### Common Operations by Library

=== "Rust (glam)"
    ```rust
    use glam::{Vec3, Mat4, Quat};
    
    // Vector
    let v = Vec3::new(1.0, 2.0, 3.0);
    let length = v.length();
    let normalized = v.normalize();
    
    // Matrix
    let m = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
    let transformed = m.transform_point3(v);
    
    // Quaternion
    let q = Quat::from_rotation_y(90_f32.to_radians());
    let rotated = q * v;
    ```

=== "C++ (glm)"
    ```cpp
    #include <glm/glm.hpp>
    #include <glm/gtc/quaternion.hpp>
    
    // Vector
    glm::vec3 v(1.0f, 2.0f, 3.0f);
    float length = glm::length(v);
    glm::vec3 normalized = glm::normalize(v);
    
    // Matrix
    glm::mat4 m = glm::translate(glm::mat4(1.0f), glm::vec3(1.0f, 0.0f, 0.0f));
    glm::vec3 transformed = glm::vec3(m * glm::vec4(v, 1.0f));
    
    // Quaternion
    glm::quat q = glm::angleAxis(glm::radians(90.0f), glm::vec3(0.0f, 1.0f, 0.0f));
    glm::vec3 rotated = q * v;
    ```

=== "C# (Unity.Mathematics)"
    ```csharp
    using Unity.Mathematics;
    
    // Vector
    float3 v = new float3(1.0f, 2.0f, 3.0f);
    float length = math.length(v);
    float3 normalized = math.normalize(v);
    
    // Matrix
    float4x4 m = float4x4.Translate(new float3(1.0f, 0.0f, 0.0f));
    float3 transformed = math.transform(m, v);
    
    // Quaternion
    quaternion q = quaternion.RotateY(math.radians(90.0f));
    float3 rotated = math.rotate(q, v);
    ```

## Additional Resources

### Textbooks
- **3D Math Primer for Graphics and Game Development** by Fletcher Dunn & Ian Parberry (accessible introduction)
- **Mathematics for 3D Game Programming and Computer Graphics** by Eric Lengyel (comprehensive reference)
- **Essential Mathematics for Games and Interactive Applications** by James M. Van Verth & Lars M. Bishop

### Online Resources
- **Immersive Linear Algebra** - Interactive web book with visualizations
- **3Blue1Brown** - YouTube series "Essence of Linear Algebra"
- **Khan Academy** - Linear algebra and trigonometry courses
- **Math3D** - Interactive 3D math visualizations

### Library Documentation
- [glam docs](https://docs.rs/glam/) - Rust
- [GLM docs](https://glm.g-truc.net/) - C++
- [Unity.Mathematics docs](https://docs.unity3d.com/Packages/com.unity.mathematics@latest) - C#

## Next Steps

Ready to begin? Start with **[Vectors](vectors.md)**, the foundation of 3D mathematics.

Or jump to a specific topic:

<div class="grid cards" markdown>

- :fontawesome-solid-arrow-right-long: **[Vectors](vectors.md)**  
  Positions, directions, and basic operations

- :fontawesome-solid-table-cells: **[Matrices](matrices.md)**  
  Transformations and projections

- :fontawesome-solid-rotate: **[Quaternions](quaternions.md)**  
  Smooth rotations without gimbal lock

- :fontawesome-solid-globe: **[Coordinate Spaces](coordinate-spaces.md)**  
  Understanding different reference frames

- :fontawesome-solid-chart-line: **[Interpolation](interpolation.md)**  
  Smooth transitions and blending

</div>

---

!!! tip "Study Tips"
    - Work through examples with pen and paper
    - Implement algorithms yourself before using libraries
    - Visualize concepts with simple diagrams
    - Test understanding by explaining to others
    - Build small demos to see math in action
