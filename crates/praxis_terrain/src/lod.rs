//! Level of Detail (LOD) system for terrain chunks.

use praxis_math::Vec3;

/// LOD information for a single terrain chunk.
#[derive(Debug, Clone)]
pub struct ChunkLod {
    /// Current active LOD level (0 = highest detail).
    pub current_level: usize,

    /// Target LOD level based on camera distance.
    pub target_level: usize,

    /// Number of LOD levels available.
    pub num_levels: usize,

    /// Transition progress from current to target level (0.0 to 1.0).
    pub transition_t: f32,
}

impl ChunkLod {
    /// Creates a new chunk LOD with the specified number of levels.
    pub fn new(num_levels: usize) -> Self {
        Self {
            current_level: 0,
            target_level: 0,
            num_levels,
            transition_t: 0.0,
        }
    }

    /// Updates the target LOD level based on distance.
    pub fn update_target(&mut self, distance: f32, lod_distances: &[f32]) {
        self.target_level = lod_distances
            .iter()
            .position(|&d| distance < d)
            .unwrap_or(self.num_levels - 1)
            .min(self.num_levels - 1);
    }

    /// Updates the transition between LOD levels.
    pub fn update_transition(&mut self, delta_time: f32, transition_speed: f32) {
        if self.current_level != self.target_level {
            self.transition_t += delta_time * transition_speed;

            if self.transition_t >= 1.0 {
                self.current_level = self.target_level;
                self.transition_t = 0.0;
            }
        } else {
            self.transition_t = 0.0;
        }
    }

    /// Returns whether the chunk is currently transitioning between LOD levels.
    pub fn is_transitioning(&self) -> bool {
        self.current_level != self.target_level && self.transition_t > 0.0
    }
}

/// Manager for terrain LOD system.
pub struct TerrainLodManager {
    /// Distance thresholds for each LOD level.
    pub lod_distances: Vec<f32>,

    /// Speed of LOD transitions (higher = faster).
    pub transition_speed: f32,
}

impl TerrainLodManager {
    /// Creates a new LOD manager with the specified distance thresholds.
    pub fn new(lod_distances: Vec<f32>) -> Self {
        Self {
            lod_distances,
            transition_speed: 2.0,
        }
    }

    /// Creates a LOD manager with default distances.
    pub fn default_distances(num_levels: usize) -> Self {
        let mut distances = Vec::new();
        let mut dist = 50.0;

        for _ in 0..num_levels {
            distances.push(dist);
            dist *= 2.0;
        }

        Self::new(distances)
    }

    /// Updates the LOD level for a chunk based on camera position.
    pub fn update_chunk_lod(&self, chunk_lod: &mut ChunkLod, chunk_center: Vec3, camera_pos: Vec3) {
        let distance = (camera_pos - chunk_center).length();
        chunk_lod.update_target(distance, &self.lod_distances);
    }

    /// Gets the vertex density multiplier for a given LOD level.
    ///
    /// LOD 0 = 1.0 (full density)
    /// LOD 1 = 0.5 (half density)
    /// LOD 2 = 0.25 (quarter density)
    /// etc.
    pub fn get_lod_density(&self, lod_level: usize) -> f32 {
        1.0 / (1 << lod_level) as f32
    }

    /// Calculates the number of vertices for a chunk at a given LOD level.
    pub fn get_vertex_count(&self, base_size: u32, lod_level: usize) -> u32 {
        let divisor = 1 << lod_level;
        (base_size / divisor).max(2)
    }
}

impl Default for TerrainLodManager {
    fn default() -> Self {
        Self::default_distances(4)
    }
}
