//! Advanced material system for defining surface properties.
//!
//! This module provides a comprehensive material system with:
//! - Material instancing for efficient per-object parameter overrides
//! - Material layers for blending multiple materials with mask textures
//! - Parallax occlusion mapping for enhanced depth perception
//! - Extended PBR features (clearcoat, sheen, transmission)

use crate::texture::Texture;
use praxis_utils::{debug, info};
use std::collections::HashMap;
use std::sync::Arc;

/// Material properties for shader uniforms (PBR base).
///
/// These properties can be uploaded to the GPU as uniform data to control
/// material appearance.
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

    /// Gets the base color tint.
    #[must_use]
    pub const fn base_color(&self) -> [f32; 4] {
        self.base_color
    }

    /// Gets the metallic factor.
    #[must_use]
    pub const fn metallic(&self) -> f32 {
        self.metallic
    }

    /// Gets the roughness factor.
    #[must_use]
    pub const fn roughness(&self) -> f32 {
        self.roughness
    }

    /// Gets the emissive strength.
    #[must_use]
    pub const fn emissive_strength(&self) -> f32 {
        self.emissive_strength
    }
}

/// Extended PBR properties for advanced material features.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ExtendedPbrProperties {
    /// Clearcoat strength [0.0, 1.0]. Adds a second specular layer on top of the base.
    pub clearcoat: f32,

    /// Clearcoat roughness [0.0, 1.0]. Controls roughness of the clearcoat layer.
    pub clearcoat_roughness: f32,

    /// Sheen strength [0.0, 1.0]. Adds fabric-like reflectance at grazing angles.
    pub sheen: f32,

    /// Sheen tint [0.0, 1.0]. Tints the sheen color toward the base color.
    pub sheen_tint: f32,

    /// Transmission [0.0, 1.0]. Controls light transmission through the material (glass, water).
    pub transmission: f32,

    /// Index of refraction for transmission [1.0, 3.0]. Default is 1.5 (glass).
    pub ior: f32,

    /// Anisotropy [-1.0, 1.0]. Controls directional roughness (brushed metal).
    pub anisotropy: f32,

    /// Anisotropy rotation [0.0, 1.0]. Rotates the anisotropic direction.
    pub anisotropy_rotation: f32,
}

impl Default for ExtendedPbrProperties {
    fn default() -> Self {
        Self {
            clearcoat: 0.0,
            clearcoat_roughness: 0.03,
            sheen: 0.0,
            sheen_tint: 0.0,
            transmission: 0.0,
            ior: 1.5,
            anisotropy: 0.0,
            anisotropy_rotation: 0.0,
        }
    }
}

impl ExtendedPbrProperties {
    /// Creates default extended PBR properties.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets clearcoat strength [0.0, 1.0].
    pub fn with_clearcoat(mut self, clearcoat: f32) -> Self {
        self.clearcoat = clearcoat.clamp(0.0, 1.0);
        self
    }

    /// Sets clearcoat roughness [0.0, 1.0].
    pub fn with_clearcoat_roughness(mut self, roughness: f32) -> Self {
        self.clearcoat_roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Sets sheen strength [0.0, 1.0].
    pub fn with_sheen(mut self, sheen: f32) -> Self {
        self.sheen = sheen.clamp(0.0, 1.0);
        self
    }

    /// Sets sheen tint [0.0, 1.0].
    pub fn with_sheen_tint(mut self, tint: f32) -> Self {
        self.sheen_tint = tint.clamp(0.0, 1.0);
        self
    }

    /// Sets transmission [0.0, 1.0].
    pub fn with_transmission(mut self, transmission: f32) -> Self {
        self.transmission = transmission.clamp(0.0, 1.0);
        self
    }

    /// Sets index of refraction [1.0, 3.0].
    pub fn with_ior(mut self, ior: f32) -> Self {
        self.ior = ior.clamp(1.0, 3.0);
        self
    }

    /// Sets anisotropy [-1.0, 1.0].
    pub fn with_anisotropy(mut self, anisotropy: f32) -> Self {
        self.anisotropy = anisotropy.clamp(-1.0, 1.0);
        self
    }

    /// Sets anisotropy rotation [0.0, 1.0].
    pub fn with_anisotropy_rotation(mut self, rotation: f32) -> Self {
        self.anisotropy_rotation = rotation.clamp(0.0, 1.0);
        self
    }
}

/// Parallax occlusion mapping parameters.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParallaxProperties {
    /// Height scale for parallax effect [0.0, 0.1]. Higher values = more depth.
    pub height_scale: f32,

