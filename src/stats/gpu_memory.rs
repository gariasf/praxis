use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::render::AccelerationStructureBytes;

/// A kind of GPU memory, as ADR 0017 splits the GPU memory Budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuMemoryKind {
    GeometryArenas,
    /// Every Material Texture's mips: none are streamed yet, so the Resident
    /// tails hold nothing apart from them.
    TexturePool,
    ResidentTails,
    RenderTargets,
    /// BLASes and TLASes, estimated (`AccelerationStructureBytes`), and the
    /// BLAS refit's scratch.
    AccelerationStructures,
    Staging,
    /// The Instance and light buffers, the Mesh and Material tables, and the
    /// passes' own buffers and textures that no target size sets.
    SceneAndPassBuffers,
}

impl GpuMemoryKind {
    pub const ALL: [Self; 7] = [
        Self::GeometryArenas,
        Self::TexturePool,
        Self::ResidentTails,
        Self::RenderTargets,
        Self::AccelerationStructures,
        Self::Staging,
        Self::SceneAndPassBuffers,
    ];

    pub const COUNT: usize = Self::ALL.len();

    /// Its name, as Frame stats and Bench records spell it.
    pub fn name(self) -> &'static str {
        match self {
            Self::GeometryArenas => "geometry_arenas",
            Self::TexturePool => "texture_pool",
            Self::ResidentTails => "resident_tails",
            Self::RenderTargets => "render_targets",
            Self::AccelerationStructures => "acceleration_structures",
            Self::Staging => "staging",
            Self::SceneAndPassBuffers => "scene_and_pass_buffers",
        }
    }
}

// A kind's total sits at its discriminant, and `ALL` lists the kinds in that order.
const _: () = {
    let mut index = 0;
    while index < GpuMemoryKind::COUNT {
        assert!(GpuMemoryKind::ALL[index] as usize == index);
        index += 1;
    }
};

type Totals = Arc<[AtomicU64; GpuMemoryKind::COUNT]>;

/// The ledger of one Renderer's GPU memory (ADR 0013): a running total per
/// kind, which each buffer, texture and acceleration structure created
/// through it adds to and takes back from when dropped.
#[derive(Clone)]
pub struct GpuMemory {
    totals: Totals,
    transient_saves_memory: bool,
    acceleration_structure_bytes: AccelerationStructureBytes,
}

impl GpuMemory {
    pub fn new(
        transient_saves_memory: bool,
        acceleration_structure_bytes: AccelerationStructureBytes,
    ) -> Self {
        Self {
            totals: Totals::default(),
            transient_saves_memory,
            acceleration_structure_bytes,
        }
    }

    pub fn for_adapter(adapter: &wgpu::AdapterInfo) -> Self {
        Self::new(
            adapter.transient_saves_memory == Some(true),
            AccelerationStructureBytes::estimate(adapter),
        )
    }

    pub fn acceleration_structure_bytes(&self) -> AccelerationStructureBytes {
        self.acceleration_structure_bytes
    }

    pub fn create_buffer(
        &self,
        device: &wgpu::Device,
        kind: GpuMemoryKind,
        descriptor: &wgpu::BufferDescriptor<'_>,
    ) -> Counted<wgpu::Buffer> {
        let buffer = device.create_buffer(descriptor);
        Counted {
            charge: self.charge(kind, buffer.size()),
            resource: buffer,
        }
    }

    pub fn create_texture(
        &self,
        device: &wgpu::Device,
        kind: GpuMemoryKind,
        descriptor: &wgpu::TextureDescriptor<'_>,
    ) -> Counted<wgpu::Texture> {
        let texture = device.create_texture(descriptor);
        Counted {
            charge: self.charge(kind, texture_bytes(&texture, self.transient_saves_memory)),
            resource: texture,
        }
    }

