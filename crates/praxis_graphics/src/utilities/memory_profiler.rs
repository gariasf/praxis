//! GPU memory profiling and VRAM tracking system.
//!
//! This module provides comprehensive GPU memory tracking including:
//! - Texture allocation monitoring with dimensions and format
//! - Mesh buffer tracking (vertex and index buffers)
//! - Descriptor set memory overhead
//! - Compute shader resource tracking
//! - Memory breakdown by category
//! - Peak usage tracking
//! - Integration with RenderStats for correlation
//!
//! # Architecture
//!
//! The memory profiler tracks all GPU allocations through:
//! - **`MemoryProfiler`**: Central profiling system with categorized tracking
//! - **`VramAllocation`**: Individual allocation record with metadata
//! - **`MemoryCategory`**: Classification of allocation types
//! - **`MemorySnapshot`**: Point-in-time memory state
//! - **`MemoryHistory`**: Rolling history with trend analysis
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use praxis_graphics::memory_profiler::{MemoryProfiler, MemoryCategory};
//!
//! let mut profiler = MemoryProfiler::new();
//!
//! // Record texture allocation
//! profiler.record_allocation(
//!     "brick_texture",
//!     MemoryCategory::Texture,
//!     1024 * 1024 * 4, // 1024x1024 RGBA8
//!     Some("1024x1024 RGBA8".to_string()),
//! );
//!
//! // Record mesh allocation
//! profiler.record_allocation(
//!     "character_mesh",
//!     MemoryCategory::MeshBuffer,
//!     vertex_buffer_size + index_buffer_size,
//!     Some(format!("{} verts, {} indices", vert_count, index_count)),
//! );
//!
//! // Get current memory state
//! let snapshot = profiler.snapshot();
//! println!("Total VRAM: {} MB", snapshot.total_bytes as f64 / 1_048_576.0);
//! println!("Texture memory: {} MB", snapshot.texture_bytes as f64 / 1_048_576.0);
//! ```

use std::collections::HashMap;
use std::time::Instant;

/// Category of GPU memory allocation.
///
/// Used to classify allocations for breakdown analysis and correlation
/// with rendering optimizations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    /// Texture image data (albedo, normal maps, etc.)
    Texture,
    /// Vertex and index buffer data
    MeshBuffer,
    /// Descriptor set allocations
    DescriptorSet,
    /// Uniform buffer objects
    UniformBuffer,
    /// Compute shader buffers (indirect draw, culling, etc.)
    ComputeBuffer,
    /// Render target images (framebuffers, G-buffer, shadow maps)
    RenderTarget,
    /// Other miscellaneous allocations
    Other,
}

impl MemoryCategory {
    /// Returns a human-readable name for this category.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Texture => "Textures",
            Self::MeshBuffer => "Mesh Buffers",
            Self::DescriptorSet => "Descriptor Sets",
            Self::UniformBuffer => "Uniform Buffers",
            Self::ComputeBuffer => "Compute Buffers",
            Self::RenderTarget => "Render Targets",
            Self::Other => "Other",
        }
    }
}

/// A single GPU memory allocation record.
///
/// Tracks metadata about an allocation including size, category, and
/// when it was created. Used for detailed memory profiling.
#[derive(Debug, Clone)]
pub struct VramAllocation {
    /// Unique identifier for this allocation
    pub id: String,
    /// Category of this allocation
    pub category: MemoryCategory,
    /// Size in bytes
    pub size_bytes: u64,
    /// Optional metadata (e.g., "1024x1024 RGBA8", "10000 vertices")
    pub metadata: Option<String>,
    /// When this allocation was created
    pub created_at: Instant,
}

/// Point-in-time snapshot of GPU memory state.
///
/// Captures current memory usage broken down by category, along with
/// allocation counts and peak usage tracking.
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Frame number when this snapshot was taken
    pub frame_number: u64,
    /// Total VRAM allocated in bytes
    pub total_bytes: u64,
    /// Peak VRAM usage since profiling started
    pub peak_bytes: u64,
    /// Bytes allocated to textures
    pub texture_bytes: u64,
    /// Bytes allocated to mesh buffers
    pub mesh_buffer_bytes: u64,
    /// Bytes allocated to descriptor sets
    pub descriptor_set_bytes: u64,
    /// Bytes allocated to uniform buffers
    pub uniform_buffer_bytes: u64,
    /// Bytes allocated to compute buffers
    pub compute_buffer_bytes: u64,
    /// Bytes allocated to render targets
    pub render_target_bytes: u64,
    /// Bytes allocated to other categories
    pub other_bytes: u64,
    /// Number of active allocations
    pub allocation_count: usize,
    /// Timestamp when snapshot was taken
    pub timestamp: Instant,
}

