//! Material system for defining surface properties.
//!
//! This module provides a material system that defines how surfaces should be rendered,
//! including texture references and material properties for efficient GPU resource access.
//!
//! # Material System Overview
//!
//! Materials define the visual properties of a surface, including:
//! - Albedo (base color) texture
//! - Material properties (metallic, roughness, emissive strength)
//! - Future: Normal maps, metallic-roughness maps, etc.
//!
//! Each material stores references to its textures and properties, which are then used
//! during rendering to create descriptor sets that bind GPU resources to shaders.
//!
//! # Descriptor Set Management
//!
//! Materials themselves do NOT create or store descriptor sets. Instead, they provide
//! the texture and property data needed to create descriptor sets during rendering.
//!
//! This is because descriptor sets require binding multiple resources that are not
//! available at material creation time:
//! - Binding 0: Per-object uniforms (model/view/projection matrices) - frame-dependent
//! - Binding 1: Material texture sampler - provided by the material
//! - Binding 2: Lighting data - frame-dependent
//!
//! ## Why Per-Material Descriptor Sets Are More Efficient
//!
//! While materials don't store descriptor sets themselves, the rendering system can still
//! achieve per-material efficiency by grouping draw calls:
//!
//! **Rendering Strategy:**
//!
//! 1. **Group by Material**: Sort draw commands by material ID before rendering
//! 2. **Create Once Per Material**: For each unique material in a frame, create one
//!    descriptor set that combines the material's texture with the frame's lighting data
//! 3. **Update Per Object**: For each object using that material, only update the
//!    per-object uniform data (model matrix), then draw
//!
//! **Benefits:**
//!
//! - **Reduced Allocations**: 100 objects with 10 materials = 10 descriptor sets per frame
//!   (not 100), assuming lighting doesn't change
//! - **Fewer GPU Binds**: Bind material resources once per material, not per object
//! - **Better Cache Coherency**: GPU texture cache benefits from grouped material access
//!
//! ## Example: Material Sharing Benefits
//!
//! ```text
//! Scenario: Rendering 1000 objects with 10 different materials (100 objects per material)
//!
//! Naive Approach (per-object descriptor sets):
//! - Descriptor sets per frame: 1000
//! - Texture binds per frame: 1000
//! - Memory allocations: Very high
//!
//! Grouped Approach (group by material):
//! - Descriptor sets per frame: 10
//! - Texture binds per frame: 10
//! - Memory allocations: Low
//!
//! Performance improvement: 100x reduction in descriptor sets and texture binds
//! ```
//!
//! ## Descriptor Set Lifecycle
//!
//! When rendering with materials, descriptor sets follow this lifecycle:
//!
//! 1. **Frame Start**: Rendering system begins a new frame
//! 2. **Material Grouping**: Draw commands are sorted/grouped by material ID
//! 3. **Per-Material Setup**: For each unique material:
//!    - Create a descriptor set binding:
//!      * The material's texture at binding 1
//!      * The current frame's lighting data at binding 2
//!    - This descriptor set is used for all objects sharing this material
//! 4. **Per-Object Drawing**: For each object using the current material:
//!    - Update binding 0 with per-object uniforms (model matrix)
//!    - Draw the object
//! 5. **Frame End**: All descriptor sets are automatically reclaimed by the pool
//!
//! This approach balances efficiency (few descriptor sets) with flexibility (per-object
//! transforms and per-frame lighting).
//!
//! ## Future Optimizations
//!
//! Potential future improvements include:
//! - **Persistent Material Descriptor Sets**: Cache descriptor sets that only contain
//!   material textures (binding 1), reusing them across frames
//! - **Dynamic Uniform Buffers**: Use a single large buffer with dynamic offsets for
//!   per-object data instead of separate descriptor sets
//! - **Push Constants**: Use push constants for per-object transforms if they fit in
//!   the 128-byte limit

use crate::texture::Texture;
use praxis_utils::{debug, info};
use std::collections::HashMap;

