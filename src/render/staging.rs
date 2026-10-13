use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Slots the per-frame uploads cycle through: the window keeps up to two
/// frames in flight (`desired_maximum_frame_latency`), plus the one being
/// written. A fixed count keeps `gpu_memory_bytes` the same on every run.
const SLOT_COUNT: usize = 3;

/// Staging memory for the per-frame uploads, created once and reused every
/// frame, where `Queue::write_buffer` creates a fresh staging buffer on every
/// call. Each frame fills the next slot in turn, copies it into its targets,
/// and the slot is mapped again once the GPU has run the copies. The fixed
/// turn keeps the frame's heap allocations the same on every run: wgpu grows
/// an encoder's tracking by each buffer's tracker index, so which slot a frame
/// takes changes its allocation count.
#[derive(Default)]
pub struct FrameStaging {
    slots: Vec<StagingSlot>,
    slot_size: u64,
    next_slot: usize,
}

struct StagingSlot {
    buffer: wgpu::Buffer,
    /// Set by the map callback once `buffer` is mapped for writing again.
    is_mapped: Arc<AtomicBool>,
}

impl FrameStaging {
    /// Writes `uploads`, each a target buffer and the bytes for its start,
    /// into the next slot and records their copies into `encoder`, which must
    /// be submitted. Returns the bytes uploaded: 0 when the slot never maps
    /// again, after an error is logged.
    pub fn write(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        uploads: &[(&wgpu::Buffer, &[u8])],
    ) -> u64 {
        let needed: u64 = uploads.iter().map(|(_, bytes)| bytes.len() as u64).sum();
        if needed > self.slot_size {
            self.resize(device, needed.next_power_of_two());
        }

        let slot = &self.slots[self.next_slot];
        self.next_slot = (self.next_slot + 1) % SLOT_COUNT;
        if !slot.is_mapped.load(Ordering::Acquire) {
            // Every slot is in flight: wait for the GPU to finish this one.
            if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
                tracing::error!(%error, "waiting for a frame staging slot failed");
            }
        }
        {
            let mut mapped = match slot.buffer.slice(..).get_mapped_range_mut() {
                Ok(mapped) => mapped,
                Err(error) => {
                    tracing::error!(%error, "frame staging slot not mapped; skipping the frame's uploads");
                    return 0;
                }
            };
            // An empty transition sizes the encoder's buffer tracking to
            // every buffer at once. The copies alone grow it buffer by buffer
            // in tracker-index order, which depends on what earlier States
            // created and dropped, so the frame's heap allocations would too.
            encoder.transition_resources(std::iter::empty(), std::iter::empty());
            let mut offset = 0;
            for (target, bytes) in uploads {
                let size = bytes.len() as u64;
                mapped
                    .slice(offset as usize..(offset + size) as usize)
                    .copy_from_slice(bytes);
                encoder.copy_buffer_to_buffer(&slot.buffer, offset, target, 0, size);
                offset += size;
            }
        }
        slot.buffer.unmap();
        slot.is_mapped.store(false, Ordering::Release);
        let is_mapped = slot.is_mapped.clone();
        encoder.map_buffer_on_submit(&slot.buffer, wgpu::MapMode::Write, .., move |result| {
            match result {
                Ok(()) => is_mapped.store(true, Ordering::Release),
                Err(error) => tracing::error!(%error, "frame staging slot failed to map"),
            }
        });
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