impl Default for MemorySnapshot {
    fn default() -> Self {
        Self {
            frame_number: 0,
            total_bytes: 0,
            peak_bytes: 0,
            texture_bytes: 0,
            mesh_buffer_bytes: 0,
            descriptor_set_bytes: 0,
            uniform_buffer_bytes: 0,
            compute_buffer_bytes: 0,
            render_target_bytes: 0,
            other_bytes: 0,
            allocation_count: 0,
            timestamp: Instant::now(),
        }
    }
}

impl MemorySnapshot {
    /// Creates a new empty memory snapshot.
    pub fn new(frame_number: u64) -> Self {
        Self {
            frame_number,
            timestamp: Instant::now(),
            ..Default::default()
        }
    }

    /// Returns memory usage for a specific category in bytes.
    pub fn bytes_for_category(&self, category: MemoryCategory) -> u64 {
        match category {
            MemoryCategory::Texture => self.texture_bytes,
            MemoryCategory::MeshBuffer => self.mesh_buffer_bytes,
            MemoryCategory::DescriptorSet => self.descriptor_set_bytes,
            MemoryCategory::UniformBuffer => self.uniform_buffer_bytes,
            MemoryCategory::ComputeBuffer => self.compute_buffer_bytes,
            MemoryCategory::RenderTarget => self.render_target_bytes,
            MemoryCategory::Other => self.other_bytes,
        }
    }

    /// Returns total memory usage in megabytes.
    pub fn total_mb(&self) -> f64 {
        self.total_bytes as f64 / 1_048_576.0
    }

    /// Returns peak memory usage in megabytes.
    pub fn peak_mb(&self) -> f64 {
        self.peak_bytes as f64 / 1_048_576.0
    }

    /// Returns memory usage for a category in megabytes.
    pub fn category_mb(&self, category: MemoryCategory) -> f64 {
        self.bytes_for_category(category) as f64 / 1_048_576.0
    }

    /// Returns the percentage of total memory used by a category.
    pub fn category_percentage(&self, category: MemoryCategory) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.bytes_for_category(category) as f64 / self.total_bytes as f64 * 100.0) as f32
    }
}

/// Rolling history of memory snapshots with trend analysis.
///
/// Maintains a circular buffer of memory snapshots and computes
/// statistical metrics for analysis and visualization.
#[derive(Debug, Clone)]
pub struct MemoryHistory {
    /// Recent memory snapshots (circular buffer)
    snapshots: Vec<MemorySnapshot>,
    /// Current write position in circular buffer
    write_index: usize,
    /// Maximum number of snapshots to track
    max_snapshots: usize,
    /// Number of snapshots currently stored
    count: usize,
    /// Global peak memory usage across all snapshots
    global_peak_bytes: u64,
}

