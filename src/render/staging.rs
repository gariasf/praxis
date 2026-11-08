use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::cooked::CookedTexture;
use crate::render::FRAMES_IN_FLIGHT;
use crate::stats::{Counted, GpuMemory, GpuMemoryKind};

/// Slots the per-frame uploads cycle through: the frames in flight, plus the
/// one being written. A fixed count keeps `gpu_memory_bytes` the same on
/// every run.
const SLOT_COUNT: usize = FRAMES_IN_FLIGHT + 1;

/// Staging memory for the per-frame uploads, reused every frame, where
/// `Queue::write_buffer` creates a fresh staging buffer on every call. Its
/// slots grow on demand, never past the most one frame uploads. Each frame
/// fills the next slot in turn, copies it into its targets, and requests the
/// map of the slot it fills next, last filled `FRAMES_IN_FLIGHT` writes
/// before. The fixed turn keeps the frame's heap allocations the same on
/// every run: wgpu grows an encoder's tracking by each buffer's tracker
/// index, so which slot a frame takes changes its allocation count. So does
/// the late map, for the reason the GPU stats readback maps late: the bench
/// has waited for the frame that copied from the slot, so the frame's own
/// submission maps it, whatever the GPU's timing
/// [ref:wgpu-30-handle-mapping].
pub struct FrameStaging {
    memory: GpuMemory,
    slots: Vec<StagingSlot>,
    slot_size: u64,
    /// The most one frame uploads, which no slot grows past.
    pub(crate) max_slot_size: u64,
    next_slot: usize,
}

struct StagingSlot {
    buffer: Counted<wgpu::Buffer>,
    /// Set by the map callback once `buffer` is mapped for writing again.
    is_mapped: Arc<AtomicBool>,
    /// Whether its map was requested since it was last written.
    map_requested: bool,
}

impl FrameStaging {
    /// Staging for frames that each upload at most `max_slot_size` bytes,
    /// counted in `memory`.
    pub fn new(memory: &GpuMemory, max_slot_size: u64) -> Self {
        Self {
            memory: memory.clone(),
            slots: Vec::new(),
            slot_size: 0,
            max_slot_size,
            next_slot: 0,
        }
    }

    /// Writes `uploads`, each a target buffer, an offset in it and the bytes
    /// for there, into the next slot and records their copies into `encoder`,
    /// which must be submitted. Returns the bytes uploaded: 0 when the slot
    /// failed to map again, after an error is logged and the map retried.
    pub fn write<'a>(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uploads: impl Iterator<Item = (&'a wgpu::Buffer, wgpu::BufferAddress, &'a [u8])> + Clone,
    ) -> u64 {
        let needed: u64 = uploads
            .clone()
            .map(|(_, _, bytes)| bytes.len() as u64)
            .sum();
        if needed == 0 {
            return 0;
        }
        assert!(
            needed <= self.max_slot_size,
            "a frame uploads {needed} bytes, more than the {} its staging is bounded by",
            self.max_slot_size
        );
        if needed > self.slot_size {
            self.resize(device, needed.next_power_of_two().min(self.max_slot_size));
        }

        let index = self.next_slot;
        self.next_slot = (self.next_slot + 1) % SLOT_COUNT;
        self.slots[self.next_slot].request_map();
        let slot = &mut self.slots[index];
        if !slot.is_mapped.load(Ordering::Acquire) {
            // Every slot is in flight: wait for the GPU to finish this one.
            slot.request_map();
            if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
                tracing::error!(%error, "waiting for a frame staging slot failed");
            }
            if !slot.is_mapped.load(Ordering::Acquire) {
                // Its map failed, so the buffer is unmapped: map it again.
                tracing::error!("frame staging slot not mapped; skipping the frame's uploads");
                slot.map_requested = false;
                slot.request_map();
                return 0;
            }
        }
        {
            let Ok(mut mapped) = slot.buffer.slice(..).get_mapped_range_mut() else {
                tracing::error!("frame staging slot not mapped; skipping the frame's uploads");
                return 0;
            };
            // An empty transition sizes the encoder's buffer tracking to
            // every buffer at once. The copies alone grow it buffer by buffer
            // in tracker-index order, which depends on what earlier States
            // created and dropped, so the frame's heap allocations would too.
            encoder.transition_resources(std::iter::empty(), std::iter::empty());
            let mut offset = 0;
            for (target, target_offset, bytes) in uploads {
                let size = bytes.len() as u64;
                debug_assert!(size.is_multiple_of(wgpu::COPY_BUFFER_ALIGNMENT));
                mapped
                    .slice(offset as usize..(offset + size) as usize)
                    .copy_from_slice(bytes);
                encoder.copy_buffer_to_buffer(&slot.buffer, offset, target, target_offset, size);
                offset += size;
            }
        }
        slot.buffer.unmap();
        slot.is_mapped.store(false, Ordering::Release);
        slot.map_requested = false;
        needed
    }

    /// Replaces the slots with `SLOT_COUNT` of `slot_size` bytes, mapped. A
    /// replaced slot still in flight lives until the GPU is done with it.
    fn resize(&mut self, device: &wgpu::Device, slot_size: u64) {
        self.slots = (0..SLOT_COUNT)
            .map(|_| StagingSlot {
                buffer: self.memory.create_buffer(
                    device,
                    GpuMemoryKind::Staging,
                    &wgpu::BufferDescriptor {
                        label: Some("Frame Staging Slot"),
                        size: slot_size,
                        usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                        mapped_at_creation: true,
                    },
                ),
                is_mapped: Arc::new(AtomicBool::new(true)),
                map_requested: false,
            })
            .collect();
        tracing::info!(
            old = self.slot_size,
            new = slot_size,
            "frame staging slots resized"
        );
        self.slot_size = slot_size;
    }
}

