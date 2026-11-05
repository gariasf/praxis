use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::render::AccelerationStructureBytes;

/// A kind of GPU memory, as ADR 0017 splits the GPU memory Budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuMemoryKind {
    GeometryArenas,
    /// Every Material Texture's mips larger than `RESIDENT_TAIL_LEVEL_BYTES`.
    TexturePool,
    /// Every Material Texture's mips of `RESIDENT_TAIL_LEVEL_BYTES` and
    /// smaller, its Resident tail (ADR 0017).
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

/// The bytes of the largest mip in a Texture's Resident tail (ADR 0017).
pub const RESIDENT_TAIL_LEVEL_BYTES: u64 = 64 << 10;

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

    /// A Material Texture, its Resident tail's mips counted apart from the
    /// others.
    pub fn create_material_texture(
        &self,
        device: &wgpu::Device,
        descriptor: &wgpu::TextureDescriptor<'_>,
    ) -> Counted<wgpu::Texture> {
        let texture = device.create_texture(descriptor);
        let tail_bytes: u64 = level_bytes(&texture)
            .filter(|&bytes| bytes <= RESIDENT_TAIL_LEVEL_BYTES)
            .sum();
        let pool_bytes = level_bytes(&texture).sum::<u64>() - tail_bytes;
        let mut charge = self.charge(GpuMemoryKind::TexturePool, pool_bytes);
        charge.set(GpuMemoryKind::ResidentTails, tail_bytes);
        Counted {
            charge,
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
            bytes: [0; GpuMemoryKind::COUNT],
        };
        charge.set(kind, bytes);
        charge
    }

    /// Each kind's bytes, in `GpuMemoryKind::ALL`'s order.
    pub fn bytes_by_kind(&self) -> [u64; GpuMemoryKind::COUNT] {
        std::array::from_fn(|kind| self.totals[kind].load(Ordering::Relaxed))
    }
}

/// Bytes of each kind counted in a ledger's totals until dropped.
pub struct Charge {
    totals: Totals,
    bytes: [u64; GpuMemoryKind::COUNT],
}

impl Charge {
    pub fn set(&mut self, kind: GpuMemoryKind, bytes: u64) {
        let total = &self.totals[kind as usize];
        total.fetch_add(bytes, Ordering::Relaxed);
        total.fetch_sub(self.bytes[kind as usize], Ordering::Relaxed);
        self.bytes[kind as usize] = bytes;
    }
}

impl Drop for Charge {
    fn drop(&mut self) {
        for kind in GpuMemoryKind::ALL {
            self.set(kind, 0);
        }
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
    level_bytes(texture).sum::<u64>() * u64::from(texture.sample_count())
}

/// Bytes of each level of `texture`'s mip chain at one sample.
fn level_bytes(texture: &wgpu::Texture) -> impl Iterator<Item = u64> + '_ {
    (0..texture.mip_level_count()).map(|level| {
        let size = texture.size().mip_level_size(level, texture.dimension());
        texture.format().theoretical_memory_footprint(size)
    })
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

    #[test]
    fn a_material_textures_mips_of_64_kib_and_smaller_count_as_its_resident_tail() {
        let Some(context) = gpu_context() else {
            return;
        };
        let memory = gpu_memory(&context);
        let texture = memory.create_material_texture(
            &context.device,
            &wgpu::TextureDescriptor {
                label: Some("Ledger Test Texture"),
                size: wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 10,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );
        // RGBA8 mips from 512 × 512: 1 MiB and 256 KiB, then 128 × 128 at
        // 64 KiB down to 1 × 1.
        let mut expected = [0; GpuMemoryKind::COUNT];
        expected[GpuMemoryKind::TexturePool as usize] = 1_048_576 + 262_144;
        expected[GpuMemoryKind::ResidentTails as usize] =
            65_536 + 16_384 + 4_096 + 1_024 + 256 + 64 + 16 + 4;
        assert_eq!(memory.bytes_by_kind(), expected);
        drop(texture);
        assert_eq!(memory.bytes_by_kind(), [0; GpuMemoryKind::COUNT]);
    }
}
