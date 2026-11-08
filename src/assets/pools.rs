use std::collections::{BTreeMap, HashMap};

use crate::{
    assets::material::CLASSES_PER_CHANNEL,
    assets::{MaterialChannel, MaterialHandle, Mesh, MeshData, MeshHandle, MeshletData},
    bench::Budgets,
    cooked::{AssetId, CookedMesh, CookedVertex},
    render::{LoadStaging, NO_MATERIAL, blas_bytes},
    stats::{Charge, Counted, GpuMemory, GpuMemoryBudgets, GpuMemoryKind},
};

/// Bytes of the geometry Arenas' Budget per Mesh table entry: 256 per Arena.
const GEOMETRY_BYTES_PER_MESH: u64 = 1280;

/// Bytes of the texture pool's Budget per Material table entry.
const TEXTURE_POOL_BYTES_PER_MATERIAL: u64 = 16 << 20;

/// Bytes of a Channel's share of the texture pool's Budget per Texture class.
const TEXTURE_POOL_BYTES_PER_CLASS: u64 = 64 << 10;

/// Unused entries each pool frees in one frame to make room for refused
/// loads, so a load that fits nowhere cannot empty a pool in one frame; the
/// Textures freed with a Material count, and may take the Material pool past
/// it.
pub(crate) const ENTRIES_FREED_PER_FRAME: usize = 64;

/// What each pool holds, derived from the GPU memory Budgets at device
/// creation: the geometry Arenas' Budget in equal shares and the Mesh table
/// beside it, the texture pool's the Material table and each Channel's
/// Texture classes, up to what the Capability floor binds. The shares are
/// provisional until the Budget split sets them (ADRs 0013, 0017).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoolCapacities {
    pub vertex_arena_bytes: u64,
    pub index_arena_bytes: u64,
    pub meshlet_arena_bytes: u64,
    pub meshlet_vertex_arena_bytes: u64,
    pub meshlet_triangle_arena_bytes: u64,
    pub meshes: u32,
    pub materials: u32,
    pub texture_classes_per_channel: u32,
}

impl PoolCapacities {
    pub fn from_budgets(budgets: &GpuMemoryBudgets) -> Self {
        let arena_bytes = budgets.geometry_arenas / 5;
        let entries = |bytes: u64| u32::try_from(bytes).unwrap_or(u32::MAX);
        let channels = MaterialChannel::ALL.len() as u64;
        Self {
            vertex_arena_bytes: arena_bytes,
            index_arena_bytes: arena_bytes,
            meshlet_arena_bytes: arena_bytes,
            meshlet_vertex_arena_bytes: arena_bytes,
            meshlet_triangle_arena_bytes: arena_bytes,
            meshes: entries(budgets.geometry_arenas / GEOMETRY_BYTES_PER_MESH),
            materials: entries(budgets.texture_pool / TEXTURE_POOL_BYTES_PER_MATERIAL),
            texture_classes_per_channel: entries(
                budgets.texture_pool / (channels * TEXTURE_POOL_BYTES_PER_CLASS),
            )
            .min(CLASSES_PER_CHANNEL),
        }
    }

    /// The capacities of `Budgets::checked_in`.
    pub fn checked_in() -> Self {
        Self::from_budgets(&Budgets::checked_in().gpu_memory_bytes_by_kind)
    }
}

/// What a Mesh is loaded for: the main pass draws it from its Meshlets, rays
/// trace it through its BLAS, or both.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MeshUse {
    DrawnAndTraced,
    /// Drawn alone: a Mesh the GPU bends, whose BLAS, built from its rest
    /// shape, would cast another shape's shadow.
    Drawn,
    /// Traced alone: a shadow proxy, which no pass draws, so it keeps no
    /// Meshlets and needs no Material.
    Traced,
    /// Drawn and traced, a Mesh the GPU bends whose BLAS is updated each
    /// frame from its bent shape: built to allow updates, never compacted.
    Refitted,
}

impl MeshUse {
    /// Whether the main pass draws it, so it needs Meshlets.
    pub(crate) fn drawn(self) -> bool {
        self != MeshUse::Traced
    }

    /// Whether rays trace it, so it needs a BLAS and the index stream the
    /// BLAS is built from.
    fn traced(self) -> bool {
        self != MeshUse::Drawn
    }

    /// Whether Renderables share one entry per Asset ID: a bent Mesh is its
    /// Renderable's own, as the GPU writes its vertices.
    fn shared(self) -> bool {
        matches!(self, MeshUse::DrawnAndTraced | MeshUse::Traced)
    }
}

/// The flags a refitted Mesh's BLAS is built with, which its refits repeat.
pub const REFITTED_BLAS_FLAGS: wgpu::AccelerationStructureFlags =
    wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE
        .union(wgpu::AccelerationStructureFlags::ALLOW_UPDATE);

/// A Mesh's BLAS and the build it was made by: its triangles' sizes, and
/// where they lie in the vertex and index Arenas, which a refit repeats.
pub struct MeshBlas {
    pub blas: wgpu::Blas,
    pub geometry: wgpu::BlasTriangleGeometrySizeDescriptor,
    pub first_vertex: u32,
    pub first_index: u32,
}

/// The Meshes, and the GPU tables they are drawn from: Arenas of vertices
/// and indices, the Meshlet Arenas and the Mesh table (ADRs 0017, 0019);
/// and a BLAS per Mesh that rays trace (ADR 0020).
pub struct MeshPool {
    device: wgpu::Device,
    meshes: LoadedMeshes,
    /// What freeing each loaded Mesh gives back, at its Handle's index.
    allocations: Vec<Option<MeshAllocation>>,
    /// Each Mesh loaded by Asset ID for a shared use.
    asset_handles: HashMap<(AssetId, MeshUse), SharedMesh>,
    /// The shared Meshes no load uses any more, which stay loaded until a
    /// refused load frees them.
    unused: Unused<(AssetId, MeshUse)>,
    /// The Asset IDs of the Meshes freed since `take_freed`.
    freed: Vec<AssetId>,
    /// Meshes freed to make room since `take_frame_frees`, up to
    /// `ENTRIES_FREED_PER_FRAME`.
    frame_frees: usize,
    /// Each Mesh's BLAS, at its Handle's index, built at load from its
    /// ranges of the vertex and index Arenas; none for a Mesh no ray traces.
    blases: Vec<Option<wgpu::Blas>>,
    /// The Meshes whose BLAS is built but not yet compacted, with its
    /// bytes.
    awaiting_compaction: Vec<(MeshHandle, u64)>,
    /// The BLASes of the Meshes loaded as `MeshUse::Refitted`.
    refitted_blases: Vec<MeshBlas>,
    /// Bytes of the BLASes compacted, as built, and of the others.
    compacted_blas_bytes: u64,
    uncompacted_blas_bytes: u64,
    /// The BLASes' bytes, the compacted ones' at `compacted_blas_fraction`
    /// of theirs.
    blas_charge: Charge,
    compacted_blas_fraction: f64,
    vertex_arena: Arena,
    index_arena: Arena,
    meshlet_arena: Arena,
    meshlet_vertex_arena: Arena,
    meshlet_triangle_arena: Arena,
    mesh_table: Counted<wgpu::Buffer>,
    mesh_capacity: u32,
}

