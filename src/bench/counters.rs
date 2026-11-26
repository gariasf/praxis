use crate::stats::{FrameStats, GpuFrameStats};

use super::percentile;

/// A count a bench run takes from Frame stats, held to its Baseline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Counter {
    DrawCalls,
    InstancesDrawn,
    TrianglesDrawn,
    InstancesOverflowed,
    UploadedBytes,
    GpuMemoryBytes,
    ArenaBytesInUse,
    HeapAllocations,
    RenderablesRefused,
    LightsOverflowed,
    StaticInstancesLeftOut,
    MovingInstancesLeftOut,
    BendsLeftOut,
    FroxelLightsOverflowed,
    ShadowMaskLightsOverflowed,
    LateChunks,
    LateFarProxies,
    ChunksLoaded,
    ChunksUnloaded,
    BytesRead,
    PoolEntriesFreed,
    LateMips,
    TlasBuildAllocations,
    InstanceSlotsInUse,
}

/// How a counter takes one value from the frames it counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CounterRule {
    /// The median frame's, as the counter repeats exactly on every run.
    Median,
    /// The most of any frame: a refusal, as one in any frame is a defect,
    /// whatever the IO pool's timing.
    Most,
    /// The sum over the frames, as each frame counts events.
    Total,
}

impl Counter {
    pub const ALL: [Self; 24] = [
        Self::DrawCalls,
        Self::InstancesDrawn,
        Self::TrianglesDrawn,
        Self::InstancesOverflowed,
        Self::UploadedBytes,
        Self::GpuMemoryBytes,
        Self::ArenaBytesInUse,
        Self::HeapAllocations,
        Self::RenderablesRefused,
        Self::LightsOverflowed,
        Self::StaticInstancesLeftOut,
        Self::MovingInstancesLeftOut,
        Self::BendsLeftOut,
        Self::FroxelLightsOverflowed,
        Self::ShadowMaskLightsOverflowed,
        Self::LateChunks,
        Self::LateFarProxies,
        Self::ChunksLoaded,
        Self::ChunksUnloaded,
        Self::BytesRead,
        Self::PoolEntriesFreed,
        Self::LateMips,
        Self::TlasBuildAllocations,
        Self::InstanceSlotsInUse,
    ];

    pub const COUNT: usize = Self::ALL.len();

