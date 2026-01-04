//! Terrain material system with texture splatting.

use praxis_graphics::MaterialProperties;

/// A material layer for terrain texture splatting.
#[derive(Debug, Clone)]
pub struct TerrainMaterialLayer {
    /// Name identifier for this layer.
    pub name: String,

    /// Albedo/diffuse texture name.
    pub albedo_texture: String,

    /// Normal map texture name (optional).
    pub normal_texture: Option<String>,

    /// PBR material properties.
    pub properties: MaterialProperties,

    /// Minimum height at which this layer is applied.
    pub min_height: f32,

    /// Maximum height at which this layer is applied.
    pub max_height: f32,

    /// Minimum slope angle (in degrees) for this layer.
    pub min_slope: f32,

    /// Maximum slope angle (in degrees) for this layer.
    pub max_slope: f32,

    /// Texture tiling scale.
    pub tiling: f32,
}

impl TerrainMaterialLayer {
    /// Creates a new material layer with height-based blending.
    pub fn new(
        name: impl Into<String>,
        albedo_texture: impl Into<String>,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        Self {
            name: name.into(),
            albedo_texture: albedo_texture.into(),
            normal_texture: None,
            properties: MaterialProperties::default(),
            min_height,
            max_height,
            min_slope: 0.0,
            max_slope: 90.0,
            tiling: 1.0,
        }
    }

    /// Sets the normal map texture.
    pub fn with_normal(mut self, normal_texture: impl Into<String>) -> Self {
        self.normal_texture = Some(normal_texture.into());
        self
    }

    /// Sets the material properties.
    pub fn with_properties(mut self, properties: MaterialProperties) -> Self {
        self.properties = properties;
        self
    }

    /// Sets the slope range for this layer.
    pub fn with_slope(mut self, min_slope: f32, max_slope: f32) -> Self {
        self.min_slope = min_slope;
        self.max_slope = max_slope;
        self
    }

    /// Sets the texture tiling scale.
    pub fn with_tiling(mut self, tiling: f32) -> Self {
        self.tiling = tiling;
        self
    }

    /// Calculates the blend weight for this layer at a given height and slope.
    pub fn calculate_weight(&self, height: f32, slope_degrees: f32) -> f32 {
        let height_weight = if height < self.min_height || height > self.max_height {
            0.0
        } else if height < self.min_height + 5.0 {
            (height - self.min_height) / 5.0
        } else if height > self.max_height - 5.0 {
            (self.max_height - height) / 5.0
        } else {
            1.0
        };

        let slope_weight = if slope_degrees < self.min_slope || slope_degrees > self.max_slope {
            0.0
        } else {
            1.0
        };

        height_weight * slope_weight
    }
}

/// Terrain material configuration with multiple layers.
pub struct TerrainMaterial {
    /// Material layers for texture splatting.
    pub layers: Vec<TerrainMaterialLayer>,
}

impl TerrainMaterial {
    /// Creates a new terrain material with no layers.
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

    /// Calculates blend weights for all layers at a given height and slope.
    pub fn calculate_weights(&self, height: f32, slope_degrees: f32) -> Vec<f32> {
        let mut weights: Vec<f32> = self
            .layers
            .iter()
            .map(|layer| layer.calculate_weight(height, slope_degrees))
            .collect();

        let sum: f32 = weights.iter().sum();
        if sum > 0.0 {
            for weight in &mut weights {
                *weight /= sum;
            }
        }

        weights
    }
}

impl Default for TerrainMaterial {
    fn default() -> Self {
        Self::new()
    }
}