impl MeshPool {
    /// A Mesh pool of `capacities`, counted in `memory`.
    pub fn new(device: &wgpu::Device, memory: &GpuMemory, capacities: &PoolCapacities) -> Self {
        let arena = |label, usage, element_bytes, capacity_bytes| {
            Arena::new(device, memory, label, usage, element_bytes, capacity_bytes)
        };
        let u32_bytes = size_of::<u32>() as u64;
        Self {
            device: device.clone(),
            meshes: LoadedMeshes::default(),
            allocations: Vec::new(),
            asset_handles: HashMap::new(),
            unused: Unused::default(),
            freed: Vec::new(),
            frame_frees: 0,
            blases: Vec::new(),
            awaiting_compaction: Vec::new(),
            refitted_blases: Vec::new(),
            compacted_blas_bytes: 0,
            uncompacted_blas_bytes: 0,
            blas_charge: memory.charge(GpuMemoryKind::AccelerationStructures, 0),
            compacted_blas_fraction: memory.compacted_blas_fraction(),
            // The vertex shader pulls from it (ADR 0019), BLAS builds read it.
            vertex_arena: arena(
                "Vertex Arena",
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::BLAS_INPUT,
                size_of::<CookedVertex>() as u64,
                capacities.vertex_arena_bytes,
            ),
            // The cooked index stream, which no pass draws from: BLAS builds
            // read it.
            index_arena: arena(
                "Index Arena",
                wgpu::BufferUsages::BLAS_INPUT,
                u32_bytes,
                capacities.index_arena_bytes,
            ),
            meshlet_arena: arena(
                "Meshlet Arena",
                wgpu::BufferUsages::STORAGE,
                size_of::<MeshletData>() as u64,
                capacities.meshlet_arena_bytes,
            ),
            meshlet_vertex_arena: arena(
                "Meshlet Vertex Arena",
                wgpu::BufferUsages::STORAGE,
                u32_bytes,
                capacities.meshlet_vertex_arena_bytes,
            ),
            meshlet_triangle_arena: arena(
                "Meshlet Triangle Arena",
                wgpu::BufferUsages::STORAGE,
                u32_bytes,
                capacities.meshlet_triangle_arena_bytes,
            ),
            mesh_table: memory.create_buffer(
                device,
                GpuMemoryKind::SceneAndPassBuffers,
                &wgpu::BufferDescriptor {
                    label: Some("Mesh Table"),
                    size: u64::from(capacities.meshes) * size_of::<MeshData>() as u64,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
            mesh_capacity: capacities.meshes,
        }
    }

    /// Loads `mesh` to be drawn and traced, as `load_for` loads it, its
    /// refusal logged.
    #[cfg(test)]
    pub fn load(
        &mut self,
        staging: &mut LoadStaging,
        mesh: &CookedMesh,
        materials: &[Option<MaterialHandle>],
    ) -> Option<MeshHandle> {
        self.load_for(staging, mesh, materials, MeshUse::DrawnAndTraced)
            .map_err(MeshRefusal::log)
            .ok()
    }

    /// The Handle of the Mesh `id` that `load_asset` loaded for `mesh_use`,
    /// counting one more user of it, unless a bent Mesh's use, which no two
    /// Renderables share.
    pub fn add_user(&mut self, id: AssetId, mesh_use: MeshUse) -> Option<MeshHandle> {
        let shared = self.asset_handles.get_mut(&(id, mesh_use))?;
        shared.users += 1;
        if let Some(since) = shared.unused_since.take() {
            self.unused.remove(since);
        }
        Some(shared.handle)
    }

    /// Whether `load_asset` loaded the Mesh `id` for `mesh_use`.
    pub fn is_loaded(&self, id: AssetId, mesh_use: MeshUse) -> bool {
        self.asset_handles.contains_key(&(id, mesh_use))
    }

    /// The bytes `load_for` writes through its staging for `mesh` loaded
    /// for `mesh_use`.
    pub fn written_bytes(mesh: &CookedMesh, mesh_use: MeshUse) -> u64 {
        let mut word_count = 0;
        if mesh_use.traced() {
            word_count += mesh.indices.len();
        }
        let mut meshlet_count = 0;
        if mesh_use.drawn() {
            for meshlet in mesh
                .primitives
                .iter()
                .flat_map(|primitive| &primitive.meshlets)
            {
                meshlet_count += 1;
                word_count += meshlet.vertices.len() + meshlet.triangles.len();
            }
        }
        (mesh.vertices.len() * size_of::<CookedVertex>()
            + word_count * size_of::<u32>()
            + meshlet_count * size_of::<MeshletData>()
            + size_of::<MeshData>()) as u64
    }

    /// Counts one user fewer of the Mesh `id` for `mesh_use`; one left with
    /// none stays loaded, the most recently used of the unused Meshes.
    pub fn remove_user(&mut self, id: AssetId, mesh_use: MeshUse) {
        let Some(shared) = self.asset_handles.get_mut(&(id, mesh_use)) else {
            return;
        };
        if shared.users == 0 {
            return;
        }
        shared.users -= 1;
        if shared.users == 0 {
            shared.unused_since = Some(self.unused.push((id, mesh_use)));
        }
    }

    /// How many loads returned the Mesh `id` for `mesh_use`, less those
    /// `remove_user` took back: a user per loaded Chunk that names it and
    /// per spawn that loaded it.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn users(&self, id: AssetId, mesh_use: MeshUse) -> u32 {
        self.asset_handles
            .get(&(id, mesh_use))
            .map_or(0, |shared| shared.users)
    }

    /// The Handle of `mesh`, the Mesh `id`, for `mesh_use`: the one `add_user`
    /// returns, or else a load as `load_for` does, kept for `add_user` with
    /// one user unless the Mesh is bent. A refused load frees unused Meshes,
    /// least recently used first, and is retried, each freed Mesh's Material
    /// Asset IDs, one per Primitive, pushed onto `released_materials` for the
    /// Material pool to count a user fewer of, and `mesh`'s own when it is
    /// refused still.
    pub fn load_asset(
        &mut self,
        staging: &mut LoadStaging,
        id: AssetId,
        mesh: &CookedMesh,
        materials: &[Option<MaterialHandle>],
        mesh_use: MeshUse,
        released_materials: &mut Vec<AssetId>,
    ) -> Option<MeshHandle> {
        if let Some(handle) = self.add_user(id, mesh_use) {
            return Some(handle);
        }
        let material_ids = mesh
            .primitives
            .iter()
            .zip(materials)
            .filter_map(|(primitive, handle)| handle.and(primitive.material));
        let handle = loop {
            match self.load_for(staging, mesh, materials, mesh_use) {
                Ok(handle) => break handle,
                Err(refusal) => {
                    if !self.free_least_recently_used(released_materials) {
                        refusal.log();
                        released_materials.extend(material_ids);
                        return None;
                    }
                }
            }
        };
        if mesh_use.shared() {
            let materials = material_ids.collect();
            let shared = SharedMesh {
                handle,
                users: 1,
                unused_since: None,
                materials,
            };
            self.asset_handles.insert((id, mesh_use), shared);
        }
        Some(handle)
    }

    /// Frees the least recently used unused Mesh, pushing its Material Asset
    /// IDs onto `released_materials`; false when none is unused or the frame
    /// freed `ENTRIES_FREED_PER_FRAME` already.
    pub fn free_least_recently_used(&mut self, released_materials: &mut Vec<AssetId>) -> bool {
        if self.frame_frees >= ENTRIES_FREED_PER_FRAME {
            return false;
        }
        let Some(key) = self.unused.pop_least_recent() else {
            return false;
        };
        let shared = self
            .asset_handles
            .remove(&key)
            .expect("an unused Mesh is shared");
        self.release(shared.handle);
        released_materials.extend(shared.materials);
        self.freed.push(key.0);
        self.frame_frees += 1;
        true
    }

    /// The Asset IDs of the Meshes loaded by Asset ID and freed since the
    /// last call.
    pub fn take_freed(&mut self) -> Vec<AssetId> {
        std::mem::take(&mut self.freed)
    }

    /// Meshes freed to make room since the last call, which renews the
    /// frame's `ENTRIES_FREED_PER_FRAME`.
    pub fn take_frame_frees(&mut self) -> usize {
        std::mem::take(&mut self.frame_frees)
    }

    /// Writes `mesh` through `staging` into one range of each Arena, and its
    /// entry into the Mesh table, each Primitive drawn with its Material of
    /// `materials`, in Primitive order, where no Renderable overrides it; and,
    /// where `mesh_use` has rays trace it, its index stream and its BLAS's
    /// build from its vertex and index ranges, which `compact_blases`
    /// compacts once `staging` is submitted, unless it is refitted. A Mesh no
    /// pass draws keeps no Meshlets. Returns its Handle, or why it was
    /// refused, unlogged, when the Mesh table is full or an Arena's longest
    /// free range is too short; a refused Mesh leaves the pool as it was.
    pub fn load_for(
        &mut self,
        staging: &mut LoadStaging,
        mesh: &CookedMesh,
        materials: &[Option<MaterialHandle>],
        mesh_use: MeshUse,
    ) -> Result<MeshHandle, MeshRefusal> {
        assert_eq!(
            materials.len(),
            mesh.primitives.len(),
            "a Material, or none, per Primitive"
        );
        let drawn_primitives = if mesh_use.drawn() {
            &mesh.primitives[..]
        } else {
            &[]
        };
        let meshlets = || {
            drawn_primitives
                .iter()
                .flat_map(|primitive| &primitive.meshlets)
        };
        let meshlet_count = meshlets().count();
        let meshlet_vertex_count = meshlets().map(|meshlet| meshlet.vertices.len()).sum();
        let triangle_count = meshlets().map(|meshlet| meshlet.triangles.len()).sum();
        // Only a BLAS build reads the index stream.
        let index_stream: &[u32] = if mesh_use.traced() {
            &mesh.indices
        } else {
            &[]
        };
        let counts = [
            mesh.vertices.len(),
            index_stream.len(),
            meshlet_count,
            meshlet_vertex_count,
            triangle_count,
        ];
        if self.meshes.full(self.mesh_capacity) {
            return Err(MeshRefusal::TableFull {
                capacity: self.mesh_capacity,
            });
        }
        let Some(
            [
                vertices,
                indices,
                meshlet_range,
                meshlet_vertices,
                meshlet_triangles,
            ],
        ) = allocate_all(self.arenas_mut(), counts)
        else {
            return Err(MeshRefusal::ArenaShort {
                counts,
                longest_free_ranges: self.arenas().map(Arena::longest_free_range),
            });
        };
        let handle = self.meshes.insert(Mesh {
            needs_material_handle: mesh_use.drawn() && materials.contains(&None),
            vertices: vertices.offset..vertices.offset + mesh.vertices.len() as u32,
            bounds: mesh.bounds,
            texel_densities: mesh
                .primitives
                .iter()
                .map(|primitive| primitive.texel_density)
                .collect(),
            materials: materials.to_vec(),
        });

        // A Meshlet's vertices index the vertex Arena, and its triangles
        // follow the Mesh's earlier Meshlets' in the triangle Arena.
        let mut meshlet_table = Vec::with_capacity(meshlet_count);
        let mut vertex_indices = Vec::with_capacity(meshlet_vertex_count);
        let mut triangles = Vec::with_capacity(triangle_count);
        for (primitive, material) in drawn_primitives.iter().zip(materials) {
            for meshlet in &primitive.meshlets {
                meshlet_table.push(MeshletData {
                    first_vertex: meshlet_vertices.offset + vertex_indices.len() as u32,
                    first_triangle: meshlet_triangles.offset + triangles.len() as u32,
                    triangle_count: meshlet.triangles.len() as u32,
                    material_id: material.map_or(NO_MATERIAL, MaterialHandle::index),
                });
                vertex_indices.extend(
                    meshlet
                        .vertices
                        .iter()
                        .map(|vertex| vertices.offset + vertex),
                );
                triangles.extend(
                    meshlet.triangles.iter().map(|&[first, second, third]| {
                        u32::from_le_bytes([first, second, third, 0])
                    }),
                );
            }
        }
        let mesh_data = MeshData {
            first_meshlet: meshlet_range.offset,
            meshlet_count: meshlet_count as u32,
            first_triangle: meshlet_triangles.offset,
            triangle_count: triangle_count as u32,
            bounds_min: mesh.bounds[0],
            bounds_max: mesh.bounds[1],
        };
        self.vertex_arena.write(
            staging,
            vertices.offset,
            bytemuck::cast_slice(&mesh.vertices),
        );
        self.index_arena
            .write(staging, indices.offset, bytemuck::cast_slice(index_stream));
        self.meshlet_arena.write(
            staging,
            meshlet_range.offset,
            bytemuck::cast_slice(&meshlet_table),
        );
        self.meshlet_vertex_arena.write(
            staging,
            meshlet_vertices.offset,
            bytemuck::cast_slice(&vertex_indices),
        );
        self.meshlet_triangle_arena.write(
            staging,
            meshlet_triangles.offset,
            bytemuck::cast_slice(&triangles),
        );
        staging.write(
            &self.mesh_table,
            u64::from(handle.index) * size_of::<MeshData>() as u64,
            bytemuck::bytes_of(&mesh_data),
        );
        tracing::debug!(
            vertices = mesh.vertices.len(),
            indices = index_stream.len(),
            meshlets = meshlet_count,
            "Mesh loaded"
        );
        let geometry = wgpu::BlasTriangleGeometrySizeDescriptor {
            vertex_format: wgpu::VertexFormat::Float32x3,
            vertex_count: mesh.vertices.len() as u32,
            index_format: Some(wgpu::IndexFormat::Uint32),
            index_count: Some(mesh.indices.len() as u32),
            flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
        };
        let refitted = mesh_use == MeshUse::Refitted;
        let flags = if refitted {
            REFITTED_BLAS_FLAGS
        } else {
            wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE
                | wgpu::AccelerationStructureFlags::ALLOW_COMPACTION
        };
        let built_bytes = if mesh_use.traced() {
            blas_bytes(&self.device, &self.vertex_arena.buffer, flags, &geometry)
        } else {
            0
        };
        let allocation = MeshAllocation {
            ranges: [
                vertices,
                indices,
                meshlet_range,
                meshlet_vertices,
                meshlet_triangles,
            ],
            blas_bytes: built_bytes,
            blas_compacted: false,
        };
        let index = handle.index as usize;
        if index == self.allocations.len() {
            self.allocations.push(Some(allocation));
            self.blases.push(None);
        } else {
            self.allocations[index] = Some(allocation);
        }
        if !mesh_use.traced() {
            return Ok(handle);
        }
        let blas = self.device.create_blas(
            &wgpu::CreateBlasDescriptor {
                label: Some("Mesh BLAS"),
                flags,
                update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            },
            wgpu::BlasGeometrySizeDescriptors::Triangles {
                descriptors: vec![geometry.clone()],
            },
        );
        let mesh_blas = MeshBlas {
            blas,
            geometry,
            first_vertex: vertices.offset,
            first_index: indices.offset,
        };
        staging.build_blas(&self.blas_build_entry(&mesh_blas));

        self.blases[index] = Some(mesh_blas.blas.clone());
        if refitted {
            self.refitted_blases.push(mesh_blas);
        } else {
            self.awaiting_compaction.push((handle, built_bytes));
        }
        self.uncompacted_blas_bytes += built_bytes;
        self.charge_blases();
        Ok(handle)
    }

    /// The build of `blas` from its ranges of the vertex and index Arenas:
    /// the position leads each vertex, and the cooked indices count from the
    /// Mesh's first vertex.
    fn blas_build_entry<'a>(&'a self, blas: &'a MeshBlas) -> wgpu::BlasBuildEntry<'a> {
        wgpu::BlasBuildEntry {
            blas: &blas.blas,
            geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
                size: &blas.geometry,
                vertex_buffer: &self.vertex_arena.buffer,
                first_vertex: blas.first_vertex,
                vertex_stride: size_of::<CookedVertex>() as u64,
                index_buffer: Some(self.index_arena()),
                first_index: Some(blas.first_index),
                transform_buffer: None,
                transform_buffer_offset: None,
            }]),
        }
    }

    /// Records into `encoder` a full rebuild through wgpu of each BLAS
    /// loaded as `MeshUse::Refitted`, from what its Arena ranges hold then:
    /// what a refit must match.
    pub fn rebuild_refitted_blases(&self, encoder: &mut wgpu::CommandEncoder) {
        let entries: Vec<_> = self
            .refitted_blases()
            .iter()
            .map(|blas| self.blas_build_entry(blas))
            .collect();
        encoder.build_acceleration_structures(&entries, std::iter::empty());
    }

    /// The BLASes of the Meshes loaded as `MeshUse::Refitted`.
    pub fn refitted_blases(&self) -> &[MeshBlas] {
        &self.refitted_blases
    }

    /// Compacts the BLASes of the Meshes loaded since the last call, once
    /// the staging their builds were recorded into is submitted: it waits
    /// for the GPU to build them, then to compact them. Nothing else orders
    /// a TLAS build after the copies: wgpu-core 30.0.1 records no barrier
    /// after `compact_blas`'s copy on any backend, and Metal has no
    /// acceleration-structure barrier at all (gfx-rs/wgpu#9215). A Mesh keeps
    /// its compacted BLAS, or the one built where compaction failed, as
    /// logged. Returns whether it waited.
    pub fn compact_blases(&mut self, queue: &wgpu::Queue) -> bool {
        if self.awaiting_compaction.is_empty() {
            return false;
        }
        for &(handle, _) in &self.awaiting_compaction {
            if let Some(blas) = &self.blases[handle.index as usize] {
                blas.prepare_compaction_async(|_| {});
            }
        }
        if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            tracing::error!(%error, "waiting for BLAS builds failed");
        }
        for (handle, bytes) in self.awaiting_compaction.drain(..) {
            let Some(blas) = &mut self.blases[handle.index as usize] else {
                continue;
            };
            if !blas.ready_for_compaction() {
                tracing::warn!(
                    mesh = handle.index,
                    "BLAS not compacted: not ready after its build"
                );
                continue;
            }
            *blas = queue.compact_blas(blas);
            if let Some(allocation) = &mut self.allocations[handle.index as usize] {
                allocation.blas_compacted = true;
            }
            self.uncompacted_blas_bytes -= bytes;
            self.compacted_blas_bytes += bytes;
        }
        self.charge_blases();
        // The copies wait in the queue's pending writes until a submit.
        queue.submit(std::iter::empty());
        if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            tracing::error!(%error, "waiting for BLAS compaction failed");
        }
        true
    }

    /// The BLAS of `handle`'s Mesh, or None when the pool has no such Mesh
    /// or no ray traces it.
    pub fn blas(&self, handle: MeshHandle) -> Option<&wgpu::Blas> {
        self.get(handle)?;
        self.blases[handle.index as usize].as_ref()
    }

    /// The Handle of the Mesh loaded at `index` of the Mesh table, which an
    /// Instance names, or None when none is.
    pub fn handle_at(&self, index: u32) -> Option<MeshHandle> {
        self.meshes.handle_at(index)
    }

    /// Frees `handle`'s Mesh: its ranges go back to their Arenas and its
    /// BLAS is dropped, and its index takes the next Mesh loaded, so `handle`
    /// finds nothing again; the next `prepare` rewrites the Instances that
    /// named it, so a free lands before it. A Handle that finds no Mesh is
    /// warned about, as is a refitted Mesh, which is not freed: the refit's
    /// scratch holds a region per refitted BLAS, in load order.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn free(&mut self, handle: MeshHandle) {
        let refitted = self.blas(handle).is_some_and(|blas| {
            self.refitted_blases
                .iter()
                .any(|refitted| &refitted.blas == blas)
        });
        if refitted {
            tracing::warn!(?handle, "Mesh not freed: its BLAS is refitted");
            return;
        }
        if !self.release(handle) {
            tracing::warn!(?handle, "Mesh not freed: its Handle finds no Mesh");
            return;
        }
        let freed_key = self
            .asset_handles
            .iter()
            .find_map(|(&key, shared)| (shared.handle == handle).then_some(key));
        if let Some(key) = freed_key
            && let Some(shared) = self.asset_handles.remove(&key)
        {
            if let Some(since) = shared.unused_since {
                self.unused.remove(since);
            }
            self.freed.push(key.0);
        }
    }

    /// Frees `handle`'s Mesh as `free` does, unchecked and unlogged but for
    /// its debug event; false, changing nothing, when it finds no Mesh.
    fn release(&mut self, handle: MeshHandle) -> bool {
        if !self.meshes.remove(handle) {
            return false;
        }
        let index = handle.index as usize;
        let allocation = self.allocations[index]
            .take()
            .expect("a live index holds a Mesh's allocation");
        for (arena, range) in self.arenas_mut().into_iter().zip(allocation.ranges) {
            arena.free(range);
        }
        if self.blases[index].take().is_some() {
            self.awaiting_compaction
                .retain(|&(awaiting, _)| awaiting != handle);
            if allocation.blas_compacted {
                self.compacted_blas_bytes -= allocation.blas_bytes;
            } else {
                self.uncompacted_blas_bytes -= allocation.blas_bytes;
            }
            self.charge_blases();
        }
        tracing::debug!(?handle, "Mesh freed");
        true
    }

    /// The loaded Meshes, as prepare reads them.
    pub fn meshes(&self) -> &LoadedMeshes {
        &self.meshes
    }

    pub fn vertex_arena(&self) -> &wgpu::Buffer {
        &self.vertex_arena.buffer
    }

    /// The cooked index streams, which BLAS builds read.
    pub fn index_arena(&self) -> &wgpu::Buffer {
        &self.index_arena.buffer
    }

    /// Each Mesh's `MeshData`, at its Handle's index.
    pub fn mesh_table(&self) -> &wgpu::Buffer {
        &self.mesh_table
    }

    /// The Meshes' `MeshletData`.
    pub fn meshlet_arena(&self) -> &wgpu::Buffer {
        &self.meshlet_arena.buffer
    }

    pub fn meshlet_vertex_arena(&self) -> &wgpu::Buffer {
        &self.meshlet_vertex_arena.buffer
    }

    pub fn meshlet_triangle_arena(&self) -> &wgpu::Buffer {
        &self.meshlet_triangle_arena.buffer
    }

    fn arenas(&self) -> [&Arena; 5] {
        [
            &self.vertex_arena,
            &self.index_arena,
            &self.meshlet_arena,
            &self.meshlet_vertex_arena,
            &self.meshlet_triangle_arena,
        ]
    }

    fn arenas_mut(&mut self) -> [&mut Arena; 5] {
        [
            &mut self.vertex_arena,
            &mut self.index_arena,
            &mut self.meshlet_arena,
            &mut self.meshlet_vertex_arena,
            &mut self.meshlet_triangle_arena,
        ]
    }

    fn charge_blases(&mut self) {
        let compacted_bytes =
            (self.compacted_blas_bytes as f64 * self.compacted_blas_fraction).round() as u64;
        let bytes = compacted_bytes + self.uncompacted_blas_bytes;
        self.blas_charge
            .set(GpuMemoryKind::AccelerationStructures, bytes);
    }

    /// Bytes of the Arenas that hold Meshes.
    pub fn arena_bytes_in_use(&self) -> u64 {
        self.arenas().iter().map(|arena| arena.bytes_in_use).sum()
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle)
    }
}

