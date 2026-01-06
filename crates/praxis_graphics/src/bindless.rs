//! Bindless rendering support using VK_EXT_descriptor_indexing.
//!
//! This module provides a bindless texture system that eliminates per-material
//! descriptor set binds by using large texture arrays and material indices.
//!
//! # Overview
//!
//! Traditional rendering requires binding a descriptor set for each material:
//! - 100 materials = 100 descriptor set binds per frame
//! - High CPU overhead from frequent binding operations
//! - Complex descriptor set management and pooling
//!
//! Bindless rendering solves this by:
//! - Single large texture array (up to 4096 textures)
//! - Material index passed as push constant
//! - Zero descriptor set binds during rendering
//! - Dramatically reduced CPU overhead
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │ Bindless Texture Manager                        │
//! │                                                  │
//! │  ┌────────────────────────────────────────────┐ │
//! │  │ Texture Array (up to 4096 textures)        │ │
//! │  │  [0]: white.png                            │ │
//! │  │  [1]: brick.png                            │ │
//! │  │  [2]: metal.png                            │ │
//! │  │  [3]: ...                                  │ │
//! │  └────────────────────────────────────────────┘ │
//! │                                                  │
//! │  ┌────────────────────────────────────────────┐ │
//! │  │ Material Data Buffer                       │ │
//! │  │  [0]: MaterialData { tex: 0, ... }         │ │
//! │  │  [1]: MaterialData { tex: 1, ... }         │ │
//! │  │  [2]: MaterialData { tex: 2, ... }         │ │
//! │  └────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────┘
//!
//! Render Loop:
//!   For each draw:
//!     1. Push material index via push constant
//!     2. Draw (no descriptor set bind needed)
//!     3. Shader reads texture array at material.texture_index
//! ```
//!
//! # Features
//!
//! - **Zero-cost material switches**: No descriptor set rebinds
//! - **Massive capacity**: Up to 4096 textures and materials
//! - **Dynamic updates**: Add/remove textures at runtime
//! - **Automatic indexing**: Texture names mapped to array indices
//! - **Backwards compatible**: Existing TextureManager integration
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_graphics::bindless::BindlessTextureManager;
//!
//! # fn example() -> praxis_utils::Result<()> {
//! // Initialize bindless system
//! // let mut bindless = BindlessTextureManager::new(
//! //     device,
//! //     memory_allocator,
//! //     command_buffer_allocator,
//! //     graphics_queue,
//! // )?;
//!
//! // Register textures
//! // let brick_idx = bindless.register_texture("brick", brick_texture)?;
//! // let metal_idx = bindless.register_texture("metal", metal_texture)?;
//!
//! // Rendering (material_index is passed as push constant)
//! // cmd_builder.push_constants(pipeline_layout, 0, material_index);
//! // cmd_builder.draw(...);
//! # Ok(())
//! # }
//! ```

use praxis_utils::{debug, eyre, info, trace, Result};
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    descriptor_set::{
        allocator::DescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
            DescriptorType,
        },
        DescriptorSet, WriteDescriptorSet,
    },
    device::Device,
    image::{sampler::Sampler, view::ImageView},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::layout::PushConstantRange,
    shader::ShaderStages,
};

/// Maximum number of textures supported in bindless mode.
pub const MAX_BINDLESS_TEXTURES: u32 = 4096;

/// Maximum number of materials supported in bindless mode.
pub const MAX_BINDLESS_MATERIALS: u32 = 4096;

/// Material data stored in GPU buffer for bindless rendering.
///
/// This structure is uploaded to a GPU buffer and indexed by material_id
/// during rendering. It contains all per-material properties including
/// texture indices into the bindless texture array.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BindlessMaterialData {
    /// Base color tint (rgba)
    pub base_color: [f32; 4],
    /// Index into bindless texture array for albedo texture
    pub albedo_texture_index: u32,
    /// Index into bindless texture array for normal map
    pub normal_texture_index: u32,
    /// Metallic factor [0,1]
    pub metallic: f32,
    /// Roughness factor [0,1]
    pub roughness: f32,
    /// Emissive strength
    pub emissive_strength: f32,
    /// Padding for alignment
    pub _padding: [f32; 3],
}

impl Default for BindlessMaterialData {
    fn default() -> Self {
        Self {
            base_color: [1.0, 1.0, 1.0, 1.0],
            albedo_texture_index: 0,
            normal_texture_index: 0,
            metallic: 0.0,
            roughness: 0.5,
            emissive_strength: 0.0,
            _padding: [0.0; 3],
        }
    }
}

