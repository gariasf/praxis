# Exercise 45: LRU Cache Implementation

**Difficulty**: 🟡 Intermediate | **Estimated Time**: 2-3h | **Subsystem**: Assets

## Overview

Implement a Least Recently Used (LRU) cache for managing limited resources like textures or audio buffers. LRU caching is essential for keeping frequently-used assets in memory while evicting old ones.

## Learning Objectives

- Understand cache eviction policies
- Implement doubly-linked list + hash map combination
- Learn O(1) cache operations
- Handle capacity constraints

## Requirements

### Functional Requirements

1. **Core Operations**
   - `get(key)`: Retrieve value, mark as recently used
   - `put(key, value)`: Insert value, evict LRU if at capacity
   - `remove(key)`: Remove entry

2. **LRU Behavior**
   - Track access order
   - Evict least recently used when full
   - Update order on get and put

3. **Capacity Management**
   - Fixed maximum capacity
   - Automatic eviction when full
   - Optional eviction callback

### Non-Functional Requirements

- **Performance**: All operations in O(1) time
- **Memory**: O(capacity) space
- **Thread Safety**: Optional concurrent access support

## API Design

```rust
pub struct LruCache<K, V> {
    capacity: usize,
    // Internal structures
}

impl<K, V> LruCache<K, V> 
where
    K: Hash + Eq + Clone,
{
    pub fn new(capacity: usize) -> Self;
    
    pub fn get(&mut self, key: &K) -> Option<&V>;
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    
    pub fn put(&mut self, key: K, value: V) -> Option<V>;
    pub fn remove(&mut self, key: &K) -> Option<V>;
    
    pub fn clear(&mut self);
    pub fn len(&self) -> usize;
    pub fn capacity(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

## Validation Criteria

### Correctness
- [ ] get() moves item to front
- [ ] put() evicts LRU item when full
- [ ] Maintains correct access order
- [ ] Handles capacity 1 correctly

### Performance
- [ ] All operations O(1)
- [ ] 1M operations in < 100ms
- [ ] No memory leaks

## Test Cases

```rust
#[test]
fn test_basic_operations() {
    let mut cache = LruCache::new(2);
    
    cache.put("a", 1);
    cache.put("b", 2);
    
    assert_eq!(cache.get(&"a"), Some(&1));
    assert_eq!(cache.get(&"b"), Some(&2));
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_eviction() {
    let mut cache = LruCache::new(2);
    
    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3); // Should evict "a"
    
    assert_eq!(cache.get(&"a"), None);
    assert_eq!(cache.get(&"b"), Some(&2));
    assert_eq!(cache.get(&"c"), Some(&3));
}

#[test]
fn test_get_updates_recency() {
    let mut cache = LruCache::new(2);
    
    cache.put("a", 1);
    cache.put("b", 2);
    cache.get(&"a"); // Make "a" most recent
    cache.put("c", 3); // Should evict "b", not "a"
    
    assert_eq!(cache.get(&"a"), Some(&1));
    assert_eq!(cache.get(&"b"), None);
    assert_eq!(cache.get(&"c"), Some(&3));
}