    /// Minimum number of samples for parallax mapping [4, 32].
    pub min_samples: u32,

    /// Maximum number of samples for parallax mapping [4, 64].
    pub max_samples: u32,

    /// Enable parallax occlusion mapping (0 = disabled, 1 = enabled).
    pub enabled: u32,
}

impl Default for ParallaxProperties {
    fn default() -> Self {
        Self {
            height_scale: 0.05,
            min_samples: 8,
            max_samples: 32,
            enabled: 0,
        }
    }
}

impl ParallaxProperties {
    /// Creates default parallax properties (disabled).
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables parallax occlusion mapping.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = if enabled { 1 } else { 0 };
        self
    }

    /// Sets height scale [0.0, 0.1].
    pub fn with_height_scale(mut self, scale: f32) -> Self {
        self.height_scale = scale.clamp(0.0, 0.1);
        self
    }

    /// Sets minimum samples [4, 32].
    pub fn with_min_samples(mut self, samples: u32) -> Self {
        self.min_samples = samples.clamp(4, 32);
        self
    }

    /// Sets maximum samples [4, 64].
    pub fn with_max_samples(mut self, samples: u32) -> Self {
        self.max_samples = samples.clamp(4, 64);
        self
    }
}

/// Material layer blend mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    /// Replace base with layer (mask controls opacity).
    Replace,
    /// Add layer to base.
    Add,
    /// Multiply layer with base.
    Multiply,
    /// Overlay blend mode.
    Overlay,
}

/// Material layer for multi-material blending.
#[derive(Clone)]
pub struct MaterialLayer {
    /// Layer name for identification.
    pub name: String,

    /// Material to blend.
    pub material_id: String,

    /// Blend mask texture (R channel controls blend weight).
    pub mask_texture: Option<Texture>,

    /// Blend mode for this layer.
    pub blend_mode: BlendMode,

    /// Layer opacity [0.0, 1.0].
    pub opacity: f32,

    /// UV scale for this layer.
    pub uv_scale: [f32; 2],
}

impl MaterialLayer {
    /// Creates a new material layer.
    pub fn new(name: impl Into<String>, material_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            material_id: material_id.into(),
            mask_texture: None,
            blend_mode: BlendMode::Replace,
            opacity: 1.0,
            uv_scale: [1.0, 1.0],
        }
    }

    /// Sets the blend mask texture.
    pub fn with_mask(mut self, texture: Texture) -> Self {
        self.mask_texture = Some(texture);
        self
    }

    /// Sets the blend mode.
    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    /// Sets the layer opacity.
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Sets the UV scale.
    pub fn with_uv_scale(mut self, scale: [f32; 2]) -> Self {
        self.uv_scale = scale;
        self
    }
}

/// Advanced material with instancing support and extended features.
///
/// This material supports:
/// - Base textures (albedo, normal, metallic-roughness, height, AO, emissive)
/// - Material instancing (sharing base material with per-instance overrides)
/// - Material layers (blending multiple materials)
/// - Parallax occlusion mapping
/// - Extended PBR features (clearcoat, sheen, transmission)
#[derive(Clone)]
pub struct Material {
    /// Material ID for instancing.
    pub id: String,

    /// Base material ID (for instancing). None if this is a base material.
    pub base_material_id: Option<String>,

    /// Albedo (base color) texture.
    pub albedo_texture: Texture,

    /// Normal map texture.
    pub normal_texture: Option<Texture>,

    /// Metallic-roughness texture (R=metallic, G=roughness).
    pub metallic_roughness_texture: Option<Texture>,

    /// Height map for parallax occlusion mapping.
    pub height_texture: Option<Texture>,

    /// Ambient occlusion texture.
    pub ao_texture: Option<Texture>,

    /// Emissive texture.
    pub emissive_texture: Option<Texture>,

