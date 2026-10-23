//! Each frame's GPU stats come back through a ring of buffers the CPU maps
//! for reading: the counts of the frame's draw and of the lights its Froxel
//! lists had no room for, and its passes' timestamps, a few frames late.
//! Only the main thread submits and polls, and it never
//! waits on a map while it renders (ADR 0016): a readback whose slot is not
//! ready is skipped and counted; `flush` waits, at the end of a run. Metal
//! reads timestamps resolved before their pass's submission completed as 0
//! or stale (`docs/research/wgpu-capability-matrix.md`), so a frame's
//! timestamps are resolved only once its submission completed.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

use crate::render::culling_pass::FrameDraw;
use crate::render::{FRAMES_IN_FLIGHT, TIMED_PASSES};
use crate::stats::{FrameRecorder, GpuFrameStats};

/// Frames after its own by which a frame's submission has completed: those
/// in flight and the one being recorded. Its timestamps resolve then.
const COMPLETION_LAG: u64 = FRAMES_IN_FLIGHT as u64 + 1;

/// A slot's resolve completes `COMPLETION_LAG` frames after it is recorded,
/// so the slot is read and reused that many frames later again.
const SLOT_COUNT: u64 = 2 * COMPLETION_LAG;

/// A timestamp at the start and one at the end of each timed pass.
const QUERIES_PER_FRAME: usize = 2 * TIMED_PASSES.len();

const TIMESTAMP_BYTES: u64 = QUERIES_PER_FRAME as u64 * wgpu::QUERY_SIZE as u64;

/// Where a slot holds the count of the lights the frame's Froxel lists had
/// no room for, after its draw's counts.
const FROXEL_OVERFLOW_OFFSET: u64 = size_of::<FrameDraw>() as u64;

/// Where a slot holds the frame's timestamps, after its counts.
const TIMESTAMP_OFFSET: u64 =
    (FROXEL_OVERFLOW_OFFSET + size_of::<u32>() as u64).next_multiple_of(8);

/// What a slot's map callback reported.
const MAP_PENDING: u8 = 0;
const MAP_DONE: u8 = 1;
const MAP_FAILED: u8 = 2;

/// The ring of readbacks of the frames' GPU stats.
pub struct GpuStatsReadback {
    /// `QUERIES_PER_FRAME` per slot, in one set: Apple GPUs allow 32
    /// timestamp sets per device.
    query_set: wgpu::QuerySet,
    /// Where a frame's timestamps resolve before their copy to its slot:
    /// queries resolve only into buffers the CPU cannot map.
    resolve_buffer: wgpu::Buffer,
    slots: Vec<Slot>,
    /// One past the last frame whose submission completed.
    frames_completed: Arc<AtomicU64>,
    /// The number of the frame being recorded, or of the next one.
    frame: u64,
    nanoseconds_per_tick: f64,
    /// The stats the last `begin_frame` or `flush` read back, by frame.
    read_back: Vec<GpuFrameStats>,
    /// Set while the frames read back had Instances the frame's draw had no
    /// room for, which was warned about when it was set.
    warned_of_overflow: bool,
    /// Set while the frames read back had lights their Froxel lists had no
    /// room for, which was warned about when it was set.
    warned_of_froxel_overflow: bool,
}

struct Slot {
    /// Its frame's draw, then its timestamps.
    buffer: wgpu::Buffer,
    state: SlotState,
    map_result: Arc<AtomicU8>,
    /// A bit per timed pass its frame did not run, by its place in
    /// `TIMED_PASSES`: each reads as taking no time.
    untimed: u32,
}

const _: () = assert!(TIMED_PASSES.len() <= u32::BITS as usize);

#[derive(Clone, Copy)]
enum SlotState {
    Free,
    /// The frame's counts are copied in; its timestamps wait for its
    /// submission to complete.
    Recorded {
        frame: u64,
    },
    /// The frame's timestamps are copied in too, and the buffer maps.
    Mapping {
        frame: u64,
    },
}

/// Where a frame's passes write their timestamps.
#[derive(Clone, Copy)]
pub struct FrameTimestamps<'a> {
    query_set: &'a wgpu::QuerySet,
    first_query: u32,
}