/// The Meshes a Mesh pool holds, each at its Handle's index, with what
/// prepare reads of them; a Handle whose Mesh was freed finds none.
#[derive(Default)]
pub struct LoadedMeshes {
    generations: Generations,
    /// Each Mesh, at its Handle's index; a freed one's stays until a later
    /// Mesh takes its index.
    meshes: Vec<Mesh>,
}

impl LoadedMeshes {
    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.generations
            .live(handle.index, handle.generation)
            .then(|| &self.meshes[handle.index as usize])
    }

    /// Meshes freed since the first insert: once it moves, an Instance may
    /// name an index whose Mesh is gone.
    pub fn frees(&self) -> u64 {
        self.generations.frees()
    }

    /// Whether `capacity` Meshes are loaded, so `insert` must not be called.
    pub(crate) fn full(&self, capacity: u32) -> bool {
        self.generations.full(capacity)
    }

    /// Puts `mesh` at a free index, the last freed or else a fresh one, and
    /// returns its Handle.
    pub(crate) fn insert(&mut self, mesh: Mesh) -> MeshHandle {
        let (index, generation) = self.generations.take();
        if index as usize == self.meshes.len() {
            self.meshes.push(mesh);
        } else {
            self.meshes[index as usize] = mesh;
        }
        MeshHandle { index, generation }
    }

    /// The Handle of the Mesh at `index`, or None when none is.
    pub(crate) fn handle_at(&self, index: u32) -> Option<MeshHandle> {
        let generation = self.generations.live_generation(index)?;
        Some(MeshHandle { index, generation })
    }

    /// Frees `handle`'s index, so `handle` finds nothing again; false,
    /// changing nothing, when it finds no Mesh.
    pub(crate) fn remove(&mut self, handle: MeshHandle) -> bool {
        self.generations.free(handle.index, handle.generation)
    }
}

