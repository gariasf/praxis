use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::render::FRAMES_IN_FLIGHT;

/// Slots the per-frame uploads cycle through: the frames in flight, plus the
/// one being written. A fixed count keeps `gpu_memory_bytes` the same on
/// every run.
const SLOT_COUNT: usize = FRAMES_IN_FLIGHT + 1;

/// Staging memory for the per-frame uploads, reused every frame, where
/// `Queue::write_buffer` creates a fresh staging buffer on every call. Its
/// slots grow on demand, never past the most one frame uploads. Each frame
/// fills the next slot in turn, copies it into its targets, and the slot is
/// mapped again once the GPU has run the copies. The fixed turn keeps the
/// frame's heap allocations the same on every run: wgpu grows an encoder's
/// tracking by each buffer's tracker index, so which slot a frame takes
/// changes its allocation count.
pub struct FrameStaging {
    slots: Vec<StagingSlot>,
    slot_size: u64,
    /// The most one frame uploads, which no slot grows past.
    max_slot_size: u64,
    next_slot: usize,
}

struct StagingSlot {
    buffer: wgpu::Buffer,
    /// Set by the map callback once `buffer` is mapped for writing again.
    is_mapped: Arc<AtomicBool>,
}

impl FrameStaging {
    /// Staging for frames that each upload at most `max_slot_size` bytes.
    pub fn new(max_slot_size: u64) -> Self {
        Self {
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

        let slot = &self.slots[self.next_slot];
        self.next_slot = (self.next_slot + 1) % SLOT_COUNT;
        if !slot.is_mapped.load(Ordering::Acquire) {
            // Every slot is in flight: wait for the GPU to finish this one.
            if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
                tracing::error!(%error, "waiting for a frame staging slot failed");
            }
            if !slot.is_mapped.load(Ordering::Acquire) {
                // Its map failed, so the buffer is unmapped: map it again.
                tracing::error!("frame staging slot not mapped; skipping the frame's uploads");
                slot.buffer
                    .map_async(wgpu::MapMode::Write, .., mark_mapped(&slot.is_mapped));
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
        encoder.map_buffer_on_submit(
            &slot.buffer,
            wgpu::MapMode::Write,
            ..,
            mark_mapped(&slot.is_mapped),
        );
        needed
    }

    /// Bytes of the staging slots.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.slots.len() as u64 * self.slot_size
    }

    /// Replaces the slots with `SLOT_COUNT` of `slot_size` bytes, mapped. A
    /// replaced slot still in flight lives until the GPU is done with it.
    fn resize(&mut self, device: &wgpu::Device, slot_size: u64) {
        self.slots = (0..SLOT_COUNT)
            .map(|_| StagingSlot {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Frame Staging Slot"),
                    size: slot_size,
                    usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                }),
                is_mapped: Arc::new(AtomicBool::new(true)),
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

/// The `StagingBelt`'s allocation unit at load: a larger write gets a chunk
/// of its own.
const LOAD_CHUNK_BYTES: u64 = 1 << 20;

/// Staging for load-time uploads (ADR 0017): each write goes through a
/// `StagingBelt` into a copy on one encoder, with the BLAS builds that read
/// them, which `submit` sends ahead of the next frame's. The belt goes with
/// it, so its staging buffers live only until the GPU has run the copies and
/// never count in `gpu_memory_bytes`.
pub struct LoadStaging {
    device: wgpu::Device,
    pending: Option<(wgpu::util::StagingBelt, wgpu::CommandEncoder)>,
}

impl LoadStaging {
    pub fn new(device: wgpu::Device) -> Self {
        Self {
            device,
            pending: None,
        }
    }

    /// Records a copy of `bytes` to `target` at `offset`; the length and the
    /// offset are multiples of `wgpu::COPY_BUFFER_ALIGNMENT`.
    pub fn write(&mut self, target: &wgpu::Buffer, offset: wgpu::BufferAddress, bytes: &[u8]) {
        let Some(size) = wgpu::BufferSize::new(bytes.len() as u64) else {
            return;
        };
        let (belt, encoder) = self.pending();
        belt.write_buffer(encoder, target, offset, size)
            .copy_from_slice(bytes);
    }

    /// Records a copy of `bytes`, mip `level` of `texture` as rows of blocks
    /// without padding, re-pitching each row to
    /// `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT` as it is written into the belt.
    pub fn write_texture(&mut self, texture: &wgpu::Texture, level: u32, bytes: &[u8]) {
        let format = texture.format();
        let (block_width, block_height) = format.block_dimensions();
        let block_bytes = format.block_copy_size(None).expect("a colour format");
        // A mip smaller than a block still copies a whole one.
        let size = texture
            .size()
            .mip_level_size(level, texture.dimension())
            .physical_size(format);
        let row_bytes = (size.width / block_width * block_bytes) as usize;
        let rows = size.height / block_height;
        let pitch = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize);
        assert_eq!(size.depth_or_array_layers, 1, "one layer per Texture");
        assert_eq!(
            bytes.len(),
            row_bytes * rows as usize,
            "mip {level} of a {format:?} Texture"
        );
        let Some(staged_bytes) = wgpu::BufferSize::new((pitch * rows as usize) as u64) else {
            return;
        };
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
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
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

    /// Submits the copies recorded since the last call, if any.
    pub fn submit(&mut self, queue: &wgpu::Queue) {
        if let Some((mut belt, encoder)) = self.pending.take() {
            belt.finish();
            queue.submit(std::iter::once(encoder.finish()));
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

    /// Every mip of an 8 × 8 BC5 Texture written through the belt after a
    /// 4-byte buffer write reads back as written: each mip starts at a
    /// multiple of its 16-byte blocks, mip 0's two block rows of 32 bytes
    /// sit 256 bytes apart, and mips smaller than a block copy one block.
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
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Staging Test Texture"),
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 4,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bc5RgUnorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
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
            staging.write_texture(&texture, level as u32, bytes);
        }
        staging.submit(&context.queue);

        for (level, (bytes, (across, down))) in levels.iter().zip(blocks).enumerate() {
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
                    texture: &texture,
                    mip_level: level as u32,
                    origin: wgpu::Origin3d::ZERO,
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
            let rows: Vec<u8> = mapped
                .chunks(pitch as usize)
                .flat_map(|row| &row[..across as usize * 16])
                .copied()
                .collect();
            assert_eq!(rows, *bytes, "mip {level}");
        }
    }
}