    /// Base material properties.
    pub properties: MaterialProperties,

    /// Extended PBR properties.
    pub extended_properties: ExtendedPbrProperties,

    /// Parallax properties.
    pub parallax_properties: ParallaxProperties,

    /// Material layers for multi-material blending.
    pub layers: Vec<MaterialLayer>,
}

impl Material {
    /// Creates a new material with the given albedo texture and default properties.
    pub fn new(id: impl Into<String>, albedo_texture: Texture) -> Self {
        Self {
            id: id.into(),
            base_material_id: None,
            albedo_texture,
            normal_texture: None,
            metallic_roughness_texture: None,
            height_texture: None,
            ao_texture: None,
            emissive_texture: None,
            properties: MaterialProperties::default(),
            extended_properties: ExtendedPbrProperties::default(),
            parallax_properties: ParallaxProperties::default(),
            layers: Vec::new(),
        }
    }

    /// Creates a material instance based on another material.
    pub fn instance(
        id: impl Into<String>,
        base_material_id: impl Into<String>,
        base_material: &Material,
    ) -> Self {
        let mut instance = base_material.clone();
        instance.id = id.into();
        instance.base_material_id = Some(base_material_id.into());
        instance
    }

    /// Sets the material properties.
    pub fn set_properties(&mut self, properties: MaterialProperties) {
        self.properties = properties;
    }

    /// Sets the extended PBR properties.
    pub fn set_extended_properties(&mut self, properties: ExtendedPbrProperties) {
        self.extended_properties = properties;
    }

    /// Sets the parallax properties.
    pub fn set_parallax_properties(&mut self, properties: ParallaxProperties) {
        self.parallax_properties = properties;
    }

    /// Sets the albedo texture.
    pub fn set_albedo_texture(&mut self, texture: Texture) {
        self.albedo_texture = texture;
    }

    /// Sets the normal map texture.
    pub fn set_normal_texture(&mut self, texture: Option<Texture>) {
        self.normal_texture = texture;
    }

    /// Sets the metallic-roughness texture.
    pub fn set_metallic_roughness_texture(&mut self, texture: Option<Texture>) {
        self.metallic_roughness_texture = texture;
    }

    /// Sets the height map texture.
    pub fn set_height_texture(&mut self, texture: Option<Texture>) {
        self.height_texture = texture;
    }

    /// Sets the ambient occlusion texture.
    pub fn set_ao_texture(&mut self, texture: Option<Texture>) {
        self.ao_texture = texture;
    }

    /// Sets the emissive texture.
    pub fn set_emissive_texture(&mut self, texture: Option<Texture>) {
        self.emissive_texture = texture;
    }

    /// Adds a material layer.
    pub fn add_layer(&mut self, layer: MaterialLayer) {
        self.layers.push(layer);
    }

    /// Removes a material layer by name.
    pub fn remove_layer(&mut self, name: &str) -> bool {
        if let Some(pos) = self.layers.iter().position(|l| l.name == name) {
            self.layers.remove(pos);
            true
        } else {
            false
        }
    }

    /// Gets a reference to the material properties.
    pub fn properties(&self) -> &MaterialProperties {
        &self.properties
    }

    /// Gets a reference to the extended PBR properties.
    pub fn extended_properties(&self) -> &ExtendedPbrProperties {
        &self.extended_properties
    }

    /// Gets a reference to the parallax properties.
    pub fn parallax_properties(&self) -> &ParallaxProperties {
        &self.parallax_properties
    }

    /// Gets the albedo texture.
    pub fn albedo_texture(&self) -> &Texture {
        &self.albedo_texture
    }

    /// Gets the normal texture.
    pub fn normal_texture(&self) -> Option<&Texture> {
        self.normal_texture.as_ref()
    }

    /// Gets the metallic-roughness texture.
    pub fn metallic_roughness_texture(&self) -> Option<&Texture> {
        self.metallic_roughness_texture.as_ref()
    }

    /// Gets the height texture.
    pub fn height_texture(&self) -> Option<&Texture> {
        self.height_texture.as_ref()
    }

    /// Gets the ambient occlusion texture.
    pub fn ao_texture(&self) -> Option<&Texture> {
        self.ao_texture.as_ref()
    }

