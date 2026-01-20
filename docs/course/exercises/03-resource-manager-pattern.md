# Exercise 03: Resource Manager Pattern

**Difficulty**: 🟡 Intermediate | **Estimated Time**: 3-4h | **Subsystem**: Core

## Overview

Implement a generic resource manager that handles loading, caching, and lifetime management of game assets. This pattern is fundamental to efficient asset management in game engines.

## Learning Objectives

- Understand resource handle patterns
- Implement reference counting for resource lifetime
- Design type-safe generic resource management
- Handle resource dependencies

## Requirements

### Functional Requirements

1. **Resource Loading**
   - Load resources from paths
   - Cache loaded resources (don't reload duplicates)
   - Return handles to resources

2. **Handle System**
   - Strong handles (keep resource alive)
   - Weak handles (don't prevent unloading)
   - Handle validation (detect dangling handles)

3. **Resource Lifecycle**
   - Automatic cleanup when last handle drops
   - Manual unload support
   - Clear all resources

### Non-Functional Requirements

- **Performance**: Handle lookup in O(1)
- **Memory**: Track total memory usage
- **Thread Safety**: Support concurrent access (optional advanced)

## API Design

```rust
pub struct ResourceManager<T> {
    resources: HashMap<ResourceId, Arc<T>>,
    path_to_id: HashMap<PathBuf, ResourceId>,
}

pub struct Handle<T> {
    id: ResourceId,
    resource: Arc<T>,
}

impl<T> ResourceManager<T> 
where
    T: Resource,
{
    pub fn new() -> Self;
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<Handle<T>>;
    pub fn get(&self, id: ResourceId) -> Option<Handle<T>>;
    pub fn unload(&mut self, id: ResourceId);
    pub fn clear(&mut self);
    pub fn memory_usage(&self) -> usize;
}

pub trait Resource: Sized {
    fn load_from_path(path: &Path) -> Result<Self>;
    fn memory_size(&self) -> usize;
}
```

## Validation Criteria

### Correctness
- [ ] Resources loaded only once per path
- [ ] Handles keep resources alive
- [ ] Resources freed when last handle drops
- [ ] Handle validation detects freed resources

### Performance
- [ ] O(1) resource lookup by ID
- [ ] O(1) path-to-resource lookup
- [ ] No unnecessary allocations

### Code Quality
- [ ] Generic over resource types
- [ ] Clear separation of concerns
- [ ] Comprehensive error handling

## Test Cases

```rust
#[test]
fn test_load_and_retrieve() {
    let mut manager = ResourceManager::<TestResource>::new();
    
    let handle1 = manager.load("test.txt").unwrap();
    let handle2 = manager.load("test.txt").unwrap();
    
    // Same resource should be returned
    assert_eq!(handle1.id, handle2.id);
    assert_eq!(Arc::strong_count(&handle1.resource), 3); // manager + 2 handles
}

#[test]
fn test_resource_cleanup() {
    let mut manager = ResourceManager::<TestResource>::new();
    
    {
        let _handle = manager.load("test.txt").unwrap();
        assert_eq!(manager.resources.len(), 1);
    } // handle dropped
    
    // Resource should still exist in manager
    assert_eq!(manager.resources.len(), 1);
    
    manager.collect_unused();
    assert_eq!(manager.resources.len(), 0);
}

#[test]
fn test_memory_tracking() {
    let mut manager = ResourceManager::<TestResource>::new();
    
    let initial = manager.memory_usage();
    let _handle = manager.load("test.txt").unwrap();
    let after = manager.memory_usage();
    
    assert!(after > initial);
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type ResourceId = u64;

pub struct ResourceManager<T: Resource> {
    resources: HashMap<ResourceId, Arc<T>>,
    path_to_id: HashMap<PathBuf, ResourceId>,
    next_id: ResourceId,
}

impl<T: Resource> ResourceManager<T> {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            path_to_id: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<Handle<T>, String> {
        let path = path.as_ref().to_path_buf();
        
        // Check if already loaded
        if let Some(&id) = self.path_to_id.get(&path) {
            if let Some(resource) = self.resources.get(&id) {
                return Ok(Handle {
                    id,
                    resource: Arc::clone(resource),
                });
            }
        }
        
        // Load new resource
        let resource = T::load_from_path(&path)?;
        let id = self.next_id;
        self.next_id += 1;
        
        let arc = Arc::new(resource);
        self.resources.insert(id, Arc::clone(&arc));
        self.path_to_id.insert(path, id);
        
        Ok(Handle { id, resource: arc })
    }
    
    pub fn get(&self, id: ResourceId) -> Option<Handle<T>> {
        self.resources.get(&id).map(|resource| Handle {
            id,
            resource: Arc::clone(resource),
        })
    }
    
    pub fn unload(&mut self, id: ResourceId) {
        if let Some(arc) = self.resources.remove(&id) {
            // Find and remove path mapping
            self.path_to_id.retain(|_, &mut v| v != id);
        }
    }
    
    pub fn collect_unused(&mut self) {
        // Remove resources with only 1 strong reference (manager itself)
        self.resources.retain(|_, arc| Arc::strong_count(arc) > 1);
        
        // Clean up path mappings
        self.path_to_id.retain(|_, id| self.resources.contains_key(id));
    }
    
    pub fn clear(&mut self) {
        self.resources.clear();
        self.path_to_id.clear();
    }
    
    pub fn memory_usage(&self) -> usize {
        self.resources
            .values()
            .map(|r| r.memory_size())
            .sum()
    }
    
    pub fn resource_count(&self) -> usize {
        self.resources.len()
    }
}

pub struct Handle<T: Resource> {
    pub id: ResourceId,
    resource: Arc<T>,
}

impl<T: Resource> Handle<T> {
    pub fn get(&self) -> &T {
        &self.resource
    }
}

impl<T: Resource> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            resource: Arc::clone(&self.resource),
        }
    }
}

pub trait Resource: Sized {
    fn load_from_path(path: &Path) -> Result<Self, String>;
    fn memory_size(&self) -> usize;
}

// Example implementation
#[derive(Clone)]
pub struct TextResource {
    content: String,
}

impl Resource for TextResource {
    fn load_from_path(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to load {}: {}", path.display(), e))?;
        
        Ok(Self { content })
    }
    
    fn memory_size(&self) -> usize {
        self.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    
    #[test]
    fn test_basic_loading() {
        let mut manager = ResourceManager::<TextResource>::new();
        
        // Create temp file
        let mut file = fs::File::create("test_temp.txt").unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        let handle = manager.load("test_temp.txt").unwrap();
        assert_eq!(handle.get().content, "test content");
        
        fs::remove_file("test_temp.txt").ok();
    }
    
    #[test]
    fn test_caching() {
        let mut manager = ResourceManager::<TextResource>::new();
        
        let mut file = fs::File::create("test_cache.txt").unwrap();
        file.write_all(b"cached").unwrap();
        drop(file);
        
        let handle1 = manager.load("test_cache.txt").unwrap();
        let handle2 = manager.load("test_cache.txt").unwrap();
        
        assert_eq!(handle1.id, handle2.id);
        assert_eq!(manager.resource_count(), 1);
        
        fs::remove_file("test_cache.txt").ok();
    }
}
```

</details>

## Related Resources

- [Praxis Assets Documentation](../../reference/crates.md#praxis_assets)
- [Resource Management Patterns](../../concepts/resource-management.md)

## Next Steps

- Add async loading (Exercise 43)
- Implement hot-reloading (Exercise 07)
- Study `praxis_assets` implementation