/// Material properties for shader uniforms.
///
/// These properties can be uploaded to the GPU as uniform data to control
/// material appearance beyond textures.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
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
/// A material defines the visual properties of a surface. This implementation
/// supports a single albedo (base color) texture that is multiplied with the vertex color,
/// along with basic material properties for PBR-style rendering.
///
/// # Rendering Integration
///
/// Materials store texture references and properties but do not create descriptor sets
/// themselves. During rendering, the renderer creates descriptor sets that combine:
/// - Per-object uniforms (transforms)
/// - Material textures (from this struct)
/// - Per-frame lighting data
///
/// By grouping draw calls by material, the renderer can minimize descriptor set creation
/// and GPU binding operations. See the module-level documentation for details on
/// efficient material-based rendering strategies.
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

    /// Sets the albedo texture.
    ///
    /// This replaces the current texture with a new one.
    pub fn set_albedo_texture(&mut self, texture: Texture) {
        self.albedo_texture = texture;
    }

    /// Gets a reference to the albedo texture.
    pub fn albedo_texture(&self) -> &Texture {
        &self.albedo_texture
    }

    /// Gets a reference to the material properties.
    pub fn properties(&self) -> &MaterialProperties {
        &self.properties
    }
}

/// Material asset manager that caches loaded materials.
///
/// This manager maintains a cache of materials by name, avoiding redundant
/// material creation. It provides convenient methods for creating materials
/// from textures and managing the material cache.
///
/// # Shared Resource Management
///
/// The MaterialManager allows multiple entities to reference the same material
/// by name. When 100 entities use the "brick" material, they all reference the
/// same Material instance, which means:
///
/// - **Memory Efficiency**: Texture references are Arc-based, so cloning is cheap
/// - **Consistent Appearance**: All objects with the same material look identical
/// - **Easy Updates**: Changing a material affects all objects using it
///
/// # Rendering Integration
///
/// During rendering, the renderer can query materials by name and group draw calls
/// by material for optimal performance. See the module-level documentation for
/// details on efficient rendering strategies with materials.
///
/// # Usage Example
///
/// ```rust,no_run
/// use praxis_graphics::{MaterialManager, Texture};
/// use std::sync::Arc;
///
/// # fn example(texture: Texture) {
/// let mut material_manager = MaterialManager::new();
///
/// // Create a material from a texture
/// material_manager.create_material("brick", texture);
///
/// // Get a material for use during rendering
/// if let Some(material) = material_manager.get_material("brick") {
///     // Use material.albedo_texture() for rendering
/// }
/// # }
/// ```
pub struct MaterialManager {
    /// Map of material name to material.
    materials: HashMap<String, Material>,
}

impl MaterialManager {
    /// Creates a new material manager.
    pub fn new() -> Self {
        debug!("Creating MaterialManager");
        Self {
            materials: HashMap::new(),
        }
    }

    /// Creates a material from a texture and adds it to the cache.
    ///
    /// This creates a material with default properties. If a material with the same
    /// name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the material
    /// * `albedo_texture` - The albedo texture for the material
    pub fn create_material(&mut self, name: impl Into<String>, albedo_texture: Texture) {
        let name = name.into();
        debug!("Creating material '{}'", name);

        let material = Material::new(albedo_texture);
        self.materials.insert(name.clone(), material);
        info!("Material '{}' created and cached", name);
    }

    /// Creates a material with custom properties and adds it to the cache.
    ///
    /// If a material with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the material
    /// * `albedo_texture` - The albedo texture for the material
    /// * `properties` - Material properties (metallic, roughness, etc.)
    pub fn create_material_with_properties(
        &mut self,
        name: impl Into<String>,
        albedo_texture: Texture,
        properties: MaterialProperties,
    ) {
        let name = name.into();
        debug!("Creating material '{}' with custom properties", name);

        let material = Material::with_properties(albedo_texture, properties);
        self.materials.insert(name.clone(), material);
        info!("Material '{}' created and cached", name);
    }

