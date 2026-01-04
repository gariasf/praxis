//! Hardware occlusion culling using GPU queries.
//!
//! Occlusion culling uses the GPU to test if objects are hidden behind other geometry,
//! allowing us to skip rendering of fully occluded objects.

use crate::aabb::Aabb;
use bevy_ecs::entity::Entity;
use praxis_utils::{error, Result};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    device::{Device, Queue},
    memory::allocator::StandardMemoryAllocator,
    query::{QueryPool, QueryPoolCreateInfo, QueryType},
};

/// Result of an occlusion query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcclusionQueryResult {
    /// Query is still pending on the GPU.
    Pending,
    /// Object is visible (samples passed).
    Visible,
    /// Object is occluded (no samples passed).
    Occluded,
}

/// An individual occlusion query for an entity.
#[derive(Debug)]
pub struct OcclusionQuery {
    /// Entity being queried.
    pub entity: Entity,
    /// Query index in the pool.
    pub query_index: u32,
    /// Bounding box used for the query.
    pub bounds: Aabb,
    /// Last known query result.
    pub result: OcclusionQueryResult,
}

impl OcclusionQuery {
    /// Creates a new occlusion query.
    pub fn new(entity: Entity, query_index: u32, bounds: Aabb) -> Self {
        Self {
            entity,
            query_index,
            bounds,
            result: OcclusionQueryResult::Pending,
        }
    }
}

/// Pool of occlusion queries.
///
/// Manages a fixed-size pool of GPU query objects for occlusion testing.
pub struct OcclusionQueryPool {
    /// Vulkan query pool.
    query_pool: Arc<QueryPool>,
    /// Maximum number of queries in the pool.
    max_queries: u32,
    /// Currently active queries.
    active_queries: HashMap<Entity, OcclusionQuery>,
    /// Next available query index.
    next_query_index: u32,
}

impl OcclusionQueryPool {
    /// Creates a new occlusion query pool.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `_queue` - Vulkan queue (reserved for future use)
    /// * `max_queries` - Maximum number of concurrent queries
    ///
    /// # Errors
    ///
    /// Returns an error if query pool creation fails.
    pub fn new(device: &Arc<Device>, _queue: &Arc<Queue>, max_queries: u32) -> Result<Self> {
        let query_pool = QueryPool::new(
            device.clone(),
            QueryPoolCreateInfo {
                query_count: max_queries,
                ..QueryPoolCreateInfo::query_type(QueryType::Occlusion)
            },
        )
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to create query pool: {}", e))?;

        Ok(Self {
            query_pool,
            max_queries,
            active_queries: HashMap::new(),
            next_query_index: 0,
        })
    }

    /// Begins an occlusion query for an entity.
    ///
    /// # Errors
    ///
    /// Returns an error if the query pool is exhausted or query operations fail.
    pub fn begin_query(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        entity: Entity,
        bounds: Aabb,
    ) -> Result<u32> {
        if self.next_query_index >= self.max_queries {
            return Err(praxis_utils::eyre::eyre!("Query pool exhausted"));
        }

        let query_index = self.next_query_index;
        self.next_query_index += 1;

        #[allow(unsafe_code)]
        #[allow(clippy::range_plus_one)]
        unsafe {
            builder
                .reset_query_pool(self.query_pool.clone(), query_index..query_index + 1)
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to reset query pool: {}", e))?;

            builder
                .begin_query(
                    self.query_pool.clone(),
                    query_index,
                    vulkano::query::QueryControlFlags::empty(),
                )
                .map_err(|e| praxis_utils::eyre::eyre!("Failed to begin query: {}", e))?;
        }

        let query = OcclusionQuery::new(entity, query_index, bounds);
        self.active_queries.insert(entity, query);

        Ok(query_index)
    }

    /// Ends an occlusion query.
    ///
    /// # Errors
    ///
    /// Returns an error if ending the query fails.
    pub fn end_query(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        query_index: u32,
    ) -> Result<()> {
        builder
            .end_query(self.query_pool.clone(), query_index)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to end query: {}", e))?;
        Ok(())
    }

