//! Material system for defining surface properties.
//!
//! This module provides a basic material system that defines how surfaces
//! should be rendered. Currently supports albedo (base color) textures.

use crate::texture::Texture;

/// Basic material with albedo texture.
///
/// A material defines the visual properties of a surface. This basic implementation
/// supports a single albedo (base color) texture that is multiplied with the vertex color.
#[derive(Clone)]
pub struct Material {
    /// Albedo (base color) texture.
    ///
    /// This texture defines the base color of the material. It is multiplied
    /// with the vertex color in the fragment shader.
    pub albedo_texture: Texture,
}

impl Material {
    /// Creates a new material with the given albedo texture.
    ///
    /// # Arguments
    ///
    /// * `albedo_texture` - The albedo texture for this material
    pub fn new(albedo_texture: Texture) -> Self {
        Self { albedo_texture }
    }
}
