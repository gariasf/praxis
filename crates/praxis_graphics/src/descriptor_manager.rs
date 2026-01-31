//! Descriptor set management and lifetime tracking.
//!
//! This module provides utilities for managing descriptor sets with automatic
//! lifetime tracking, pooling, and efficient reuse patterns.
//!
//! # Overview
//!
//! Descriptor sets in Vulkan bind resources (buffers, images) to shaders. Managing
//! their lifecycle correctly is critical for performance and correctness. This module
//! provides abstractions that handle:
//!
//! - **Lifetime Tracking**: Ensures descriptor sets remain alive while GPU uses them
//! - **Pooling**: Reuses descriptor sets across frames to reduce allocations
//! - **LRU Eviction**: Automatically removes unused descriptor sets to bound memory
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::descriptor_manager::{DescriptorSetCache, DescriptorSetKey};
//! # use std::sync::Arc;
//! # use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
//! # use vulkano::descriptor_set::layout::DescriptorSetLayout;
//! # use vulkano::device::Device;
//! # fn example(
//! #     device: Arc<Device>,
//! #     layout: Arc<DescriptorSetLayout>,
//! # ) -> praxis_utils::Result<()> {
//! let allocator = Arc::new(StandardDescriptorSetAllocator::new(
//!     device.clone(),
//!     Default::default(),
//! ));
//!
//! let mut cache = DescriptorSetCache::new(allocator, layout);
//!
//! // Create a descriptor set (or reuse from cache)
//! let key = DescriptorSetKey::from_hash(123456);
//! // let descriptor_set = cache.get_or_create(key, || {
//! //     // Create descriptor set...
//! // })?;
//!
//! // Advance frame and evict old sets
//! cache.next_frame();
//! # Ok(())
//! # }
//! ```

use praxis_utils::{debug, trace, Result};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use vulkano::descriptor_set::{
    allocator::DescriptorSetAllocator, layout::DescriptorSetLayout, DescriptorSet,
};

/// Key for identifying descriptor sets in the cache.
///
/// Descriptor sets are keyed by their resource bindings (textures, buffers, etc.).
/// This key is typically computed from a hash of the binding configuration.
///
/// # Example
///
/// ```rust
/// use praxis_graphics::descriptor_manager::DescriptorSetKey;
/// use std::collections::hash_map::DefaultHasher;
/// use std::hash::{Hash, Hasher};
///
/// // Create key from configuration
/// let mut hasher = DefaultHasher::new();
/// "texture_name".hash(&mut hasher);
/// 0.5f32.to_bits().hash(&mut hasher); // metallic
/// 0.8f32.to_bits().hash(&mut hasher); // roughness
///
/// let key = DescriptorSetKey::from_hash(hasher.finish());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DescriptorSetKey {
    hash: u64,
}

impl DescriptorSetKey {
    /// Creates a new descriptor set key from a hash value.
    pub fn from_hash(hash: u64) -> Self {
        Self { hash }
    }

    /// Creates a new descriptor set key by hashing the given value.
    pub fn from_hashable<T: Hash>(value: &T) -> Self {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        Self {
            hash: hasher.finish(),
        }
    }

    /// Gets the hash value of this key.
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

/// Cached descriptor set with lifetime tracking.
struct CachedDescriptorSet {
    /// The descriptor set
    descriptor_set: Arc<DescriptorSet>,
    /// Frame number when this descriptor set was last used
    last_used_frame: u64,
}

/// Cache for descriptor sets with LRU eviction.
///
/// This cache stores descriptor sets indexed by a key (typically a hash of their
/// configuration) and automatically evicts descriptor sets that haven't been used
/// recently.
///
/// # Eviction Policy
///
/// The cache uses an LRU (Least Recently Used) eviction policy:
/// - Tracks the last frame each descriptor set was used
/// - Evicts sets unused for `eviction_threshold` frames
/// - Runs eviction every 60 frames to minimize overhead
///
/// # Example
///
/// ```rust,no_run
/// use praxis_graphics::descriptor_manager::{DescriptorSetCache, DescriptorSetKey};
/// # use std::sync::Arc;
/// # use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
/// # use vulkano::descriptor_set::layout::DescriptorSetLayout;
/// # use vulkano::device::Device;
/// # fn example(
/// #     device: Arc<Device>,
/// #     layout: Arc<DescriptorSetLayout>,
/// # ) -> praxis_utils::Result<()> {
/// let allocator = Arc::new(StandardDescriptorSetAllocator::new(
///     device,
///     Default::default(),
/// ));
///
/// let mut cache = DescriptorSetCache::new(allocator, layout);
///
/// // Use descriptor sets...
/// // They are automatically cached and reused
///
/// // Clean up old sets
/// cache.next_frame();
/// # Ok(())
/// # }
/// ```
pub struct DescriptorSetCache {
    /// Cached descriptor sets indexed by key
    cache: HashMap<DescriptorSetKey, CachedDescriptorSet>,
    /// Descriptor set allocator
    allocator: Arc<dyn DescriptorSetAllocator>,
    /// Descriptor set layout
    layout: Arc<DescriptorSetLayout>,
    /// Current frame number for LRU tracking
    current_frame: u64,
    /// Number of frames a descriptor set can remain unused before eviction
    eviction_threshold: u64,
}

impl DescriptorSetCache {
    /// Creates a new descriptor set cache.
    ///
    /// # Arguments
    ///
    /// * `allocator` - Descriptor set allocator for creating new sets
    /// * `layout` - Descriptor set layout for all sets in this cache
    pub fn new(
        allocator: Arc<dyn DescriptorSetAllocator>,
        layout: Arc<DescriptorSetLayout>,
    ) -> Self {
        Self {
            cache: HashMap::new(),
            allocator,
            layout,
            current_frame: 0,
            eviction_threshold: 60,
        }
    }