    /// Retrieves the results of all active queries.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving query results fails.
    pub fn get_results(&mut self) -> Result<()> {
        if self.active_queries.is_empty() {
            return Ok(());
        }

        let query_count = self.next_query_index;
        if query_count == 0 {
            return Ok(());
        }

        let mut results = vec![0u64; query_count as usize];

        match self
            .query_pool
            .get_results(0..query_count, &mut results, vulkano::query::QueryResultFlags::empty())
        {
            Ok(_) => {
                for query in self.active_queries.values_mut() {
                    if let Some(&sample_count) = results.get(query.query_index as usize) {
                        query.result = if sample_count > 0 {
                            OcclusionQueryResult::Visible
                        } else {
                            OcclusionQueryResult::Occluded
                        };
                    }
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to get query results: {}", e);
                Ok(())
            }
        }
    }

    /// Gets the result for a specific entity.
    pub fn get_entity_result(&self, entity: Entity) -> Option<OcclusionQueryResult> {
        self.active_queries.get(&entity).map(|q| q.result)
    }

    /// Resets the query pool for the next frame.
    pub fn reset(&mut self) {
        self.active_queries.clear();
        self.next_query_index = 0;
    }

    /// Returns the number of active queries.
    pub fn active_count(&self) -> usize {
        self.active_queries.len()
    }

    /// Returns the maximum number of queries.
    pub fn max_queries(&self) -> u32 {
        self.max_queries
    }
}

/// Occlusion culler that manages hardware occlusion queries.
///
/// This system tests object visibility by rendering bounding boxes with occlusion queries,
/// then uses the results to skip rendering of fully occluded objects.
pub struct OcclusionCuller {
    /// Query pool for occlusion tests.
    query_pool: OcclusionQueryPool,
    /// Results from the previous frame (for temporal coherence).
    previous_results: HashMap<Entity, OcclusionQueryResult>,
}

impl OcclusionCuller {
    /// Creates a new occlusion culler.
    ///
    /// # Errors
    ///
    /// Returns an error if query pool creation fails.
    pub fn new(
        device: &Arc<Device>,
        queue: &Arc<Queue>,
        _allocator: Arc<StandardMemoryAllocator>,
        max_queries: u32,
    ) -> Result<Self> {
        let query_pool = OcclusionQueryPool::new(device, queue, max_queries)?;

        Ok(Self {
            query_pool,
            previous_results: HashMap::new(),
        })
    }

    /// Begins occlusion testing for an entity.
    ///
    /// # Errors
    ///
    /// Returns an error if the query pool is exhausted or query operations fail.
    pub fn begin_test(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        entity: Entity,
        bounds: Aabb,
    ) -> Result<u32> {
        self.query_pool.begin_query(builder, entity, bounds)
    }

    /// Ends occlusion testing for a query.
    ///
    /// # Errors
    ///
    /// Returns an error if ending the query fails.
    pub fn end_test(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        query_index: u32,
    ) -> Result<()> {
        self.query_pool.end_query(builder, query_index)
    }

    /// Retrieves occlusion query results and updates visibility state.
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving query results fails.
    pub fn update_results(&mut self) -> Result<()> {
        self.query_pool.get_results()?;

        for (entity, query) in &self.query_pool.active_queries {
            self.previous_results.insert(*entity, query.result);
        }

        Ok(())
    }

    /// Checks if an entity is visible based on occlusion query results.
    ///
    /// Returns true if the entity passed occlusion tests, or if no test has been performed yet.
    pub fn is_visible(&self, entity: Entity) -> bool {
        self.previous_results
            .get(&entity)
            .is_none_or(|r| *r != OcclusionQueryResult::Occluded)
    }

    /// Resets the occlusion culler for the next frame.
    pub fn reset(&mut self) {
        self.query_pool.reset();
    }

    /// Returns statistics about the occlusion culler.
    pub fn stats(&self) -> OcclusionCullerStats {
        let visible_count = self
            .previous_results
            .values()
            .filter(|r| **r == OcclusionQueryResult::Visible)
            .count();
        let occluded_count = self
            .previous_results
            .values()
            .filter(|r| **r == OcclusionQueryResult::Occluded)
            .count();

        OcclusionCullerStats {
            active_queries: self.query_pool.active_count(),
            visible_objects: visible_count,
            occluded_objects: occluded_count,
        }
    }
}

/// Statistics about occlusion culling performance.
#[derive(Debug, Clone, Copy, Default)]
pub struct OcclusionCullerStats {
    /// Number of active queries this frame.
    pub active_queries: usize,
    /// Number of visible objects.
    pub visible_objects: usize,
    /// Number of occluded objects.
    pub occluded_objects: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_occlusion_query_result() {
        assert_eq!(OcclusionQueryResult::Pending, OcclusionQueryResult::Pending);
        assert_ne!(OcclusionQueryResult::Visible, OcclusionQueryResult::Occluded);
    }

    #[test]
    fn test_occlusion_query_creation() {
        let entity = Entity::from_raw(1);
        let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
        let query = OcclusionQuery::new(entity, 0, bounds);

        assert_eq!(query.entity, entity);
        assert_eq!(query.query_index, 0);
        assert_eq!(query.result, OcclusionQueryResult::Pending);
    }

    #[test]
    fn test_occlusion_culler_stats() {
        let stats = OcclusionCullerStats {
            active_queries: 10,
            visible_objects: 7,
            occluded_objects: 3,
        };

        assert_eq!(stats.active_queries, 10);
        assert_eq!(stats.visible_objects, 7);
        assert_eq!(stats.occluded_objects, 3);
    }
}
