//! High-level terrain system.

use crate::chunk::{TerrainChunk, TerrainChunkId};
use crate::heightmap::TerrainHeightmap;
use crate::lod::TerrainLodManager;
use crate::material::{TerrainMaterial, TerrainMaterialLayer};
use crate::mesh::TerrainMesh;
use crate::renderer::{TerrainRenderer, VegetationRenderer};
use crate::splatmap::SplatMap;
use crate::vegetation::{VegetationDistributor, VegetationLayer};
use praxis_graphics::{GpuMesh, MeshData};
use praxis_math::Vec3;
use praxis_utils::{eyre, info, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::command_buffer::allocator::CommandBufferAllocator;
use vulkano::device::{Device, Queue};
use vulkano::memory::allocator::StandardMemoryAllocator;

/// Configuration for the terrain system.
#[derive(Debug, Clone)]
pub struct TerrainConfig {
    /// Size of each chunk in world units.
    pub chunk_size: f32,

    /// Number of vertices per chunk side at LOD 0.
    pub vertices_per_chunk: u32,

    /// Maximum terrain height.
    pub max_height: f32,

    /// Number of LOD levels.
    pub lod_levels: usize,

    /// Distance thresholds for LOD levels.
    pub lod_distances: Vec<f32>,

    /// World size (used for coordinate mapping).
    pub world_size: f32,

    /// Scale factor for world coordinates.
    pub world_scale: f32,

    /// Enable frustum culling for chunks.
    pub enable_frustum_culling: bool,

    /// Enable occlusion culling for chunks.
    pub enable_occlusion_culling: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64.0,
            vertices_per_chunk: 65,
            max_height: 100.0,
            lod_levels: 4,
            lod_distances: vec![50.0, 100.0, 200.0, 400.0],
            world_size: 1024.0,
            world_scale: 1.0,
            enable_frustum_culling: true,
            enable_occlusion_culling: false,
        }
    }
}

/// High-level terrain system managing heightmap, chunks, LOD, materials, and vegetation.
pub struct TerrainSystem {
    /// Terrain configuration.
    pub config: TerrainConfig,

    /// Heightmap data.
    pub heightmap: TerrainHeightmap,

    /// All terrain chunks.
    chunks: HashMap<TerrainChunkId, TerrainChunk>,

    /// LOD manager.
    lod_manager: TerrainLodManager,

    /// Material configuration.
    pub material: TerrainMaterial,

    /// Splat map for material blending.
    pub splatmap: SplatMap,

    /// Vegetation layers.
    pub vegetation_layers: Vec<VegetationLayer>,

    /// Terrain renderer.
    terrain_renderer: Option<TerrainRenderer>,

    /// Vegetation renderer.
    vegetation_renderer: Option<VegetationRenderer>,

    /// Memory allocator for GPU resources.
    memory_allocator: Option<Arc<StandardMemoryAllocator>>,

    /// Command buffer allocator for GPU operations.
    command_buffer_allocator: Option<Arc<dyn CommandBufferAllocator>>,

    /// Transfer queue for GPU operations.
    transfer_queue: Option<Arc<Queue>>,

    /// Chunks pending mesh generation.
    pending_chunks: Vec<TerrainChunkId>,

    /// Maximum number of chunks to generate per frame.
    max_chunks_per_frame: usize,
}

impl TerrainSystem {
    /// Creates a new terrain system with the given configuration and heightmap.
    pub fn new(config: TerrainConfig, heightmap: TerrainHeightmap) -> Result<Self> {
        let lod_manager = TerrainLodManager::new(config.lod_distances.clone());

        let splatmap = SplatMap::new(heightmap.width, heightmap.height);

        info!(
            "Created terrain system: {}x{} heightmap, {} LOD levels, {:.1}m chunks",
            heightmap.width, heightmap.height, config.lod_levels, config.chunk_size
        );

        Ok(Self {
            config,
            heightmap,
            chunks: HashMap::new(),
            lod_manager,
            material: TerrainMaterial::new(),
            splatmap,
            vegetation_layers: Vec::new(),
            terrain_renderer: None,
            vegetation_renderer: None,
            memory_allocator: None,
            command_buffer_allocator: None,
            transfer_queue: None,
            pending_chunks: Vec::new(),
            max_chunks_per_frame: 8,
        })
    }

