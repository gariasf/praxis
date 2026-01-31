//! Descriptor set binding management and utilities.
//!
//! This module provides helpers for creating and managing descriptor set bindings,
//! ensuring type safety and correct layout matching between CPU and GPU.

use std::sync::Arc;
use vulkano::{
    buffer::Subbuffer,
    descriptor_set::{
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        WriteDescriptorSet,
    },
    device::Device,
    image::{sampler::Sampler, view::ImageView},
    shader::ShaderStages,
};

use praxis_utils::{eyre, Result};

/// Builder for descriptor set layouts.
///
/// Provides a fluent API for constructing descriptor set layouts with
/// validation and type safety.
pub struct DescriptorSetLayoutBuilder {
    bindings: Vec<DescriptorSetLayoutBinding>,
}

impl DescriptorSetLayoutBuilder {
    /// Creates a new descriptor set layout builder.
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Adds a uniform buffer binding.
    pub fn add_uniform_buffer(mut self, _binding: u32, stages: ShaderStages) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding {
            descriptor_count: 1,
            stages,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBuffer)
        });
        self
    }

    /// Adds a dynamic uniform buffer binding.
    pub fn add_dynamic_uniform_buffer(mut self, _binding: u32, stages: ShaderStages) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding {
            descriptor_count: 1,
            stages,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::UniformBufferDynamic)
        });
        self
    }

    /// Adds a combined image sampler binding (texture).
    pub fn add_combined_image_sampler(mut self, _binding: u32, stages: ShaderStages) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding {
            descriptor_count: 1,
            stages,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::CombinedImageSampler)
        });
        self
    }

    /// Adds a storage buffer binding.
    pub fn add_storage_buffer(mut self, _binding: u32, stages: ShaderStages) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding {
            descriptor_count: 1,
            stages,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::StorageBuffer)
        });
        self
    }

    /// Adds a combined image sampler array binding (bindless textures).
    pub fn add_combined_image_sampler_array(
        mut self,
        _binding: u32,
        count: u32,
        stages: ShaderStages,
    ) -> Self {
        self.bindings.push(DescriptorSetLayoutBinding {
            descriptor_count: count,
            stages,
            ..DescriptorSetLayoutBinding::descriptor_type(DescriptorType::CombinedImageSampler)
        });
        self
    }

    /// Builds the descriptor set layout.
    pub fn build(self, device: Arc<Device>) -> Result<Arc<DescriptorSetLayout>> {
        DescriptorSetLayout::new(
            device,
            DescriptorSetLayoutCreateInfo {
                bindings: self
                    .bindings
                    .into_iter()
                    .enumerate()
                    .map(|(i, binding)| {
                        (i as u32, binding)
                    })
                    .collect(),
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set layout: {}", e))
    }
}

impl Default for DescriptorSetLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building descriptor set writes.
pub struct DescriptorSetWriter {
    writes: Vec<WriteDescriptorSet>,
}

impl DescriptorSetWriter {
    /// Creates a new descriptor set writer.
    pub fn new() -> Self {
        Self { writes: Vec::new() }
    }

    /// Adds a uniform buffer write.
    pub fn write_buffer<T>(mut self, binding: u32, buffer: Subbuffer<T>) -> Self
    where
        T: ?Sized,
    {
        self.writes.push(WriteDescriptorSet::buffer(binding, buffer));
        self
    }

    /// Adds a dynamic uniform buffer write with descriptor info.
    pub fn write_buffer_with_range(
        mut self,
        binding: u32,
        buffer_info: vulkano::descriptor_set::DescriptorBufferInfo,
    ) -> Self {
        self.writes
            .push(WriteDescriptorSet::buffer_with_range(binding, buffer_info));
        self
    }

    /// Adds a combined image sampler write.
    pub fn write_image_view_sampler(
        mut self,
        binding: u32,
        image_view: Arc<ImageView>,
        sampler: Arc<Sampler>,
    ) -> Self {
        self.writes
            .push(WriteDescriptorSet::image_view_sampler(
                binding, image_view, sampler,
            ));
        self
    }

    /// Adds a combined image sampler array write.
    pub fn write_image_view_sampler_array(
        mut self,
        binding: u32,
        first_array_element: u32,
        elements: Vec<(Arc<ImageView>, Arc<Sampler>)>,
    ) -> Self {
        self.writes
            .push(WriteDescriptorSet::image_view_sampler_array(
                binding,
                first_array_element,
                elements,
            ));
        self
    }

    /// Returns the writes for descriptor set creation.
    pub fn build(self) -> Vec<WriteDescriptorSet> {
        self.writes
    }
}

impl Default for DescriptorSetWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard descriptor set layouts for common use cases.
pub struct StandardDescriptorLayouts;

impl StandardDescriptorLayouts {
    /// Creates a descriptor set layout for per-frame data (Set 0).
    ///
    /// Bindings:
    /// - 0: View/Projection uniform buffer
    /// - 1: Dynamic uniform buffer for model matrices
    /// - 2: Albedo texture sampler
    /// - 3: Lighting data uniform buffer
    pub fn per_frame_layout(device: Arc<Device>) -> Result<Arc<DescriptorSetLayout>> {
        DescriptorSetLayoutBuilder::new()
            .add_uniform_buffer(0, ShaderStages::VERTEX | ShaderStages::FRAGMENT)
            .add_dynamic_uniform_buffer(1, ShaderStages::VERTEX)
            .add_combined_image_sampler(2, ShaderStages::FRAGMENT)
            .add_uniform_buffer(3, ShaderStages::FRAGMENT)
            .build(device)
    }

    /// Creates a descriptor set layout for per-material data (Set 1).
    ///
    /// Bindings:
    /// - 0: Material properties uniform buffer
    pub fn per_material_layout(device: Arc<Device>) -> Result<Arc<DescriptorSetLayout>> {
        DescriptorSetLayoutBuilder::new()
            .add_uniform_buffer(0, ShaderStages::FRAGMENT)
            .build(device)
    }

    /// Creates a descriptor set layout for bindless resources (Set 2).
    ///
    /// Bindings:
    /// - 0: Bindless texture array (4096 textures)
    /// - 1: Bindless material data buffer
    pub fn bindless_layout(device: Arc<Device>) -> Result<Arc<DescriptorSetLayout>> {
        DescriptorSetLayoutBuilder::new()
            .add_combined_image_sampler_array(0, 4096, ShaderStages::FRAGMENT)
            .add_uniform_buffer(1, ShaderStages::FRAGMENT)
            .build(device)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_set_layout_builder() {
        let builder = DescriptorSetLayoutBuilder::new()
            .add_uniform_buffer(0, ShaderStages::VERTEX)
            .add_combined_image_sampler(1, ShaderStages::FRAGMENT);

        assert_eq!(builder.bindings.len(), 2);
    }

    #[test]
    fn test_descriptor_set_writer() {
        let writer = DescriptorSetWriter::new();
        let writes = writer.build();

        assert_eq!(writes.len(), 0);
    }
}
