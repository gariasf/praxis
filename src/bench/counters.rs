use std::ops::{Index, IndexMut};

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

// A counter's value sits at its discriminant, and `ALL` lists the counters in that order.
const _: () = {
    let mut index = 0;
    while index < Counter::COUNT {
        assert!(Counter::ALL[index] as usize == index);
        index += 1;
    }
};

/// Each `Counter`'s value over a run's frames, by its rule. Results and
/// Bench records hold them flat, by name, in `Counter::ALL`'s order.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Counters([u64; Counter::COUNT]);

impl Counters {
    /// The counters of `frames`, each with its GPU stats.
    pub(crate) fn of(frames: &[FrameStats]) -> Self {
        Self(Counter::ALL.map(|counter| counter.value_over(frames)))
    }

    /// Whether the frames these count loaded or unloaded a Chunk or read a
    /// byte.
    pub(super) fn streamed(&self) -> bool {
        self[Counter::ChunksLoaded] + self[Counter::ChunksUnloaded] + self[Counter::BytesRead] > 0
    }
}

impl Index<Counter> for Counters {
    type Output = u64;

    fn index(&self, counter: Counter) -> &u64 {
        &self.0[counter as usize]
    }
}

impl IndexMut<Counter> for Counters {
    fn index_mut(&mut self, counter: Counter) -> &mut u64 {
        &mut self.0[counter as usize]
    }
}

#[cfg(test)]
impl FromIterator<(Counter, u64)> for Counters {
    fn from_iter<Values: IntoIterator<Item = (Counter, u64)>>(values: Values) -> Self {
        let mut counters = Self::default();
        for (counter, value) in values {
            counters[counter] = value;
        }
        counters
    }
}

impl serde::Serialize for Counters {
    fn serialize<Serializer: serde::Serializer>(
        &self,
        serializer: Serializer,
    ) -> Result<Serializer::Ok, Serializer::Error> {
        use serde::ser::SerializeStruct as _;
        let mut fields = serializer.serialize_struct("Counters", Counter::COUNT)?;
        for counter in Counter::ALL {
            fields.serialize_field(counter.name(), &self[counter])?;
        }
        fields.end()
    }
}

/// Skips a field naming no counter: a record holding them flattened hands
/// it its other fields too.
impl<'de> serde::Deserialize<'de> for Counters {
    fn deserialize<Deserializer: serde::Deserializer<'de>>(
        deserializer: Deserializer,
    ) -> Result<Self, Deserializer::Error> {
        deserializer.deserialize_map(CountersVisitor)
    }
}

struct CountersVisitor;

impl<'de> serde::de::Visitor<'de> for CountersVisitor {
    type Value = Counters;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("counters by name")
    }

    fn visit_map<Fields: serde::de::MapAccess<'de>>(
        self,
        mut fields: Fields,
    ) -> Result<Counters, Fields::Error> {
        use serde::de::Error as _;
        let mut values_read = [None; Counter::COUNT];
        while let Some(name) = fields.next_key::<String>()? {
            let named_counter = Counter::ALL
                .into_iter()
                .find(|counter| counter.name() == name);
            match named_counter {
                Some(counter) if values_read[counter as usize].is_some() => {
                    return Err(Fields::Error::duplicate_field(counter.name()));
                }
                Some(counter) => values_read[counter as usize] = Some(fields.next_value()?),
                None => {
                    fields.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }
        let mut counters = Counters::default();
        for counter in Counter::ALL {
            counters[counter] = match values_read[counter as usize] {
                Some(value) => value,
                None if counter.absent_reads_as_none() => 0,
                None => return Err(Fields::Error::missing_field(counter.name())),
            };
        }
        Ok(counters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::Measurements;
    use crate::bench::tests::baseline;

    #[test]
    fn each_counter_is_named_as_results_spell_it() {
        let counters: Counters = Counter::ALL.into_iter().zip(1..).collect();
        let json = serde_json::to_value(counters).unwrap();
        assert_eq!(json.as_object().unwrap().len(), Counter::COUNT);
        for (counter, value) in Counter::ALL.into_iter().zip(1..) {
            assert_eq!(json[counter.name()], value, "{counter:?}");
        }
        assert_eq!(serde_json::from_value::<Counters>(json).unwrap(), counters);
    }

    /// `measurements` read back from its JSON without `counter` at the top
    /// level, where the measured frames' counters sit.
    fn read_without(counter: Counter, measurements: &Measurements) -> anyhow::Result<Measurements> {
        let serde_json::Value::Object(mut json) = serde_json::to_value(measurements).unwrap()
        else {
            panic!("a result is a JSON object");
        };
        json.remove(counter.name()).expect("the result holds it");
        Measurements::from_baseline_json(serde_json::Value::Object(json).to_string().as_bytes())
    }

    #[test]
    fn a_result_without_draw_calls_is_refused() {
        let error = read_without(Counter::DrawCalls, &baseline()).expect_err("refused");
        assert!(
            format!("{error:#}").contains("missing field `draw_calls`"),
            "{error:#}"
        );
    }

    #[test]
    fn a_result_without_late_mips_reads_them_as_none() {
        let mut late = baseline();
        late.counters[Counter::LateMips] = 12;
        let read = read_without(Counter::LateMips, &late).expect("read");
        assert_eq!(read, baseline());
    }
}