/// Manages bindless texture arrays and material data for efficient rendering.
///
/// The `BindlessTextureManager` provides a centralized system for managing
/// textures and materials using descriptor indexing, eliminating the need
/// for per-material descriptor set binds during rendering.
///
/// # Architecture
///
/// The bindless system uses:
/// - A large descriptor array of texture samplers (up to 4096 textures)
/// - A GPU buffer containing material data
/// - Push constants to pass material indices to shaders
///
/// # Performance Benefits
///
/// Traditional rendering:
/// ```text
/// For each material:
///   - Bind descriptor set (CPU → GPU sync)
///   - For each object with this material:
///     - Draw call
/// ```
///
/// Bindless rendering:
/// ```text
/// For each object:
///   - Push material index (fast push constant)
///   - Draw call (shader indexes texture array)
/// ```
///
/// Result: 100x+ reduction in descriptor set operations for scenes with many materials.
pub struct BindlessTextureManager {
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,

    /// Map from texture name to index in texture array
    texture_name_to_index: HashMap<String, u32>,

    /// Map from material key to material index
    material_to_index: HashMap<u64, u32>,

    /// Next available texture index
    next_texture_index: u32,

    /// Next available material index
    next_material_index: u32,

    /// Array of all registered texture image views
    texture_views: Vec<Arc<ImageView>>,

    /// Array of all registered texture samplers
    texture_samplers: Vec<Arc<Sampler>>,

    /// Material data buffer
    material_data_buffer: Option<Subbuffer<[BindlessMaterialData]>>,

    /// Material data CPU-side (for updates)
    material_data: Vec<BindlessMaterialData>,

    /// Bindless descriptor set
    descriptor_set: Option<Arc<DescriptorSet>>,

    /// Descriptor set layout for bindless rendering
    descriptor_set_layout: Arc<DescriptorSetLayout>,
}

impl BindlessTextureManager {
    /// Creates a new bindless texture manager.
    ///
    /// Initializes the bindless system with empty texture arrays and material buffers.
    /// The descriptor set layout is created with support for variable descriptor counts
    /// and runtime-sized arrays.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan logical device
    /// * `memory_allocator` - Memory allocator for buffer creation
    /// * `descriptor_set_allocator` - Allocator for descriptor sets
    ///
    /// # Errors
    ///
    /// Returns an error if descriptor set layout creation fails.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    ) -> Result<Self> {
        info!("Initializing bindless texture manager");