    /// Initializes the terrain system with Vulkan resources.
    pub fn initialize_rendering(
        &mut self,
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
        transfer_queue: Arc<Queue>,
    ) {
        self.terrain_renderer = Some(TerrainRenderer::new(
            device.clone(),
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
        ));

        self.vegetation_renderer = Some(VegetationRenderer::new(
            device,
            memory_allocator.clone(),
            command_buffer_allocator.clone(),
        ));

        self.memory_allocator = Some(memory_allocator);
        self.command_buffer_allocator = Some(command_buffer_allocator);
        self.transfer_queue = Some(transfer_queue);
        info!("Terrain rendering initialized");
    }

    /// Adds a material layer to the terrain.
    pub fn add_material_layer(
        &mut self,
        name: impl Into<String>,
        min_height: f32,
        max_height: f32,
    ) -> Result<()> {
        let name_str = name.into();
        let layer = TerrainMaterialLayer::new(
            name_str.clone(),
            format!("{name_str}_albedo"),
            min_height,
            max_height,
        );
        self.material.add_layer(layer);
        Ok(())
    }

    /// Adds a vegetation layer to the terrain.
    pub fn add_vegetation_layer(
        &mut self,
        name: impl Into<String>,
        density: f32,
        min_height: f32,
        max_height: f32,
    ) -> Result<()> {
        let name_str = name.into();
        let layer = VegetationLayer::new(
            name_str.clone(),
            format!("{name_str}_mesh"),
            format!("{name_str}_material"),
            density,
        )
        .with_height_range(min_height, max_height);

        self.vegetation_layers.push(layer);
        Ok(())
    }

    /// Updates the terrain system based on camera position.
    pub fn update(&mut self, camera_pos: Vec3) {
        let camera_chunk_x = (camera_pos.x / self.config.chunk_size).floor() as i32;
        let camera_chunk_z = (camera_pos.z / self.config.chunk_size).floor() as i32;

        let view_distance = self.config.lod_distances.last().copied().unwrap_or(400.0);
        let chunk_view_distance = (view_distance / self.config.chunk_size).ceil() as i32 + 1;

        self.pending_chunks.clear();

        for z in (camera_chunk_z - chunk_view_distance)..=(camera_chunk_z + chunk_view_distance) {
            for x in (camera_chunk_x - chunk_view_distance)..=(camera_chunk_x + chunk_view_distance)
            {
                let chunk_id = TerrainChunkId::new(x, z);

                if !self.chunks.contains_key(&chunk_id) {
                    self.pending_chunks.push(chunk_id);
                }

                if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                    let chunk_center = chunk_id.world_position(self.config.chunk_size)
                        + Vec3::new(
                            self.config.chunk_size * 0.5,
                            0.0,
                            self.config.chunk_size * 0.5,
                        );
                    self.lod_manager
                        .update_chunk_lod(&mut chunk.lod, chunk_center, camera_pos);
                }
            }
        }

        let chunks_to_create = self
            .pending_chunks
            .iter()
            .take(self.max_chunks_per_frame)
            .copied()
            .collect::<Vec<_>>();

        for chunk_id in chunks_to_create {
            self.create_chunk(chunk_id);
        }

