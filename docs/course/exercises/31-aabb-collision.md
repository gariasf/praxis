# Exercise 31: AABB Collision Detection

**Difficulty**: 🟢 Beginner | **Estimated Time**: 2-3h | **Subsystem**: Physics

## Overview

Implement Axis-Aligned Bounding Box (AABB) collision detection. AABBs are the foundation of broad-phase collision detection in physics engines.

## Learning Objectives

- Understand AABB representation and properties
- Implement intersection tests
- Learn sweep tests (moving AABBs)
- Optimize with early exit conditions

## Requirements

### Functional Requirements

1. **AABB Structure**
   - Store minimum and maximum points
   - Calculate center and extents
   - Construct from points or center+extents

2. **Intersection Tests**
   - Static AABB vs AABB
   - AABB vs point
   - Ray vs AABB
   - Sweep test (moving AABB vs static AABB)

3. **Utilities**
   - Expand AABB by point/AABB
   - Compute surface area
   - Compute volume

### Non-Functional Requirements

- **Performance**: 1 million AABB tests in < 10ms
- **Accuracy**: Handle edge cases (touching boxes, zero-size boxes)
- **Robustness**: No numerical instability

## API Design

```rust
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self;
    pub fn from_center_extents(center: Vec3, extents: Vec3) -> Self;
    pub fn from_points(points: &[Vec3]) -> Self;
    
    pub fn center(&self) -> Vec3;
    pub fn extents(&self) -> Vec3;
    pub fn surface_area(&self) -> f32;
    pub fn volume(&self) -> f32;
    
    pub fn contains_point(&self, point: Vec3) -> bool;
    pub fn intersects(&self, other: &AABB) -> bool;
    pub fn intersects_ray(&self, origin: Vec3, direction: Vec3) -> Option<f32>;
    
    pub fn expand_by_point(&mut self, point: Vec3);
    pub fn expand_by_aabb(&mut self, aabb: &AABB);
    pub fn merge(&self, other: &AABB) -> AABB;
}
```

## Validation Criteria

### Correctness
- [ ] Detects overlapping AABBs
- [ ] Rejects non-overlapping AABBs
- [ ] Handles edge cases (touching, contained, identical)
- [ ] Ray intersection accurate
- [ ] Sweep test finds first contact

### Performance
- [ ] 1M intersection tests in < 10ms
- [ ] No allocations in hot path
- [ ] SIMD-friendly layout (optional)

## Test Cases

```rust
#[test]
fn test_basic_intersection() {
    let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    let b = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    
    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
}

#[test]
fn test_no_intersection() {
    let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    let b = AABB::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));
    
    assert!(!a.intersects(&b));
}

#[test]
fn test_touching_boxes() {
    let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    let b = AABB::new(Vec3::ONE, Vec3::new(2.0, 2.0, 2.0));
    
    // Touching at single point - should be considered intersection
    assert!(a.intersects(&b));
}

#[test]
fn test_contained() {
    let outer = AABB::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
    let inner = AABB::new(Vec3::ONE, Vec3::new(2.0, 2.0, 2.0));
    
    assert!(outer.intersects(&inner));
    assert!(inner.intersects(&outer));
}

#[test]
fn test_point_containment() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    
    assert!(aabb.contains_point(Vec3::new(0.5, 0.5, 0.5)));
    assert!(aabb.contains_point(Vec3::ZERO)); // Edge case: on boundary
    assert!(!aabb.contains_point(Vec3::new(2.0, 0.5, 0.5)));
}

#[test]
fn test_ray_intersection() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    
    // Ray from outside pointing at box
    let origin = Vec3::new(-1.0, 0.5, 0.5);
    let direction = Vec3::X;
    
    let t = aabb.intersects_ray(origin, direction);
    assert!(t.is_some());
    assert!((t.unwrap() - 1.0).abs() < 0.001);
}

#[test]
fn test_merge() {
    let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    let b = AABB::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));
    
    let merged = a.merge(&b);
    assert_eq!(merged.min, Vec3::ZERO);
    assert_eq!(merged.max, Vec3::new(3.0, 1.0, 1.0));
}
```

## Performance Targets