impl GpuStatsReadback {
    /// The ring, empty; timestamps tick every `nanoseconds_per_tick`.
    pub fn new(device: &wgpu::Device, nanoseconds_per_tick: f32) -> Self {
        let buffer = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        Self {
            // Timestamp queries are in the Capability floor (ADR 0021).
            query_set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("GPU Stats Timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: (SLOT_COUNT as usize * QUERIES_PER_FRAME) as u32,
            }),
            resolve_buffer: buffer(
                "GPU Stats Resolve Buffer",
                TIMESTAMP_BYTES,
                wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            ),
            slots: (0..SLOT_COUNT)
                .map(|_| Slot {
                    buffer: buffer(
                        "GPU Stats Readback Slot",
                        TIMESTAMP_OFFSET + TIMESTAMP_BYTES,
                        wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    ),
                    state: SlotState::Free,
                    map_result: Arc::new(AtomicU8::new(MAP_PENDING)),
                    untimed: 0,
                })
                .collect(),
            frames_completed: Arc::new(AtomicU64::new(0)),
            frame: 0,
            nanoseconds_per_tick: f64::from(nanoseconds_per_tick),
            read_back: Vec::with_capacity(SLOT_COUNT as usize),
            warned_of_overflow: false,
            warned_of_froxel_overflow: false,
        }
    }

    /// Starts recording the next frame into `encoder`: reads back the slot
    /// it takes, resolves the timestamps of the frame `COMPLETION_LAG`
    /// before it, and returns where its passes write theirs; `None` when its
    /// slot is still mapping, a skip counted in `recorder`.
    pub fn begin_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        recorder: &mut FrameRecorder,
    ) -> Option<FrameTimestamps<'_>> {
        self.read_back.clear();
        if let Some(resolved) = self.frame.checked_sub(COMPLETION_LAG) {
            let index = slot_index(resolved);
            if let SlotState::Recorded { frame } = self.slots[index].state {
                if self.frames_completed.load(Ordering::Acquire) > frame {
                    self.resolve(encoder, index, frame);
                } else {
                    // Its timestamps may not be written yet.
                    self.slots[index].state = SlotState::Free;
                    recorder.gpu_readbacks_skipped += 1;
                }
            }
        }

        let index = slot_index(self.frame);
        if !self.read(index) {
            recorder.gpu_readbacks_skipped += 1;
            return None;
        }
        self.slots[index].state = SlotState::Recorded { frame: self.frame };
        self.slots[index].untimed = 0;
        self.frame_timestamps()
    }

    /// Where the passes of the frame `begin_frame` started write their
    /// timestamps; `None` when its readback was skipped.
    pub fn frame_timestamps(&self) -> Option<FrameTimestamps<'_>> {
        let index = slot_index(self.frame);
        matches!(self.slots[index].state, SlotState::Recorded { frame } if frame == self.frame)
            .then(|| FrameTimestamps {
                query_set: &self.query_set,
                first_query: (index * QUERIES_PER_FRAME) as u32,
            })
    }

    /// Marks `pass`, of `TIMED_PASSES`, as one the frame `begin_frame`
    /// started does not run, so it reads as taking no time.
    pub fn pass_untimed(&mut self, pass: &str) {
        let slot = &mut self.slots[slot_index(self.frame)];
        if matches!(slot.state, SlotState::Recorded { frame } if frame == self.frame) {
            slot.untimed |= 1 << pass_position(pass);
        }
    }

    /// Ends the frame `begin_frame` started: copies its draw's counts,
    /// `frame_draw`, and the lights its Froxel lists had no room for,
    /// `froxel_lights_overflowed`, into its slot, and marks it completed once
    /// the GPU has run it.
    pub fn end_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_draw: &wgpu::Buffer,
        froxel_lights_overflowed: &wgpu::Buffer,
    ) {
        let slot = &self.slots[slot_index(self.frame)];
        if matches!(slot.state, SlotState::Recorded { frame } if frame == self.frame) {
            let size = size_of::<FrameDraw>() as u64;
            encoder.copy_buffer_to_buffer(frame_draw, 0, &slot.buffer, 0, size);
            encoder.copy_buffer_to_buffer(
                froxel_lights_overflowed,
                0,
                &slot.buffer,
                FROXEL_OVERFLOW_OFFSET,
                size_of::<u32>() as u64,
            );
        }
        let (frame, frames_completed) = (self.frame, self.frames_completed.clone());
        encoder.on_submitted_work_done(move || {
            frames_completed.fetch_max(frame + 1, Ordering::Release);
        });
        self.frame += 1;
    }

    /// Waits for the GPU to finish every frame recorded, then reads back
    /// every one not yet read: the end of a run.
    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<()> {
        self.read_back.clear();
        device.poll(wgpu::PollType::wait_indefinitely())?;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("GPU Stats Flush Encoder"),
        });
        // Every slot once, oldest frame first.
        let slots = (self.frame..self.frame + SLOT_COUNT).map(slot_index);
        for index in slots.clone() {
            if let SlotState::Recorded { frame } = self.slots[index].state {
                self.resolve(&mut encoder, index, frame);
            }
        }
        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::PollType::wait_indefinitely())?;
        for index in slots {
            self.read(index);
        }
        Ok(())
    }

    /// The stats the last `begin_frame` or `flush` read back, by frame.
    pub fn read_back(&self) -> &[GpuFrameStats] {
        &self.read_back
    }

    /// Bytes of the resolve buffer and the slots.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.resolve_buffer.size()
            + self
                .slots
                .iter()
                .map(|slot| slot.buffer.size())
                .sum::<u64>()
    }

    /// Records the copy of `frame`'s timestamps, from slot `index`'s
    /// queries, into the slot, which maps once the copy is done.
    fn resolve(&mut self, encoder: &mut wgpu::CommandEncoder, index: usize, frame: u64) {
        let first_query = (index * QUERIES_PER_FRAME) as u32;
        let queries = first_query..first_query + QUERIES_PER_FRAME as u32;
        encoder.resolve_query_set(&self.query_set, queries, &self.resolve_buffer, 0);
        let slot = &mut self.slots[index];
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            0,
            &slot.buffer,
            TIMESTAMP_OFFSET,
            TIMESTAMP_BYTES,
        );
        slot.map_result.store(MAP_PENDING, Ordering::Relaxed);
        let map_result = slot.map_result.clone();
        encoder.map_buffer_on_submit(&slot.buffer, wgpu::MapMode::Read, .., move |result| {
            let reported = if result.is_ok() { MAP_DONE } else { MAP_FAILED };
            map_result.store(reported, Ordering::Release);
        });
        slot.state = SlotState::Mapping { frame };
    }

    /// Reads slot `index` back if it is mapped, and frees it. Returns whether
    /// it is free; one still mapping is not. A frame whose draw had no room
    /// for some Instances, or whose Froxel lists for some lights, is warned
    /// about, once until a frame read back has room for all; a skipped
    /// readback neither warns nor clears it.
    fn read(&mut self, index: usize) -> bool {
        let slot = &mut self.slots[index];
        let SlotState::Mapping { frame } = slot.state else {
            return true;
        };
        match slot.map_result.load(Ordering::Acquire) {
            MAP_PENDING => return false,
            MAP_DONE => {
                match slot.buffer.get_mapped_range(..) {
                    Ok(view) => {
                        let draw: FrameDraw =
                            bytemuck::pod_read_unaligned(&view[..size_of::<FrameDraw>()]);
                        let overflowed = draw.instances_overflowed > 0;
                        if overflowed && !self.warned_of_overflow {
                            tracing::warn!(
                                frame,
                                instances_overflowed = draw.instances_overflowed,
                                "The frame's draw is full: Instances whose Meshlets do not fit are not drawn"
                            );
                        }
                        self.warned_of_overflow = overflowed;
                        let froxel_lights_overflowed: u32 = bytemuck::pod_read_unaligned(
                            &view[FROXEL_OVERFLOW_OFFSET as usize..][..size_of::<u32>()],
                        );
                        let froxel_overflowed = froxel_lights_overflowed > 0;
                        if froxel_overflowed && !self.warned_of_froxel_overflow {
                            tracing::warn!(
                                frame,
                                froxel_lights_overflowed,
                                "The Froxel lists are full: lights past a list's capacity do not light its Froxel"
                            );
                        }
                        self.warned_of_froxel_overflow = froxel_overflowed;
                        let ticks: [u64; QUERIES_PER_FRAME] = bytemuck::pod_read_unaligned(
                            &view[TIMESTAMP_OFFSET as usize..][..TIMESTAMP_BYTES as usize],
                        );
                        let time = |begin: u64, end: u64| {
                            let ticks = end.saturating_sub(begin) as f64;
                            Duration::from_nanos((ticks * self.nanoseconds_per_tick) as u64)
                        };
                        let untimed = slot.untimed;
                        let timed = |pass: usize| untimed & (1 << pass) == 0;
                        let first_timed = (0..TIMED_PASSES.len()).find(|&pass| timed(pass));
                        self.read_back.push(GpuFrameStats {
                            frame,
                            pass_times: std::array::from_fn(|pass| {
                                let (begin, end) = (ticks[2 * pass], ticks[2 * pass + 1]);
                                let taken = if timed(pass) {
                                    time(begin, end)
                                } else {
                                    Duration::ZERO
                                };
                                (TIMED_PASSES[pass], taken)
                            }),
                            frame_time: first_timed.map_or(Duration::ZERO, |pass| {
                                time(ticks[2 * pass], ticks[QUERIES_PER_FRAME - 1])
                            }),
                            instances_drawn: draw.instances_drawn,
                            triangles_drawn: draw.triangles_drawn(),
                            instances_overflowed: draw.instances_overflowed,
                            froxel_lights_overflowed,
                        });
                    }
                    Err(error) => tracing::error!(frame, %error, "GPU stats readback not mapped"),
                }
                slot.buffer.unmap();
            }
            _ => tracing::error!(frame, "GPU stats readback failed to map"),
        }
        slot.state = SlotState::Free;
        true
    }
}

