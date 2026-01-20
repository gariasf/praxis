# Decision Tree: Custom Math Library vs Using Existing Libraries

```
┌──────────────────────────────────────────────────┐
│ Should I write my own math library or use one?  │
└──────────────────────────────────────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ Is this a learning project?       │
        └───────────────────────────────────┘
                /                   \
               /                     \
       Yes, learning           No, production
              │                       │
              ▼                       ▼
    ┌──────────────────┐      ┌─────────────┐
    │ Write your own   │      │ Use library │
    │ (educational)    │      │ (strongly   │
    └──────────────────┘      │  rec.)      │
              │               └─────────────┘
              ▼
    ┌──────────────────────┐
    │ What do you want to  │
    │ learn?               │
    └──────────────────────┘
          /          \
         /            \
   3D Math      Performance
   Fundamentals   Optimization
        │              │
        ▼              ▼
   ┌─────────┐   ┌──────────┐
   │ Basic   │   │ SIMD     │
   │ Custom  │   │ Custom   │
   └─────────┘   └──────────┘
```

## Quick Decision Matrix

| Factor | Custom Math | Existing Library |
|--------|-------------|------------------|
| **Learning 3D math** | ✅ Excellent | ❌ Miss fundamentals |
| **Production code** | ❌ Risky | ✅ Battle-tested |
| **Time to market** | ❌ Slow | ✅ Fast |
| **Performance** | ⚠️ Depends on skill | ✅ Highly optimized |
| **SIMD optimization** | ❌ Very difficult | ✅ Built-in |
| **Bug-free** | ❌ Likely has bugs | ✅ Well-tested |
| **Maintenance** | ❌ Your responsibility | ✅ Community maintained |
| **Platform support** | ⚠️ Limited | ✅ Multi-platform |
| **Special features** | ✅ Total control | ⚠️ May lack specifics |
| **Documentation** | ❌ You write it | ✅ Extensive docs |

## Detailed Analysis

### Using Existing Math Libraries (Recommended for Production)

**Popular libraries by language:**

**Rust:**
- `glam` - Praxis uses this (SIMD, zero-cost)
- `nalgebra` - Generic, scientific computing
- `cgmath` - Game-focused, no SIMD
- `ultraviolet` - SIMD, modern

**C++:**
- `GLM` - Industry standard, GLSL-like API
- `Eigen` - Scientific computing, excellent
- `DirectXMath` - Microsoft, SIMD optimized
- `MathFu` - Google, mobile-optimized

**C#:**
- `System.Numerics` - Built-in, SIMD
- `Unity.Mathematics` - Unity's DOTS math
- `SharpDX` - DirectX wrapper

#### Choose Existing Library If:

**✅ High Priority (MOST PROJECTS):**
- Building **production engine or game**
- **Time constrained** (want to focus on game logic)
- Need **performance** (SIMD optimization)
- Want **battle-tested** code (no subtle bugs)
- Need **multi-platform** support
- Team has **mixed skill levels**
- Want **community support** and documentation

**Example Use Cases:**
- Commercial game development
- Production game engines
- Professional projects
- Hobby projects (unless learning math)
- Team projects

**Pros:**
- **Performance**: SIMD-optimized out of the box
- **Correctness**: Thousands of hours of testing
- **Time savings**: Focus on game, not math
- **Features**: Comprehensive APIs (matrices, quaternions, etc.)
- **Platform support**: Works on x86, ARM, WASM, etc.
- **Documentation**: Extensive examples and guides
- **Community**: Get help when stuck
- **Maintenance**: Updates and bug fixes handled
- **Interop**: Works with other libraries

**Cons:**
- **Learning**: Miss fundamentals if you never implement
- **Overkill**: May include features you don't need
- **API style**: Might not match your preferences
- **Dependencies**: External dependency in project
- **Black box**: Harder to debug internals

