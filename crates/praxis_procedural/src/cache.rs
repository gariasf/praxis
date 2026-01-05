//! Caching system for procedurally generated textures.
//!
//! This module provides a caching layer to avoid redundant texture generation.
//! Textures are cached based on their generation parameters and graph structure.

use crate::generator::TextureGenerationParams;
use crate::graph::TextureGraph;
use praxis_utils::{debug, info, trace};
use std::collections::HashMap;

/// Key for identifying cached textures.
///
/// The key is based on the texture graph structure and generation parameters.
/// Two textures with the same key are guaranteed to be identical.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureCacheKey {
    /// Hash of the texture graph structure
    graph_hash: u64,
    /// Width of the texture
    width: u32,
    /// Height of the texture
    height: u32,
    /// Random seed used for generation
    seed: u32,
}

impl TextureCacheKey {
    /// Creates a new cache key from a texture graph and parameters.
    pub fn new(graph: &TextureGraph, params: TextureGenerationParams) -> Self {
        Self {
            graph_hash: Self::hash_graph(graph),
            width: params.width,
            height: params.height,
            seed: params.seed,
        }
    }

    fn hash_graph(graph: &TextureGraph) -> u64 {
        use seahash::SeaHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = SeaHasher::new();

        if let Some(output) = graph.output() {
            output.hash(&mut hasher);
        }

        let mut nodes: Vec<_> = graph.nodes().collect();
        nodes.sort_by_key(|(id, _)| id.0);

        for (id, node) in nodes {
            id.hash(&mut hasher);

            let node_bytes = format!("{node:?}");
            node_bytes.hash(&mut hasher);
        }

        graph.seed().hash(&mut hasher);

        hasher.finish()
    }
}

/// Cache entry for a generated texture.
#[derive(Clone)]
pub struct CachedTexture {
    /// RGBA8 texture data
    pub data: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Number of times this texture has been accessed
    pub access_count: u64,
}

/// Statistics about the texture cache.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStatistics {
    /// Total number of cache lookups
    pub total_lookups: u64,
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Current number of cached textures
    pub cached_count: usize,
    /// Total memory used by cached textures (bytes)
    pub memory_used: usize,
}

impl CacheStatistics {
    /// Calculates the cache hit rate as a percentage.
    pub fn hit_rate(&self) -> f64 {
        if self.total_lookups == 0 {
            0.0
        } else {
            (self.hits as f64 / self.total_lookups as f64) * 100.0
        }
    }
}

/// Cache for procedurally generated textures.
///
/// This cache stores generated texture data in memory to avoid redundant
/// generation of identical textures. It provides automatic eviction when
/// the cache grows too large.
pub struct ProceduralTextureCache {
    /// Cached textures
    cache: HashMap<TextureCacheKey, CachedTexture>,
    /// Maximum number of textures to cache
    max_entries: usize,
    /// Maximum memory usage in bytes
    max_memory: usize,
    /// Current memory usage in bytes
    current_memory: usize,
    /// Cache statistics
    stats: CacheStatistics,
}

impl ProceduralTextureCache {
    /// Creates a new texture cache.
    ///
    /// # Arguments
    ///
    /// * `max_entries` - Maximum number of textures to cache (0 = unlimited)
    /// * `max_memory` - Maximum memory usage in bytes (0 = unlimited)
    pub fn new(max_entries: usize, max_memory: usize) -> Self {
        info!(
            "Created procedural texture cache (max entries: {}, max memory: {} MB)",
            if max_entries == 0 {
                "unlimited".to_string()
            } else {
                max_entries.to_string()
            },
            if max_memory == 0 {
                "unlimited".to_string()
            } else {
                format!("{}", max_memory / 1024 / 1024)
            }
        );

        Self {
            cache: HashMap::new(),
            max_entries,
            max_memory,
            current_memory: 0,
            stats: CacheStatistics::default(),
        }
    }

    /// Creates a new cache with default limits.
    ///
    /// Default limits: 1000 entries, 512 MB memory
    pub fn with_defaults() -> Self {
        Self::new(1000, 512 * 1024 * 1024)
    }

    /// Looks up a texture in the cache.
    ///
    /// Returns the cached texture data if found, otherwise returns `None`.
    pub fn get(&mut self, key: &TextureCacheKey) -> Option<Vec<u8>> {
        self.stats.total_lookups += 1;

        if let Some(entry) = self.cache.get_mut(key) {
            self.stats.hits += 1;
            entry.access_count += 1;
            trace!("Cache hit for texture {}x{}", entry.width, entry.height);
            Some(entry.data.clone())
        } else {
            self.stats.misses += 1;
            trace!("Cache miss");
            None
        }
    }

