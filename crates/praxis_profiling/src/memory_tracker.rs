//! Memory allocation tracking and leak detection.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Information about a memory allocation.
#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    /// Size in bytes
    pub size: usize,
    /// Allocation timestamp
    pub timestamp: Instant,
    /// Allocation location (file:line)
    pub location: String,
    /// Allocation type/category
    pub category: String,
    /// Call stack (if available)
    pub call_stack: Option<Vec<String>>,
}

/// Statistics about memory usage.
#[derive(Debug, Clone, Default)]
pub struct MemoryStatistics {
    /// Total allocated bytes
    pub total_allocated: usize,
    /// Total deallocated bytes
    pub total_deallocated: usize,
    /// Current allocated bytes
    pub current_allocated: usize,
    /// Peak allocated bytes
    pub peak_allocated: usize,
    /// Number of active allocations
    pub allocation_count: usize,
    /// Allocations by category
    pub allocations_by_category: HashMap<String, usize>,
    /// Bytes by category
    pub bytes_by_category: HashMap<String, usize>,
}

/// Tracks memory allocations and detects leaks.
pub struct AllocationTracker {
    /// Active allocations
    allocations: Arc<Mutex<HashMap<usize, MemoryAllocation>>>,
    /// Statistics
    stats: Arc<Mutex<MemoryStatistics>>,
    /// Next allocation ID
    next_id: Arc<Mutex<usize>>,
    /// Whether tracking is enabled
    enabled: bool,
}

impl AllocationTracker {
    /// Creates a new allocation tracker.
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(MemoryStatistics::default())),
            next_id: Arc::new(Mutex::new(0)),
            enabled: true,
        }
    }

    /// Enables or disables tracking.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Records a memory allocation.
    ///
    /// Returns an allocation ID that should be used for deallocation.
    pub fn track_allocation(
        &self,
        size: usize,
        location: String,
        category: String,
    ) -> usize {
        if !self.enabled {
            return 0;
        }

        let mut next_id = self.next_id.lock();
        let id = *next_id;
        *next_id += 1;
        drop(next_id);

        let allocation = MemoryAllocation {
            size,
            timestamp: Instant::now(),
            location,
            category: category.clone(),
            call_stack: None, // Could be populated with backtrace in debug builds
        };

        let mut allocations = self.allocations.lock();
        allocations.insert(id, allocation);

        let mut stats = self.stats.lock();
        stats.total_allocated += size;
        stats.current_allocated += size;
        stats.allocation_count = allocations.len();

        if stats.current_allocated > stats.peak_allocated {
            stats.peak_allocated = stats.current_allocated;
        }

        *stats.allocations_by_category.entry(category.clone()).or_insert(0) += 1;
        *stats.bytes_by_category.entry(category).or_insert(0) += size;

        id
    }

    /// Records a memory deallocation.
    pub fn track_deallocation(&self, id: usize) {
        if !self.enabled || id == 0 {
            return;
        }

        let mut allocations = self.allocations.lock();
        if let Some(allocation) = allocations.remove(&id) {
            let mut stats = self.stats.lock();
            stats.total_deallocated += allocation.size;
            stats.current_allocated = stats.current_allocated.saturating_sub(allocation.size);
            stats.allocation_count = allocations.len();

            if let Some(count) = stats.allocations_by_category.get_mut(&allocation.category) {
                *count = count.saturating_sub(1);
            }
            if let Some(bytes) = stats.bytes_by_category.get_mut(&allocation.category) {
                *bytes = bytes.saturating_sub(allocation.size);
            }
        }
    }

    /// Returns current memory statistics.
    pub fn statistics(&self) -> MemoryStatistics {
        self.stats.lock().clone()
    }

    /// Returns all active allocations.
    pub fn active_allocations(&self) -> Vec<(usize, MemoryAllocation)> {
        let allocations = self.allocations.lock();
        allocations.iter().map(|(k, v)| (*k, v.clone())).collect()
    }

    /// Resets statistics (but keeps tracking allocations).
    pub fn reset_statistics(&self) {
        let mut stats = self.stats.lock();
        let current = stats.current_allocated;
        *stats = MemoryStatistics {
            current_allocated: current,
            peak_allocated: current,
            allocation_count: self.allocations.lock().len(),
            ..Default::default()
        };
    }
}