**Praxis Example (using glam):**
```rust
use glam::{Vec3, Quat, Mat4};

// Vectors
let position = Vec3::new(1.0, 2.0, 3.0);
let velocity = Vec3::new(0.1, 0.0, 0.0);
let new_pos = position + velocity * dt;

// Quaternions (complex math, easy API)
let rotation = Quat::from_rotation_y(angle);
let rotated = rotation * direction;

// Matrices (SIMD-optimized)
let view = Mat4::look_at_rh(eye, target, up);
let proj = Mat4::perspective_rh(fov, aspect, near, far);
let mvp = proj * view * model;

// All of this is:
// - SIMD optimized
// - Cross-platform
// - Bug-free (tested extensively)
// - Fast to write
```

**Performance:**
```rust
// glam (SIMD):
1,000,000 matrix multiplies: ~5ms

// Naive custom (no SIMD):
1,000,000 matrix multiplies: ~50ms

// 10x performance difference!
```

### Writing Custom Math Library (Educational)

#### Choose Custom Math If:

**✅ High Priority:**
- **Learning project** (want to understand 3D math)
- **Educational engine** (teaching fundamentals)
- **Research** (exploring new techniques)
- Have **unique requirements** library doesn't support
- Want **zero dependencies** (embedded, special platforms)
- **Performance tuning** for very specific use case

**Example Use Cases:**
- Learning 3D math fundamentals
- University coursework
- Blog posts/tutorials on math
- Extremely constrained platforms
- Research engines

**Pros:**
- **Learning**: Deep understanding of 3D math
- **Control**: Exact behavior you want
- **No dependencies**: Self-contained
- **Custom features**: Exact API you need
- **Educational**: Great for teaching
- **Debugging**: Can step through everything

**Cons:**
- **Time consuming**: Weeks to months of work
- **Bugs**: Will have subtle bugs (gimbal lock, numerical stability)
- **Performance**: Hard to beat SIMD libraries
- **Maintenance**: All on you
- **Testing**: Need comprehensive test suite
- **Platform-specific**: SIMD requires per-platform code
- **Opportunity cost**: Time not spent on game logic

**Basic Custom Implementation:**
```rust
// Simple vector (no SIMD)
#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        Vec3 {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

// Operator overloading
impl std::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

// ... many more operators and methods needed
```

**What you need to implement (minimum):**
- Vec2, Vec3, Vec4
- Mat2, Mat3, Mat4
- Quaternion (complex!)
- Operators (+, -, *, /)
- Geometric operations (dot, cross, normalize, etc.)
- Matrix operations (multiply, inverse, transpose, etc.)
- Transformations (translate, rotate, scale)
- Projections (perspective, orthographic)
- View matrices (look-at)
- Interpolation (lerp, slerp)

**Estimated effort:**
- Basic implementation: 2-4 weeks
- Comprehensive: 2-3 months
- SIMD optimized: +6-12 months
- Battle-tested: Years of use

## What's Hard About Custom Math?

### 1. Quaternions

Quaternions are notoriously tricky:

```rust
// Looks simple...
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// But has subtle issues:
// - Normalization required (numerical drift)
// - Double-cover (q and -q represent same rotation)
// - Gimbal lock when converting from Euler
// - Slerp requires special math
// - Multiplication order matters
```

**Common bugs:**
- Forgetting to normalize after operations
- Wrong multiplication order
- Gimbal lock in Euler conversions
- Incorrect slerp implementation

**glam handles all of this correctly.**

### 2. SIMD Optimization

SIMD (Single Instruction Multiple Data) provides massive speedups:

```rust
// Scalar (custom): 1 operation at a time
let result = Vec3 {
    x: a.x + b.x,  // 1 instruction
    y: a.y + b.y,  // 1 instruction
    z: a.z + b.z,  // 1 instruction
};

// SIMD (glam): All at once
// Uses __m128 (SSE) on x86
let result = _mm_add_ps(a, b); // 3 adds in 1 instruction
```

**Platform-specific SIMD:**
- **x86/x64**: SSE, SSE2, AVX, AVX2, AVX-512
- **ARM**: NEON, SVE
- **WASM**: SIMD128
- **Other**: AltiVec (PowerPC), etc.