impl MemoryHistory {
    /// Creates a new memory history tracker.
    ///
    /// # Arguments
    ///
    /// * `max_snapshots` - Maximum number of snapshots to track
    ///
    /// # Recommended Values
    ///
    /// - **300**: ~5 seconds at 60 FPS
    /// - **1800**: ~30 seconds at 60 FPS
    /// - **18000**: ~5 minutes at 60 FPS
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: vec![MemorySnapshot::default(); max_snapshots],
            write_index: 0,
            max_snapshots,
            count: 0,
            global_peak_bytes: 0,
        }
    }

    /// Records a new memory snapshot.
    ///
    /// If the history is full, the oldest snapshot is overwritten.
    pub fn record(&mut self, snapshot: MemorySnapshot) {
        self.global_peak_bytes = self.global_peak_bytes.max(snapshot.total_bytes);
        self.snapshots[self.write_index] = snapshot;
        self.write_index = (self.write_index + 1) % self.max_snapshots;
        self.count = (self.count + 1).min(self.max_snapshots);
    }

    /// Returns the number of snapshots currently stored.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if no snapshots have been recorded.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Clears all recorded snapshots.
    pub fn clear(&mut self) {
        self.write_index = 0;
        self.count = 0;
        self.global_peak_bytes = 0;
    }

    /// Returns the most recent snapshot.
    pub fn latest(&self) -> Option<&MemorySnapshot> {
        if self.count == 0 {
            return None;
        }
        let index = (self.write_index + self.max_snapshots - 1) % self.max_snapshots;
        Some(&self.snapshots[index])
    }

    /// Returns an iterator over all snapshots (oldest to newest).
    pub fn iter(&self) -> impl Iterator<Item = &MemorySnapshot> {
        let start = if self.count < self.max_snapshots {
            0
        } else {
            self.write_index
        };

        (0..self.count).map(move |i| {
            let index = (start + i) % self.max_snapshots;
            &self.snapshots[index]
        })
    }

    /// Returns the global peak memory usage across all tracked snapshots.
    pub fn global_peak_bytes(&self) -> u64 {
        self.global_peak_bytes
    }

    /// Returns the global peak memory usage in megabytes.
    pub fn global_peak_mb(&self) -> f64 {
        self.global_peak_bytes as f64 / 1_048_576.0
    }

    /// Returns average total memory usage in bytes.
    pub fn avg_total_bytes(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: u64 = self.iter().map(|s| s.total_bytes).sum();
        sum as f64 / self.count as f64
    }

    /// Returns average memory usage for a category in bytes.
    pub fn avg_category_bytes(&self, category: MemoryCategory) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: u64 = self.iter().map(|s| s.bytes_for_category(category)).sum();
        sum as f64 / self.count as f64
    }

    /// Returns memory usage history for a category as a vector for graphing.
    pub fn category_history(&self, category: MemoryCategory) -> Vec<f32> {
        self.iter()
            .map(|s| s.bytes_for_category(category) as f32 / 1_048_576.0)
            .collect()
    }

    /// Returns total memory usage history as a vector for graphing (in MB).
    pub fn total_history(&self) -> Vec<f32> {
        self.iter()
            .map(|s| s.total_bytes as f32 / 1_048_576.0)
            .collect()
    }

    /// Returns allocation count history as a vector for graphing.
    pub fn allocation_count_history(&self) -> Vec<f32> {
        self.iter().map(|s| s.allocation_count as f32).collect()
    }
}

impl Default for MemoryHistory {
    fn default() -> Self {
        Self::new(300) // Default to 300 frames (~5 seconds at 60 FPS)
    }
}

/// Main GPU memory profiler.
///
/// Tracks all GPU memory allocations with categorization, metadata, and
/// historical trend analysis. Integrates with RenderStats to correlate
/// memory usage with rendering optimizations.
#[derive(Debug)]
pub struct MemoryProfiler {
    /// All active allocations indexed by ID
    allocations: HashMap<String, VramAllocation>,
    /// Current memory usage by category
    category_totals: HashMap<MemoryCategory, u64>,
    /// Peak memory usage since profiling started
    peak_total_bytes: u64,
    /// Historical snapshots
    history: MemoryHistory,
    /// Current frame number
    current_frame: u64,
    /// Whether profiling is enabled
    enabled: bool,
}

