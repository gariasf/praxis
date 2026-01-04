//! Terrain chunk management.

use crate::heightmap::TerrainHeightmap;
use crate::lod::ChunkLod;
use praxis_graphics::GpuMesh;
use praxis_math::Vec3;

/// Unique identifier for a terrain chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerrainChunkId {
    /// X coordinate in the chunk grid.
    pub x: i32,
    /// Z coordinate in the chunk grid.
    pub z: i32,
}

impl TerrainChunkId {
    /// Creates a new chunk ID.
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// Calculates the world-space position of the chunk's origin.
    pub fn world_position(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(self.x as f32 * chunk_size, 0.0, self.z as f32 * chunk_size)
    }

    /// Calculates the distance from this chunk to a point.
    pub fn distance_to(&self, point: Vec3, chunk_size: f32) -> f32 {
        let chunk_center =
            self.world_position(chunk_size) + Vec3::new(chunk_size * 0.5, 0.0, chunk_size * 0.5);
        (point - chunk_center).length()
    }
}

/// A terrain chunk containing mesh data and LOD information.
pub struct TerrainChunk {
    /// Unique identifier for this chunk.
    pub id: TerrainChunkId,

    /// LOD information for this chunk.
    pub lod: ChunkLod,

    /// GPU meshes for each LOD level.
    pub meshes: Vec<Option<GpuMesh>>,

    /// Bounding box minimum point.
    pub bounds_min: Vec3,

    /// Bounding box maximum point.
    pub bounds_max: Vec3,

    /// Whether this chunk needs mesh regeneration.
    pub dirty: bool,
}

impl TerrainChunk {
    /// Creates a new terrain chunk.
    pub fn new(id: TerrainChunkId, chunk_size: f32, lod_levels: usize) -> Self {
        let world_pos = id.world_position(chunk_size);

        Self {
            id,
            lod: ChunkLod::new(lod_levels),
            meshes: vec![None; lod_levels],
            bounds_min: world_pos,
            bounds_max: world_pos + Vec3::new(chunk_size, 0.0, chunk_size),
            dirty: true,
        }
    }

    /// Updates the bounding box based on heightmap data.
    pub fn update_bounds(&mut self, heightmap: &TerrainHeightmap, chunk_size: f32) {
        let grid_start_x = (self.id.x * chunk_size as i32) as u32;
        let grid_start_z = (self.id.z * chunk_size as i32) as u32;
        let grid_size = chunk_size as u32;

        let mut min_height = f32::MAX;
        let mut max_height = f32::MIN;

        for z in grid_start_z..grid_start_z + grid_size {
            for x in grid_start_x..grid_start_x + grid_size {
                let h = heightmap.get_height(x, z);
                min_height = min_height.min(h);
                max_height = max_height.max(h);
            }
        }

        self.bounds_min.y = min_height;
        self.bounds_max.y = max_height;
    }

    /// Checks if a point is inside this chunk's bounds.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.bounds_min.x
            && point.x <= self.bounds_max.x
            && point.y >= self.bounds_min.y
            && point.y <= self.bounds_max.y
            && point.z >= self.bounds_min.z
            && point.z <= self.bounds_max.z
    }

    /// Marks this chunk as needing mesh regeneration.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag after mesh regeneration.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