    /// Gets a descriptor set from the cache or creates a new one.
    ///
    /// If a descriptor set with the given key exists in the cache, it is returned
    /// and its last used frame is updated. Otherwise, the provided closure is called
    /// to create a new descriptor set, which is then cached.
    ///
    /// # Arguments
    ///
    /// * `key` - Key identifying this descriptor set configuration
    /// * `create` - Closure that creates the descriptor set if not cached
    ///
    /// # Returns
    ///
    /// The cached or newly created descriptor set.
    pub fn get_or_create<F>(
        &mut self,
        key: DescriptorSetKey,
        create: F,
    ) -> Result<Arc<DescriptorSet>>
    where
        F: FnOnce() -> Result<Arc<DescriptorSet>>,
    {
        if let Some(cached) = self.cache.get_mut(&key) {
            trace!("Reusing cached descriptor set (key: {})", key.hash);
            cached.last_used_frame = self.current_frame;
            return Ok(cached.descriptor_set.clone());
        }

        trace!("Creating new descriptor set (key: {})", key.hash);
        let descriptor_set = create()?;

        let cached = CachedDescriptorSet {
            descriptor_set: descriptor_set.clone(),
            last_used_frame: self.current_frame,
        };

        self.cache.insert(key, cached);
        Ok(descriptor_set)
    }

    /// Advances to the next frame and evicts unused descriptor sets.
    ///
    /// This should be called at the start of each frame. It increments the frame
    /// counter and periodically evicts descriptor sets that haven't been used
    /// within the eviction threshold.
    pub fn next_frame(&mut self) {
        self.current_frame += 1;

        // Only run eviction check occasionally to reduce overhead
        if self.current_frame % 60 != 0 {
            return;
        }

        let eviction_cutoff = self.current_frame.saturating_sub(self.eviction_threshold);
        let count_before = self.cache.len();

        self.cache.retain(|key, cached| {
            let should_keep = cached.last_used_frame >= eviction_cutoff;
            if !should_keep {
                trace!(
                    "Evicting descriptor set (key: {}, last used: frame {}, current: frame {})",
                    key.hash,
                    cached.last_used_frame,
                    self.current_frame
                );
            }
            should_keep
        });

        let evicted = count_before - self.cache.len();
        if evicted > 0 {
            debug!(
                "Evicted {} descriptor sets (unused for {} frames)",
                evicted, self.eviction_threshold
            );
        }
    }

    /// Clears all cached descriptor sets.
    ///
    /// This should be called when descriptor set configurations change and the
    /// cache needs to be invalidated.
    pub fn clear(&mut self) {
        debug!("Clearing descriptor set cache ({} sets)", self.cache.len());
        self.cache.clear();
        self.current_frame = 0;
    }

    /// Gets the number of cached descriptor sets.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Sets the eviction threshold (number of frames before eviction).
    pub fn set_eviction_threshold(&mut self, threshold: u64) {
        self.eviction_threshold = threshold;
    }

    /// Gets the current eviction threshold.
    pub fn eviction_threshold(&self) -> u64 {
        self.eviction_threshold
    }

    /// Gets the current frame number.
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Gets the descriptor set layout.
    pub fn layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.layout
    }