    /// Gets the emissive texture.
    pub fn emissive_texture(&self) -> Option<&Texture> {
        self.emissive_texture.as_ref()
    }

    /// Gets the material layers.
    pub fn layers(&self) -> &[MaterialLayer] {
        &self.layers
    }

    /// Checks if this is a material instance.
    pub fn is_instance(&self) -> bool {
        self.base_material_id.is_some()
    }
}

/// Material asset manager with instancing support.
pub struct MaterialManager {
    /// Map of material ID to material.
    materials: HashMap<String, Arc<Material>>,
}

impl MaterialManager {
    /// Creates a new material manager.
    pub fn new() -> Self {
        debug!("Creating MaterialManager");
        Self {
            materials: HashMap::new(),
        }
    }

    /// Adds a material to the manager.
    pub fn add_material(&mut self, material: Material) {
        let id = material.id.clone();
        debug!("Adding material '{}'", id);
        self.materials.insert(id, Arc::new(material));
        info!("Material added to manager");
    }

    /// Creates a material from a texture and adds it to the manager.
    pub fn create_material(&mut self, id: impl Into<String>, albedo_texture: Texture) {
        let id = id.into();
        debug!("Creating material '{}'", id);
        let material = Material::new(id.clone(), albedo_texture);
        self.materials.insert(id, Arc::new(material));
        info!("Material created and cached");
    }

    /// Creates a material instance based on another material.
    ///
    /// This is efficient for per-object parameter overrides without duplicating texture data.
    pub fn create_instance(
        &mut self,
        id: impl Into<String>,
        base_material_id: &str,
    ) -> Result<(), String> {
        let id = id.into();
        let base = self
            .materials
            .get(base_material_id)
            .ok_or_else(|| format!("Base material '{base_material_id}' not found"))?;

        debug!(
            "Creating material instance '{}' from '{}'",
            id, base_material_id
        );
        let instance = Material::instance(&id, base_material_id, base);
        self.materials.insert(id, Arc::new(instance));
        info!("Material instance created");
        Ok(())
    }

    /// Gets a material by ID.
    pub fn get_material(&self, id: &str) -> Option<Arc<Material>> {
        self.materials.get(id).cloned()
    }

    /// Gets a mutable reference to a material by ID.
    ///
    /// Note: This will create a new Arc if the material is shared.
    pub fn get_material_mut(&mut self, id: &str) -> Option<&mut Material> {
        self.materials.get_mut(id).and_then(Arc::get_mut)
    }

    /// Checks if a material exists.
    pub fn contains_material(&self, id: &str) -> bool {
        self.materials.contains_key(id)
    }