**Implementing SIMD yourself:**
```rust
// You need platform-specific code:
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

pub struct Vec3 {
    #[cfg(target_arch = "x86_64")]
    inner: __m128,
    
    #[cfg(target_arch = "aarch64")]
    inner: float32x4_t,
    
    // Fallback for other platforms
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    inner: [f32; 4],
}

// Now implement operations for each platform...
// This is HARD and error-prone!
```

**glam does this for you:**
- Automatically uses best SIMD for platform
- Falls back gracefully
- Tested on all platforms

### 3. Numerical Stability

Floating-point math has subtle issues:

```rust
// Catastrophic cancellation
let a = 1.0e10;
let b = 1.0e-10;
let result = (a + b) - a; // Should be b, might be 0.0!

// Accumulated error in rotations
let mut rot = Quat::identity();
for _ in 0..10000 {
    rot = rot * small_rotation; // Accumulates error
}
// rot is no longer normalized!
```

**Solutions:**
- Careful operation ordering
- Periodic re-normalization
- Kahan summation for accumulation
- Double precision where needed

**Libraries handle this properly.**

### 4. Matrix Inverse

Matrix inverse is complex:

```rust
// 4x4 matrix inverse requires:
// - Computing determinant
// - Computing cofactor matrix
// - Handling singular matrices
// - Numerical stability considerations

// ~200 lines of careful code
// Easy to get wrong
// Performance-critical

pub fn inverse(&self) -> Option<Mat4> {
    // Compute determinant
    let det = /* complex calculation */;
    
    if det.abs() < EPSILON {
        return None; // Singular matrix
    }
    
    // Compute inverse using cofactors
    // ... 150+ lines of math
}
```

**glam's inverse:**
- SIMD optimized
- Numerically stable
- Well-tested
- Fast

### 5. Look-At Matrix

Building view matrices is tricky:

```rust
// Seems simple, but has edge cases
pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let forward = (target - eye).normalize();
    
    // Bug: What if forward == up?
    // Results in zero cross product!
    let right = up.cross(forward).normalize();
    let up = forward.cross(right);
    
    // Build matrix... (more code)
}
```

**Edge cases:**
- Eye == target (zero direction)
- Forward parallel to up (gimbal lock)
- Denormalized inputs

**Libraries handle all edge cases.**

## Hybrid Approach: Wrapper Around Library

**Best of both worlds:**

```rust
// Your custom types
pub struct Vector3 {
    inner: glam::Vec3, // Use library internally
}

impl Vector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: glam::Vec3::new(x, y, z),
        }
    }
    
    // Your API
    pub fn magnitude(&self) -> f32 {
        self.inner.length() // Delegate to glam
    }
    
    // Add custom methods library doesn't have
    pub fn custom_operation(&self) -> Vector3 {
        // Your special logic
    }
}
```

**When to use:**
- Want custom API style
- Need additional features
- Gradual migration from custom math
- Teaching (show both approaches)

**Pros:**
- Performance of library
- API you prefer
- Extensibility
- Easy migration

**Cons:**
- Wrapper overhead (usually negligible)
- Maintenance (keep wrapper updated)
- Two layers to understand

## What If Libraries Don't Support Your Need?

**Rare cases where custom makes sense:**

### 1. Fixed-Point Math (Console/Embedded)

```rust
// Some old consoles or embedded systems need fixed-point
pub struct Fixed16 {
    value: i32, // 16.16 fixed-point
}

impl Fixed16 {
    pub fn from_float(f: f32) -> Self {
        Self {
            value: (f * 65536.0) as i32,
        }
    }
    
    pub fn mul(self, other: Fixed16) -> Fixed16 {
        Self {
            value: ((self.value as i64 * other.value as i64) >> 16) as i32,
        }
    }
}
```

**When needed:**
- Retro platforms (GBA, DS)
- Deterministic math (lockstep networking)
- No floating-point unit

### 2. Double-Precision

```rust
// Most game libs use f32, but you need f64 for:
// - Large worlds (space sims)
// - Scientific accuracy
// - Precision at distance

pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

**When needed:**
- Space simulations
- CAD/engineering software
- Astronomical scales

**Note:** Some libraries support this (nalgebra, cgmath with feature flags)

### 3. Custom Coordinate System

```rust
// Z-up coordinate system (default is Y-up)
pub struct Vec3ZUp {
    pub x: f32,
    pub y: f32,
    pub z: f32, // Z is up
}