impl StagingSlot {
    /// Requests the map of its buffer for writing, unless it is mapped or
    /// its map was requested since it was last written.
    fn request_map(&mut self) {
        if !self.map_requested && !self.is_mapped.load(Ordering::Acquire) {
            self.map_requested = true;
            self.buffer
                .map_async(wgpu::MapMode::Write, .., mark_mapped(&self.is_mapped));
        }
    }
}

/// The `StagingBelt`'s allocation unit at load: a larger write gets a chunk
/// of its own.
const LOAD_CHUNK_BYTES: u64 = 1 << 20;

/// Staging for load-time uploads (ADR 0017): each write goes through a
/// `StagingBelt` into a copy on one encoder, with the BLAS builds that read
/// them and the copies that move Textures between Texture classes, which
/// `submit` sends ahead of the next frame's. The belt goes with it, so its
/// staging buffers live only until the GPU has run the copies and never
/// count in `gpu_memory_bytes`.
pub struct LoadStaging {
    device: wgpu::Device,
    pending: Option<(wgpu::util::StagingBelt, wgpu::CommandEncoder)>,
    /// Bytes written into the belt since the last submit, row padding
    /// included.
    written_bytes: u64,
    /// Bytes the pending Texture moves copy.
    moved_bytes: u64,
}

impl LoadStaging {
    pub fn new(device: wgpu::Device) -> Self {
        Self {
            device,
            pending: None,
            written_bytes: 0,
            moved_bytes: 0,
        }
    }

    /// Records a copy of `bytes` to `target` at `offset`; the length and the
    /// offset are multiples of `wgpu::COPY_BUFFER_ALIGNMENT`.
    pub fn write(&mut self, target: &wgpu::Buffer, offset: wgpu::BufferAddress, bytes: &[u8]) {
        let Some(size) = wgpu::BufferSize::new(bytes.len() as u64) else {
            return;
        };
        self.written_bytes += size.get();
        let (belt, encoder) = self.pending();
        belt.write_buffer(encoder, target, offset, size)
            .copy_from_slice(bytes);
    }

    /// Bytes written into the belt since the last submit.
    pub fn written_bytes(&self) -> u64 {
        self.written_bytes
    }

