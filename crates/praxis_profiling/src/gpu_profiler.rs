//! GPU profiling using Vulkan timestamp queries.

use praxis_utils::{debug, error, warn, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    device::{Device, Queue},
    query::{QueryPool, QueryPoolCreateInfo, QueryResultFlags, QueryType},
};

/// Represents a GPU timestamp query pair (start and end).
#[derive(Debug, Clone)]
pub struct TimestampQuery {
    /// Name of the query
    pub name: String,
    /// Query pool
    pub pool: Arc<QueryPool>,
    /// Start query index
    pub start_index: u32,
    /// End query index
    pub end_index: u32,
}

/// GPU timestamp measurement result.
#[derive(Debug, Clone)]
pub struct GpuTimestamp {
    /// Name of the measurement
    pub name: String,
    /// Duration in nanoseconds
    pub duration_ns: u64,
    /// Start timestamp
    pub start_ns: u64,
    /// End timestamp
    pub end_ns: u64,
}

impl GpuTimestamp {
    /// Returns the duration as a `Duration`.
    pub fn duration(&self) -> Duration {
        Duration::from_nanos(self.duration_ns)
    }
}

/// GPU profiler for tracking GPU execution times using Vulkan timestamp queries.
pub struct GpuProfiler {
    /// Vulkan device
    #[allow(dead_code)]
    device: Arc<Device>,
    /// Graphics queue
    #[allow(dead_code)]
    queue: Arc<Queue>,
    /// Query pools for double buffering
    query_pools: Vec<Arc<QueryPool>>,
    /// Current query pool index
    current_pool_index: usize,
    /// Number of queries per pool
    queries_per_pool: u32,
    /// Next query index in current pool
    next_query_index: u32,
    /// Active queries in current frame
    active_queries: HashMap<String, (Arc<QueryPool>, u32, u32)>,
    /// Timestamp period in nanoseconds
    timestamp_period: f32,
    /// Maximum number of queries per frame
    #[allow(dead_code)]
    max_queries_per_frame: u32,
}

