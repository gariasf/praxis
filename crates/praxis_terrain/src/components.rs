//! ECS components for terrain system.

use crate::chunk::TerrainChunkId;
use crate::material::TerrainMaterialLayer;
use crate::vegetation::VegetationInstance;
use bevy_ecs::component::Component;

/// Component marking an entity as terrain.
#[derive(Component, Debug, Clone)]
pub struct Terrain {
    /// Identifier for the terrain system this entity belongs to.
    pub terrain_id: String,

    /// Chunk ID if this is a terrain chunk entity.
    pub chunk_id: Option<TerrainChunkId>,
}

impl Terrain {
    /// Creates a new terrain component.
    pub fn new(terrain_id: impl Into<String>) -> Self {
        Self {
            terrain_id: terrain_id.into(),
            chunk_id: None,
        }
    }

    /// Creates a terrain component for a specific chunk.
    pub fn for_chunk(terrain_id: impl Into<String>, chunk_id: TerrainChunkId) -> Self {
        Self {
            terrain_id: terrain_id.into(),
            chunk_id: Some(chunk_id),
        }
    }
}

/// Component storing terrain material layers.
#[derive(Component, Debug, Clone)]
pub struct TerrainMaterialLayers {
    /// Material layers for texture splatting.
    pub layers: Vec<TerrainMaterialLayer>,
}

impl TerrainMaterialLayers {
    /// Creates a new material layers component.
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Adds a material layer.
    pub fn add_layer(&mut self, layer: TerrainMaterialLayer) {
        self.layers.push(layer);
    }

    /// Gets a layer by name.
    pub fn get_layer(&self, name: &str) -> Option<&TerrainMaterialLayer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Gets a mutable reference to a layer by name.
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut TerrainMaterialLayer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }
}

impl Default for TerrainMaterialLayers {
    fn default() -> Self {
        Self::new()
    }
}

/// Component storing vegetation instances for a terrain area.
#[derive(Component, Debug, Clone)]
pub struct VegetationInstances {
    /// Name identifier for the vegetation layer.
    pub layer_name: String,

    /// All vegetation instances.
    pub instances: Vec<VegetationInstance>,
}

impl VegetationInstances {
    /// Creates a new vegetation instances component.
    pub fn new(layer_name: impl Into<String>) -> Self {
        Self {
            layer_name: layer_name.into(),
            instances: Vec::new(),
        }
    }

    /// Adds an instance.
    pub fn add_instance(&mut self, instance: VegetationInstance) {
        self.instances.push(instance);
    }

    /// Gets the number of instances.
    pub fn count(&self) -> usize {
        self.instances.len()
    }
}