    /// Its name, as results and Bench records spell it.
    pub fn name(self) -> &'static str {
        match self {
            Self::DrawCalls => "draw_calls",
            Self::InstancesDrawn => "instances_drawn",
            Self::TrianglesDrawn => "triangles_drawn",
            Self::InstancesOverflowed => "instances_overflowed",
            Self::UploadedBytes => "uploaded_bytes",
            Self::GpuMemoryBytes => "gpu_memory_bytes",
            Self::ArenaBytesInUse => "arena_bytes_in_use",
            Self::HeapAllocations => "heap_allocations",
            Self::RenderablesRefused => "renderables_refused",
            Self::LightsOverflowed => "lights_overflowed",
            Self::StaticInstancesLeftOut => "static_instances_left_out",
            Self::MovingInstancesLeftOut => "moving_instances_left_out",
            Self::BendsLeftOut => "bends_left_out",
            Self::FroxelLightsOverflowed => "froxel_lights_overflowed",
            Self::ShadowMaskLightsOverflowed => "shadow_mask_lights_overflowed",
            Self::LateChunks => "late_chunks",
            Self::LateFarProxies => "late_far_proxies",
            Self::ChunksLoaded => "chunks_loaded",
            Self::ChunksUnloaded => "chunks_unloaded",
            Self::BytesRead => "bytes_read",
            Self::PoolEntriesFreed => "pool_entries_freed",
            Self::LateMips => "late_mips",
            Self::TlasBuildAllocations => "tlas_build_allocations",
            Self::InstanceSlotsInUse => "instance_slots_in_use",
        }
    }

    pub fn rule(self) -> CounterRule {
        match self {
            Self::DrawCalls
            | Self::InstancesDrawn
            | Self::TrianglesDrawn
            | Self::UploadedBytes
            | Self::GpuMemoryBytes
            | Self::ArenaBytesInUse
            | Self::HeapAllocations
            | Self::TlasBuildAllocations
            | Self::InstanceSlotsInUse => CounterRule::Median,
            Self::InstancesOverflowed
            | Self::RenderablesRefused
            | Self::LightsOverflowed
            | Self::StaticInstancesLeftOut
            | Self::MovingInstancesLeftOut
            | Self::BendsLeftOut
            | Self::FroxelLightsOverflowed
            | Self::ShadowMaskLightsOverflowed => CounterRule::Most,
            Self::LateChunks
            | Self::LateFarProxies
            | Self::ChunksLoaded
            | Self::ChunksUnloaded
            | Self::BytesRead
            | Self::PoolEntriesFreed
            | Self::LateMips => CounterRule::Total,
        }
    }

    /// Whether a comparison judges it, rather than only reporting it: the
    /// late mips depend on when the IO pool's reads land, and the heap
    /// allocations already hold the TLAS build's.
    pub fn judged(self) -> bool {
        match self {
            Self::LateMips | Self::TlasBuildAllocations => false,
            Self::DrawCalls
            | Self::InstancesDrawn
            | Self::TrianglesDrawn
            | Self::InstancesOverflowed
            | Self::UploadedBytes
            | Self::GpuMemoryBytes
            | Self::ArenaBytesInUse
            | Self::HeapAllocations
            | Self::RenderablesRefused
            | Self::LightsOverflowed
            | Self::StaticInstancesLeftOut
            | Self::MovingInstancesLeftOut
            | Self::BendsLeftOut
            | Self::FroxelLightsOverflowed
            | Self::ShadowMaskLightsOverflowed
            | Self::LateChunks
            | Self::LateFarProxies
            | Self::ChunksLoaded
            | Self::ChunksUnloaded
            | Self::BytesRead
            | Self::PoolEntriesFreed
            | Self::InstanceSlotsInUse => true,
        }
    }

    /// Whether a record without it reads it as none, as Baselines recorded
    /// before results carried it do, rather than being refused.
    #[cfg(test)]
    pub fn absent_reads_as_none(self) -> bool {
        match self {
            Self::DrawCalls
            | Self::InstancesDrawn
            | Self::TrianglesDrawn
            | Self::InstancesOverflowed
            | Self::UploadedBytes
            | Self::GpuMemoryBytes
            | Self::ArenaBytesInUse
            | Self::HeapAllocations => false,
            Self::RenderablesRefused
            | Self::LightsOverflowed
            | Self::StaticInstancesLeftOut
            | Self::MovingInstancesLeftOut
            | Self::BendsLeftOut
            | Self::FroxelLightsOverflowed
            | Self::ShadowMaskLightsOverflowed
            | Self::LateChunks
            | Self::LateFarProxies
            | Self::ChunksLoaded
            | Self::ChunksUnloaded
            | Self::BytesRead
            | Self::PoolEntriesFreed
            | Self::LateMips
            | Self::TlasBuildAllocations
            | Self::InstanceSlotsInUse => true,
        }
    }

    /// Its value in one frame: none of what the GPU measured in a frame
    /// whose GPU stats are missing.
    pub fn value_of(self, stats: &FrameStats) -> u64 {
        let gpu_value = |value: fn(&GpuFrameStats) -> u32| -> u64 {
            stats.gpu.as_ref().map_or(0, value).into()
        };
        match self {
            Self::DrawCalls => stats.draw_calls.into(),
            Self::InstancesDrawn => gpu_value(|gpu| gpu.instances_drawn),
            Self::TrianglesDrawn => gpu_value(|gpu| gpu.triangles_drawn),
            Self::InstancesOverflowed => gpu_value(|gpu| gpu.instances_overflowed),
            Self::UploadedBytes => stats.uploaded_bytes,
            Self::GpuMemoryBytes => stats.gpu_memory_bytes,
            Self::ArenaBytesInUse => stats.arena_bytes_in_use,
            Self::HeapAllocations => stats.heap_allocations,
            Self::RenderablesRefused => stats.renderables_refused.into(),
            Self::LightsOverflowed => stats.lights_overflowed.into(),
            Self::StaticInstancesLeftOut => stats.static_instances_left_out.into(),
            Self::MovingInstancesLeftOut => stats.moving_instances_left_out.into(),
            Self::BendsLeftOut => stats.bends_left_out.into(),
            Self::FroxelLightsOverflowed => gpu_value(|gpu| gpu.froxel_lights_overflowed),
            Self::ShadowMaskLightsOverflowed => gpu_value(|gpu| gpu.shadow_mask_lights_overflowed),
            Self::LateChunks => stats.late_chunks.into(),
            Self::LateFarProxies => stats.late_far_proxies.into(),
            Self::ChunksLoaded => stats.chunks_loaded.into(),
            Self::ChunksUnloaded => stats.chunks_unloaded.into(),
            Self::BytesRead => stats.bytes_read,
            Self::PoolEntriesFreed => stats.pool_entries_freed.into(),
            Self::LateMips => stats.late_mips.into(),
            Self::TlasBuildAllocations => stats.tlas_build_allocations,
            Self::InstanceSlotsInUse => stats.instance_slots_in_use.into(),
        }
    }

    /// Its value over `frames`, by its rule.
    pub fn value_over(self, frames: &[FrameStats]) -> u64 {
        let values = frames.iter().map(|stats| self.value_of(stats));
        match self.rule() {
            CounterRule::Median => {
                percentile(values.map(|value| value as f64).collect(), 0.5) as u64
            }
            CounterRule::Most => values.max().unwrap_or(0),
            CounterRule::Total => values.sum(),
        }
    }
}