impl Default for AllocationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Detects memory leaks by tracking allocations over time.
pub struct LeakDetector {
    /// Allocation tracker
    tracker: Arc<AllocationTracker>,
    /// Snapshot of allocations at checkpoint
    checkpoint: Mutex<HashMap<usize, MemoryAllocation>>,
    /// Checkpoint timestamp
    checkpoint_time: Mutex<Option<Instant>>,
}

impl LeakDetector {
    /// Creates a new leak detector.
    pub fn new(tracker: Arc<AllocationTracker>) -> Self {
        Self {
            tracker,
            checkpoint: Mutex::new(HashMap::new()),
            checkpoint_time: Mutex::new(None),
        }
    }

    /// Creates a checkpoint of current allocations.
    pub fn checkpoint(&self) {
        let allocations = self.tracker.allocations.lock();
        let mut checkpoint = self.checkpoint.lock();
        *checkpoint = allocations.clone();
        *self.checkpoint_time.lock() = Some(Instant::now());
    }

    /// Detects potential leaks by comparing current allocations to checkpoint.
    ///
    /// Returns allocations that exist now but didn't exist at checkpoint,
    /// or existed at checkpoint and still exist now (potential leaks).
    pub fn detect_leaks(&self, min_age: Duration) -> Vec<(usize, MemoryAllocation)> {
        let checkpoint_time_guard = self.checkpoint_time.lock();
        let Some(_checkpoint_time) = *checkpoint_time_guard else {
            return Vec::new();
        };
        drop(checkpoint_time_guard);

        let now = Instant::now();
        let allocations = self.tracker.allocations.lock();
        let checkpoint = self.checkpoint.lock();

        allocations
            .iter()
            .filter(|(id, alloc)| {
                // Check if allocation is old enough
                let age = now.duration_since(alloc.timestamp);
                if age < min_age {
                    return false;
                }

                // Check if it existed at checkpoint (potential leak)
                checkpoint.contains_key(id)
            })
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }

    /// Returns the number of new allocations since checkpoint.
    pub fn new_allocations_count(&self) -> usize {
        let allocations = self.tracker.allocations.lock();
        let checkpoint = self.checkpoint.lock();

        allocations
            .keys()
            .filter(|id| !checkpoint.contains_key(id))
            .count()
    }

    /// Returns total bytes of new allocations since checkpoint.
    pub fn new_allocations_bytes(&self) -> usize {
        let allocations = self.tracker.allocations.lock();
        let checkpoint = self.checkpoint.lock();

        allocations
            .iter()
            .filter(|(id, _)| !checkpoint.contains_key(id))
            .map(|(_, alloc)| alloc.size)
            .sum()
    }
}

/// RAII guard for tracking an allocation.
pub struct AllocationGuard {
    tracker: Arc<AllocationTracker>,
    id: usize,
}

impl AllocationGuard {
    /// Creates a new allocation guard.
    #[allow(dead_code)]
    pub fn new(tracker: Arc<AllocationTracker>, size: usize, location: String, category: String) -> Self {
        let id = tracker.track_allocation(size, location, category);
        Self { tracker, id }
    }
}

impl Drop for AllocationGuard {
    fn drop(&mut self) {
        self.tracker.track_deallocation(self.id);
    }
}

/// Macro for tracking allocations with automatic location.
#[macro_export]
macro_rules! track_allocation {
    ($tracker:expr, $size:expr, $category:expr) => {{
        let location = format!("{}:{}", file!(), line!());
        $tracker.track_allocation($size, location, $category.to_string())
    }};
}
