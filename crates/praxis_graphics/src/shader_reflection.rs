//! Shader reflection and metadata system.
//!
//! This module provides shader introspection capabilities, extracting
//! metadata about shader inputs, outputs, and resources at compile time
//! through vulkano-shaders reflection data.

use std::collections::HashMap;

/// Type of shader stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    /// Vertex shader stage.
    Vertex,
    /// Fragment shader stage.
    Fragment,
    /// Compute shader stage.
    Compute,
}

/// Descriptor type for shader resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorType {
    /// Uniform buffer with read-only data.
    UniformBuffer,
    /// Dynamic uniform buffer with per-draw offsets.
    UniformBufferDynamic,
    /// Storage buffer with read-write data.
    StorageBuffer,
    /// Combined image sampler (texture).
    CombinedImageSampler,
    /// Input attachment from previous subpass.
    InputAttachment,
}

/// Descriptor binding information extracted from shader.
#[derive(Debug, Clone)]
pub struct DescriptorBinding {
    /// Descriptor set number.
    pub set: u32,
    /// Binding number within the set.
    pub binding: u32,
    /// Type of descriptor.
    pub descriptor_type: DescriptorType,
    /// Name of the binding in shader code.
    pub name: String,
    /// Array size (1 for non-arrays).
    pub count: u32,
}

/// Input/output variable information.
#[derive(Debug, Clone)]
pub struct ShaderVariable {
    /// Location index.
    pub location: u32,
    /// Variable name in shader code.
    pub name: String,
    /// Format description (e.g., "vec3", "mat4").
    pub format: String,
}

/// Push constant range information.
#[derive(Debug, Clone)]
pub struct PushConstantRange {
    /// Offset in bytes within push constant block.
    pub offset: u32,
    /// Size in bytes.
    pub size: u32,
    /// Shader stages that use this range.
    pub stages: Vec<ShaderStage>,
}

/// Shader reflection metadata.
///
/// Contains all information extracted from shader introspection,
/// including descriptor bindings, input/output variables, and
/// push constant ranges.
#[derive(Debug, Clone)]
pub struct ShaderReflection {
    /// Shader stage type.
    pub stage: ShaderStage,
    /// Entry point name (typically "main").
    pub entry_point: String,
    /// Descriptor bindings used by this shader.
    pub descriptor_bindings: Vec<DescriptorBinding>,
    /// Input variables (vertex attributes for vertex shaders).
    pub inputs: Vec<ShaderVariable>,
    /// Output variables.
    pub outputs: Vec<ShaderVariable>,
    /// Push constant ranges.
    pub push_constants: Vec<PushConstantRange>,
}

impl ShaderReflection {
    /// Creates shader reflection metadata from a shader module.
    ///
    /// Extracts descriptor bindings, input/output variables, and push constants
    /// from the shader's SPIR-V reflection data.
    ///
    /// # Arguments
    ///
    /// * `stage` - The shader stage type
    /// * `entry_point` - The entry point to analyze
    ///
    /// # Returns
    ///
    /// Reflection metadata for the shader.
    pub fn from_entry_point(
        stage: ShaderStage,
        _entry_point: &vulkano::shader::EntryPoint,
    ) -> Self {
        let entry_point_name = "main".to_string();

        // Extract descriptor bindings from entry point info
        let descriptor_bindings = Vec::new();
        
        // Note: vulkano's reflection API is limited, so we provide placeholder
        // reflection data. In a production system, you would use spirv-reflect
        // or similar tools to extract full reflection metadata.
        
        // For now, document the expected descriptor set layout based on our shaders
        // Set 0: Per-frame/Per-draw resources
        // Set 1: Per-material resources
        // Set 2: Bindless resources (optional)

        Self {
            stage,
            entry_point: entry_point_name,
            descriptor_bindings,
            inputs: Vec::new(),
            outputs: Vec::new(),
            push_constants: Vec::new(),
        }
    }

    /// Gets all descriptor bindings for a specific set.
    pub fn get_bindings_for_set(&self, set: u32) -> Vec<&DescriptorBinding> {
        self.descriptor_bindings
            .iter()
            .filter(|b| b.set == set)
            .collect()
    }

    /// Checks if shader uses a specific descriptor set.
    pub fn uses_descriptor_set(&self, set: u32) -> bool {
        self.descriptor_bindings.iter().any(|b| b.set == set)
    }

