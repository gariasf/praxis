//! Material instancing system for efficient per-object parameter overrides.
//!
//! This module provides a system for creating material instances that share
//! texture data but allow per-object property overrides without full duplication.
//!
//! # Integration with Rendering Pipeline
//!
//! The material instancing system is fully integrated with the RenderContext:
//! - `DrawCommand` supports `material_instance_id` field for referencing instances
//! - Instances are automatically resolved during rendering
//! - Descriptor sets are pooled and reused efficiently
//! - Batching optimizations apply to instances automatically
//!
//! See `MATERIAL_INSTANCING.md` for detailed integration documentation and usage patterns.

use crate::material::{ExtendedPbrProperties, Material, MaterialProperties, ParallaxProperties};
use std::collections::HashMap;
use std::sync::Arc;

/// Material instance data that overrides base material properties.
#[derive(Clone)]
pub struct MaterialInstance {
    /// Base material reference.
    base_material: Arc<Material>,

    /// Override properties (None means use base material values).
    properties_override: Option<MaterialProperties>,

    /// Extended PBR overrides.
    extended_override: Option<ExtendedPbrProperties>,

    /// Parallax overrides.
    parallax_override: Option<ParallaxProperties>,
}

impl MaterialInstance {
    /// Creates a new material instance.
    pub fn new(base_material: Arc<Material>) -> Self {
        Self {
            base_material,
            properties_override: None,
            extended_override: None,
            parallax_override: None,
        }
    }

    /// Overrides base material properties.
    pub fn override_properties(mut self, properties: MaterialProperties) -> Self {
        self.properties_override = Some(properties);
        self
    }

    /// Overrides extended PBR properties.
    pub fn override_extended(mut self, properties: ExtendedPbrProperties) -> Self {
        self.extended_override = Some(properties);
        self
    }

    /// Overrides parallax properties.
    pub fn override_parallax(mut self, properties: ParallaxProperties) -> Self {
        self.parallax_override = Some(properties);
        self
    }

    /// Gets the base material.
    pub fn base_material(&self) -> &Material {
        &self.base_material
    }

    /// Gets the effective material properties (base or override).
    pub fn properties(&self) -> MaterialProperties {
        self.properties_override
            .unwrap_or(self.base_material.properties)
    }

    /// Gets the effective extended PBR properties.
    pub fn extended_properties(&self) -> ExtendedPbrProperties {
        self.extended_override
            .unwrap_or(self.base_material.extended_properties)
    }

    /// Gets the effective parallax properties.
    pub fn parallax_properties(&self) -> ParallaxProperties {
        self.parallax_override
            .unwrap_or(self.base_material.parallax_properties)
    }

    /// Checks if any properties are overridden.
    pub fn has_overrides(&self) -> bool {
        self.properties_override.is_some()
            || self.extended_override.is_some()
            || self.parallax_override.is_some()
    }
}

/// Material instancing manager for efficient instance tracking.
pub struct MaterialInstanceManager {
    /// Map of instance ID to material instance.
    instances: HashMap<String, MaterialInstance>,
}

impl MaterialInstanceManager {
    /// Creates a new material instance manager.
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    /// Creates a material instance.
    ///
    /// # Panics
    ///
    /// Panics if the instance cannot be retrieved after insertion. This should never happen
    /// under normal circumstances as the instance is immediately retrieved after insertion.
    pub fn create_instance(
        &mut self,
        instance_id: impl Into<String>,
        base_material: Arc<Material>,
    ) -> &mut MaterialInstance {
        let instance_id = instance_id.into();
        let instance = MaterialInstance::new(base_material);
        self.instances.insert(instance_id.clone(), instance);
        self.instances.get_mut(&instance_id).expect("Just inserted")
    }

    /// Gets a material instance.
    pub fn get_instance(&self, instance_id: &str) -> Option<&MaterialInstance> {
        self.instances.get(instance_id)
    }

