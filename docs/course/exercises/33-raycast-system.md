# Exercise 33: Raycast System

**Difficulty**: 🟡 Intermediate | **Estimated Time**: 3-4h | **Subsystem**: Physics

## Overview

Implement raycasting for querying objects along a ray. Essential for shooting mechanics, line-of-sight checks, and editor picking.

## Learning Objectives

- Understand ray-object intersection algorithms
- Learn spatial acceleration for raycasts
- Implement closest-hit and all-hits queries
- Handle edge cases and numerical precision

## Requirements

### Functional Requirements

1. **Ray Representation**
   - Origin and direction
   - Optional maximum distance
   - Ray-AABB intersection
   - Ray-sphere intersection
   - Ray-triangle intersection

2. **Query Types**
   - Raycast: Find closest hit
   - RaycastAll: Find all hits
   - Linecast: Check if ray is blocked
   - Filter by layers/groups

3. **Hit Information**
   - Hit point
   - Hit normal
   - Hit distance
   - Hit entity
   - UV coordinates (optional)

### Non-Functional Requirements

- **Performance**: 1000 raycasts in < 1ms (with spatial acceleration)
- **Accuracy**: Precise intersection math
- **Robustness**: Handle degenerate cases

## API Design

```rust
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3, // Must be normalized
    pub max_distance: f32,
}

pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub entity: Entity,
}

pub trait Raycastable {
    fn raycast(&self, ray: &Ray) -> Option<RayHit>;
}

pub struct RaycastSystem {
    spatial_structure: Octree<Entity>,
}

impl RaycastSystem {
    pub fn raycast(&self, ray: &Ray) -> Option<RayHit>;
    pub fn raycast_all(&self, ray: &Ray) -> Vec<RayHit>;
    pub fn linecast(&self, start: Vec3, end: Vec3) -> bool;
}
```

## Validation Criteria

### Correctness
- [ ] Ray-AABB intersection correct
- [ ] Ray-sphere intersection correct
- [ ] Ray-triangle intersection correct
- [ ] Returns closest hit
- [ ] Handles rays parallel to surfaces
- [ ] Handles rays starting inside objects

### Performance
- [ ] 1000 raycasts in < 1ms (with octree)
- [ ] Gracefully handles scenes with 10,000+ objects

## Test Cases

```rust
#[test]
fn test_ray_aabb_intersection() {
    let ray = Ray {
        origin: Vec3::new(-5.0, 0.0, 0.0),
        direction: Vec3::X,
        max_distance: f32::MAX,
    };
    
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    
    let hit = ray.intersect_aabb(&aabb);
    assert!(hit.is_some());
    assert_eq!(hit.unwrap(), 5.0);
}

#[test]
fn test_ray_sphere_intersection() {
    let ray = Ray {
        origin: Vec3::new(0.0, 0.0, -5.0),
        direction: Vec3::Z,
        max_distance: f32::MAX,
    };
    
    let sphere = Sphere {
        center: Vec3::ZERO,
        radius: 1.0,
    };
    
    let hit = ray.intersect_sphere(&sphere);
    assert!(hit.is_some());
    assert!((hit.unwrap() - 4.0).abs() < 0.001);
}

#[test]
fn test_ray_triangle_intersection() {
    let ray = Ray {
        origin: Vec3::new(0.5, 0.5, -5.0),
        direction: Vec3::Z,
        max_distance: f32::MAX,
    };
    
    let triangle = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ];
    
    let hit = ray.intersect_triangle(&triangle);
    assert!(hit.is_some());
}

#[test]
fn test_raycast_returns_closest() {
    let mut system = RaycastSystem::new();
    
    // Add two objects along ray
    system.add_sphere(Vec3::new(0.0, 0.0, 5.0), 1.0, entity1);
    system.add_sphere(Vec3::new(0.0, 0.0, 10.0), 1.0, entity2);
    
    let ray = Ray {
        origin: Vec3::ZERO,
        direction: Vec3::Z,
        max_distance: f32::MAX,
    };
    
    let hit = system.raycast(&ray).unwrap();
    assert_eq!(hit.entity, entity1); // Closer object
}
```

## Algorithms

### Ray-AABB Intersection (Slab Method)
```rust
impl Ray {
    pub fn intersect_aabb(&self, aabb: &AABB) -> Option<f32> {
        let inv_dir = 1.0 / self.direction;
        
        let t1 = (aabb.min - self.origin) * inv_dir;
        let t2 = (aabb.max - self.origin) * inv_dir;
        
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
}
```

### Ray-Sphere Intersection (Geometric)
```rust
impl Ray {
    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        
        let discriminant = b * b - 4.0 * a * c;
        
        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t >= 0.0 {
                Some(t)
            } else {
                let t = (-b + discriminant.sqrt()) / (2.0 * a);
                if t >= 0.0 {
                    Some(t)
                } else {
                    None
                }
            }
        }
    }
}
```