    /// Records a copy of `bytes`, mip `level` of array layer `slice` of
    /// `texture` as rows of blocks without padding, re-pitching each row to
    /// `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT` as it is written into the belt.
    pub fn write_texture(&mut self, texture: &wgpu::Texture, level: u32, slice: u32, bytes: &[u8]) {
        let format = texture.format();
        let MipCopy {
            size,
            row_bytes,
            rows,
            pitch,
        } = MipCopy::of(format, texture.width(), texture.height(), level);
        assert_eq!(
            bytes.len(),
            row_bytes * rows as usize,
            "mip {level} of a {format:?} Texture"
        );
        let Some(staged_bytes) = wgpu::BufferSize::new((pitch * rows as usize) as u64) else {
            return;
        };
        self.written_bytes += staged_bytes.get();
        let (belt, encoder) = self.pending();
        // A copy's offset is a multiple of the block size, which divides 256.
        let alignment = wgpu::BufferSize::new(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.into())
            .expect("256 is not zero");
        let staged = belt.allocate(staged_bytes, alignment);
        {
            // The belt maps every chunk before handing out a slice of it.
            let mut view = staged
                .get_mapped_range_mut()
                .expect("a StagingBelt slice is mapped");
            for (index, row) in bytes.chunks(row_bytes).enumerate() {
                view.slice(index * pitch..index * pitch + row_bytes)
                    .copy_from_slice(row);
            }
        }
        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: staged.buffer(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: staged.offset(),
                    bytes_per_row: Some(pitch as u32),
                    rows_per_image: Some(rows),
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: slice,
                },
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
    }

    /// Records a copy of the mips of array layer `source_slice` of `source`
    /// from its mip `first_source_level` on into layer `target_slice` of
    /// `target`, from its mip `first_target_level` on: a Texture's move
    /// between Texture classes.
    pub fn copy_texture(
        &mut self,
        source: &wgpu::Texture,
        source_slice: u32,
        first_source_level: u32,
        target: &wgpu::Texture,
        target_slice: u32,
        first_target_level: u32,
    ) {
        let format = source.format();
        let (block_width, block_height) = format.block_dimensions();
        let block_bytes = format.block_copy_size(None).expect("a colour format");
        let mut moved_bytes = 0;
        let (_, encoder) = self.pending();
        for level in first_source_level..source.mip_level_count() {
            // A mip smaller than a block still copies a whole one.
            let size = wgpu::Extent3d {
                depth_or_array_layers: 1,
                ..source
                    .size()
                    .mip_level_size(level, source.dimension())
                    .physical_size(format)
            };
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: source,
                    mip_level: level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: source_slice,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: target,
                    mip_level: first_target_level + level - first_source_level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: target_slice,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                size,
            );
            let blocks = (size.width / block_width) * (size.height / block_height);
            moved_bytes += u64::from(blocks * block_bytes);
        }
        self.moved_bytes += moved_bytes;
    }

    /// Records `entry`'s BLAS build after the copies recorded so far, so it
    /// reads what they write.
    pub fn build_blas(&mut self, entry: &wgpu::BlasBuildEntry<'_>) {
        let (_, encoder) = self.pending();
        encoder.build_acceleration_structures(std::iter::once(entry), std::iter::empty());
    }

    /// The belt and encoder of the copies not yet submitted.
    fn pending(&mut self) -> &mut (wgpu::util::StagingBelt, wgpu::CommandEncoder) {
        let device = &self.device;
        self.pending.get_or_insert_with(|| {
            (
                wgpu::util::StagingBelt::new(device.clone(), LOAD_CHUNK_BYTES),
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Load Encoder"),
                }),
            )
        })
    }

    /// Submits the copies recorded since the last call, if any, and returns
    /// the bytes their Texture moves copy.
    pub fn submit(&mut self, queue: &wgpu::Queue) -> u64 {
        if let Some((mut belt, encoder)) = self.pending.take() {
            belt.finish();
            queue.submit(std::iter::once(encoder.finish()));
        }
        self.written_bytes = 0;
        std::mem::take(&mut self.moved_bytes)
    }
}

/// The bytes `write_texture` writes into the belt for every mip `texture`
/// holds, row padding included.
pub fn texture_written_bytes(texture: &CookedTexture) -> u64 {
    let end = texture.first_level + texture.levels.len() as u32;
    mips_written_bytes(
        texture.format,
        texture.width,
        texture.height,
        texture.first_level..end,
    )
}

/// The bytes `write_texture` writes into the belt for mips `levels` of a
/// `width` × `height` Texture in `format`, row padding included.
pub fn mips_written_bytes(
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    levels: std::ops::Range<u32>,
) -> u64 {
    levels
        .map(|level| {
            let copy = MipCopy::of(format, width, height, level);
            (copy.pitch * copy.rows as usize) as u64
        })
        .sum()
}