    /// Adds a pre-created material to the cache.
    ///
    /// This is useful when you need to create a material with custom configuration
    /// before adding it to the manager.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the material
    /// * `material` - The material to add
    pub fn add_material(&mut self, name: impl Into<String>, material: Material) {
        let name = name.into();
        debug!("Adding material '{}' to cache", name);
        self.materials.insert(name, material);
    }

    /// Gets a material by name.
    ///
    /// Returns `None` if the material doesn't exist.
    pub fn get_material(&self, name: &str) -> Option<&Material> {
        self.materials.get(name)
    }

    /// Gets a mutable reference to a material by name.
    ///
    /// This allows modifying material properties or textures after creation.
    pub fn get_material_mut(&mut self, name: &str) -> Option<&mut Material> {
        self.materials.get_mut(name)
    }

    /// Checks if a material exists in the cache.
    pub fn contains_material(&self, name: &str) -> bool {
        self.materials.contains_key(name)
    }

    /// Removes a material from the cache.
    ///
    /// Returns `true` if the material existed and was removed.
    pub fn remove_material(&mut self, name: &str) -> bool {
        if self.materials.remove(name).is_some() {
            debug!("Material '{}' removed from cache", name);
            true
        } else {
            false
        }
    }

    /// Returns the number of cached materials.
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Clears all cached materials.
    pub fn clear(&mut self) {
        debug!("Clearing {} cached materials", self.materials.len());
        self.materials.clear();
    }
}

impl Default for MaterialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_properties_defaults() {
        let props = MaterialProperties::default();
        assert_eq!(props.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(props.metallic, 0.0);
        assert_eq!(props.roughness, 0.5);
        assert_eq!(props.emissive_strength, 0.0);
    }