impl MemoryProfiler {
    /// Creates a new memory profiler with default settings.
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            category_totals: HashMap::new(),
            peak_total_bytes: 0,
            history: MemoryHistory::default(),
            current_frame: 0,
            enabled: true,
        }
    }

    /// Creates a new memory profiler with custom history size.
    pub fn with_history_size(history_size: usize) -> Self {
        Self {
            allocations: HashMap::new(),
            category_totals: HashMap::new(),
            peak_total_bytes: 0,
            history: MemoryHistory::new(history_size),
            current_frame: 0,
            enabled: true,
        }
    }

    /// Enables or disables memory profiling.
    ///
    /// When disabled, allocation tracking continues but snapshots are not recorded.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether profiling is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Records a new GPU memory allocation.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this allocation (e.g., "brick_texture")
    /// * `category` - Category of allocation
    /// * `size_bytes` - Size in bytes
    /// * `metadata` - Optional metadata string (e.g., "1024x1024 RGBA8")
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use praxis_graphics::memory_profiler::{MemoryProfiler, MemoryCategory};
    /// # let mut profiler = MemoryProfiler::new();
    /// profiler.record_allocation(
    ///     "character_model",
    ///     MemoryCategory::MeshBuffer,
    ///     vertex_size + index_size,
    ///     Some(format!("10000 verts, 15000 indices")),
    /// );
    /// ```
    pub fn record_allocation(
        &mut self,
        id: impl Into<String>,
        category: MemoryCategory,
        size_bytes: u64,
        metadata: Option<String>,
    ) {
        let id = id.into();

        // If allocation already exists, free it first
        if let Some(old_alloc) = self.allocations.remove(&id) {
            self.free_allocation_internal(&old_alloc);
        }

        // Record new allocation
        let allocation = VramAllocation {
            id: id.clone(),
            category,
            size_bytes,
            metadata,
            created_at: Instant::now(),
        };

        *self.category_totals.entry(category).or_insert(0) += size_bytes;
        self.allocations.insert(id, allocation);

        // Update peak tracking
        let total = self.total_allocated_bytes();
        self.peak_total_bytes = self.peak_total_bytes.max(total);
    }

    /// Frees a previously recorded allocation.
    ///
    /// # Arguments
    ///
    /// * `id` - Identifier of the allocation to free
    ///
    /// # Returns
    ///
    /// True if the allocation was found and freed, false otherwise.
    pub fn free_allocation(&mut self, id: &str) -> bool {
        if let Some(allocation) = self.allocations.remove(id) {
            self.free_allocation_internal(&allocation);
            true
        } else {
            false
        }
    }

    /// Internal helper to free an allocation's memory from category totals.
    fn free_allocation_internal(&mut self, allocation: &VramAllocation) {
        if let Some(total) = self.category_totals.get_mut(&allocation.category) {
            *total = total.saturating_sub(allocation.size_bytes);
        }
    }

    /// Returns the total currently allocated VRAM in bytes.
    pub fn total_allocated_bytes(&self) -> u64 {
        self.category_totals.values().sum()
    }

    /// Returns the total allocated VRAM in megabytes.
    pub fn total_allocated_mb(&self) -> f64 {
        self.total_allocated_bytes() as f64 / 1_048_576.0
    }

    /// Returns the peak VRAM usage in bytes.
    pub fn peak_bytes(&self) -> u64 {
        self.peak_total_bytes
    }

    /// Returns the peak VRAM usage in megabytes.
    pub fn peak_mb(&self) -> f64 {
        self.peak_total_bytes as f64 / 1_048_576.0
    }

    /// Returns bytes allocated for a specific category.
    pub fn category_bytes(&self, category: MemoryCategory) -> u64 {
        self.category_totals.get(&category).copied().unwrap_or(0)
    }

    /// Returns megabytes allocated for a specific category.
    pub fn category_mb(&self, category: MemoryCategory) -> f64 {
        self.category_bytes(category) as f64 / 1_048_576.0
    }

    /// Returns the number of active allocations.
    pub fn allocation_count(&self) -> usize {
        self.allocations.len()
    }

    /// Returns an iterator over all active allocations.
    pub fn allocations(&self) -> impl Iterator<Item = &VramAllocation> {
        self.allocations.values()
    }

    /// Returns allocations for a specific category.
    pub fn allocations_for_category(
        &self,
        category: MemoryCategory,
    ) -> impl Iterator<Item = &VramAllocation> {
        self.allocations
            .values()
            .filter(move |a| a.category == category)
    }

    /// Creates a memory snapshot of the current state.
    ///
    /// Captures all current memory metrics in a point-in-time snapshot.
    pub fn snapshot(&self) -> MemorySnapshot {
        let total_bytes = self.total_allocated_bytes();

        MemorySnapshot {
            frame_number: self.current_frame,
            total_bytes,
            peak_bytes: self.peak_total_bytes,
            texture_bytes: self.category_bytes(MemoryCategory::Texture),
            mesh_buffer_bytes: self.category_bytes(MemoryCategory::MeshBuffer),
            descriptor_set_bytes: self.category_bytes(MemoryCategory::DescriptorSet),
            uniform_buffer_bytes: self.category_bytes(MemoryCategory::UniformBuffer),
            compute_buffer_bytes: self.category_bytes(MemoryCategory::ComputeBuffer),
            render_target_bytes: self.category_bytes(MemoryCategory::RenderTarget),
            other_bytes: self.category_bytes(MemoryCategory::Other),
            allocation_count: self.allocation_count(),
            timestamp: Instant::now(),
        }
    }

    /// Begins a new frame and optionally records a snapshot.
    ///
    /// Should be called at the start of each frame to maintain accurate
    /// frame numbering and historical tracking.
    pub fn begin_frame(&mut self) {
        self.current_frame += 1;

        if self.enabled {
            let snapshot = self.snapshot();
            self.history.record(snapshot);
        }
    }

    /// Returns a reference to the memory history.
    pub fn history(&self) -> &MemoryHistory {
        &self.history
    }

    /// Clears all allocations and resets profiling state.
    ///
    /// This does not clear historical snapshots. Use `history().clear()` for that.
    pub fn clear_allocations(&mut self) {
        self.allocations.clear();
        self.category_totals.clear();
    }

    /// Resets all profiling data including history.
    pub fn reset(&mut self) {
        self.allocations.clear();
        self.category_totals.clear();
        self.peak_total_bytes = 0;
        self.history.clear();
        self.current_frame = 0;
    }

    /// Returns the current frame number.
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }
}