/// A Mesh loaded by Asset ID for a shared use: its Handle, how many loads
/// returned it, less those `remove_user` took back, when its last user left,
/// and the Material Asset IDs its Primitives name, which count it as a user.
struct SharedMesh {
    handle: MeshHandle,
    users: u32,
    unused_since: Option<u64>,
    materials: Vec<AssetId>,
}

/// Why `load_for` refused a Mesh.
#[derive(Debug)]
pub enum MeshRefusal {
    TableFull {
        capacity: u32,
    },
    /// The elements it needed of each Arena, and each one's longest free
    /// range, in `MeshPool::arenas` order.
    ArenaShort {
        counts: [usize; 5],
        longest_free_ranges: [u32; 5],
    },
}

impl MeshRefusal {
    pub fn log(self) {
        match self {
            MeshRefusal::TableFull { capacity } => {
                tracing::error!(capacity, "Mesh refused: the Mesh table is full");
            }
            MeshRefusal::ArenaShort {
                counts: [vertices, indices, meshlets, meshlet_vertices, triangles],
                longest_free_ranges:
                    [
                        longest_free_vertex_range,
                        longest_free_index_range,
                        longest_free_meshlet_range,
                        longest_free_meshlet_vertex_range,
                        longest_free_meshlet_triangle_range,
                    ],
            } => tracing::error!(
                vertices,
                indices,
                meshlets,
                meshlet_vertices,
                triangles,
                longest_free_vertex_range,
                longest_free_index_range,
                longest_free_meshlet_range,
                longest_free_meshlet_vertex_range,
                longest_free_meshlet_triangle_range,
                "Mesh refused: an Arena's longest free range is too short"
            ),
        }
    }
}