    /// Gets the descriptor set allocator.
    pub fn allocator(&self) -> &Arc<dyn DescriptorSetAllocator> {
        &self.allocator
    }
}

/// Lifetime tracker for GPU resources.
///
/// This utility tracks when GPU resources (buffers, images, descriptor sets) are
/// last used and ensures they remain alive while the GPU might still be accessing them.
///
/// # Frame-Based Lifetime
///
/// Resources are tracked by frame number:
/// - When a resource is used, its "last used frame" is updated
/// - Resources are kept alive for N frames after last use (typically 2-3 for in-flight frames)
/// - After the grace period, resources can be safely freed
///
/// # Example
///
/// ```rust
/// use praxis_graphics::descriptor_manager::ResourceLifetimeTracker;
/// use std::sync::Arc;
///
/// let mut tracker = ResourceLifetimeTracker::new(3); // 3 frames in flight
///
/// // Track a resource
/// let resource_id = 1;
/// tracker.mark_used(resource_id);
///
/// // Check if resource can be freed
/// tracker.next_frame();
/// tracker.next_frame();
/// tracker.next_frame();
/// tracker.next_frame(); // 4 frames later
///
/// assert!(tracker.can_free(resource_id));
/// ```
pub struct ResourceLifetimeTracker {
    /// Map of resource ID to last used frame
    last_used: HashMap<u64, u64>,
    /// Current frame number
    current_frame: u64,
    /// Number of frames to keep resources alive after last use
    grace_period: u64,
}

impl ResourceLifetimeTracker {
    /// Creates a new resource lifetime tracker.
    ///
    /// # Arguments
    ///
    /// * `grace_period` - Number of frames to keep resources alive after last use
    pub fn new(grace_period: u64) -> Self {
        Self {
            last_used: HashMap::new(),
            current_frame: 0,
            grace_period,
        }
    }

    /// Marks a resource as used in the current frame.
    pub fn mark_used(&mut self, resource_id: u64) {
        self.last_used.insert(resource_id, self.current_frame);
    }

    /// Checks if a resource can be safely freed.
    ///
    /// A resource can be freed if it hasn't been used within the grace period.
    pub fn can_free(&self, resource_id: u64) -> bool {
        if let Some(&last_used_frame) = self.last_used.get(&resource_id) {
            let frames_since_use = self.current_frame.saturating_sub(last_used_frame);
            frames_since_use > self.grace_period
        } else {
            // Resource not tracked, can be freed
            true
        }
    }

    /// Removes a resource from tracking.
    pub fn remove(&mut self, resource_id: u64) {
        self.last_used.remove(&resource_id);
    }

    /// Advances to the next frame.
    pub fn next_frame(&mut self) {
        self.current_frame += 1;
    }

    /// Gets the current frame number.
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Clears all tracked resources.
    pub fn clear(&mut self) {
        self.last_used.clear();
        self.current_frame = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_set_key_from_hash() {
        let key1 = DescriptorSetKey::from_hash(12345);
        let key2 = DescriptorSetKey::from_hash(12345);
        let key3 = DescriptorSetKey::from_hash(67890);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_descriptor_set_key_from_hashable() {
        let key1 = DescriptorSetKey::from_hashable(&"test");
        let key2 = DescriptorSetKey::from_hashable(&"test");
        let key3 = DescriptorSetKey::from_hashable(&"different");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_resource_lifetime_tracker() {
        let mut tracker = ResourceLifetimeTracker::new(2);

        // Mark resource as used
        tracker.mark_used(1);
        assert!(!tracker.can_free(1)); // Frame 0

        // Advance frames
        tracker.next_frame(); // Frame 1
        assert!(!tracker.can_free(1));

        tracker.next_frame(); // Frame 2
        assert!(!tracker.can_free(1));

        tracker.next_frame(); // Frame 3 (2 frames after grace period)
        assert!(tracker.can_free(1));
    }

    #[test]
    fn test_resource_lifetime_tracker_remove() {
        let mut tracker = ResourceLifetimeTracker::new(2);

        tracker.mark_used(1);
        assert!(!tracker.can_free(1));

        tracker.remove(1);
        assert!(tracker.can_free(1)); // Can free after removal
    }

    #[test]
    fn test_resource_lifetime_tracker_clear() {
        let mut tracker = ResourceLifetimeTracker::new(2);

        tracker.mark_used(1);
        tracker.mark_used(2);
        tracker.next_frame();

        tracker.clear();
        assert_eq!(tracker.current_frame(), 0);
        assert!(tracker.can_free(1));
        assert!(tracker.can_free(2));
    }
}