        self.chunks.retain(|id, _| {
            id.distance_to(camera_pos, self.config.chunk_size) < view_distance * 1.5
        });
    }

    /// Creates a new terrain chunk.
    fn create_chunk(&mut self, chunk_id: TerrainChunkId) {
        let mut chunk = TerrainChunk::new(chunk_id, self.config.chunk_size, self.config.lod_levels);
        chunk.update_bounds(&self.heightmap, self.config.chunk_size);

        if let (Some(memory_allocator), Some(command_buffer_allocator), Some(transfer_queue)) = (
            &self.memory_allocator,
            &self.command_buffer_allocator,
            &self.transfer_queue,
        ) {
            for lod_level in 0..self.config.lod_levels {
                if let Ok(mesh_data) = self.generate_chunk_mesh(chunk_id, lod_level) {
                    if let Ok(gpu_mesh) = self.upload_mesh_to_gpu(
                        &mesh_data,
                        memory_allocator,
                        command_buffer_allocator,
                        transfer_queue,
                    ) {
                        chunk.meshes[lod_level] = Some(gpu_mesh);
                    }
                }
            }
        }

        self.chunks.insert(chunk_id, chunk);
    }

    /// Generates mesh data for a chunk at a specific LOD level.
    fn generate_chunk_mesh(&self, chunk_id: TerrainChunkId, lod_level: usize) -> Result<MeshData> {
        if lod_level >= self.config.lod_levels {
            return Err(eyre::eyre!(
                "LOD level {} exceeds configured levels {}",
                lod_level,
                self.config.lod_levels
            ));
        }

        let vertices_per_side = self
            .lod_manager
            .get_vertex_count(self.config.vertices_per_chunk, lod_level)
            .max(2);

        if vertices_per_side > 1024 {
            return Err(eyre::eyre!(
                "vertices_per_side {} exceeds maximum 1024",
                vertices_per_side
            ));
        }

        TerrainMesh::generate_chunk(
            &self.heightmap,
            chunk_id.x,
            chunk_id.z,
            self.config.chunk_size,
            vertices_per_side,
            self.config.world_scale,
        )
    }

    /// Uploads mesh data to GPU.
    fn upload_mesh_to_gpu(
        &self,
        mesh_data: &MeshData,
        memory_allocator: &Arc<StandardMemoryAllocator>,
        command_buffer_allocator: &Arc<dyn CommandBufferAllocator>,
        transfer_queue: &Arc<Queue>,
    ) -> Result<GpuMesh> {
        mesh_data
            .upload(
                memory_allocator.clone(),
                command_buffer_allocator.clone(),
                transfer_queue.clone(),
            )
            .map_err(|e| eyre::eyre!("Failed to upload terrain mesh to GPU: {}", e))
    }

    /// Generates vegetation for the entire terrain using parallel processing.
    pub fn generate_vegetation(&mut self) -> Result<()> {
        info!(
            "Generating vegetation for {} layers...",
            self.vegetation_layers.len()
        );

        if self.config.world_size <= 0.0 {
            return Err(eyre::eyre!("Invalid world_size: must be positive"));
        }

        let bounds_min = Vec3::ZERO;
        let bounds_max = Vec3::new(self.config.world_size, 0.0, self.config.world_size);

        self.vegetation_layers.par_iter_mut().for_each(|layer| {
            layer.clear_instances();

            let height_fn =
                |x: f32, z: f32| self.heightmap.get_height_at(x, z, self.config.world_size);

            let normal_fn = |x: f32, z: f32| {
                if self.config.world_size <= 0.0
                    || self.heightmap.width == 0
                    || self.heightmap.height == 0
                {
                    return Vec3::Y;
                }
                let grid_x = ((x / self.config.world_size * self.heightmap.width as f32)
                    .clamp(0.0, self.heightmap.width as f32 - 1.0))
                    as u32;
                let grid_z = ((z / self.config.world_size * self.heightmap.height as f32)
                    .clamp(0.0, self.heightmap.height as f32 - 1.0))
                    as u32;
                self.heightmap
                    .calculate_normal(grid_x, grid_z, self.config.world_scale)
            };

            if let Ok(instances) = VegetationDistributor::distribute(
                layer, bounds_min, bounds_max, height_fn, normal_fn,
            ) {
                layer.instances = instances;
            }
        });

        let total_instances: usize = self
            .vegetation_layers
            .iter()
            .map(|l| l.instances.len())
            .sum();
        info!("Generated {} total vegetation instances", total_instances);

        Ok(())
    }

    /// Generates vegetation for a specific area (for painting tools).
    pub fn generate_vegetation_in_area(
        &mut self,
        layer_index: usize,
        center: Vec3,
        radius: f32,
        density: f32,
    ) -> Result<()> {
        if layer_index >= self.vegetation_layers.len() {
            return Err(eyre::eyre!("Invalid vegetation layer index"));
        }

        if radius <= 0.0 || density < 0.0 || self.config.world_size <= 0.0 {
            return Err(eyre::eyre!("Invalid parameters for vegetation generation"));
        }

        let layer = &mut self.vegetation_layers[layer_index];
        let bounds_min = Vec3::new(center.x - radius, 0.0, center.z - radius);
        let bounds_max = Vec3::new(center.x + radius, 0.0, center.z + radius);

        let height_fn = |x: f32, z: f32| self.heightmap.get_height_at(x, z, self.config.world_size);

        let normal_fn = |x: f32, z: f32| {
            if self.config.world_size <= 0.0
                || self.heightmap.width == 0
                || self.heightmap.height == 0
            {
                return Vec3::Y;
            }
            let grid_x = ((x / self.config.world_size * self.heightmap.width as f32)
                .clamp(0.0, self.heightmap.width as f32 - 1.0)) as u32;
            let grid_z = ((z / self.config.world_size * self.heightmap.height as f32)
                .clamp(0.0, self.heightmap.height as f32 - 1.0)) as u32;
            self.heightmap
                .calculate_normal(grid_x, grid_z, self.config.world_scale)
        };

        let mut temp_layer = layer.clone();
        temp_layer.density = density;

        let new_instances = VegetationDistributor::distribute(
            &temp_layer,
            bounds_min,
            bounds_max,
            height_fn,
            normal_fn,
        )?;

        layer.instances.extend(new_instances);

        Ok(())
    }

    /// Gets a reference to a terrain chunk.
    pub fn get_chunk(&self, chunk_id: TerrainChunkId) -> Option<&TerrainChunk> {
        self.chunks.get(&chunk_id)
    }

    /// Gets a mutable reference to a terrain chunk.
    pub fn get_chunk_mut(&mut self, chunk_id: TerrainChunkId) -> Option<&mut TerrainChunk> {
        self.chunks.get_mut(&chunk_id)
    }

    /// Gets all active chunks.
    pub fn chunks(&self) -> &HashMap<TerrainChunkId, TerrainChunk> {
        &self.chunks
    }

    /// Gets the number of active chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Marks all chunks as dirty, requiring mesh regeneration.
    pub fn mark_all_chunks_dirty(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.mark_dirty();
        }
    }

    /// Marks chunks in a specific area as dirty.
    pub fn mark_area_chunks_dirty(&mut self, center: Vec3, radius: f32) {
        let chunk_radius = (radius / self.config.chunk_size).ceil() as i32;
        let center_chunk_x = (center.x / self.config.chunk_size).floor() as i32;
        let center_chunk_z = (center.z / self.config.chunk_size).floor() as i32;

        for z in (center_chunk_z - chunk_radius)..=(center_chunk_z + chunk_radius) {
            for x in (center_chunk_x - chunk_radius)..=(center_chunk_x + chunk_radius) {
                let chunk_id = TerrainChunkId::new(x, z);
                if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                    chunk.mark_dirty();
                }
            }
        }
    }

    /// Regenerates meshes for dirty chunks using parallel processing.
    pub fn regenerate_dirty_chunks(&mut self) -> Result<()> {
        if self.memory_allocator.is_none()
            || self.command_buffer_allocator.is_none()
            || self.transfer_queue.is_none()
        {
            return Ok(());
        }

        let memory_allocator = self.memory_allocator.as_ref().unwrap();
        let command_buffer_allocator = self.command_buffer_allocator.as_ref().unwrap();
        let transfer_queue = self.transfer_queue.as_ref().unwrap();

        let dirty_chunks: Vec<TerrainChunkId> = self
            .chunks
            .iter()
            .filter(|(_, chunk)| chunk.dirty)
            .map(|(id, _)| *id)
            .collect();

        if dirty_chunks.is_empty() {
            return Ok(());
        }

        info!("Regenerating {} dirty chunks...", dirty_chunks.len());

        for chunk_id in dirty_chunks {
            for lod_level in 0..self.config.lod_levels {
                if let Ok(mesh_data) = self.generate_chunk_mesh(chunk_id, lod_level) {
                    if let Ok(gpu_mesh) = self.upload_mesh_to_gpu(
                        &mesh_data,
                        memory_allocator,
                        command_buffer_allocator,
                        transfer_queue,
                    ) {
                        if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                            chunk.meshes[lod_level] = Some(gpu_mesh);
                        }
                    }
                }
            }

            if let Some(chunk) = self.chunks.get_mut(&chunk_id) {
                chunk.clear_dirty();
            }
        }

        Ok(())
    }

    /// Gets the terrain renderer.
    pub fn terrain_renderer(&self) -> Option<&TerrainRenderer> {
        self.terrain_renderer.as_ref()
    }

    /// Gets the terrain renderer mutably.
    pub fn terrain_renderer_mut(&mut self) -> Option<&mut TerrainRenderer> {
        self.terrain_renderer.as_mut()
    }

    /// Gets the vegetation renderer.
    pub fn vegetation_renderer(&self) -> Option<&VegetationRenderer> {
        self.vegetation_renderer.as_ref()
    }

    /// Gets the vegetation renderer mutably.
    pub fn vegetation_renderer_mut(&mut self) -> Option<&mut VegetationRenderer> {
        self.vegetation_renderer.as_mut()
    }

    /// Gets the LOD manager.
    pub fn lod_manager(&self) -> &TerrainLodManager {
        &self.lod_manager
    }

    /// Gets the LOD manager mutably.
    pub fn lod_manager_mut(&mut self) -> &mut TerrainLodManager {
        &mut self.lod_manager
    }
}