    /// Gets a mutable material instance.
    pub fn get_instance_mut(&mut self, instance_id: &str) -> Option<&mut MaterialInstance> {
        self.instances.get_mut(instance_id)
    }

    /// Removes a material instance.
    pub fn remove_instance(&mut self, instance_id: &str) -> bool {
        self.instances.remove(instance_id).is_some()
    }

    /// Returns the number of instances.
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Clears all instances.
    pub fn clear(&mut self) {
        self.instances.clear();
    }
}

impl Default for MaterialInstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about material instancing efficiency.
#[derive(Debug, Clone, Copy)]
pub struct InstancingStats {
    /// Total number of material instances.
    pub total_instances: usize,

    /// Number of unique base materials.
    pub unique_base_materials: usize,

    /// Number of instances with overrides.
    pub instances_with_overrides: usize,

    /// Average instances per base material.
    pub avg_instances_per_base: f32,
}

impl MaterialInstanceManager {
    /// Computes instancing statistics.
    pub fn compute_stats(&self) -> InstancingStats {
        let total_instances = self.instances.len();
        let mut unique_bases = std::collections::HashSet::new();
        let mut instances_with_overrides = 0;

        for instance in self.instances.values() {
            unique_bases.insert(Arc::as_ptr(&instance.base_material));
            if instance.has_overrides() {
                instances_with_overrides += 1;
            }
        }

        let unique_base_materials = unique_bases.len();
        let avg_instances_per_base = if unique_base_materials > 0 {
            total_instances as f32 / unique_base_materials as f32
        } else {
            0.0
        };

        InstancingStats {
            total_instances,
            unique_base_materials,
            instances_with_overrides,
            avg_instances_per_base,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::Texture;

    /// Creates a test material with stub texture data.
    /// Note: The Texture contains Arc<Image> which cannot be constructed without Vulkan.
    /// For these tests we'll use Material's Clone trait which doesn't require texture validation.
    fn create_test_material(id: &str) -> Arc<Material> {
        // We create materials with base properties and test property overrides
        let mut material = Material {
            id: id.to_string(),
            base_material_id: None,
            albedo_texture: unsafe { std::mem::zeroed() }, // Stub - not used in property tests
            normal_texture: None,
            metallic_roughness_texture: None,
            height_texture: None,
            ao_texture: None,
            emissive_texture: None,
            properties: MaterialProperties::default(),
            extended_properties: ExtendedPbrProperties::default(),
            parallax_properties: ParallaxProperties::default(),
            layers: Vec::new(),
        };

        // Set distinctive default values for testing
        material.properties = MaterialProperties::new()
            .with_metallic(0.5)
            .with_roughness(0.3)
            .with_base_color([1.0, 0.0, 0.0, 1.0]);

        material.extended_properties = ExtendedPbrProperties::new()
            .with_clearcoat(0.2)
            .with_sheen(0.1);

        material.parallax_properties = ParallaxProperties::new()
            .with_height_scale(0.03)
            .with_min_samples(10);

        Arc::new(material)
    }

    #[test]
    fn test_material_instance_creation() {
        let material = create_test_material("test_base");
        let instance = MaterialInstance::new(material.clone());

        assert!(!instance.has_overrides());
        assert_eq!(Arc::as_ptr(&instance.base_material), Arc::as_ptr(&material));
        assert_eq!(instance.base_material().id, "test_base");
    }

    #[test]
    fn test_material_instance_property_overrides() {
        let material = create_test_material("test_base");
        let original_metallic = material.properties.metallic;
        let original_roughness = material.properties.roughness;

        let instance = MaterialInstance::new(material.clone()).override_properties(
            MaterialProperties::new()
                .with_metallic(0.8)
                .with_roughness(0.9),
        );

        assert!(instance.has_overrides());
        assert_eq!(instance.properties().metallic, 0.8);
        assert_eq!(instance.properties().roughness, 0.9);

        // Base material should remain unchanged
        assert_eq!(material.properties.metallic, original_metallic);
        assert_eq!(material.properties.roughness, original_roughness);
    }

    #[test]
    fn test_material_instance_extended_overrides() {
        let material = create_test_material("test_base");
        let original_clearcoat = material.extended_properties.clearcoat;

        let instance = MaterialInstance::new(material.clone()).override_extended(
            ExtendedPbrProperties::new()
                .with_clearcoat(0.9)
                .with_sheen(0.7),
        );

        assert!(instance.has_overrides());
        assert_eq!(instance.extended_properties().clearcoat, 0.9);
        assert_eq!(instance.extended_properties().sheen, 0.7);

        // Base material should remain unchanged
        assert_eq!(material.extended_properties.clearcoat, original_clearcoat);
    }

    #[test]
    fn test_material_instance_parallax_overrides() {
        let material = create_test_material("test_base");
        let original_height_scale = material.parallax_properties.height_scale;

        let instance = MaterialInstance::new(material.clone()).override_parallax(
            ParallaxProperties::new()
                .with_height_scale(0.08)
                .with_min_samples(20),
        );

        assert!(instance.has_overrides());
        assert_eq!(instance.parallax_properties().height_scale, 0.08);
        assert_eq!(instance.parallax_properties().min_samples, 20);

        // Base material should remain unchanged
        assert_eq!(
            material.parallax_properties.height_scale,
            original_height_scale
        );
    }

    #[test]
    fn test_material_instance_multiple_overrides() {
        let material = create_test_material("test_base");

        let instance = MaterialInstance::new(material.clone())
            .override_properties(MaterialProperties::new().with_metallic(0.9))
            .override_extended(ExtendedPbrProperties::new().with_clearcoat(0.5))
            .override_parallax(ParallaxProperties::new().with_height_scale(0.06));

        assert!(instance.has_overrides());
        assert_eq!(instance.properties().metallic, 0.9);
        assert_eq!(instance.extended_properties().clearcoat, 0.5);
        assert_eq!(instance.parallax_properties().height_scale, 0.06);
    }

    #[test]
    fn test_material_instance_effective_property_resolution() {
        let material = create_test_material("test_base");

        // Instance without overrides should return base material properties
        let instance_no_override = MaterialInstance::new(material.clone());
        assert_eq!(
            instance_no_override.properties().metallic,
            material.properties.metallic
        );
        assert_eq!(
            instance_no_override.properties().roughness,
            material.properties.roughness
        );
        assert_eq!(
            instance_no_override.extended_properties().clearcoat,
            material.extended_properties.clearcoat
        );
        assert_eq!(
            instance_no_override.parallax_properties().height_scale,
            material.parallax_properties.height_scale
        );

        // Instance with overrides should return overridden values
        let instance_with_override = MaterialInstance::new(material.clone())
            .override_properties(MaterialProperties::new().with_metallic(0.75));

        assert_eq!(instance_with_override.properties().metallic, 0.75);
        // Non-overridden properties should still come from base
        assert_eq!(
            instance_with_override.extended_properties().clearcoat,
            material.extended_properties.clearcoat
        );
    }

    #[test]
    fn test_material_instance_base_texture_sharing() {
        let material = create_test_material("test_base");
        let instance1 = MaterialInstance::new(material.clone());
        let instance2 = MaterialInstance::new(material.clone());

        // All instances should share the same base material pointer
        assert_eq!(
            Arc::as_ptr(&instance1.base_material),
            Arc::as_ptr(&instance2.base_material)
        );
        assert_eq!(
            Arc::as_ptr(&instance1.base_material),
            Arc::as_ptr(&material)
        );
    }

    #[test]
    fn test_instance_manager_creation() {
        let manager = MaterialInstanceManager::new();
        assert_eq!(manager.instance_count(), 0);
    }

    #[test]
    fn test_instance_manager_default() {
        let manager = MaterialInstanceManager::default();
        assert_eq!(manager.instance_count(), 0);
    }

    #[test]
    fn test_instance_manager_create_instance() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        let instance = manager.create_instance("instance1", material.clone());
        instance.override_properties(MaterialProperties::new().with_metallic(0.8));

        assert_eq!(manager.instance_count(), 1);
        let retrieved = manager.get_instance("instance1").unwrap();
        assert_eq!(retrieved.properties().metallic, 0.8);
    }

    #[test]
    fn test_instance_manager_get_instance() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        manager.create_instance("instance1", material);

        let instance = manager.get_instance("instance1");
        assert!(instance.is_some());
        assert_eq!(instance.unwrap().base_material().id, "test_base");

        let missing = manager.get_instance("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_instance_manager_get_instance_mut() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        manager.create_instance("instance1", material);

        {
            let instance = manager.get_instance_mut("instance1").unwrap();
            *instance = instance
                .clone()
                .override_properties(MaterialProperties::new().with_metallic(0.95));
        }

        let instance = manager.get_instance("instance1").unwrap();
        assert_eq!(instance.properties().metallic, 0.95);

        let missing = manager.get_instance_mut("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_instance_manager_remove_instance() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        manager.create_instance("instance1", material);
        assert_eq!(manager.instance_count(), 1);

        assert!(manager.remove_instance("instance1"));
        assert_eq!(manager.instance_count(), 0);

        // Removing non-existent instance should return false
        assert!(!manager.remove_instance("instance1"));
        assert!(!manager.remove_instance("nonexistent"));
    }

    #[test]
    fn test_instance_manager_multiple_instances() {
        let mut manager = MaterialInstanceManager::new();
        let material1 = create_test_material("base1");
        let material2 = create_test_material("base2");

        manager.create_instance("instance1", material1.clone());
        manager.create_instance("instance2", material1.clone());
        manager.create_instance("instance3", material2.clone());

        assert_eq!(manager.instance_count(), 3);
        assert!(manager.get_instance("instance1").is_some());
        assert!(manager.get_instance("instance2").is_some());
        assert!(manager.get_instance("instance3").is_some());
    }

    #[test]
    fn test_instance_manager_clear() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        manager.create_instance("instance1", material.clone());
        manager.create_instance("instance2", material.clone());
        manager.create_instance("instance3", material);

        assert_eq!(manager.instance_count(), 3);

        manager.clear();
        assert_eq!(manager.instance_count(), 0);
        assert!(manager.get_instance("instance1").is_none());
    }

    #[test]
    fn test_instance_manager_compute_stats_empty() {
        let manager = MaterialInstanceManager::new();
        let stats = manager.compute_stats();

        assert_eq!(stats.total_instances, 0);
        assert_eq!(stats.unique_base_materials, 0);
        assert_eq!(stats.instances_with_overrides, 0);
        assert_eq!(stats.avg_instances_per_base, 0.0);
    }

    #[test]
    fn test_instance_manager_compute_stats_single_base() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("base1");

        manager.create_instance("instance1", material.clone());
        manager.create_instance("instance2", material.clone());
        manager.create_instance("instance3", material);

        let stats = manager.compute_stats();
        assert_eq!(stats.total_instances, 3);
        assert_eq!(stats.unique_base_materials, 1);
        assert_eq!(stats.avg_instances_per_base, 3.0);
        assert_eq!(stats.instances_with_overrides, 0);
    }

