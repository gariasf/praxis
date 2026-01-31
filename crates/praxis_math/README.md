# praxis_math

Math library for Praxis engine using glam.

## Overview

Thin wrapper around `glam` providing SIMD-optimized 3D mathematics with serialization support.

## Types

- **Vectors**: `Vec2`, `Vec3`, `Vec4`
- **Matrices**: `Mat2`, `Mat3`, `Mat4`
- **Quaternions**: `Quat`
- **Affine Transforms**: `Affine2`, `Affine3`

## Features

- SIMD optimization for high performance
- Serialization via `serde`
- Comprehensive 3D math operations
- Column-major matrices (Vulkan-compatible)

## Example

```rust
use praxis_math::{Vec3, Quat, Mat4};

// Vectors
let position = Vec3::new(1.0, 2.0, 3.0);
let direction = Vec3::Z;

// Quaternions for rotation
let rotation = Quat::from_rotation_y(90_f32.to_radians());

// Matrices
let transform = Mat4::from_rotation_translation(rotation, position);
```

## Dependencies

- `glam`: SIMD-optimized 3D math library
- `serde`: Serialization support

## Usage

```toml
praxis_math = { path = "../praxis_math", version = "0.1.0" }
```