// Or left-handed coordinates
// Or custom basis
```

**When needed:**
- Matching existing tools/data
- Scientific conventions
- Legacy integration

**Note:** Often easier to just convert at boundaries

## Language-Specific Recommendations

### Rust
**Strong recommendation: glam**

Why glam:
- Zero-cost abstractions
- SIMD optimized
- No allocations
- Compiles to optimal code
- Widely used in Rust gamedev

```rust
// Praxis uses glam
[dependencies]
glam = "0.24" # Current version
```

Alternatives:
- `nalgebra`: More features, heavier
- `cgmath`: Simpler, no SIMD
- `ultraviolet`: Modern, good alternative

### C++
**Strong recommendation: GLM**

Why GLM:
- Industry standard
- GLSL-like syntax
- Header-only
- Well-documented

```cpp
#include <glm/glm.hpp>
#include <glm/gtc/matrix_transform.hpp>

glm::vec3 position(1.0f, 2.0f, 3.0f);
glm::mat4 view = glm::lookAt(eye, target, up);
```

Alternatives:
- `Eigen`: Scientific computing
- `DirectXMath`: Windows/Xbox
- `MathFu`: Google, mobile

### C#
**Strong recommendation: System.Numerics or Unity.Mathematics**

Unity.Mathematics for Unity DOTS:
```csharp
using Unity.Mathematics;

float3 position = new float3(1, 2, 3);
quaternion rotation = quaternion.AxisAngle(float3(0, 1, 0), angle);
float4x4 matrix = float4x4.TRS(position, rotation, scale);
```

System.Numerics for .NET:
```csharp
using System.Numerics;

Vector3 position = new Vector3(1, 2, 3);
Matrix4x4 transform = Matrix4x4.CreateTranslation(position);
```

### JavaScript/TypeScript
**Strong recommendation: gl-matrix**

```javascript
import { vec3, mat4 } from 'gl-matrix';

let position = vec3.fromValues(1, 2, 3);
let view = mat4.create();
mat4.lookAt(view, eye, target, up);
```

Alternative:
- `three.js` (includes math)

## Testing Custom Math

**If you do write custom math, you MUST test thoroughly:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vector_normalize() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let n = v.normalize();
        
        // Test length is 1
        assert!((n.length() - 1.0).abs() < 0.0001);
        
        // Test direction preserved
        let expected = Vec3::new(0.6, 0.8, 0.0);
        assert!((n.x - expected.x).abs() < 0.0001);
    }
    
    #[test]
    fn test_quaternion_slerp() {
        // Test cases from known-good implementations
        // ...
    }
    
    #[test]
    fn test_matrix_inverse() {
        let m = Mat4::from_translation(Vec3::new(1, 2, 3));
        let inv = m.inverse().unwrap();
        let identity = m * inv;
        
        // Should be identity
        assert!(identity.is_identity());
    }
    
    // Need hundreds of tests!
    // - Edge cases (zero vectors, parallel vectors, etc.)
    // - Numerical stability
    // - Known results from other libraries
}
```

**Test categories needed:**
1. **Basic operations**: Add, subtract, multiply, etc.
2. **Geometric**: Dot, cross, normalize, etc.
3. **Transformations**: Translate, rotate, scale
4. **Edge cases**: Zero vectors, parallel vectors, singular matrices
5. **Numerical stability**: Accumulated error, precision
6. **Performance**: Benchmark against known libraries

**Effort:**
- Writing tests: 1-2 weeks
- Finding edge cases: Ongoing
- Debugging failures: Frustrating

## Decision Checklist

| Question | Custom | Library |
|----------|--------|---------|
| Learning project? | ✓ | |
| Production project? | | ✓ |
| Time constrained? | | ✓ |
| Need SIMD performance? | | ✓ |
| Want to focus on game logic? | | ✓ |
| Exploring 3D math fundamentals? | ✓ | |
| Need battle-tested code? | | ✓ |
| Have unique requirements? | ✓ | |
| Want community support? | | ✓ |
| Embedded/special platform? | ⚠️ | ⚠️ |