### Ray-Triangle Intersection (Möller-Trumbore)
```rust
impl Ray {
    pub fn intersect_triangle(&self, v0: Vec3, v1: Vec3, v2: Vec3) 
        -> Option<(f32, Vec2)> // (distance, barycentric coords)
    {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        
        let h = self.direction.cross(edge2);
        let a = edge1.dot(h);
        
        if a.abs() < 1e-8 {
            return None; // Ray parallel to triangle
        }
        
        let f = 1.0 / a;
        let s = self.origin - v0;
        let u = f * s.dot(h);
        
        if u < 0.0 || u > 1.0 {
            return None;
        }
        
        let q = s.cross(edge1);
        let v = f * self.direction.dot(q);
        
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        
        let t = f * edge2.dot(q);
        
        if t > 1e-8 {
            Some((t, Vec2::new(u, v)))
        } else {
            None
        }
    }
}
```

## Performance Targets

| Operation | Without Octree | With Octree |
|-----------|----------------|-------------|
| 100 raycasts | 10ms | 0.1ms |
| 1000 raycasts | 100ms | 1ms |
| 10000 objects | Very slow | Fast |

## Hints & Guidance

### Spatial Acceleration
Without spatial structure, raycast is O(n) where n is object count. Use octree or BVH:

```rust
fn raycast_with_octree(&self, ray: &Ray) -> Option<RayHit> {
    let mut candidates = Vec::new();
    self.octree.ray_query(ray, &mut candidates);
    
    let mut closest_hit = None;
    let mut closest_dist = ray.max_distance;
    
    for entity in candidates {
        if let Some(hit) = self.test_entity(entity, ray) {
            if hit.distance < closest_dist {
                closest_dist = hit.distance;
                closest_hit = Some(hit);
            }
        }
    }
    
    closest_hit
}
```

### Normal Calculation
For sphere:
```rust
let normal = (hit_point - sphere.center).normalize();
```

For triangle:
```rust
let normal = edge1.cross(edge2).normalize();
```

For AABB:
```rust
// Find which face was hit based on hit point
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use glam::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_distance: f32,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            max_distance: f32::MAX,
        }
    }
    
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

// AABB intersection
pub fn ray_aabb_intersection(ray: &Ray, min: Vec3, max: Vec3) -> Option<f32> {
    let inv_dir = Vec3::ONE / ray.direction;
    
    let t1 = (min - ray.origin) * inv_dir;
    let t2 = (max - ray.origin) * inv_dir;
    
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);
    
    let t_near = tmin.x.max(tmin.y).max(tmin.z);
    let t_far = tmax.x.min(tmax.y).min(tmax.z);
    
    if t_near > t_far || t_far < 0.0 || t_near > ray.max_distance {
        None
    } else {
        Some(t_near.max(0.0))
    }
}

// Sphere intersection
pub fn ray_sphere_intersection(
    ray: &Ray,
    center: Vec3,
    radius: f32,
) -> Option<RayHit> {
    let oc = ray.origin - center;
    
    let a = ray.direction.dot(ray.direction);
    let half_b = oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    
    let discriminant = half_b * half_b - a * c;
    
    if discriminant < 0.0 {
        return None;
    }
    
    let sqrt_d = discriminant.sqrt();
    let mut t = (-half_b - sqrt_d) / a;
    
    if t < 0.0 {
        t = (-half_b + sqrt_d) / a;
        if t < 0.0 {
            return None;
        }
    }
    
    if t > ray.max_distance {
        return None;
    }
    
    let point = ray.at(t);
    let normal = (point - center).normalize();
    
    Some(RayHit {
        point,
        normal,
        distance: t,
    })
}

// Triangle intersection (Möller-Trumbore algorithm)
pub fn ray_triangle_intersection(
    ray: &Ray,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<RayHit> {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    
    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);
    
    if a.abs() < 1e-8 {
        return None;
    }
    
    let f = 1.0 / a;
    let s = ray.origin - v0;
    let u = f * s.dot(h);
    
    if u < 0.0 || u > 1.0 {
        return None;
    }
    
    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);
    
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    
    let t = f * edge2.dot(q);
    
    if t < 1e-8 || t > ray.max_distance {
        return None;
    }
    
    let point = ray.at(t);
    let normal = edge1.cross(edge2).normalize();
    
    Some(RayHit {
        point,
        normal,
        distance: t,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ray_sphere() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -5.0), Vec3::Z);
        let hit = ray_sphere_intersection(&ray, Vec3::ZERO, 1.0);
        
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert!((hit.distance - 4.0).abs() < 0.001);
    }
}
```

</details>

## Related Resources

- [Ray Tracing in One Weekend](https://raytracing.github.io/)
- [Scratchapixel - Ray-Triangle Intersection](https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/moller-trumbore-ray-triangle-intersection)
- [Praxis Physics Documentation](../../reference/crates.md#praxis_physics)

## Next Steps

- Integrate with physics engine (Exercise 34)
- Use for editor picking (Exercise 56)
- Implement raycasting against mesh colliders