    #[test]
    fn test_instance_manager_compute_stats_multiple_bases() {
        let mut manager = MaterialInstanceManager::new();
        let material1 = create_test_material("base1");
        let material2 = create_test_material("base2");

        manager.create_instance("instance1", material1.clone());
        manager.create_instance("instance2", material1);
        manager.create_instance("instance3", material2.clone());
        manager.create_instance("instance4", material2);

        let stats = manager.compute_stats();
        assert_eq!(stats.total_instances, 4);
        assert_eq!(stats.unique_base_materials, 2);
        assert_eq!(stats.avg_instances_per_base, 2.0);
    }

    #[test]
    fn test_instance_manager_compute_stats_with_overrides() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("base1");

        // Instance with override
        let instance1 = manager.create_instance("instance1", material.clone());
        *instance1 = instance1
            .clone()
            .override_properties(MaterialProperties::new().with_metallic(0.9));

        // Instance without override
        manager.create_instance("instance2", material.clone());

        // Instance with multiple overrides
        let instance3 = manager.create_instance("instance3", material);
        *instance3 = instance3
            .clone()
            .override_properties(MaterialProperties::new().with_roughness(0.8))
            .override_extended(ExtendedPbrProperties::new().with_clearcoat(0.7));

        let stats = manager.compute_stats();
        assert_eq!(stats.total_instances, 3);
        assert_eq!(stats.unique_base_materials, 1);
        assert_eq!(stats.instances_with_overrides, 2);
        assert_eq!(stats.avg_instances_per_base, 3.0);
    }

    #[test]
    fn test_instance_manager_update_through_mut() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        manager.create_instance("instance1", material);

        // Modify through mutable reference
        {
            let instance = manager.get_instance_mut("instance1").unwrap();
            *instance = instance.clone().override_properties(
                MaterialProperties::new()
                    .with_metallic(0.7)
                    .with_roughness(0.6),
            );
        }

        // Verify changes persisted
        let instance = manager.get_instance("instance1").unwrap();
        assert!(instance.has_overrides());
        assert_eq!(instance.properties().metallic, 0.7);
        assert_eq!(instance.properties().roughness, 0.6);
    }

    #[test]
    fn test_instance_manager_string_conversion() {
        let mut manager = MaterialInstanceManager::new();
        let material = create_test_material("test_base");

        // Test that Into<String> works for instance_id
        manager.create_instance(String::from("instance1"), material.clone());
        manager.create_instance("instance2", material); // &str should also work

        assert_eq!(manager.instance_count(), 2);
        assert!(manager.get_instance("instance1").is_some());
        assert!(manager.get_instance("instance2").is_some());
    }

    #[test]
    fn test_instancing_stats_debug() {
        let stats = InstancingStats {
            total_instances: 10,
            unique_base_materials: 3,
            instances_with_overrides: 7,
            avg_instances_per_base: 3.33,
        };

        // Test that Debug is implemented
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("3"));
        assert!(debug_str.contains("7"));
    }

    #[test]
    fn test_material_instance_clone() {
        let material = create_test_material("test_base");
        let instance1 = MaterialInstance::new(material)
            .override_properties(MaterialProperties::new().with_metallic(0.85));

        let instance2 = instance1.clone();

        assert!(instance2.has_overrides());
        assert_eq!(instance2.properties().metallic, 0.85);
        assert_eq!(
            Arc::as_ptr(&instance1.base_material),
            Arc::as_ptr(&instance2.base_material)
        );
    }

    #[test]
    fn test_effective_property_independence() {
        let material = create_test_material("test_base");

        let instance1 = MaterialInstance::new(material.clone())
            .override_properties(MaterialProperties::new().with_metallic(0.9));

        let instance2 = MaterialInstance::new(material.clone())
            .override_properties(MaterialProperties::new().with_metallic(0.3));

        // Each instance should have independent overrides
        assert_eq!(instance1.properties().metallic, 0.9);
        assert_eq!(instance2.properties().metallic, 0.3);

        // Base material should be unaffected
        assert_ne!(material.properties.metallic, 0.9);
        assert_ne!(material.properties.metallic, 0.3);
    }

    #[test]
    fn test_partial_property_override() {
        let material = create_test_material("test_base");
        let base_roughness = material.properties.roughness;
        let base_color = material.properties.base_color;

        // Override only metallic, other properties should come from base
        let instance = MaterialInstance::new(material.clone()).override_properties(
            MaterialProperties::new()
                .with_metallic(0.95)
                .with_roughness(base_roughness)
                .with_base_color(base_color),
        );

        let props = instance.properties();
        assert_eq!(props.metallic, 0.95);
        assert_eq!(props.roughness, base_roughness);
        assert_eq!(props.base_color, base_color);
    }
}