    /// Removes a material.
    pub fn remove_material(&mut self, id: &str) -> bool {
        if self.materials.remove(id).is_some() {
            debug!("Material '{}' removed from cache", id);
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
    fn test_extended_pbr_defaults() {
        let props = ExtendedPbrProperties::default();
        assert_eq!(props.clearcoat, 0.0);
        assert_eq!(props.sheen, 0.0);
        assert_eq!(props.transmission, 0.0);
        assert_eq!(props.ior, 1.5);
    }

    #[test]
    fn test_parallax_properties_defaults() {
        let props = ParallaxProperties::default();
        assert_eq!(props.height_scale, 0.05);
        assert_eq!(props.enabled, 0);
    }

    #[test]
    fn test_extended_pbr_builder() {
        let props = ExtendedPbrProperties::new()
            .with_clearcoat(0.8)
            .with_transmission(0.9);

        assert_eq!(props.clearcoat, 0.8);
        assert_eq!(props.transmission, 0.9);
    }

    #[test]
    fn test_parallax_builder() {
        let props = ParallaxProperties::new()
            .enabled(true)
            .with_height_scale(0.08);

        assert_eq!(props.enabled, 1);
        assert_eq!(props.height_scale, 0.08);
    }

    #[test]
    fn test_material_layer() {
        let layer = MaterialLayer::new("layer1", "material1")
            .with_opacity(0.5)
            .with_blend_mode(BlendMode::Multiply);

        assert_eq!(layer.name, "layer1");
        assert_eq!(layer.opacity, 0.5);
        assert_eq!(layer.blend_mode, BlendMode::Multiply);
    }

    #[test]
    fn test_extended_pbr_clearcoat() {
        let props = ExtendedPbrProperties::new()
            .with_clearcoat(0.9)
            .with_clearcoat_roughness(0.05);

        assert_eq!(props.clearcoat, 0.9);
        assert_eq!(props.clearcoat_roughness, 0.05);
    }

    #[test]
    fn test_extended_pbr_sheen() {
        let props = ExtendedPbrProperties::new()
            .with_sheen(0.7)
            .with_sheen_tint(0.5);

        assert_eq!(props.sheen, 0.7);
        assert_eq!(props.sheen_tint, 0.5);
    }

    #[test]
    fn test_extended_pbr_transmission() {
        let props = ExtendedPbrProperties::new()
            .with_transmission(0.9)
            .with_ior(1.5);

        assert_eq!(props.transmission, 0.9);
        assert_eq!(props.ior, 1.5);
    }

    #[test]
    fn test_extended_pbr_anisotropy() {
        let props = ExtendedPbrProperties::new()
            .with_anisotropy(0.7)
            .with_anisotropy_rotation(0.25);

        assert_eq!(props.anisotropy, 0.7);
        assert_eq!(props.anisotropy_rotation, 0.25);
    }

    #[test]
    fn test_parallax_enabled() {
        let props = ParallaxProperties::new().enabled(true);
        assert_eq!(props.enabled, 1);

        let props2 = ParallaxProperties::new().enabled(false);
        assert_eq!(props2.enabled, 0);
    }

    #[test]
    fn test_parallax_height_scale() {
        let props = ParallaxProperties::new().with_height_scale(0.08);
        assert_eq!(props.height_scale, 0.08);
    }

    #[test]
    fn test_parallax_samples() {
        let props = ParallaxProperties::new()
            .with_min_samples(16)
            .with_max_samples(48);

        assert_eq!(props.min_samples, 16);
        assert_eq!(props.max_samples, 48);
    }

    #[test]
    fn test_material_properties_size() {
        assert_eq!(std::mem::size_of::<MaterialProperties>(), 32);
        assert_eq!(std::mem::size_of::<ExtendedPbrProperties>(), 32);
        assert_eq!(std::mem::size_of::<ParallaxProperties>(), 16);
    }

    #[test]
    fn test_blend_mode_equality() {
        assert_eq!(BlendMode::Replace, BlendMode::Replace);
        assert_ne!(BlendMode::Add, BlendMode::Multiply);
    }

    #[test]
    fn test_layer_opacity_clamping() {
        let layer = MaterialLayer::new("test", "mat").with_opacity(2.0);
        assert_eq!(layer.opacity, 1.0);

        let layer2 = MaterialLayer::new("test", "mat").with_opacity(-0.5);
        assert_eq!(layer2.opacity, 0.0);
    }

    #[test]
    fn test_clearcoat_clamping() {
        let props = ExtendedPbrProperties::new().with_clearcoat(1.5);
        assert_eq!(props.clearcoat, 1.0);

        let props2 = ExtendedPbrProperties::new().with_clearcoat(-0.5);
        assert_eq!(props2.clearcoat, 0.0);
    }

    #[test]
    fn test_ior_clamping() {
        let props = ExtendedPbrProperties::new().with_ior(5.0);
        assert_eq!(props.ior, 3.0);

        let props2 = ExtendedPbrProperties::new().with_ior(0.5);
        assert_eq!(props2.ior, 1.0);
    }

    #[test]
    fn test_parallax_height_scale_clamping() {
        let props = ParallaxProperties::new().with_height_scale(0.5);
        assert_eq!(props.height_scale, 0.1);

        let props2 = ParallaxProperties::new().with_height_scale(-0.1);
        assert_eq!(props2.height_scale, 0.0);
    }

    #[test]
    fn test_parallax_sample_clamping() {
        let props = ParallaxProperties::new()
            .with_min_samples(2)
            .with_max_samples(100);

        assert_eq!(props.min_samples, 4);
        assert_eq!(props.max_samples, 64);
    }
}
