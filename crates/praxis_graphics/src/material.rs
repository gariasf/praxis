//! Material system for defining surface properties.
//!
//! This module provides a basic material system that defines how surfaces
//! should be rendered. Currently supports albedo (base color) textures and
//! basic material properties.

use crate::texture::Texture;

/// Material properties for shader uniforms.
///
/// These properties can be uploaded to the GPU as uniform data to control
/// material appearance beyond textures.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialProperties {
    /// Base color tint (RGBA). Multiplied with the albedo texture.
    pub base_color: [f32; 4],

    /// Metallic factor [0.0, 1.0]. Controls how metallic the surface appears.
    pub metallic: f32,

    /// Roughness factor [0.0, 1.0]. Controls surface roughness (0 = smooth, 1 = rough).
    pub roughness: f32,

    /// Emissive strength. Controls how much the material glows.
    pub emissive_strength: f32,

    /// Padding for alignment.
    _padding: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive_strength: 0.0,
            _padding: 0.0,
        }
    }
}

impl MaterialProperties {
    /// Creates default material properties (white, non-metallic, medium roughness).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base color tint.
    pub fn with_base_color(mut self, color: [f32; 4]) -> Self {
        self.base_color = color;
        self
    }

    /// Sets the metallic factor [0.0, 1.0].
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    /// Sets the roughness factor [0.0, 1.0].
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Sets the emissive strength.
    pub fn with_emissive_strength(mut self, strength: f32) -> Self {
        self.emissive_strength = strength;
        self
    }
}

/// Basic material with albedo texture and properties.
///
/// A material defines the visual properties of a surface. This basic implementation
/// supports a single albedo (base color) texture that is multiplied with the vertex color,
/// along with basic material properties for PBR-style rendering.
#[derive(Clone)]
pub struct Material {
    /// Albedo (base color) texture.
    ///
    /// This texture defines the base color of the material. It is multiplied
    /// with the vertex color and base color tint in the fragment shader.
    pub albedo_texture: Texture,

    /// Material properties (metallic, roughness, emissive, etc.).
    pub properties: MaterialProperties,
}

impl Material {
    /// Creates a new material with the given albedo texture and default properties.
    ///
    /// # Arguments
    ///
    /// * `albedo_texture` - The albedo texture for this material
    pub fn new(albedo_texture: Texture) -> Self {
        Self {
            albedo_texture,
            properties: MaterialProperties::default(),
        }
    }

    /// Creates a new material with the given texture and properties.
    ///
    /// # Arguments
    ///
    /// * `albedo_texture` - The albedo texture for this material
    /// * `properties` - Material properties (metallic, roughness, etc.)
    pub fn with_properties(albedo_texture: Texture, properties: MaterialProperties) -> Self {
        Self {
            albedo_texture,
            properties,
        }
    }

    /// Sets the material properties.
    pub fn set_properties(&mut self, properties: MaterialProperties) {
        self.properties = properties;
    }
}
