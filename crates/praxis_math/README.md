# Praxis Math

Mathematics library for the Praxis game engine.

## Overview

Vectors, matrices, quaternions, and geometric primitives built on glam with SIMD acceleration.

**Key Features:**
- Vec2, Vec3, Vec4 with standard operations
- Mat3, Mat4 for transformations
- Quat for rotations
- AABB, spheres, planes, rays
- Serialization support
- Right-handed coordinate system (Vulkan conventions)

## Quick Start

### Vectors

```rust
use praxis_math::Vec3;

let v1 = Vec3::new(1.0, 2.0, 3.0);
let v2 = Vec3::new(4.0, 5.0, 6.0);

let sum = v1 + v2;
let dot = v1.dot(v2);
let cross = v1.cross(v2);
let normalized = v1.normalize();
```

### Matrices

```rust
use praxis_math::Mat4;

let translation = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
let rotation = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
let scale = Mat4::from_scale(Vec3::splat(2.0));

let transform = translation * rotation * scale;

let proj = Mat4::perspective_rh(70.0_f32.to_radians(), 16.0/9.0, 0.1, 1000.0);
```

### Quaternions

```rust
use praxis_math::Quat;

let quat = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
let interpolated = q1.slerp(q2, 0.5);
```

### Geometric Primitives

```rust
use praxis_math::Aabb;

let aabb = Aabb::from_min_max(Vec3::new(-1.0, -1.0, -1.0), Vec3::ONE);

if aabb.contains_point(point) {
    // Point inside
}

if aabb.intersects(&other_aabb) {
    // AABBs intersect
}
```

## Coordinate System

Right-handed coordinate system:
- X: Right
- Y: Up
- Z: Forward (into screen)

## Performance

SIMD acceleration (SSE2 on x86, NEON on ARM) with automatic scalar fallback.

## Dependencies

- `glam` 0.30.4: Core math with SIMD

## API Stability

**Status:** Stable

Re-exports glam types with minimal additions. API follows glam's stability guarantees. AABB and geometric primitives are stable. Breaking changes will be documented in the changelog.