| Test | Target |
|------|--------|
| 1M intersection tests | < 10ms |
| 1M point tests | < 5ms |
| 100K ray tests | < 10ms |

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        debug_assert!(min.x <= max.x && min.y <= max.y && min.z <= max.z);
        Self { min, max }
    }
    
    pub fn from_center_extents(center: Vec3, extents: Vec3) -> Self {
        Self {
            min: center - extents,
            max: center + extents,
        }
    }
    
    pub fn from_points(points: &[Vec3]) -> Self {
        assert!(!points.is_empty());
        
        let mut min = points[0];
        let mut max = points[0];
        
        for &point in points.iter().skip(1) {
            min = min.min(point);
            max = max.max(point);
        }
        
        Self { min, max }
    }
    
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    
    pub fn extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
    
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    
    pub fn surface_area(&self) -> f32 {
        let size = self.size();
        2.0 * (size.x * size.y + size.y * size.z + size.z * size.x)
    }
    
    pub fn volume(&self) -> f32 {
        let size = self.size();
        size.x * size.y * size.z
    }
    
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }
    
    pub fn intersects(&self, other: &AABB) -> bool {
        // Separating axis test - if separated on any axis, no intersection
        if self.max.x < other.min.x || self.min.x > other.max.x {
            return false;
        }
        if self.max.y < other.min.y || self.min.y > other.max.y {
            return false;
        }
        if self.max.z < other.min.z || self.min.z > other.max.z {
            return false;
        }
        true
    }
    
    pub fn intersects_ray(&self, origin: Vec3, direction: Vec3) -> Option<f32> {
        let inv_dir = 1.0 / direction;
        
        let t1 = (self.min - origin) * inv_dir;
        let t2 = (self.max - origin) * inv_dir;
        
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        
        let t_near = tmin.x.max(tmin.y).max(tmin.z);
        let t_far = tmax.x.min(tmax.y).min(tmax.z);
        
        if t_near > t_far || t_far < 0.0 {
            None
        } else {
            Some(t_near.max(0.0))
        }
    }
    
    pub fn expand_by_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
    
    pub fn expand_by_aabb(&mut self, aabb: &AABB) {
        self.min = self.min.min(aabb.min);
        self.max = self.max.max(aabb.max);
    }
    
    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    
    pub fn grow(&self, amount: f32) -> AABB {
        AABB {
            min: self.min - Vec3::splat(amount),
            max: self.max + Vec3::splat(amount),
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        Self {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_properties() {
        let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.center(), Vec3::splat(0.5));
        assert_eq!(aabb.extents(), Vec3::splat(0.5));
        assert_eq!(aabb.volume(), 1.0);
    }
}
```

</details>

### C++ (Alternative)

<details>
<summary>Click to reveal C++ implementation</summary>

```cpp
#include <glm/glm.hpp>
#include <algorithm>

struct AABB {
    glm::vec3 min;
    glm::vec3 max;
    
    AABB() : min(FLT_MAX), max(-FLT_MAX) {}
    
    AABB(const glm::vec3& min, const glm::vec3& max) 
        : min(min), max(max) {}
    
    static AABB fromCenterExtents(const glm::vec3& center, const glm::vec3& extents) {
        return AABB(center - extents, center + extents);
    }
    
    glm::vec3 center() const {
        return (min + max) * 0.5f;
    }
    
    glm::vec3 extents() const {
        return (max - min) * 0.5f;
    }
    
    bool intersects(const AABB& other) const {
        return (max.x >= other.min.x && min.x <= other.max.x) &&
               (max.y >= other.min.y && min.y <= other.max.y) &&
               (max.z >= other.min.z && min.z <= other.max.z);
    }
    
    bool containsPoint(const glm::vec3& point) const {
        return point.x >= min.x && point.x <= max.x &&
               point.y >= min.y && point.y <= max.y &&
               point.z >= min.z && point.z <= max.z;
    }
    
    void expandByPoint(const glm::vec3& point) {
        min = glm::min(min, point);
        max = glm::max(max, point);
    }
    
    AABB merge(const AABB& other) const {
        return AABB(glm::min(min, other.min), glm::max(max, other.max));
    }
};
```

</details>

## Related Resources

- [Real-Time Collision Detection](https://realtimecollisiondetection.net/)
- [Praxis Physics Documentation](../../reference/crates.md#praxis_physics)

## Next Steps

- Implement sphere collision (Exercise 32)
- Build spatial partitioning (Exercise 36)
- Integrate with physics engine (Exercise 34)