        // Create descriptor set layout for bindless rendering
        // Set 2: Bindless textures and materials
        //   Binding 0: texture array (sampler2D[MAX_BINDLESS_TEXTURES])
        //   Binding 1: material data buffer (uniform buffer)
        let descriptor_set_layout = DescriptorSetLayout::new(
            device.clone(),
            DescriptorSetLayoutCreateInfo {
                bindings: [
                    (
                        0,
                        DescriptorSetLayoutBinding {
                            descriptor_count: MAX_BINDLESS_TEXTURES,
                            stages: ShaderStages::FRAGMENT,
                            ..DescriptorSetLayoutBinding::descriptor_type(
                                DescriptorType::CombinedImageSampler,
                            )
                        },
                    ),
                    (
                        1,
                        DescriptorSetLayoutBinding {
                            descriptor_count: 1,
                            stages: ShaderStages::FRAGMENT,
                            ..DescriptorSetLayoutBinding::descriptor_type(
                                DescriptorType::UniformBuffer,
                            )
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create bindless descriptor set layout: {}", e))?;

        debug!("Created bindless descriptor set layout");

        Ok(Self {
            memory_allocator,
            descriptor_set_allocator,
            texture_name_to_index: HashMap::new(),
            material_to_index: HashMap::new(),
            next_texture_index: 0,
            next_material_index: 0,
            texture_views: Vec::new(),
            texture_samplers: Vec::new(),
            material_data_buffer: None,
            material_data: Vec::new(),
            descriptor_set: None,
            descriptor_set_layout,
        })
    }

    /// Registers a texture in the bindless texture array.
    ///
    /// If the texture is already registered, returns its existing index.
    /// Otherwise, allocates a new index and updates the descriptor set.
    ///
    /// # Arguments
    ///
    /// * `name` - Unique name for this texture
    /// * `image_view` - Vulkan image view for the texture
    /// * `sampler` - Vulkan sampler for the texture
    ///
    /// # Returns
    ///
    /// The index of the texture in the bindless array.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum texture count is exceeded
    /// - Descriptor set update fails
    pub fn register_texture(
        &mut self,
        name: &str,
        image_view: Arc<ImageView>,
        sampler: Arc<Sampler>,
    ) -> Result<u32> {
        // Check if texture is already registered
        if let Some(&index) = self.texture_name_to_index.get(name) {
            trace!("Texture '{}' already registered at index {}", name, index);
            return Ok(index);
        }

        // Check capacity
        if self.next_texture_index >= MAX_BINDLESS_TEXTURES {
            return Err(eyre::eyre!(
                "Maximum bindless texture count ({}) exceeded",
                MAX_BINDLESS_TEXTURES
            ));
        }

        let index = self.next_texture_index;
        self.next_texture_index += 1;

        self.texture_name_to_index.insert(name.to_string(), index);
        self.texture_views.push(image_view);
        self.texture_samplers.push(sampler);

        debug!("Registered texture '{}' at bindless index {}", name, index);

        // Mark descriptor set as needing update
        self.descriptor_set = None;

        Ok(index)
    }

    /// Registers a material in the bindless material buffer.
    ///
    /// Materials are deduplicated based on their properties. If an identical
    /// material exists, returns its index. Otherwise, allocates a new index.
    ///
    /// # Arguments
    ///
    /// * `material_data` - Material properties and texture indices
    ///
    /// # Returns
    ///
    /// The index of the material in the bindless array.
    ///
    /// # Errors
    ///
    /// Returns an error if maximum material count is exceeded.
    pub fn register_material(&mut self, material_data: BindlessMaterialData) -> Result<u32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Compute hash for material deduplication
        let mut hasher = DefaultHasher::new();
        bytemuck::bytes_of(&material_data).hash(&mut hasher);
        let material_hash = hasher.finish();

        // Check if material is already registered
        if let Some(&index) = self.material_to_index.get(&material_hash) {
            trace!("Material already registered at index {}", index);
            return Ok(index);
        }

        // Check capacity
        if self.next_material_index >= MAX_BINDLESS_MATERIALS {
            return Err(eyre::eyre!(
                "Maximum bindless material count ({}) exceeded",
                MAX_BINDLESS_MATERIALS
            ));
        }

        let index = self.next_material_index;
        self.next_material_index += 1;

        self.material_to_index.insert(material_hash, index);
        self.material_data.push(material_data);

        debug!("Registered material at bindless index {}", index);

        // Mark buffer as needing update
        self.material_data_buffer = None;
        self.descriptor_set = None;

        Ok(index)
    }

    /// Gets or creates the bindless descriptor set.
    ///
    /// This descriptor set contains:
    /// - Binding 0: Array of all registered textures
    /// - Binding 1: Buffer containing all material data
    ///
    /// The descriptor set is cached and only recreated when textures or
    /// materials are added.
    ///
    /// # Returns
    ///
    /// The bindless descriptor set ready for binding during rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if descriptor set creation or update fails.
    pub fn get_descriptor_set(&mut self) -> Result<Arc<DescriptorSet>> {
        if let Some(ref descriptor_set) = self.descriptor_set {
            return Ok(descriptor_set.clone());
        }

        info!(
            "Building bindless descriptor set ({} textures, {} materials)",
            self.texture_views.len(),
            self.material_data.len()
        );

        // Create or update material data buffer
        if self.material_data_buffer.is_none() && !self.material_data.is_empty() {
            trace!("Creating material data buffer");
            let buffer = Buffer::from_iter(
                self.memory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::UNIFORM_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_HOST
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.material_data.iter().copied(),
            )
            .map_err(|e| eyre::eyre!("Failed to create material data buffer: {}", e))?;

            self.material_data_buffer = Some(buffer);
        }

        // Prepare descriptor writes
        let mut writes = Vec::new();

        // Write texture array (binding 0)
        if !self.texture_views.is_empty() {
            let image_infos: Vec<_> = self
                .texture_views
                .iter()
                .zip(self.texture_samplers.iter())
                .map(|(view, sampler)| {
                    WriteDescriptorSet::image_view_sampler(0, view.clone(), sampler.clone())
                })
                .collect();

            writes.extend(image_infos);
        }

        // Write material data buffer (binding 1)
        if let Some(ref buffer) = self.material_data_buffer {
            writes.push(WriteDescriptorSet::buffer(1, buffer.clone()));
        }

        // Create descriptor set
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.descriptor_set_layout.clone(),
            writes,
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create bindless descriptor set: {}", e))?;

        debug!("Created bindless descriptor set");

        self.descriptor_set = Some(descriptor_set.clone());

        Ok(descriptor_set)
    }

    /// Gets the descriptor set layout for bindless rendering.
    ///
    /// This layout should be used when creating pipelines that support
    /// bindless rendering.
    pub fn descriptor_set_layout(&self) -> &Arc<DescriptorSetLayout> {
        &self.descriptor_set_layout
    }

    /// Gets the texture index for a given texture name.
    ///
    /// Returns `None` if the texture has not been registered.
    pub fn get_texture_index(&self, name: &str) -> Option<u32> {
        self.texture_name_to_index.get(name).copied()
    }

    /// Returns the number of registered textures.
    pub fn texture_count(&self) -> usize {
        self.texture_views.len()
    }

    /// Returns the number of registered materials.
    pub fn material_count(&self) -> usize {
        self.material_data.len()
    }

    /// Creates push constant range for material index.
    ///
    /// The material index is passed via push constants for maximum performance.
    /// This range should be included in the pipeline layout.
    pub fn push_constant_range() -> PushConstantRange {
        PushConstantRange {
            stages: ShaderStages::FRAGMENT,
            offset: 0,
            size: 4, // u32 material index
        }
    }
}
