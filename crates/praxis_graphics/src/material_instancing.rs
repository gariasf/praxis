//! Material instancing system for efficient per-object parameter overrides.
//!
//! This module provides a system for creating material instances that share
//! texture data but allow per-object property overrides without full duplication.

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

// Temporarily disabled due to unsafe zeroed() initialization of Texture with Arc
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::texture::Texture;
//
//     fn create_dummy_texture() -> Texture {
//         // Create a minimal texture for testing
//         // In real tests, you'd use a proper texture creation method
//         unsafe { std::mem::zeroed() }
//     }
//
//     #[test]
//     fn test_material_instance_creation() {
//         let material = Arc::new(Material::new("test", create_dummy_texture()));
//         let instance = MaterialInstance::new(material.clone());
//
//         assert!(!instance.has_overrides());
//         assert_eq!(
//             Arc::as_ptr(&instance.base_material),
//             Arc::as_ptr(&material)
//         );
//     }
//
//     #[test]
//     fn test_material_instance_overrides() {
//         let material = Arc::new(Material::new("test", create_dummy_texture()));
//         let mut instance = MaterialInstance::new(material);
//
//         assert!(!instance.has_overrides());
//
//         instance = instance.override_properties(MaterialProperties::new().with_metallic(0.8));
//         assert!(instance.has_overrides());
//         assert_eq!(instance.properties().metallic, 0.8);
//     }
//
//     #[test]
//     fn test_instance_manager() {
//         let mut manager = MaterialInstanceManager::new();
//         let material = Arc::new(Material::new("test", create_dummy_texture()));
//
//         manager.create_instance("instance1", material.clone());
//         manager.create_instance("instance2", material.clone());
//
//         assert_eq!(manager.instance_count(), 2);
//
//         let stats = manager.compute_stats();
//         assert_eq!(stats.total_instances, 2);
//         assert_eq!(stats.unique_base_materials, 1);
//         assert_eq!(stats.avg_instances_per_base, 2.0);
//     }
//
//     #[test]
//     fn test_instance_removal() {
//         let mut manager = MaterialInstanceManager::new();
//         let material = Arc::new(Material::new("test", create_dummy_texture()));
//
//         manager.create_instance("instance1", material);
//         assert_eq!(manager.instance_count(), 1);
//
//         assert!(manager.remove_instance("instance1"));
//         assert_eq!(manager.instance_count(), 0);
//         assert!(!manager.remove_instance("instance1"));
//     }
// }