    /// Inserts a texture into the cache.
    ///
    /// If the cache is full, the least recently used texture will be evicted.
    pub fn insert(&mut self, key: TextureCacheKey, data: Vec<u8>, width: u32, height: u32) {
        let entry_size = data.len();

        while self.should_evict(entry_size) {
            self.evict_lru();
        }

        self.cache.insert(
            key,
            CachedTexture {
                data,
                width,
                height,
                access_count: 0,
            },
        );

        self.current_memory += entry_size;
        self.stats.cached_count = self.cache.len();
        self.stats.memory_used = self.current_memory;

        debug!(
            "Cached texture {}x{} ({} bytes, {} total cached)",
            width,
            height,
            entry_size,
            self.cache.len()
        );
    }

    /// Clears the entire cache.
    pub fn clear(&mut self) {
        let count = self.cache.len();
        self.cache.clear();
        self.current_memory = 0;
        self.stats.cached_count = 0;
        self.stats.memory_used = 0;
        info!("Cleared texture cache ({} textures)", count);
    }

    /// Gets cache statistics.
    pub fn statistics(&self) -> CacheStatistics {
        self.stats
    }

    /// Resets cache statistics.
    pub fn reset_statistics(&mut self) {
        self.stats.total_lookups = 0;
        self.stats.hits = 0;
        self.stats.misses = 0;
    }

    fn should_evict(&self, new_entry_size: usize) -> bool {
        if self.cache.is_empty() {
            return false;
        }

        let exceeds_count = self.max_entries > 0 && self.cache.len() >= self.max_entries;
        let exceeds_memory =
            self.max_memory > 0 && self.current_memory + new_entry_size > self.max_memory;

        exceeds_count || exceeds_memory
    }

    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        let lru_key = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            if let Some(entry) = self.cache.remove(&key) {
                self.current_memory -= entry.data.len();
                self.stats.cached_count = self.cache.len();
                self.stats.memory_used = self.current_memory;
                debug!(
                    "Evicted texture {}x{} from cache ({} remaining)",
                    entry.width,
                    entry.height,
                    self.cache.len()
                );
            }
        }
    }

    /// Removes specific texture from cache by key.
    pub fn remove(&mut self, key: &TextureCacheKey) -> bool {
        if let Some(entry) = self.cache.remove(key) {
            self.current_memory -= entry.data.len();
            self.stats.cached_count = self.cache.len();
            self.stats.memory_used = self.current_memory;
            debug!(
                "Removed texture {}x{} from cache",
                entry.width, entry.height
            );
            true
        } else {
            false
        }
    }

    /// Returns the number of cached textures.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Returns current memory usage in bytes.
    pub fn memory_usage(&self) -> usize {
        self.current_memory
    }

    /// Performs cache maintenance, evicting entries if needed.
    ///
    /// This can be called periodically to keep the cache within limits.
    pub fn maintain(&mut self) {
        while self.should_evict(0) {
            self.evict_lru();
        }
    }
}

impl Default for ProceduralTextureCache {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{NoiseType, TextureNode};

    fn create_simple_graph() -> TextureGraph {
        let mut graph = TextureGraph::new();
        let node = graph.add_node(TextureNode::Noise {
            noise_type: NoiseType::Perlin,
            scale: 1.0,
            octaves: 1,
            persistence: 0.5,
            lacunarity: 2.0,
        });
        graph.set_output(node);
        graph
    }

    #[test]
    fn test_cache_key_equality() {
        let graph1 = create_simple_graph();
        let graph2 = create_simple_graph();

        let params = TextureGenerationParams {
            width: 512,
            height: 512,
            seed: 0,
        };

        let key1 = TextureCacheKey::new(&graph1, params);
        let key2 = TextureCacheKey::new(&graph2, params);

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_insertion_and_retrieval() {
        let mut cache = ProceduralTextureCache::new(10, 1024 * 1024);

        let graph = create_simple_graph();
        let params = TextureGenerationParams {
            width: 64,
            height: 64,
            seed: 0,
        };
        let key = TextureCacheKey::new(&graph, params);

        let data = vec![255u8; 64 * 64 * 4];
        cache.insert(key.clone(), data.clone(), 64, 64);

        assert_eq!(cache.len(), 1);

        let retrieved = cache.get(&key).unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_cache_eviction_by_count() {
        let mut cache = ProceduralTextureCache::new(2, 0);

        for i in 0..3 {
            let mut graph = create_simple_graph();
            graph.set_seed(i);
            let params = TextureGenerationParams {
                width: 64,
                height: 64,
                seed: i,
            };
            let key = TextureCacheKey::new(&graph, params);
            let data = vec![i as u8; 64 * 64 * 4];
            cache.insert(key, data, 64, 64);
        }

        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_statistics() {
        let mut cache = ProceduralTextureCache::new(10, 1024 * 1024);

        let graph = create_simple_graph();
        let params = TextureGenerationParams {
            width: 64,
            height: 64,
            seed: 0,
        };
        let key = TextureCacheKey::new(&graph, params);

        cache.get(&key);
        cache.insert(key.clone(), vec![0; 64 * 64 * 4], 64, 64);
        cache.get(&key);

        let stats = cache.statistics();
        assert_eq!(stats.total_lookups, 2);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }
}