    /// A TLAS, counted by its estimated bytes.
    pub fn create_tlas(
        &self,
        device: &wgpu::Device,
        descriptor: &wgpu::CreateTlasDescriptor<'_>,
    ) -> Counted<wgpu::Tlas> {
        let bytes = self
            .acceleration_structure_bytes
            .tlas(descriptor.max_instances as usize);
        Counted {
            charge: self.charge(GpuMemoryKind::AccelerationStructures, bytes),
            resource: device.create_tlas(descriptor),
        }
    }

    /// `bytes` of `kind` until the Charge is dropped or set again: for
    /// memory wgpu gives no size of, such as acceleration structures.
    pub fn charge(&self, kind: GpuMemoryKind, bytes: u64) -> Charge {
        let mut charge = Charge {
            totals: Arc::clone(&self.totals),
            kind,
            bytes: 0,
        };
        charge.set(bytes);
        charge
    }

    /// Each kind's bytes, in `GpuMemoryKind::ALL`'s order.
    pub fn bytes_by_kind(&self) -> [u64; GpuMemoryKind::COUNT] {
        std::array::from_fn(|kind| self.totals[kind].load(Ordering::Relaxed))
    }
}

/// Bytes of one kind counted in a ledger's total until dropped.
pub struct Charge {
    totals: Totals,
    kind: GpuMemoryKind,
    bytes: u64,
}

impl Charge {
    pub fn set(&mut self, bytes: u64) {
        let total = &self.totals[self.kind as usize];
        total.fetch_add(bytes, Ordering::Relaxed);
        total.fetch_sub(self.bytes, Ordering::Relaxed);
        self.bytes = bytes;
    }
}

impl Drop for Charge {
    fn drop(&mut self) {
        self.set(0);
    }
}

/// A GPU resource created through a ledger, counted there while it lives.
pub struct Counted<T> {
    resource: T,
    charge: Charge,
}

impl<T> Counted<T> {
    /// `convert`'s result of the resource, counted in its place: a view of
    /// a texture, which keeps the texture alive.
    pub fn map<U>(self, convert: impl FnOnce(T) -> U) -> Counted<U> {
        Counted {
            resource: convert(self.resource),
            charge: self.charge,
        }
    }
}

impl<T> Deref for Counted<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.resource
    }
}

impl<T> DerefMut for Counted<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.resource
    }
}

/// Bytes of `texture`'s mip chain at every sample, as ADR 0013 counts GPU
/// memory. A transient attachment counts 0 where the adapter keeps it in tile
/// memory (memoryless on Apple GPUs).
fn texture_bytes(texture: &wgpu::Texture, transient_saves_memory: bool) -> u64 {
    if transient_saves_memory
        && texture
            .usage()
            .contains(wgpu::TextureUsages::TRANSIENT_ATTACHMENT)
    {
        return 0;
    }
    let mip_chain_bytes: u64 = (0..texture.mip_level_count())
        .map(|level| {
            let size = texture.size().mip_level_size(level, texture.dimension());
            texture.format().theoretical_memory_footprint(size)
        })
        .sum();
    mip_chain_bytes * u64::from(texture.sample_count())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::assets::pools::tests::gpu_context;
    use crate::gpu::GpuContext;

    /// The ledger for `context`'s adapter.
    pub(crate) fn gpu_memory(context: &GpuContext) -> GpuMemory {
        GpuMemory::for_adapter(&context.adapter.get_info())
    }

    #[test]
    fn a_buffer_created_through_the_ledger_counts_in_its_kind_until_dropped() {
        let Some(context) = gpu_context() else {
            return;
        };
        let memory = gpu_memory(&context);
        let buffer = memory.create_buffer(
            &context.device,
            GpuMemoryKind::Staging,
            &wgpu::BufferDescriptor {
                label: Some("Ledger Test Buffer"),
                size: 1024,
                usage: wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let staging = GpuMemoryKind::Staging as usize;
        let mut expected = [0; GpuMemoryKind::COUNT];
        expected[staging] = 1024;
        assert_eq!(memory.bytes_by_kind(), expected);
        drop(buffer);
        assert_eq!(memory.bytes_by_kind(), [0; GpuMemoryKind::COUNT]);
    }
}