/// A pool's unused entries in the order their last users left, keyed by
/// the tick `push` returned, so the least recently used comes first.
pub(crate) struct Unused<Key> {
    order: BTreeMap<u64, Key>,
    next_tick: u64,
}

impl<Key> Default for Unused<Key> {
    fn default() -> Self {
        Self {
            order: BTreeMap::new(),
            next_tick: 0,
        }
    }
}

impl<Key> Unused<Key> {
    /// Adds `key` as the most recently used, returning its tick.
    pub(crate) fn push(&mut self, key: Key) -> u64 {
        let tick = self.next_tick;
        self.next_tick += 1;
        self.order.insert(tick, key);
        tick
    }

    /// Takes out the key `push` returned `tick` for.
    pub(crate) fn remove(&mut self, tick: u64) {
        self.order.remove(&tick);
    }

    pub(crate) fn pop_least_recent(&mut self) -> Option<Key> {
        self.order.pop_first().map(|(_, key)| key)
    }
}

/// What freeing a loaded Mesh gives back: its range of each Arena, in
/// `MeshPool::arenas` order, and its BLAS's bytes as built, compacted or not.
struct MeshAllocation {
    ranges: [offset_allocator::Allocation; 5],
    blas_bytes: u64,
    blas_compacted: bool,
}

/// The indices of a pool's table, each with the generation its Handles
/// carry: a freed index's generation moves on, so no earlier Handle finds
/// the entry a later take puts there.
#[derive(Default)]
pub(crate) struct Generations {
    indices: Vec<IndexGeneration>,
    free: Vec<u32>,
    frees: u64,
}

#[derive(Clone, Copy)]
struct IndexGeneration {
    generation: u32,
    live: bool,
}

impl Generations {
    /// Whether `capacity` indices are live, so `take` must not be called.
    pub(crate) fn full(&self, capacity: u32) -> bool {
        self.free.is_empty() && self.indices.len() as u32 >= capacity
    }

    /// A free index, the last freed or else a fresh one, made live, and its
    /// generation.
    pub(crate) fn take(&mut self) -> (u32, u32) {
        let index = self.free.pop().unwrap_or_else(|| {
            self.indices.push(IndexGeneration {
                generation: 0,
                live: false,
            });
            self.indices.len() as u32 - 1
        });
        let taken = &mut self.indices[index as usize];
        taken.live = true;
        (index, taken.generation)
    }

    /// Whether `index` is live in `generation`.
    pub(crate) fn live(&self, index: u32, generation: u32) -> bool {
        self.live_generation(index) == Some(generation)
    }

    /// The generation `index` is live in, or None when it is free.
    pub(crate) fn live_generation(&self, index: u32) -> Option<u32> {
        let held = self.indices.get(index as usize)?;
        held.live.then_some(held.generation)
    }

    /// Frees `index` when it is live in `generation`, moving its generation
    /// on; false, changing nothing, otherwise.
    pub(crate) fn free(&mut self, index: u32, generation: u32) -> bool {
        if !self.live(index, generation) {
            return false;
        }
        let freed = &mut self.indices[index as usize];
        freed.live = false;
        freed.generation = freed.generation.wrapping_add(1);
        self.free.push(index);
        self.frees += 1;
        true
    }

    /// Indices freed since the first take.
    pub(crate) fn frees(&self) -> u64 {
        self.frees
    }
}

/// A range of each of `counts` elements in each of `arenas`, or None, with
/// every Arena as it was, when one's longest free range is too short.
fn allocate_all<const N: usize>(
    mut arenas: [&mut Arena; N],
    counts: [usize; N],
) -> Option<[offset_allocator::Allocation; N]> {
    let mut ranges = Vec::with_capacity(N);
    for (arena, count) in arenas.iter_mut().zip(counts) {
        match arena.allocate(count) {
            Some(range) => ranges.push(range),
            None => break,
        }
    }
    if ranges.len() < N {
        for (arena, range) in arenas.iter_mut().zip(ranges) {
            arena.free(range);
        }
        return None;
    }
    ranges.try_into().ok()
}

/// A GPU buffer of fixed capacity, never grown, sub-allocated in ranges of
/// elements by `offset-allocator` (ADR 0017).
struct Arena {
    buffer: Counted<wgpu::Buffer>,
    allocator: offset_allocator::Allocator,
    element_bytes: u64,
    bytes_in_use: u64,
}

impl Arena {
    fn new(
        device: &wgpu::Device,
        memory: &GpuMemory,
        label: &str,
        usage: wgpu::BufferUsages,
        element_bytes: u64,
        capacity_bytes: u64,
    ) -> Self {
        // wgpu-hal's largest buffer on DX12, and on Linux and Android Vulkan
        // other than NVIDIA's (ADR 0017).
        assert!(
            capacity_bytes < 1 << 31,
            "an Arena of {capacity_bytes} bytes is past 2^31 - 1"
        );
        Self {
            buffer: memory.create_buffer(
                device,
                GpuMemoryKind::GeometryArenas,
                &wgpu::BufferDescriptor {
                    label: Some(label),
                    size: capacity_bytes,
                    usage: usage | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                },
            ),
            allocator: offset_allocator::Allocator::new((capacity_bytes / element_bytes) as u32),
            element_bytes,
            bytes_in_use: 0,
        }
    }

    /// A range of `count` elements, or None when `longest_free_range` is
    /// shorter.
    fn allocate(&mut self, count: usize) -> Option<offset_allocator::Allocation> {
        let allocation = self.allocator.allocate(u32::try_from(count).ok()?)?;
        self.bytes_in_use +=
            u64::from(self.allocator.allocation_size(allocation)) * self.element_bytes;
        Some(allocation)
    }

    fn free(&mut self, allocation: offset_allocator::Allocation) {
        self.bytes_in_use -=
            u64::from(self.allocator.allocation_size(allocation)) * self.element_bytes;
        self.allocator.free(allocation);
    }

    /// Writes `bytes` from element `first_element` on.
    fn write(&self, staging: &mut LoadStaging, first_element: u32, bytes: &[u8]) {
        let offset = u64::from(first_element) * self.element_bytes;
        staging.write(&self.buffer, offset, bytes);
    }