**Score:**
- **Mostly Custom**: Only if learning or very special needs
- **Mostly Library**: Use existing library (vast majority of cases)

## Migration Path

### Starting Custom, Moving to Library

**Common path for learning projects:**

1. **Implement basics** (vectors, matrices)
   - Learn fundamentals
   - Build intuition

2. **Hit complexity wall** (quaternions, SIMD)
   - Realize depth of problem
   - Appreciate library work

3. **Switch to library** (glam, GLM, etc.)
   - Retain understanding
   - Gain production quality
   - Focus on game

**Praxis recommendation:**
- Write basic Vec3/Mat4 yourself first (learning)
- Then switch to glam for real work (production)

### Wrapping Library (Custom API)

If you want custom API:

```rust
// 1. Use library internally
pub struct Transform {
    position: glam::Vec3,
    rotation: glam::Quat,
    scale: glam::Vec3,
}

// 2. Provide your API
impl Transform {
    pub fn translate(&mut self, delta: [f32; 3]) {
        self.position += glam::Vec3::from(delta);
    }
    
    // Your preferred API style
}
```

## Recommended Reading

**If Writing Custom:**
- *3D Math Primer for Graphics and Game Development* by Dunn & Parberry
- *Essential Mathematics for Games* by Van Verth & Bishop
- *Real-Time Rendering* by Akenine-Möller et al.

**Library Documentation:**
- [glam docs](https://docs.rs/glam/)
- [GLM manual](https://glm.g-truc.net/)
- [DirectXMath](https://docs.microsoft.com/en-us/windows/win32/dxmath/directxmath-portal)

**Learning Resources:**
- [Immersive Linear Algebra](http://immersivemath.com/)
- [3Blue1Brown - Essence of Linear Algebra](https://www.youtube.com/playlist?list=PLZHQObOWTQDPD3MizzM2xVFitgF8hE_ab)
- [Quaternions Visualized](https://eater.net/quaternions)

## Performance Comparison

### Matrix Multiply (1,000,000 iterations)

**Custom (scalar):**
```rust
// Naive implementation
impl Mul for Mat4 {
    fn mul(self, other: Mat4) -> Mat4 {
        let mut result = Mat4::zero();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self[i][k] * other[k][j];
                }
            }
        }
        result
    }
}
```
**Time: ~50ms**

**glam (SIMD):**
```rust
let result = m1 * m2; // Uses __m128 instructions
```
**Time: ~5ms** (10x faster!)

**Why:** SIMD processes 4 floats at once, glam uses optimized algorithms

### Vector Normalization (10,000,000 vectors)

**Custom (scalar):**
```rust
let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
let normalized = Vec3 {
    x: v.x / len,
    y: v.y / len,
    z: v.z / len,
};
```
**Time: ~80ms**

**glam (SIMD):**
```rust
let normalized = v.normalize();
```
**Time: ~20ms** (4x faster!)

## Conclusion

**TL;DR:**
- **Production project? → Use existing library (glam, GLM, etc.)**
- **Learning 3D math? → Write basic custom, then switch to library**
- **Very special needs? → Consider custom (rare)**
- **Want custom API? → Wrap existing library**

**Praxis Choice: glam**
- SIMD optimized (5-10x faster than naive)
- Zero-cost abstractions
- Battle-tested (used by Bevy and other engines)
- Excellent Rust integration
- Comprehensive feature set
- Well-documented

**Why Praxis doesn't use custom math:**
1. **Focus**: Want to teach engine architecture, not math implementation
2. **Performance**: glam is faster than we could write
3. **Correctness**: glam has years of testing and bug fixes
4. **Time**: Implementing quality math lib would take months
5. **Educational**: Can still teach 3D math concepts using glam API

**When to write custom:**
1. You're learning 3D math fundamentals (do it!)
2. You have unique requirements libraries don't support (rare)
3. You're on a platform without library support (very rare)

**99% of the time: Use a library. Focus on your game, not math bugs.**