impl<'a> FrameTimestamps<'a> {
    /// The timestamp writes of `pass`, a compute pass of `TIMED_PASSES`.
    pub fn compute_pass(self, pass: &str) -> wgpu::ComputePassTimestampWrites<'a> {
        let first = self.first_query_of(pass);
        wgpu::ComputePassTimestampWrites {
            query_set: self.query_set,
            beginning_of_pass_write_index: Some(first),
            end_of_pass_write_index: Some(first + 1),
        }
    }

    /// The timestamp writes of `pass`, a render pass of `TIMED_PASSES`.
    pub fn render_pass(self, pass: &str) -> wgpu::RenderPassTimestampWrites<'a> {
        let first = self.first_query_of(pass);
        wgpu::RenderPassTimestampWrites {
            query_set: self.query_set,
            beginning_of_pass_write_index: Some(first),
            end_of_pass_write_index: Some(first + 1),
        }
    }

    /// The timestamp writes that bracket `pass`, of `TIMED_PASSES`, with
    /// two compute passes around it: the end of the first, the start of the
    /// second.
    pub fn bracket(self, pass: &str) -> [wgpu::ComputePassTimestampWrites<'a>; 2] {
        let first = self.first_query_of(pass);
        [
            wgpu::ComputePassTimestampWrites {
                query_set: self.query_set,
                beginning_of_pass_write_index: None,
                end_of_pass_write_index: Some(first),
            },
            wgpu::ComputePassTimestampWrites {
                query_set: self.query_set,
                beginning_of_pass_write_index: Some(first + 1),
                end_of_pass_write_index: None,
            },
        ]
    }

    fn first_query_of(self, pass: &str) -> u32 {
        self.first_query + 2 * pass_position(pass) as u32
    }
}

