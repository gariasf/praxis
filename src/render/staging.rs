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
