use crate::{
    assets::{MaterialHandle, Mesh, MeshData, MeshHandle, MeshletData},
    cooked::{CookedMesh, CookedVertex},
    render::{AccelerationStructureBytes, LoadStaging, NO_MATERIAL},
};

/// Bytes of each Arena: one provisional capacity for the app, tests, bench
/// and CI until ADR 0013's split of the 8 GB Budget replaces it.
pub const ARENA_CAPACITY_BYTES: u64 = 16 << 20;

/// Meshes the Mesh table holds, provisional as `ARENA_CAPACITY_BYTES` is.
pub const MESH_CAPACITY: u32 = 1 << 16;

/// What a Mesh is loaded for: the main pass draws it from its Meshlets, rays
/// trace it through its BLAS, or both.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    fn drawn(self) -> bool {
        self != MeshUse::Traced
    }

    /// Whether rays trace it, so it needs a BLAS and the index stream the
    /// BLAS is built from.
    fn traced(self) -> bool {
        self != MeshUse::Drawn
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
/// and a BLAS per Mesh, which rays trace (ADR 0020).
pub struct MeshPool {
    device: wgpu::Device,
    meshes: Vec<Mesh>,
    /// Each Mesh's BLAS, at its Handle's index, built at load from its
    /// ranges of the vertex and index Arenas; none for a Mesh no ray traces.
    blases: Vec<Option<wgpu::Blas>>,
    /// The Meshes whose BLAS is built but not yet compacted, with its
    /// triangles.
    awaiting_compaction: Vec<(MeshHandle, u64)>,
    /// The BLASes of the Meshes loaded as `MeshUse::Refitted`.
    refitted_blases: Vec<MeshBlas>,
    /// Triangles of the BLASes compacted, and of the others.
    compacted_blas_triangles: u64,
    uncompacted_blas_triangles: u64,
    vertex_arena: Arena,
    index_arena: Arena,
    meshlet_arena: Arena,
    meshlet_vertex_arena: Arena,
    meshlet_triangle_arena: Arena,
    mesh_table: wgpu::Buffer,
}

impl MeshPool {
    /// A Mesh pool whose Arenas hold `arena_bytes` each.
    pub fn new(device: &wgpu::Device, arena_bytes: u64) -> Self {
        let arena = |label, usage, element_bytes| {
            Arena::new(device, label, usage, element_bytes, arena_bytes)
        };
        let u32_bytes = size_of::<u32>() as u64;
        Self {
            device: device.clone(),
            meshes: Vec::new(),
            blases: Vec::new(),
            awaiting_compaction: Vec::new(),
            refitted_blases: Vec::new(),
            compacted_blas_triangles: 0,
            uncompacted_blas_triangles: 0,
            // The vertex shader pulls from it (ADR 0019), BLAS builds read it.
            vertex_arena: arena(
                "Vertex Arena",
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::BLAS_INPUT,
                size_of::<CookedVertex>() as u64,
            ),
            // The cooked index stream, which no pass draws from: BLAS builds
            // read it.
            index_arena: arena("Index Arena", wgpu::BufferUsages::BLAS_INPUT, u32_bytes),
            meshlet_arena: arena(
                "Meshlet Arena",
                wgpu::BufferUsages::STORAGE,
                size_of::<MeshletData>() as u64,
            ),
            meshlet_vertex_arena: arena(
                "Meshlet Vertex Arena",
                wgpu::BufferUsages::STORAGE,
                u32_bytes,
            ),
            meshlet_triangle_arena: arena(
                "Meshlet Triangle Arena",
                wgpu::BufferUsages::STORAGE,
                u32_bytes,
            ),
            mesh_table: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Mesh Table"),
                size: u64::from(MESH_CAPACITY) * size_of::<MeshData>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    /// Loads `mesh` to be drawn and traced, as `load_for` loads it.
    pub fn load(
        &mut self,
        staging: &mut LoadStaging,
        mesh: &CookedMesh,
        materials: &[Option<MaterialHandle>],
    ) -> Option<MeshHandle> {
        self.load_for(staging, mesh, materials, MeshUse::DrawnAndTraced)
    }

    /// Writes `mesh` through `staging` into one range of each Arena, and its
    /// entry into the Mesh table, each Primitive drawn with its Material of
    /// `materials`, in Primitive order, where no Renderable overrides it; and,
    /// where `mesh_use` has rays trace it, its index stream and its BLAS's
    /// build from its vertex and index ranges, which `compact_blases`
    /// compacts once `staging` is submitted, unless it is refitted. A Mesh no pass draws keeps no
    /// Meshlets. Returns its Handle, or None
    /// after logging when the Mesh table is full or an Arena's longest free
    /// range is too short; a refused Mesh leaves the pool as it was.
    pub fn load_for(
        &mut self,
        staging: &mut LoadStaging,
        mesh: &CookedMesh,
        materials: &[Option<MaterialHandle>],
        mesh_use: MeshUse,
    ) -> Option<MeshHandle> {
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
        let handle = MeshHandle(self.meshes.len() as u32);
        if handle.0 == MESH_CAPACITY {
            tracing::error!(
                capacity = MESH_CAPACITY,
                "Mesh refused: the Mesh table is full"
            );
            return None;
        }
        let Some(
            [
                vertices,
                indices,
                meshlet_range,
                meshlet_vertices,
                meshlet_triangles,
            ],
        ) = allocate_all(
            [
                &mut self.vertex_arena,
                &mut self.index_arena,
                &mut self.meshlet_arena,
                &mut self.meshlet_vertex_arena,
                &mut self.meshlet_triangle_arena,
            ],
            counts,
        )
        else {
            tracing::error!(
                vertices = mesh.vertices.len(),
                indices = index_stream.len(),
                meshlets = meshlet_count,
                meshlet_vertices = meshlet_vertex_count,
                triangles = triangle_count,
                longest_free_vertex_range = self.vertex_arena.longest_free_range(),
                longest_free_index_range = self.index_arena.longest_free_range(),
                longest_free_meshlet_range = self.meshlet_arena.longest_free_range(),
                longest_free_meshlet_vertex_range = self.meshlet_vertex_arena.longest_free_range(),
                longest_free_meshlet_triangle_range =
                    self.meshlet_triangle_arena.longest_free_range(),
                "Mesh refused: an Arena's longest free range is too short"
            );
            return None;
        };

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
            u64::from(handle.0) * size_of::<MeshData>() as u64,
            bytemuck::bytes_of(&mesh_data),
        );
        tracing::debug!(
            vertices = mesh.vertices.len(),
            indices = index_stream.len(),
            meshlets = meshlet_count,
            "Mesh loaded"
        );
        self.meshes.push(Mesh {
            needs_material_handle: mesh_use.drawn() && materials.contains(&None),
            vertices: vertices.offset..vertices.offset + mesh.vertices.len() as u32,
        });
        if !mesh_use.traced() {
            self.blases.push(None);
            return Some(handle);
        }
        let geometry = wgpu::BlasTriangleGeometrySizeDescriptor {
            vertex_format: wgpu::VertexFormat::Float32x3,
            vertex_count: mesh.vertices.len() as u32,
            index_format: Some(wgpu::IndexFormat::Uint32),
            index_count: Some(mesh.indices.len() as u32),
            flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
        };
        let refitted = mesh_use == MeshUse::Refitted;
        let blas = self.device.create_blas(
            &wgpu::CreateBlasDescriptor {
                label: Some("Mesh BLAS"),
                flags: if refitted {
                    REFITTED_BLAS_FLAGS
                } else {
                    wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE
                        | wgpu::AccelerationStructureFlags::ALLOW_COMPACTION
                },
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

        let blas_triangles = mesh.indices.len() as u64 / 3;
        self.blases.push(Some(mesh_blas.blas.clone()));
        if refitted {
            self.refitted_blases.push(mesh_blas);
        } else {
            self.awaiting_compaction.push((handle, blas_triangles));
        }
        self.uncompacted_blas_triangles += blas_triangles;
        Some(handle)
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
    /// logged.
    pub fn compact_blases(&mut self, queue: &wgpu::Queue) {
        if self.awaiting_compaction.is_empty() {
            return;
        }
        for &(handle, _) in &self.awaiting_compaction {
            if let Some(blas) = &self.blases[handle.0 as usize] {
                blas.prepare_compaction_async(|_| {});
            }
        }
        if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            tracing::error!(%error, "waiting for BLAS builds failed");
        }
        for (handle, triangles) in self.awaiting_compaction.drain(..) {
            let Some(blas) = &mut self.blases[handle.0 as usize] else {
                continue;
            };
            if !blas.ready_for_compaction() {
                tracing::warn!(
                    mesh = handle.0,
                    "BLAS not compacted: not ready after its build"
                );
                continue;
            }
            *blas = queue.compact_blas(blas);
            self.uncompacted_blas_triangles -= triangles;
            self.compacted_blas_triangles += triangles;
        }
        // The copies wait in the queue's pending writes until a submit.
        queue.submit(std::iter::empty());
        if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
            tracing::error!(%error, "waiting for BLAS compaction failed");
        }
    }

    /// The BLAS of `handle`'s Mesh, or None when the pool has no such Mesh
    /// or no ray traces it.
    pub fn blas(&self, handle: MeshHandle) -> Option<&wgpu::Blas> {
        self.blases.get(handle.0 as usize)?.as_ref()
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

    /// Bytes of the Arenas and the Mesh table, whatever they hold.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.arenas()
            .iter()
            .map(|arena| arena.buffer.size())
            .sum::<u64>()
            + self.mesh_table.size()
    }

    /// Bytes of the BLASes, as `acceleration_structure_bytes` estimates
    /// them.
    pub fn estimated_blas_bytes(
        &self,
        acceleration_structure_bytes: &AccelerationStructureBytes,
    ) -> u64 {
        acceleration_structure_bytes.blases(
            self.compacted_blas_triangles,
            self.uncompacted_blas_triangles,
        )
    }

    /// Bytes of the Arenas that hold Meshes.
    pub fn arena_bytes_in_use(&self) -> u64 {
        self.arenas().iter().map(|arena| arena.bytes_in_use).sum()
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle.0 as usize)
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
    buffer: wgpu::Buffer,
    allocator: offset_allocator::Allocator,
    element_bytes: u64,
    bytes_in_use: u64,
}

impl Arena {
    fn new(
        device: &wgpu::Device,
        label: &str,
        usage: wgpu::BufferUsages,
        element_bytes: u64,
        capacity_bytes: u64,
    ) -> Self {
        // The DX12 and Linux non-NVIDIA Vulkan buffer limit (ADR 0017).
        assert!(
            capacity_bytes < 1 << 31,
            "an Arena of {capacity_bytes} bytes is past 2^31 - 1"
        );
        Self {
            buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: capacity_bytes,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
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

    /// Room for four 28-byte vertices, or 28 indices.
    const CAPACITY: u64 = 4 * 28;

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
                meshlets,
            }],
            indices,
            bounds: [[0.0; 3]; 2],
        }
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
        let mut pool = MeshPool::new(&context.device, ARENA_CAPACITY_BYTES);
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
            meshlets: CookedMeshlet::split(&[[0, 1, 2], [2, 1, 3]][first_index / 3..][..1], [1])
                .expect("one Meshlet"),
        };
        let two_primitives = CookedMesh {
            vertices: (0..4).map(|index| vertex(index as f32)).collect(),
            indices: indices.clone(),
            primitives: vec![primitive(0), primitive(3)],
            bounds: [[-1.0, -2.0, -3.0], [4.0, 5.0, 6.0]],
        };
        let material = MaterialHandle(5);
        let handle = pool
            .load(&mut staging, &two_primitives, &[Some(material), None])
            .expect("the Mesh fits");
        staging.submit(&context.queue);

        let mesh_table: Vec<MeshData> = read_storage(&context, &pool.mesh_table, 2);
        let mesh_data = mesh_table[handle.0 as usize];
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

    /// A Mesh loaded after one whose triangle is degenerate, so that its
    /// ranges of the vertex and index Arenas start past their first elements,
    /// is traced where its triangle is once its BLAS is built and compacted;
    /// its BLAS counts by the estimate's bytes per triangle, then per
    /// compacted triangle.
    #[test]
    fn a_meshs_blas_holds_its_triangle_from_its_arena_ranges_and_is_compacted() {
        let Some(context) = gpu_context() else {
            return;
        };
        let bytes = AccelerationStructureBytes {
            per_blas_triangle: 100.0,
            per_compacted_blas_triangle: 50.0,
            per_tlas_instance: 0.0,
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let mut pool = MeshPool::new(&context.device, ARENA_CAPACITY_BYTES);
        pool.load(&mut staging, &mesh(3, vec![0, 0, 0]), &[None])
            .expect("a triangle fits");
        let handle = pool
            .load(&mut staging, &triangle_about_x(), &[None])
            .expect("a triangle fits");
        assert_eq!(pool.estimated_blas_bytes(&bytes), 2 * 100);
        staging.submit(&context.queue);
        pool.compact_blases(&context.queue);
        assert_eq!(pool.estimated_blas_bytes(&bytes), 2 * 50);

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
            let mut pool = MeshPool::new(&context.device, 4 << 10);
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
    /// z 0.
    pub(crate) fn triangle_about_x() -> CookedMesh {
        let mut triangle = mesh(3, vec![0, 1, 2]);
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
        let mut pool = MeshPool::new(&context.device, CAPACITY);
        pool.load(&mut staging, &mesh(3, vec![0, 1, 2]), &[None])
            .expect("a triangle fits");
        let gpu_memory_bytes = pool.gpu_memory_bytes();
        let arena_bytes_in_use = pool.arena_bytes_in_use();

        // One vertex past the vertex Arena, then one index past the index
        // Arena while the vertices fit.
        for past_capacity in [mesh(2, vec![0, 1, 0]), mesh(1, vec![0; 26])] {
            let (handle, log) = logged(|| pool.load(&mut staging, &past_capacity, &[None]));
            assert!(handle.is_none(), "a Mesh past capacity loaded");
            assert!(log.contains("Mesh refused"), "not logged: {log:?}");
            assert_eq!(pool.gpu_memory_bytes(), gpu_memory_bytes);
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