impl Default for MemoryProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_profiler_basic() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation(
            "test_texture",
            MemoryCategory::Texture,
            1024 * 1024 * 4,
            Some("1024x1024 RGBA8".to_string()),
        );

        assert_eq!(profiler.allocation_count(), 1);
        assert_eq!(profiler.total_allocated_bytes(), 4_194_304);
        assert_eq!(profiler.category_bytes(MemoryCategory::Texture), 4_194_304);
    }

    #[test]
    fn test_allocation_replacement() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation("test", MemoryCategory::Texture, 1000, None);
        assert_eq!(profiler.total_allocated_bytes(), 1000);

        // Re-recording should replace, not add
        profiler.record_allocation("test", MemoryCategory::Texture, 2000, None);
        assert_eq!(profiler.allocation_count(), 1);
        assert_eq!(profiler.total_allocated_bytes(), 2000);
    }

    #[test]
    fn test_free_allocation() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation("test", MemoryCategory::MeshBuffer, 5000, None);
        assert_eq!(profiler.total_allocated_bytes(), 5000);

        assert!(profiler.free_allocation("test"));
        assert_eq!(profiler.total_allocated_bytes(), 0);
        assert_eq!(profiler.allocation_count(), 0);
    }

    #[test]
    fn test_peak_tracking() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation("alloc1", MemoryCategory::Texture, 1000, None);
        profiler.record_allocation("alloc2", MemoryCategory::MeshBuffer, 2000, None);
        assert_eq!(profiler.peak_bytes(), 3000);

        profiler.free_allocation("alloc1");
        assert_eq!(profiler.total_allocated_bytes(), 2000);
        assert_eq!(profiler.peak_bytes(), 3000); // Peak should remain
    }

    #[test]
    fn test_memory_snapshot() {
        let mut profiler = MemoryProfiler::new();

        profiler.record_allocation("tex", MemoryCategory::Texture, 1000, None);
        profiler.record_allocation("mesh", MemoryCategory::MeshBuffer, 2000, None);

        let snapshot = profiler.snapshot();
        assert_eq!(snapshot.total_bytes, 3000);
        assert_eq!(snapshot.texture_bytes, 1000);
        assert_eq!(snapshot.mesh_buffer_bytes, 2000);
        assert_eq!(snapshot.allocation_count, 2);
    }

    #[test]
    fn test_memory_history() {
        let mut history = MemoryHistory::new(3);

        let snap1 = MemorySnapshot {
            frame_number: 1,
            total_bytes: 1000,
            ..Default::default()
        };
        history.record(snap1);

        let snap2 = MemorySnapshot {
            frame_number: 2,
            total_bytes: 2000,
            ..Default::default()
        };
        history.record(snap2);

        assert_eq!(history.len(), 2);
        assert_eq!(history.global_peak_bytes(), 2000);
        assert_eq!(history.avg_total_bytes(), 1500.0);
    }

    #[test]
    fn test_category_percentage() {
        let snapshot = MemorySnapshot {
            frame_number: 1,
            total_bytes: 1000,
            texture_bytes: 400,
            mesh_buffer_bytes: 600,
            ..Default::default()
        };

        assert_eq!(snapshot.category_percentage(MemoryCategory::Texture), 40.0);
        assert_eq!(
            snapshot.category_percentage(MemoryCategory::MeshBuffer),
            60.0
        );
    }
}