#[test]
fn test_put_updates_value() {
    let mut cache = LruCache::new(2);
    
    cache.put("a", 1);
    let old = cache.put("a", 10);
    
    assert_eq!(old, Some(1));
    assert_eq!(cache.get(&"a"), Some(&10));
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_capacity_one() {
    let mut cache = LruCache::new(1);
    
    cache.put("a", 1);
    assert_eq!(cache.get(&"a"), Some(&1));
    
    cache.put("b", 2);
    assert_eq!(cache.get(&"a"), None);
    assert_eq!(cache.get(&"b"), Some(&2));
}
```

## Performance Targets

| Operation | Target |
|-----------|--------|
| get() | < 10ns |
| put() | < 50ns |
| 1M mixed operations | < 100ms |

## Hints & Guidance

### Data Structure
Combine two structures:
- **HashMap**: For O(1) key lookup
- **Doubly-Linked List**: For O(1) reordering

```
HashMap: key -> ListNode
List: [MRU] <-> Node <-> Node <-> [LRU]
```

### Implementation Approaches

**Approach 1: Custom Linked List**
- Implement your own doubly-linked list
- Store raw pointers or indices
- Most control, but requires unsafe code

**Approach 2: Vec + HashMap**
- Vec as circular buffer
- HashMap stores indices into Vec
- Safe but needs index remapping

**Approach 3: Library Crate**
- Use `linked-hash-map` or similar
- Less learning but functional

### Move-to-Front Operation
```rust
// Pseudocode
fn move_to_front(key) {
    node = map[key]
    remove_from_list(node)
    insert_at_front(node)
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, Box<Node<K, V>>>,
    head: *mut Node<K, V>,
    tail: *mut Node<K, V>,
}

struct Node<K, V> {
    key: K,
    value: V,
    prev: *mut Node<K, V>,
    next: *mut Node<K, V>,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
{
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            capacity,
            map: HashMap::new(),
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if !self.map.contains_key(key) {
            return None;
        }
        
        // Move to front
        let node_ptr = self.map[key].as_ref() as *const Node<K, V> as *mut Node<K, V>;
        unsafe {
            self.remove_from_list(node_ptr);
            self.insert_at_front(node_ptr);
            Some(&(*node_ptr).value)
        }
    }
    
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if !self.map.contains_key(key) {
            return None;
        }
        
        let node_ptr = self.map[key].as_mut() as *mut Node<K, V>;
        unsafe {
            self.remove_from_list(node_ptr);
            self.insert_at_front(node_ptr);
            Some(&mut (*node_ptr).value)
        }
    }
    
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        // If key exists, update value and move to front
        if let Some(node_box) = self.map.get_mut(&key) {
            let node_ptr = node_box.as_mut() as *mut Node<K, V>;
            unsafe {
                let old_value = std::mem::replace(&mut (*node_ptr).value, value);
                self.remove_from_list(node_ptr);
                self.insert_at_front(node_ptr);
                return Some(old_value);
            }
        }
        
        // Evict if at capacity
        if self.map.len() >= self.capacity {
            self.evict_lru();
        }
        
        // Insert new node
        let mut node = Box::new(Node {
            key: key.clone(),
            value,
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        });
        
        let node_ptr = node.as_mut() as *mut Node<K, V>;
        self.map.insert(key, node);
        
        unsafe {
            self.insert_at_front(node_ptr);
        }
        
        None
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(mut node) = self.map.remove(key) {
            let node_ptr = node.as_mut() as *mut Node<K, V>;
            unsafe {
                self.remove_from_list(node_ptr);
            }
            Some(node.value)
        } else {
            None
        }
    }
    
    pub fn clear(&mut self) {
        self.map.clear();
        self.head = std::ptr::null_mut();
        self.tail = std::ptr::null_mut();
    }
    
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    
    unsafe fn remove_from_list(&mut self, node: *mut Node<K, V>) {
        if !(*node).prev.is_null() {
            (*(*node).prev).next = (*node).next;
        } else {
            self.head = (*node).next;
        }
        
        if !(*node).next.is_null() {
            (*(*node).next).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }
        
        (*node).prev = std::ptr::null_mut();
        (*node).next = std::ptr::null_mut();
    }
    
    unsafe fn insert_at_front(&mut self, node: *mut Node<K, V>) {
        (*node).next = self.head;
        (*node).prev = std::ptr::null_mut();
        
        if !self.head.is_null() {
            (*self.head).prev = node;
        }
        
        self.head = node;
        
        if self.tail.is_null() {
            self.tail = node;
        }
    }
    
    fn evict_lru(&mut self) {
        if self.tail.is_null() {
            return;
        }
        
        unsafe {
            let key = (*self.tail).key.clone();
            self.remove(&key);
        }
    }
}

// Safe alternative using indices
pub struct SafeLruCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    list: Vec<(K, V)>,
    order: Vec<usize>,
}

impl<K, V> SafeLruCache<K, V>
where
    K: Hash + Eq + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            list: Vec::new(),
            order: Vec::new(),
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.map.get(key).map(|&idx| {
            self.touch(idx);
            &self.list[idx].1
        })
    }
    
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        if let Some(&idx) = self.map.get(&key) {
            let old = std::mem::replace(&mut self.list[idx].1, value);
            self.touch(idx);
            return Some(old);
        }
        
        if self.list.len() >= self.capacity {
            let lru_idx = self.order.remove(0);
            let (old_key, _) = &self.list[lru_idx];
            self.map.remove(old_key);
            self.list[lru_idx] = (key.clone(), value);
            self.map.insert(key, lru_idx);
            self.order.push(lru_idx);
        } else {
            let idx = self.list.len();
            self.list.push((key.clone(), value));
            self.map.insert(key, idx);
            self.order.push(idx);
        }
        
        None
    }
    
    fn touch(&mut self, idx: usize) {
        if let Some(pos) = self.order.iter().position(|&i| i == idx) {
            self.order.remove(pos);
            self.order.push(idx);
        }
    }
}
```

</details>

## Related Resources

- [LRU Cache on Wikipedia](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_recently_used_(LRU))
- [Praxis Procedural Cache](../../reference/crates.md#praxis_procedural)

## Next Steps

- Add size-based eviction (not just count)
- Implement concurrent LRU cache
- Use for texture streaming (Exercise 47)