    /// Elements in the longest range `allocate` can return.
    fn longest_free_range(&self) -> u32 {
        self.allocator.storage_report().largest_free_region
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::capture::submit_and_map;
    use crate::cooked::{CookedMeshlet, CookedPrimitive};
    use crate::gpu::GpuContext;
    use crate::stats::gpu_memory::tests::gpu_memory;

    /// Room for four 28-byte vertices, or 28 indices.
    const CAPACITY: u64 = 4 * 28;

    /// The checked-in capacities with every Arena of `bytes`.
    fn arenas_of(bytes: u64) -> PoolCapacities {
        PoolCapacities {
            vertex_arena_bytes: bytes,
            index_arena_bytes: bytes,
            meshlet_arena_bytes: bytes,
            meshlet_vertex_arena_bytes: bytes,
            meshlet_triangle_arena_bytes: bytes,
            ..PoolCapacities::checked_in()
        }
    }

    #[test]
    fn the_checked_in_budgets_keep_the_capacities_before_them() {
        assert_eq!(
            PoolCapacities::checked_in(),
            PoolCapacities {
                vertex_arena_bytes: 16 << 20,
                index_arena_bytes: 16 << 20,
                meshlet_arena_bytes: 16 << 20,
                meshlet_vertex_arena_bytes: 16 << 20,
                meshlet_triangle_arena_bytes: 16 << 20,
                meshes: 1 << 16,
                materials: 256,
                texture_classes_per_channel: 13_107,
            }
        );
    }

    /// Budgets for two Meshes make a Mesh table of two and five Arenas of
    /// 512 bytes: a third triangle, which the Arenas have room for, is
    /// refused and logged.
    #[test]
    fn a_mesh_past_the_mesh_table_its_budget_sizes_is_refused() {
        let Some(context) = gpu_context() else {
            return;
        };
        let capacities = PoolCapacities::from_budgets(&GpuMemoryBudgets {
            geometry_arenas: 2560,
            ..GpuMemoryBudgets::each(0)
        });
        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let mut pool = MeshPool::new(&context.device, &memory, &capacities);
        let bytes_by_kind = memory.bytes_by_kind();
        assert_eq!(bytes_by_kind[GpuMemoryKind::GeometryArenas as usize], 2560);
        assert_eq!(
            bytes_by_kind[GpuMemoryKind::SceneAndPassBuffers as usize],
            2 * size_of::<MeshData>() as u64
        );
        let triangle = mesh(3, vec![0, 1, 2]);
        for _ in 0..2 {
            pool.load(&mut staging, &triangle, &[None])
                .expect("a triangle fits");
        }
        let (handle, log) = logged(|| pool.load(&mut staging, &triangle, &[None]));
        assert!(handle.is_none(), "a Mesh past the Mesh table loaded");
        assert!(
            log.contains("the Mesh table is full"),
            "not logged: {log:?}"
        );
    }

    /// A one-Primitive Mesh of `vertex_count` vertices and `indices`, its
    /// whole triangles one Meshlet.
    pub(crate) fn mesh(vertex_count: usize, indices: Vec<u32>) -> CookedMesh {
        let (triangles, _) = indices.as_chunks::<3>();
        let meshlets = CookedMeshlet::split(triangles, [triangles.len()]).expect("one Meshlet");
        CookedMesh {
            vertices: vec![CookedVertex::default(); vertex_count],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: indices.len() as u32,
                material: None,
                texel_density: 0.0,
                meshlets,
            }],
            indices,
            bounds: [[0.0; 3]; 2],
        }
    }

    #[test]
    fn a_mesh_loaded_twice_keeps_one_entry_unless_it_is_bent() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MeshPool::new(
            &context.device,
            &gpu_memory(&context),
            &PoolCapacities::checked_in(),
        );
        let triangle = mesh(3, vec![0, 1, 2]);
        let mut load = |id, mesh_use| {
            pool.load_asset(
                &mut staging,
                AssetId(id),
                &triangle,
                &[None],
                mesh_use,
                &mut Vec::new(),
            )
            .expect("a triangle fits")
            .index
        };
        let handles = [
            MeshUse::DrawnAndTraced,
            MeshUse::DrawnAndTraced,
            MeshUse::Traced,
            MeshUse::Traced,
            MeshUse::Drawn,
            MeshUse::Drawn,
            MeshUse::Refitted,
            MeshUse::Refitted,
        ]
        .map(|mesh_use| load(1, mesh_use));
        let other = load(2, MeshUse::DrawnAndTraced);

        assert_eq!(handles, [0, 0, 1, 1, 2, 3, 4, 5]);
        assert_eq!(other, 6);
        assert_eq!(pool.allocations.len(), 7);
    }

    /// A Mesh of two Primitives, loaded after another Mesh, reaches the
    /// tables as the GPU reads them: its entry at its Handle, then each of
    /// its Meshlets, with its Primitive's Material ID, whose triangles' corners
    /// pull the Mesh's own vertices from the vertex Arena, in cooked order.
    #[test]
    fn a_meshs_meshlets_reach_the_tables_with_their_primitives_material_ids() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MeshPool::new(
            &context.device,
            &gpu_memory(&context),
            &PoolCapacities::checked_in(),
        );
        pool.load(&mut staging, &mesh(3, vec![0, 1, 2]), &[None])
            .expect("a triangle fits");
        // Each vertex's x is its index, so a pulled vertex names itself.
        let vertex = |x| CookedVertex {
            position: [x, 1.0, 2.0],
            ..CookedVertex::default()
        };
        let indices = vec![0, 1, 2, 2, 1, 3];
        let primitive = |first_index: usize| CookedPrimitive {
            first_index: first_index as u32,
            index_count: 3,
            material: None,
            texel_density: first_index as f32,
            meshlets: CookedMeshlet::split(&[[0, 1, 2], [2, 1, 3]][first_index / 3..][..1], [1])
                .expect("one Meshlet"),
        };
        let two_primitives = CookedMesh {
            vertices: (0..4).map(|index| vertex(index as f32)).collect(),
            indices: indices.clone(),
            primitives: vec![primitive(0), primitive(3)],
            bounds: [[-1.0, -2.0, -3.0], [4.0, 5.0, 6.0]],
        };
        let material = MaterialHandle {
            index: 5,
            generation: 0,
        };
        let handle = pool
            .load(&mut staging, &two_primitives, &[Some(material), None])
            .expect("the Mesh fits");
        staging.submit(&context.queue);
        assert_eq!(pool.get(handle).unwrap().texel_densities, [0.0, 3.0]);

        let mesh_table: Vec<MeshData> = read_storage(&context, &pool.mesh_table, 2);
        let mesh_data = mesh_table[handle.index as usize];
        assert_eq!((mesh_data.meshlet_count, mesh_data.triangle_count), (2, 2));
        assert_eq!(
            [mesh_data.bounds_min, mesh_data.bounds_max],
            two_primitives.bounds
        );
        let meshlets: Vec<MeshletData> = read_storage(&context, &pool.meshlet_arena.buffer, 64);
        let meshlet_vertices: Vec<u32> =
            read_storage(&context, &pool.meshlet_vertex_arena.buffer, 64);
        let triangles: Vec<u32> = read_storage(&context, &pool.meshlet_triangle_arena.buffer, 64);
        let vertices: Vec<CookedVertex> = read_storage(&context, &pool.vertex_arena.buffer, 64);
        let mut pulled = Vec::new();
        for (meshlet_in_mesh, material_id) in
            [material.index(), NO_MATERIAL].into_iter().enumerate()
        {
            let meshlet = meshlets[mesh_data.first_meshlet as usize + meshlet_in_mesh];
            assert_eq!(meshlet.material_id, material_id);
            assert_eq!(
                meshlet.first_triangle,
                mesh_data.first_triangle + meshlet_in_mesh as u32
            );
            let corners = triangles[meshlet.first_triangle as usize].to_le_bytes();
            for &corner in &corners[..3] {
                let vertex = meshlet_vertices[(meshlet.first_vertex + u32::from(corner)) as usize];
                pulled.push(vertices[vertex as usize].position[0] as u32);
            }
        }
        assert_eq!(pulled, indices);
    }

    /// A freed Mesh gives back its Arena ranges and its BLAS's bytes; the
    /// next Mesh loaded takes its index in a new generation, which the freed
    /// Handle does not find, and its entry in the Mesh table is the new one.
    #[test]
    fn a_freed_meshs_index_takes_the_next_mesh_and_refuses_the_old_handle() {
        let Some(context) = gpu_context() else {
            return;
        };
        let memory = gpu_memory(&context);
        let acceleration_structure_bytes =
            || memory.bytes_by_kind()[GpuMemoryKind::AccelerationStructures as usize];
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MeshPool::new(&context.device, &memory, &PoolCapacities::checked_in());
        let refitted = pool
            .load_for(
                &mut staging,
                &mesh(3, vec![0, 1, 2]),
                &[None],
                MeshUse::Refitted,
            )
            .expect("a triangle fits");
        let (_, log) = logged(|| pool.free(refitted));
        assert!(log.contains("its BLAS is refitted"), "not logged: {log:?}");
        assert!(pool.get(refitted).is_some());
        assert_eq!(pool.meshes().frees(), 0);
        let arena_bytes_in_use = pool.arena_bytes_in_use();
        let blas_bytes = acceleration_structure_bytes();
        let freed = pool
            .load_asset(
                &mut staging,
                AssetId(1),
                &mesh(4, vec![0, 1, 2, 2, 1, 3]),
                &[None],
                MeshUse::DrawnAndTraced,
                &mut Vec::new(),
            )
            .expect("a quad fits");
        let quad_arena_bytes = pool.arena_bytes_in_use() - arena_bytes_in_use;
        assert!(acceleration_structure_bytes() > blas_bytes);

        pool.free(freed);
        assert_eq!(pool.arena_bytes_in_use(), arena_bytes_in_use);
        assert_eq!(acceleration_structure_bytes(), blas_bytes);
        assert!(pool.get(freed).is_none() && pool.blas(freed).is_none());
        assert_eq!(pool.handle_at(freed.index), None);
        assert_eq!(pool.users(AssetId(1), MeshUse::DrawnAndTraced), 0);
        assert_eq!(pool.meshes().frees(), 1);
        let (_, log) = logged(|| pool.free(freed));
        assert!(log.contains("Mesh not freed"), "not logged: {log:?}");
        assert_eq!(pool.meshes().frees(), 1);

        let mut bounds = [[0.0; 3], [1.0; 3]];
        let mut replacement = mesh(4, vec![0, 1, 2, 2, 1, 3]);
        replacement.bounds = bounds;
        let loaded = pool
            .load(&mut staging, &replacement, &[None])
            .expect("a quad fits");
        assert_eq!(loaded.index, freed.index);
        assert_ne!(loaded, freed);
        assert!(pool.get(freed).is_none() && pool.blas(freed).is_none());
        assert!(pool.get(loaded).is_some() && pool.blas(loaded).is_some());
        assert_eq!(pool.handle_at(loaded.index), Some(loaded));
        assert_eq!(
            pool.arena_bytes_in_use(),
            arena_bytes_in_use + quad_arena_bytes
        );
        staging.submit(&context.queue);
        let mesh_table: Vec<MeshData> = read_storage(&context, &pool.mesh_table, 2);
        let entry = mesh_table[loaded.index as usize];
        bounds = [entry.bounds_min, entry.bounds_max];
        assert_eq!(bounds, replacement.bounds);
        assert_eq!(entry.triangle_count, 2);
    }

    /// Arenas full of Meshes whose last users left: the next load frees the
    /// one whose last user left first, takes its ranges and succeeds, giving
    /// back the Material its Primitive named; the freed Handle finds nothing,
    /// and with every Mesh used again a load is refused.
    #[test]
    fn a_load_into_full_arenas_frees_the_least_recently_used_unused_mesh() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MeshPool::new(&context.device, &gpu_memory(&context), &arenas_of(CAPACITY));
        let mut released = Vec::new();
        let mut load = |pool: &mut MeshPool, released: &mut Vec<AssetId>, id| {
            let mut two_vertices = mesh(2, vec![0, 1, 1]);
            two_vertices.primitives[0].material = Some(AssetId(id + 100));
            let material = MaterialHandle {
                index: id as u32,
                generation: 0,
            };
            pool.load_asset(
                &mut staging,
                AssetId(id),
                &two_vertices,
                &[Some(material)],
                MeshUse::DrawnAndTraced,
                released,
            )
        };
        let first = load(&mut pool, &mut released, 1).expect("two vertices fit");
        let second = load(&mut pool, &mut released, 2).expect("two more fit");
        let second_vertices = pool.get(second).unwrap().vertices.clone();
        pool.remove_user(AssetId(2), MeshUse::DrawnAndTraced);
        pool.remove_user(AssetId(1), MeshUse::DrawnAndTraced);

        let third = load(&mut pool, &mut released, 3).expect("an unused Mesh is freed");
        assert!(pool.get(second).is_none() && pool.blas(second).is_none());
        assert!(pool.get(first).is_some());
        assert_eq!(pool.get(third).unwrap().vertices, second_vertices);
        assert_eq!(pool.add_user(AssetId(2), MeshUse::DrawnAndTraced), None);
        assert_eq!(released, [AssetId(102)]);
        assert_eq!(pool.take_freed(), [AssetId(2)]);

        assert_eq!(
            pool.add_user(AssetId(1), MeshUse::DrawnAndTraced),
            Some(first)
        );
        let (refused, log) = logged(|| load(&mut pool, &mut released, 4));
        assert!(refused.is_none(), "a Mesh in use was freed");
        assert!(log.contains("Mesh refused"), "not logged: {log:?}");
        assert!(pool.get(first).is_some() && pool.get(third).is_some());
        assert!(pool.take_freed().is_empty());
    }

    /// A Mesh loaded after one whose triangle is degenerate, so that its
    /// ranges of the vertex and index Arenas start past their first elements,
    /// is traced where its triangle is once its BLAS is built and compacted;
    /// each BLAS counts the bytes its driver gives for a triangle's, then
    /// the ledger's fraction of them.
    #[test]
    fn a_meshs_blas_holds_its_triangle_from_its_arena_ranges_and_is_compacted() {
        let Some(context) = gpu_context() else {
            return;
        };
        let memory = GpuMemory::new(false, 0.25);
        let acceleration_structure_bytes =
            || memory.bytes_by_kind()[GpuMemoryKind::AccelerationStructures as usize];
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MeshPool::new(&context.device, &memory, &PoolCapacities::checked_in());
        pool.load(&mut staging, &mesh(3, vec![0, 0, 0]), &[None])
            .expect("a triangle fits");
        let handle = pool
            .load(&mut staging, &triangle_about_x(), &[None])
            .expect("a triangle fits");
        let triangle_bytes = blas_bytes(
            &context.device,
            pool.vertex_arena(),
            wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE
                | wgpu::AccelerationStructureFlags::ALLOW_COMPACTION,
            &wgpu::BlasTriangleGeometrySizeDescriptor {
                vertex_format: wgpu::VertexFormat::Float32x3,
                vertex_count: 3,
                index_format: Some(wgpu::IndexFormat::Uint32),
                index_count: Some(3),
                flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
            },
        );
        assert!(triangle_bytes > 0);
        assert_eq!(acceleration_structure_bytes(), 2 * triangle_bytes);
        staging.submit(&context.queue);
        pool.compact_blases(&context.queue);
        assert_eq!(
            acceleration_structure_bytes(),
            (2.0 * triangle_bytes as f64 * 0.25).round() as u64
        );

        let blas = pool.blas(handle).expect("the triangle's BLAS");
        assert_eq!(
            trace_triangle_about_x(&context, blas),
            [Some(7), None, None]
        );
    }

    /// A TLAS built once `compact_blases` returns holds the compacted BLAS
    /// while other devices load the GPU, as parallel tests do: no barrier
    /// orders a TLAS build after the copies that write its BLASes, so
    /// `compact_blases` waits for them. Eight threads, each with its
    /// own device, load and trace a triangle 25 times.
    #[test]
    fn a_tlas_built_after_compaction_holds_its_blas_while_other_devices_load_the_gpu() {
        if gpu_context().is_none() {
            return;
        }
        let load_and_trace = || {
            let context = gpu_context().expect("a GPU adapter, as found above");
            let mut staging = LoadStaging::new(context.device.clone());
            // Room for its 25 triangles; eight pools of full Arenas would
            // take 660 MiB.
            let mut pool = MeshPool::new(
                &context.device,
                &gpu_memory(&context),
                &PoolCapacities {
                    meshes: 25,
                    ..arenas_of(4 << 10)
                },
            );
            (0..25)
                .map(|_| {
                    let handle = pool
                        .load(&mut staging, &triangle_about_x(), &[None])
                        .expect("a triangle fits");
                    staging.submit(&context.queue);
                    pool.compact_blases(&context.queue);
                    let blas = pool.blas(handle).expect("the triangle's BLAS");
                    trace_triangle_about_x(&context, blas)
                })
                .collect::<Vec<_>>()
        };
        std::thread::scope(|scope| {
            let threads: Vec<_> = (0..8).map(|_| scope.spawn(load_and_trace)).collect();
            for thread in threads {
                for traced in thread.join().expect("a tracing thread finishes") {
                    assert_eq!(traced, [Some(7), None, None]);
                }
            }
        });
    }

    /// What `trace_down` finds at a point on `triangle_about_x`'s triangle and
    /// at two points off it, in a TLAS of `blas` alone, tagged 7.
    fn trace_triangle_about_x(context: &GpuContext, blas: &wgpu::Blas) -> Vec<Option<u32>> {
        let mut tlas = context.device.create_tlas(&wgpu::CreateTlasDescriptor {
            label: Some("Test TLAS"),
            max_instances: 1,
            flags: wgpu::AccelerationStructureFlags::empty(),
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        });
        let identity = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
        tlas[0] = Some(wgpu::TlasInstance::new(blas, identity, 7, 0xff));
        let mut encoder = context.device.create_command_encoder(&Default::default());
        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&tlas));
        context.queue.submit(std::iter::once(encoder.finish()));
        let points = [[0.8, 0.0, 0.0], [0.0, 0.0, 0.0], [0.8, 0.5, 0.0]];
        trace_down(context, &tlas, &points)
    }

    /// A one-Primitive Mesh of a triangle about (0.8, 0) in mesh space, at
    /// z 0, with the bounds that hold it.
    pub(crate) fn triangle_about_x() -> CookedMesh {
        let mut triangle = mesh(3, vec![0, 1, 2]);
        triangle.bounds = [[0.6, -0.2, 0.0], [1.0, 0.2, 0.0]];
        triangle.vertices = [[0.6, -0.2], [1.0, -0.2], [0.8, 0.2]]
            .map(|[x, y]| CookedVertex {
                position: [x, y, 0.0],
                ..CookedVertex::default()
            })
            .to_vec();
        triangle
    }

    /// What a ray down -Z from 1 m above each of `points`, in `tlas`'s
    /// frame, hits within 2 m: the tag of the Instance it hits.
    pub(crate) fn trace_down(
        context: &GpuContext,
        tlas: &wgpu::Tlas,
        points: &[[f32; 3]],
    ) -> Vec<Option<u32>> {
        let device = &context.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Trace Down"),
            source: wgpu::ShaderSource::Wgsl(
                "enable wgpu_ray_query;
                @group(0) @binding(0) var tlas: acceleration_structure;
                @group(0) @binding(1) var<storage, read> points: array<vec4<f32>>;
                @group(0) @binding(2) var<storage, read_write> tags: array<u32>;
                @compute @workgroup_size(1)
                fn trace_down(@builtin(global_invocation_id) invocation: vec3<u32>) {
                    let origin = points[invocation.x].xyz + vec3(0.0, 0.0, 1.0);
                    let down = vec3(0.0, 0.0, -1.0);
                    let ray = RayDesc(RAY_FLAG_NONE, 0xffu, 0.0, 2.0, origin, down);
                    var query: ray_query;
                    rayQueryInitialize(&query, tlas, ray);
                    while rayQueryProceed(&query) {}
                    let hit = rayQueryGetCommittedIntersection(&query);
                    let missed = hit.kind == RAY_QUERY_INTERSECTION_NONE;
                    tags[invocation.x] = select(hit.instance_custom_data, 0xffffffffu, missed);
                }"
                .into(),
            ),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Trace Down"),
            layout: None,
            module: &module,
            entry_point: Some("trace_down"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let points: Vec<[f32; 4]> = points.iter().map(|&[x, y, z]| [x, y, z, 0.0]).collect();
        let point_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Trace Down Points"),
                contents: bytemuck::cast_slice(&points),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let size = (points.len() * size_of::<u32>()) as wgpu::BufferAddress;
        let buffer_of = |usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Trace Down Tags"),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let tags = buffer_of(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC);
        let read = buffer_of(wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Trace Down"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: tlas.as_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: tags.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(points.len() as u32, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&tags, 0, &read, 0, size);
        submit_and_map(context, encoder, &read).expect("read the tags back");
        let words = read.get_mapped_range(..).expect("mapped");
        bytemuck::cast_slice::<u8, u32>(&words)
            .iter()
            .map(|&tag| (tag != u32::MAX).then_some(tag))
            .collect()
    }

    /// The first `count` elements of `buffer`, a storage buffer, as a shader
    /// reads them.
    pub(crate) fn read_storage<T: bytemuck::Pod>(
        context: &GpuContext,
        buffer: &wgpu::Buffer,
        count: usize,
    ) -> Vec<T> {
        let device = &context.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Storage Read-back"),
            source: wgpu::ShaderSource::Wgsl(
                "@group(0) @binding(0) var<storage, read> source: array<u32>;
                @group(0) @binding(1) var<storage, read_write> copy: array<u32>;
                @compute @workgroup_size(64)
                fn copy_words(@builtin(global_invocation_id) invocation: vec3<u32>) {
                    if invocation.x < arrayLength(&copy) {
                        copy[invocation.x] = source[invocation.x];
                    }
                }"
                .into(),
            ),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Storage Read-back"),
            layout: None,
            module: &module,
            entry_point: Some("copy_words"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let size = (count * size_of::<T>()) as wgpu::BufferAddress;
        let buffer_of = |usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Storage Read-back"),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let copy = buffer_of(wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC);
        let read = buffer_of(wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Storage Read-back"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: copy.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((size as u32 / 4).div_ceil(64), 1, 1);
        }
        encoder.copy_buffer_to_buffer(&copy, 0, &read, 0, size);
        submit_and_map(context, encoder, &read).expect("read the storage buffer back");
        let words = read.get_mapped_range(..).expect("mapped");
        bytemuck::cast_slice(&words).to_vec()
    }

    /// A headless context, or `None` after saying so on a machine without a
    /// GPU adapter, where these tests skip.
    pub(crate) fn gpu_context() -> Option<GpuContext> {
        match pollster::block_on(GpuContext::headless()) {
            Ok(context) => Some(context),
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                None
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        }
    }

    #[test]
    fn a_mesh_past_capacity_is_refused_and_logged_and_changes_nothing() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let mut pool = MeshPool::new(&context.device, &memory, &arenas_of(CAPACITY));
        pool.load(&mut staging, &mesh(3, vec![0, 1, 2]), &[None])
            .expect("a triangle fits");
        let gpu_memory_bytes = memory.bytes_by_kind();
        let arena_bytes_in_use = pool.arena_bytes_in_use();

        // One vertex past the vertex Arena, then one index past the index
        // Arena while the vertices fit.
        for past_capacity in [mesh(2, vec![0, 1, 0]), mesh(1, vec![0; 26])] {
            let (handle, log) = logged(|| pool.load(&mut staging, &past_capacity, &[None]));
            assert!(handle.is_none(), "a Mesh past capacity loaded");
            assert!(log.contains("Mesh refused"), "not logged: {log:?}");
            assert_eq!(memory.bytes_by_kind(), gpu_memory_bytes);
            assert_eq!(pool.arena_bytes_in_use(), arena_bytes_in_use);
        }
        // The second refusal gave back the last vertex it took.
        pool.load(&mut staging, &mesh(1, vec![0, 0, 0]), &[None])
            .expect("the last vertex fits");
    }

    /// What `run` returns, and the text of the events it logged.
    pub(crate) fn logged<T>(run: impl FnOnce() -> T) -> (T, String) {
        #[derive(Clone, Default)]
        struct Log(Arc<Mutex<Vec<u8>>>);
        impl Write for Log {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let log = Log::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer({
                let log = log.clone();
                move || log.clone()
            })
            .with_ansi(false)
            .finish();
        let result = tracing::subscriber::with_default(subscriber, run);
        let text = String::from_utf8_lossy(&log.0.lock().unwrap()).into_owned();
        (result, text)
    }
}