/// One mip's copy from the belt, as rows of blocks.
struct MipCopy {
    /// Its size in texels, whole blocks: a mip smaller than a block still
    /// copies a whole one.
    size: wgpu::Extent3d,
    row_bytes: usize,
    rows: u32,
    /// The bytes between rows in the belt.
    pitch: usize,
}

impl MipCopy {
    fn of(format: wgpu::TextureFormat, width: u32, height: u32, level: u32) -> Self {
        let (block_width, block_height) = format.block_dimensions();
        let block_bytes = format.block_copy_size(None).expect("a colour format");
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        }
        .mip_level_size(level, wgpu::TextureDimension::D2)
        .physical_size(format);
        let row_bytes = (size.width / block_width * block_bytes) as usize;
        let pitch = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
        Self {
            size,
            row_bytes,
            rows: size.height / block_height,
            pitch,
        }
    }
}

/// A map callback that sets `is_mapped` on success. A failure leaves it unset
/// for `write` to report; dropping a slot whose map is pending, as `resize`
/// does, fails it too.
fn mark_mapped(
    is_mapped: &Arc<AtomicBool>,
) -> impl FnOnce(Result<(), wgpu::BufferAsyncError>) + use<> {
    let is_mapped = is_mapped.clone();
    move |result| {
        if result.is_ok() {
            is_mapped.store(true, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::capture::submit_and_map;
    use crate::gpu::GpuContext;

    /// Every mip of slice 1 of an 8 × 8 BC5 texture array written through
    /// the belt after a 4-byte buffer write reads back as written: each mip
    /// starts at a multiple of its 16-byte blocks, mip 0's two block rows of
    /// 32 bytes sit 256 bytes apart, and mips smaller than a block copy one
    /// block. Copied from its mip 1 on into a 4 × 4 texture, each mip lands
    /// one level finer.
    #[test]
    fn every_mip_reads_back_as_written() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let texture_descriptor = wgpu::TextureDescriptor {
            label: Some("Staging Test Texture"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 2,
            },
            mip_level_count: 4,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc5RgUnorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = context.device.create_texture(&texture_descriptor);
        // Blocks across and down per mip, of 16 bytes each, every byte
        // distinct.
        let blocks: [(u32, u32); 4] = [(2, 2), (1, 1), (1, 1), (1, 1)];
        let levels: Vec<Vec<u8>> = blocks
            .iter()
            .enumerate()
            .map(|(level, (across, down))| {
                (0..across * down * 16)
                    .map(|byte| (level as u32 * 64 + byte) as u8)
                    .collect()
            })
            .collect();
        let mut staging = LoadStaging::new(context.device.clone());
        let word = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Test Word"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        staging.write(&word, 0, &[1, 2, 3, 4]);
        for (level, bytes) in levels.iter().enumerate() {
            staging.write_texture(&texture, level as u32, 1, bytes);
        }
        staging.submit(&context.queue);

        let read_back = |texture: &wgpu::Texture, level: u32, slice: u32, (across, down)| {
            let pitch = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
            let read_back = context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Staging Test Read-back"),
                size: u64::from(pitch * down),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let mut encoder = context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: level,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: slice,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &read_back,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(pitch),
                        rows_per_image: None,
                    },
                },
                wgpu::Extent3d {
                    width: 4 * across,
                    height: 4 * down,
                    depth_or_array_layers: 1,
                },
            );
            submit_and_map(&context, encoder, &read_back).expect("read-back mapped");
            let mapped = read_back.get_mapped_range(..).expect("read-back range");
            mapped
                .chunks(pitch as usize)
                .flat_map(|row| &row[..across as usize * 16])
                .copied()
                .collect::<Vec<u8>>()
        };
        for (level, (bytes, blocks)) in levels.iter().zip(blocks).enumerate() {
            let read = read_back(&texture, level as u32, 1, blocks);
            assert_eq!(read, *bytes, "mip {level}");
        }

        let coarser = context.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 3,
            label: Some("Staging Test Coarser Texture"),
            ..texture_descriptor
        });
        staging.copy_texture(&texture, 1, 1, &coarser, 0, 0);
        staging.submit(&context.queue);
        for (level, bytes) in levels[1..].iter().enumerate() {
            let read = read_back(&coarser, level as u32, 0, (1, 1));
            assert_eq!(read, *bytes, "mip {level} from mip {}", level + 1);
        }
    }
}
