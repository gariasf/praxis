use crate::{
    assets::{MaterialHandle, Mesh, MeshHandle, Primitive},
    cooked::{CookedMesh, CookedVertex},
    render::LoadStaging,
};

/// Bytes of each Arena: one provisional capacity for the app, tests, bench
/// and CI until M4's Budget split replaces it (decision on #17, item 4).
pub const ARENA_CAPACITY_BYTES: u64 = 16 << 20;

pub struct MeshPool {
    meshes: Vec<Mesh>,
    vertex_arena: Arena,
    index_arena: Arena,
}

impl MeshPool {
    /// A Mesh pool with a vertex Arena and an index Arena of `arena_bytes`
    /// each.
    pub fn new(device: &wgpu::Device, arena_bytes: u64) -> Self {
        Self {
            meshes: Vec::new(),
            // The vertex shader pulls from it (ADR 0019).
            vertex_arena: Arena::new(
                device,
                "Vertex Arena",
                wgpu::BufferUsages::STORAGE,
                std::mem::size_of::<CookedVertex>() as u64,
                arena_bytes,
            ),
            index_arena: Arena::new(
                device,
                "Index Arena",
                wgpu::BufferUsages::INDEX,
                std::mem::size_of::<u32>() as u64,
                arena_bytes,
            ),
        }
    }

    /// Writes `mesh`'s vertices and indices through `staging` into one range
    /// of each Arena, drawn with `material` where no Renderable overrides
    /// it. Returns its Handle, or None after logging when an Arena's longest
    /// free range is too short; a refused Mesh leaves the Arenas as they were.
    pub fn load(
        &mut self,
        staging: &mut LoadStaging,
        mesh: &CookedMesh,
        material: Option<MaterialHandle>,
    ) -> Option<MeshHandle> {
        let (vertex_count, index_count) = (mesh.vertices.len(), mesh.indices.len());
        let Some(vertex_range) = self.vertex_arena.allocate(vertex_count) else {
            return self.refuse(vertex_count, index_count);
        };
        let Some(index_range) = self.index_arena.allocate(index_count) else {
            self.vertex_arena.free(vertex_range);
            return self.refuse(vertex_count, index_count);
        };
        self.vertex_arena.write(
            staging,
            vertex_range.offset,
            bytemuck::cast_slice(&mesh.vertices),
        );
        self.index_arena.write(
            staging,
            index_range.offset,
            bytemuck::cast_slice(&mesh.indices),
        );
        // A Primitive's indices count from the Mesh's first vertex.
        let primitives = mesh
            .primitives
            .iter()
            .map(|primitive| Primitive {
                vertex_offset: vertex_range.offset,
                index_offset: index_range.offset + primitive.first_index,
                index_count: primitive.index_count,
            })
            .collect();
        tracing::debug!(
            vertices = vertex_count,
            indices = index_count,
            "Mesh loaded"
        );

        let handle = MeshHandle(self.meshes.len() as u32);
        self.meshes.push(Mesh {
            primitives,
            material,
        });
        Some(handle)
    }

    fn refuse(&self, vertex_count: usize, index_count: usize) -> Option<MeshHandle> {
        tracing::error!(
            vertices = vertex_count,
            indices = index_count,
            longest_free_vertex_range = self.vertex_arena.longest_free_range(),
            longest_free_index_range = self.index_arena.longest_free_range(),
            "Mesh refused: an Arena's longest free range is too short"
        );
        None
    }

    pub fn vertex_arena(&self) -> &wgpu::Buffer {
        &self.vertex_arena.buffer
    }

    pub fn index_buffer(&self) -> &wgpu::Buffer {
        &self.index_arena.buffer
    }

    /// Bytes of the vertex and index Arenas, whatever they hold.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.vertex_arena.buffer.size() + self.index_arena.buffer.size()
    }

    /// Bytes of the vertex and index Arenas that hold Meshes.
    pub fn arena_bytes_in_use(&self) -> u64 {
        self.vertex_arena.bytes_in_use + self.index_arena.bytes_in_use
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle.0 as usize)
    }
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
    use crate::cooked::CookedPrimitive;
    use crate::gpu::GpuContext;

    /// Room for four 28-byte vertices, or 28 indices.
    const CAPACITY: u64 = 4 * 28;

    /// A one-Primitive Mesh of `vertex_count` vertices and `indices`.
    pub(crate) fn mesh(vertex_count: usize, indices: Vec<u32>) -> CookedMesh {
        CookedMesh {
            vertices: vec![CookedVertex::default(); vertex_count],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: indices.len() as u32,
                material: None,
                meshlets: Vec::new(),
            }],
            indices,
            bounds: [[0.0; 3]; 2],
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
        pool.load(&mut staging, &mesh(3, vec![0, 1, 2]), None)
            .expect("a triangle fits");
        let buffers = (pool.vertex_arena().clone(), pool.index_buffer().clone());
        let gpu_memory_bytes = pool.gpu_memory_bytes();
        let arena_bytes_in_use = pool.arena_bytes_in_use();

        // One vertex past the vertex Arena, then one index past the index
        // Arena while the vertices fit.
        for past_capacity in [mesh(2, vec![0, 1, 0]), mesh(1, vec![0; 26])] {
            let (handle, log) = logged(|| pool.load(&mut staging, &past_capacity, None));
            assert!(handle.is_none(), "a Mesh past capacity loaded");
            assert!(log.contains("Mesh refused"), "not logged: {log:?}");
            assert_eq!(
                (pool.vertex_arena(), pool.index_buffer()),
                (&buffers.0, &buffers.1)
            );
            assert_eq!(pool.gpu_memory_bytes(), gpu_memory_bytes);
            assert_eq!(pool.arena_bytes_in_use(), arena_bytes_in_use);
        }
        // The second refusal gave back the last vertex it took.
        pool.load(&mut staging, &mesh(1, vec![0, 0, 0]), None)
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