impl GpuProfiler {
    /// Creates a new GPU profiler.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `queue` - Graphics queue
    /// * `max_queries_per_frame` - Maximum number of timestamp queries per frame
    /// * `num_buffered_frames` - Number of frames to buffer (typically 2-3)
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        max_queries_per_frame: u32,
        num_buffered_frames: u32,
    ) -> Result<Self> {
        // Check if timestamp queries are supported
        let physical_device = device.physical_device();
        let queue_family_properties = physical_device.queue_family_properties();
        let queue_family_index = queue.queue_family_index();

        let timestamp_valid_bits = queue_family_properties
            .get(queue_family_index as usize)
            .and_then(|props| props.timestamp_valid_bits)
            .unwrap_or(0);

        if timestamp_valid_bits == 0 {
            warn!("GPU timestamp queries not supported on this device");
        }

        let timestamp_period = physical_device.properties().timestamp_period;

        // Create query pools for each buffered frame
        let queries_per_pool = max_queries_per_frame * 2; // *2 for start/end pairs
        let mut query_pools = Vec::with_capacity(num_buffered_frames as usize);

        for _ in 0..num_buffered_frames {
            let pool = QueryPool::new(
                device.clone(),
                QueryPoolCreateInfo {
                    query_count: queries_per_pool,
                    ..QueryPoolCreateInfo::query_type(QueryType::Timestamp)
                },
            )
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to create query pool: {}", e))?;

            query_pools.push(pool);
        }

        debug!(
            "Created GPU profiler with {} query pools, {} queries each (timestamp period: {}ns)",
            num_buffered_frames, queries_per_pool, timestamp_period
        );

        Ok(Self {
            device,
            queue,
            query_pools,
            current_pool_index: 0,
            queries_per_pool,
            next_query_index: 0,
            active_queries: HashMap::new(),
            timestamp_period,
            max_queries_per_frame,
        })
    }

    /// Begins a new frame of GPU profiling.
    pub fn begin_frame(&mut self) {
        // Move to next query pool
        self.current_pool_index = (self.current_pool_index + 1) % self.query_pools.len();
        self.next_query_index = 0;
        self.active_queries.clear();

        // Note: Query pool reset needs to be done on the GPU timeline
        // This is typically done at the start of command buffer recording
    }

    /// Resets the query pool at the start of command buffer recording.
    ///
    /// This should be called at the beginning of your command buffer.
    pub fn reset_queries(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<()> {
        let pool = self.query_pools[self.current_pool_index].clone();

        // Reset all queries in the pool
        // Note: This must be done on the GPU timeline, not CPU
        unsafe {
            builder
                .reset_query_pool(pool.clone(), 0..self.queries_per_pool)
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to reset query pool: {}", e))?;
        }

        Ok(())
    }

    /// Begins a GPU timing measurement.
    ///
    /// Returns the query indices (start, end) to use with command buffer.
    pub fn begin_query(&mut self, name: impl Into<String>) -> Option<(Arc<QueryPool>, u32, u32)> {
        // Ensure we don't exceed the query pool capacity
        if self.next_query_index + 2 > self.queries_per_pool {
            error!(
                "GPU profiler: Query pool exhausted (used {}/{})",
                self.next_query_index, self.queries_per_pool
            );
            return None;
        }

        let name = name.into();
        let pool = self.query_pools[self.current_pool_index].clone();
        let start_index = self.next_query_index;
        let end_index = self.next_query_index + 1;
        self.next_query_index += 2;

        self.active_queries
            .insert(name.clone(), (pool.clone(), start_index, end_index));

        debug!(
            "GPU profiler: Started query '{}' (indices {}-{})",
            name, start_index, end_index
        );

        Some((pool, start_index, end_index))
    }

    /// Writes a timestamp to the command buffer.
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder
    /// * `pool` - Query pool
    /// * `query_index` - Query index to write to
    /// * `stage` - Pipeline stage for the timestamp
    pub fn write_timestamp(
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pool: Arc<QueryPool>,
        query_index: u32,
        stage: vulkano::sync::PipelineStage,
    ) -> Result<()> {
        unsafe {
            builder
                .write_timestamp(pool, query_index, stage)
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to write timestamp: {}", e))?;
        }
        Ok(())
    }

    /// Collects the results of GPU timing measurements from the previous frame.
    ///
    /// This should be called after the GPU has finished executing commands.
    /// Uses WAIT flag to ensure results are available before reading.
    pub fn collect_results(&self) -> Result<Vec<GpuTimestamp>> {
        // Get the previous frame's query pool
        let prev_pool_index = if self.current_pool_index == 0 {
            self.query_pools.len() - 1
        } else {
            self.current_pool_index - 1
        };

        let pool = self.query_pools[prev_pool_index].clone();
        let mut results = Vec::new();

        // Only read queries that were actually used in the previous frame
        // Use the max query index from previous frame to avoid reading uninitialized queries
        let query_count = self.next_query_index.min(self.queries_per_pool);
        
        if query_count == 0 {
            // No queries were used in this frame
            return Ok(results);
        }

        let mut timestamps = vec![0u64; query_count as usize];

        // Try to get query results with WAIT flag to ensure availability
        // WAIT ensures GPU operations complete before reading results
        let result = pool.get_results(
            0..query_count,
            &mut timestamps,
            QueryResultFlags::WAIT,
        );

        match result {
            Ok(_) => {
                // Process timestamp pairs
                for i in (0..query_count).step_by(2) {
                    if (i + 1) >= query_count {
                        break; // Incomplete pair
                    }

                    let start_ts = timestamps[i as usize];
                    let end_ts = timestamps[(i + 1) as usize];

                    // Validate timestamp values before processing
                    if start_ts > 0 && end_ts > 0 && end_ts >= start_ts {
                        let duration_raw = end_ts - start_ts;
                        let duration_ns =
                            (duration_raw as f64 * self.timestamp_period as f64) as u64;

                        // Sanity check: duration should be reasonable (< 1 second)
                        if duration_ns < 1_000_000_000 {
                            results.push(GpuTimestamp {
                                name: format!("Query_{}", i / 2),
                                duration_ns,
                                start_ns: (start_ts as f64 * self.timestamp_period as f64) as u64,
                                end_ns: (end_ts as f64 * self.timestamp_period as f64) as u64,
                            });
                        } else {
                            debug!(
                                "GPU profiler: Ignoring query {} with excessive duration: {}ms",
                                i / 2,
                                duration_ns / 1_000_000
                            );
                        }
                    }
                }
            }
            Err(e) => {
                // Results not ready yet, this is expected if GPU is still working
                debug!("GPU query results not ready: {:?}", e);
            }
        }

        Ok(results)
    }

    /// Returns the number of active queries in the current frame.
    pub fn active_query_count(&self) -> usize {
        self.active_queries.len()
    }

    /// Returns whether timestamp queries are supported.
    pub fn is_supported(&self) -> bool {
        self.timestamp_period > 0.0
    }
}

/// Helper for inserting GPU profiling markers in command buffers.
pub struct GpuProfileScope {
    builder: *mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    pool: Arc<QueryPool>,
    end_index: u32,
}

impl GpuProfileScope {
    /// Creates a new GPU profile scope.
    ///
    /// # Safety
    ///
    /// The builder pointer must remain valid for the lifetime of this scope.
    #[allow(dead_code)]
    pub unsafe fn new(
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        profiler: &mut GpuProfiler,
        name: &str,
    ) -> Option<Self> {
        if let Some((pool, start_index, end_index)) = profiler.begin_query(name) {
            // Write start timestamp
            if let Err(e) = GpuProfiler::write_timestamp(
                builder,
                pool.clone(),
                start_index,
                vulkano::sync::PipelineStage::TopOfPipe,
            ) {
                error!("Failed to write start timestamp: {}", e);
                return None;
            }

            Some(Self {
                builder: builder as *mut _,
                pool,
                end_index,
            })
        } else {
            None
        }
    }
}

impl Drop for GpuProfileScope {
    fn drop(&mut self) {
        // Write end timestamp
        unsafe {
            let builder = &mut *self.builder;
            if let Err(e) = GpuProfiler::write_timestamp(
                builder,
                self.pool.clone(),
                self.end_index,
                vulkano::sync::PipelineStage::BottomOfPipe,
            ) {
                error!("Failed to write end timestamp: {}", e);
            }
        }
    }
}