    #[test]
    fn test_material_properties_builder() {
        let props = MaterialProperties::new()
            .with_base_color([0.5, 0.5, 0.5, 1.0])
            .with_metallic(0.8)
            .with_roughness(0.2)
            .with_emissive_strength(1.5);

        assert_eq!(props.base_color, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(props.metallic, 0.8);
        assert_eq!(props.roughness, 0.2);
        assert_eq!(props.emissive_strength, 1.5);
    }

    #[test]
    fn test_material_properties_clamping() {
        let props = MaterialProperties::new()
            .with_metallic(1.5) // Should clamp to 1.0
            .with_roughness(-0.5); // Should clamp to 0.0

        assert_eq!(props.metallic, 1.0);
        assert_eq!(props.roughness, 0.0);
    }

    #[test]
    fn test_material_manager_creation() {
        let manager = MaterialManager::new();
        assert_eq!(manager.material_count(), 0);
    }

    #[test]
    fn test_material_manager_contains() {
        let mut manager = MaterialManager::new();
        assert!(!manager.contains_material("test"));
    }

    #[test]
    fn test_material_manager_remove_nonexistent() {
        let mut manager = MaterialManager::new();
        assert!(!manager.remove_material("nonexistent"));
    }

    #[test]
    fn test_material_manager_clear() {
        let mut manager = MaterialManager::new();
        manager.clear();
        assert_eq!(manager.material_count(), 0);
    }

    #[test]
    fn test_material_properties_metallic_clamping_upper() {
        let props = MaterialProperties::new().with_metallic(2.5);
        assert_eq!(props.metallic, 1.0);
    }

    #[test]
    fn test_material_properties_metallic_clamping_lower() {
        let props = MaterialProperties::new().with_metallic(-1.0);
        assert_eq!(props.metallic, 0.0);
    }

    #[test]
    fn test_material_properties_roughness_clamping_upper() {
        let props = MaterialProperties::new().with_roughness(10.0);
        assert_eq!(props.roughness, 1.0);
    }

    #[test]
    fn test_material_properties_roughness_clamping_lower() {
        let props = MaterialProperties::new().with_roughness(-5.0);
        assert_eq!(props.roughness, 0.0);
    }

    #[test]
    fn test_material_properties_emissive_no_clamping() {
        let props = MaterialProperties::new().with_emissive_strength(5.0);
        assert_eq!(props.emissive_strength, 5.0);

        let props2 = MaterialProperties::new().with_emissive_strength(-1.0);
        assert_eq!(props2.emissive_strength, -1.0);
    }

    #[test]
    fn test_material_properties_base_color() {
        let color = [0.2, 0.4, 0.6, 0.8];
        let props = MaterialProperties::new().with_base_color(color);
        assert_eq!(props.base_color, color);
    }

    #[test]
    fn test_material_properties_chaining() {
        let props = MaterialProperties::new()
            .with_base_color([0.5, 0.5, 0.5, 1.0])
            .with_metallic(0.7)
            .with_roughness(0.3)
            .with_emissive_strength(2.0);

        assert_eq!(props.base_color, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(props.metallic, 0.7);
        assert_eq!(props.roughness, 0.3);
        assert_eq!(props.emissive_strength, 2.0);
    }

    #[test]
    fn test_material_properties_size() {
        assert_eq!(std::mem::size_of::<MaterialProperties>(), 32);
    }

    #[test]
    fn test_material_properties_alignment() {
        assert_eq!(std::mem::align_of::<MaterialProperties>(), 4);
    }

    #[test]
    fn test_material_properties_pod() {
        let props = MaterialProperties::default();
        let bytes = bytemuck::bytes_of(&props);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_material_properties_equality() {
        let props1 = MaterialProperties::new().with_metallic(0.5);
        let props2 = MaterialProperties::new().with_metallic(0.5);
        assert_eq!(props1, props2);
    }

    #[test]
    fn test_material_properties_inequality() {
        let props1 = MaterialProperties::new().with_metallic(0.5);
        let props2 = MaterialProperties::new().with_metallic(0.6);
        assert_ne!(props1, props2);
    }

    #[test]
    fn test_material_properties_clone() {
        let props1 = MaterialProperties::new().with_roughness(0.8);
        let props2 = props1;
        assert_eq!(props1.roughness, props2.roughness);
    }

    #[test]
    fn test_material_properties_copy() {
        let props1 = MaterialProperties::new();
        let props2 = props1;
        assert_eq!(props1.metallic, props2.metallic);
    }

    #[test]
    fn test_material_properties_zero_metallic() {
        let props = MaterialProperties::new().with_metallic(0.0);
        assert_eq!(props.metallic, 0.0);
    }

    #[test]
    fn test_material_properties_full_metallic() {
        let props = MaterialProperties::new().with_metallic(1.0);
        assert_eq!(props.metallic, 1.0);
    }

    #[test]
    fn test_material_properties_zero_roughness() {
        let props = MaterialProperties::new().with_roughness(0.0);
        assert_eq!(props.roughness, 0.0);
    }

    #[test]
    fn test_material_properties_full_roughness() {
        let props = MaterialProperties::new().with_roughness(1.0);
        assert_eq!(props.roughness, 1.0);
    }

    #[test]
    fn test_material_manager_default() {
        let manager = MaterialManager::default();
        assert_eq!(manager.material_count(), 0);
    }

    #[test]
    fn test_material_properties_realistic_metal() {
        let metal = MaterialProperties::new()
            .with_metallic(1.0)
            .with_roughness(0.2);

        assert_eq!(metal.metallic, 1.0);
        assert_eq!(metal.roughness, 0.2);
    }

    #[test]
    fn test_material_properties_realistic_plastic() {
        let plastic = MaterialProperties::new()
            .with_metallic(0.0)
            .with_roughness(0.6);

        assert_eq!(plastic.metallic, 0.0);
        assert_eq!(plastic.roughness, 0.6);
    }

    #[test]
    fn test_material_properties_emissive_object() {
        let emissive = MaterialProperties::new()
            .with_emissive_strength(3.0)
            .with_base_color([1.0, 0.8, 0.0, 1.0]);

        assert_eq!(emissive.emissive_strength, 3.0);
        assert_eq!(emissive.base_color, [1.0, 0.8, 0.0, 1.0]);
    }
}