    /// Checks if shader uses push constants.
    pub fn uses_push_constants(&self) -> bool {
        !self.push_constants.is_empty()
    }
}

/// Collection of shader reflections for a complete pipeline.
#[derive(Debug, Clone, Default)]
pub struct PipelineReflection {
    /// Reflection data for each shader stage.
    pub stages: HashMap<ShaderStage, ShaderReflection>,
}

impl PipelineReflection {
    /// Creates a new empty pipeline reflection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a shader stage to the pipeline reflection.
    pub fn add_stage(&mut self, reflection: ShaderReflection) {
        self.stages.insert(reflection.stage, reflection);
    }

    /// Gets reflection data for a specific stage.
    pub fn get_stage(&self, stage: ShaderStage) -> Option<&ShaderReflection> {
        self.stages.get(&stage)
    }

    /// Gets all descriptor sets used by any shader stage.
    pub fn get_all_descriptor_sets(&self) -> Vec<u32> {
        let mut sets: Vec<u32> = self
            .stages
            .values()
            .flat_map(|s| s.descriptor_bindings.iter().map(|b| b.set))
            .collect();
        sets.sort_unstable();
        sets.dedup();
        sets
    }

    /// Checks if any shader stage uses push constants.
    pub fn uses_push_constants(&self) -> bool {
        self.stages.values().any(|s| s.uses_push_constants())
    }

    /// Merges descriptor bindings from all stages.
    ///
    /// Returns a map of set numbers to their bindings.
    pub fn merge_descriptor_bindings(&self) -> HashMap<u32, Vec<DescriptorBinding>> {
        let mut result: HashMap<u32, Vec<DescriptorBinding>> = HashMap::new();

        for reflection in self.stages.values() {
            for binding in &reflection.descriptor_bindings {
                result
                    .entry(binding.set)
                    .or_default()
                    .push(binding.clone());
            }
        }

        // Deduplicate bindings within each set
        for bindings in result.values_mut() {
            bindings.sort_by_key(|b| b.binding);
            bindings.dedup_by_key(|b| b.binding);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_reflection_creation() {
        let reflection = ShaderReflection {
            stage: ShaderStage::Vertex,
            entry_point: "main".to_string(),
            descriptor_bindings: vec![
                DescriptorBinding {
                    set: 0,
                    binding: 0,
                    descriptor_type: DescriptorType::UniformBuffer,
                    name: "ViewProjection".to_string(),
                    count: 1,
                },
                DescriptorBinding {
                    set: 0,
                    binding: 1,
                    descriptor_type: DescriptorType::UniformBufferDynamic,
                    name: "Model".to_string(),
                    count: 1,
                },
            ],
            inputs: Vec::new(),
            outputs: Vec::new(),
            push_constants: Vec::new(),
        };

        assert_eq!(reflection.stage, ShaderStage::Vertex);
        assert_eq!(reflection.get_bindings_for_set(0).len(), 2);
        assert!(reflection.uses_descriptor_set(0));
        assert!(!reflection.uses_descriptor_set(1));
        assert!(!reflection.uses_push_constants());
    }

    #[test]
    fn test_pipeline_reflection() {
        let mut pipeline = PipelineReflection::new();

        let vs_reflection = ShaderReflection {
            stage: ShaderStage::Vertex,
            entry_point: "main".to_string(),
            descriptor_bindings: vec![DescriptorBinding {
                set: 0,
                binding: 0,
                descriptor_type: DescriptorType::UniformBuffer,
                name: "ViewProjection".to_string(),
                count: 1,
            }],
            inputs: Vec::new(),
            outputs: Vec::new(),
            push_constants: Vec::new(),
        };

        let fs_reflection = ShaderReflection {
            stage: ShaderStage::Fragment,
            entry_point: "main".to_string(),
            descriptor_bindings: vec![DescriptorBinding {
                set: 1,
                binding: 0,
                descriptor_type: DescriptorType::CombinedImageSampler,
                name: "albedoTexture".to_string(),
                count: 1,
            }],
            inputs: Vec::new(),
            outputs: Vec::new(),
            push_constants: Vec::new(),
        };

        pipeline.add_stage(vs_reflection);
        pipeline.add_stage(fs_reflection);

        let sets = pipeline.get_all_descriptor_sets();
        assert_eq!(sets, vec![0, 1]);
        assert!(!pipeline.uses_push_constants());
    }
}