/// Where `pass` is in `TIMED_PASSES`.
fn pass_position(pass: &str) -> usize {
    let Some(position) = TIMED_PASSES.iter().position(|&timed| timed == pass) else {
        panic!("{pass} is not a timed pass");
    };
    position
}

fn slot_index(frame: u64) -> usize {
    (frame % SLOT_COUNT) as usize
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::gpu::GpuContext;

    /// Frames that never complete, as their encoders are never submitted:
    /// from `COMPLETION_LAG` frames on, each skips the timestamps of the
    /// frame that far back, counted, and none is read back. Then, in a ring
    /// whose frames complete, a slot whose map was never requested, as the
    /// encoder requesting it was dropped, is skipped when the ring comes
    /// round to it, and the next slot is read: no frame waits.
    #[test]
    fn readbacks_not_ready_are_skipped_and_counted() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let new_ring = || GpuStatsReadback::new(&context.device, 1.0);
        let counts = |label, size| {
            context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        };
        let frame_draw = counts("Readback Test Frame Draw", size_of::<FrameDraw>() as u64);
        let froxel_lights_overflowed =
            counts("Readback Test Froxel Overflow", size_of::<u32>() as u64);
        // Records a frame, and submits it and waits for it if `completes`:
        // whether it is timed, the readbacks it skipped, the frames it read.
        let frame = |readback: &mut GpuStatsReadback, completes: bool| {
            let descriptor = wgpu::CommandEncoderDescriptor::default();
            let mut encoder = context.device.create_command_encoder(&descriptor);
            let mut recorder = FrameRecorder::new();
            let timed = readback.begin_frame(&mut encoder, &mut recorder).is_some();
            readback.end_frame(&mut encoder, &frame_draw, &froxel_lights_overflowed);
            if completes {
                context.queue.submit([encoder.finish()]);
                let wait = context.device.poll(wgpu::PollType::wait_indefinitely());
                wait.expect("the frame completes");
            }
            let read: Vec<u64> = readback.read_back().iter().map(|gpu| gpu.frame).collect();
            (timed, recorder.gpu_readbacks_skipped, read)
        };

        let mut never_completes = new_ring();
        for number in 0..2 * SLOT_COUNT {
            let skipped = u32::from(number >= COMPLETION_LAG);
            let recorded = frame(&mut never_completes, false);
            assert_eq!(recorded, (true, skipped, vec![]), "frame {number}");
        }

        let mut ring = new_ring();
        for number in 0..SLOT_COUNT {
            // Frame 3 resolves frame 0's timestamps, never to be mapped.
            let completes = number != COMPLETION_LAG;
            assert_eq!(
                frame(&mut ring, completes),
                (true, 0, vec![]),
                "frame {number}"
            );
        }
        assert_eq!(frame(&mut ring, true), (false, 1, vec![]), "frame 6");
        assert_eq!(frame(&mut ring, true), (true, 0, vec![1]), "frame 7");
    }
}
