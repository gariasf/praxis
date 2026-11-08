//! Each frame's GPU stats come back through a ring of buffers the CPU maps
//! for reading: the counts of the frame's draw, of its lights and of the
//! lights its shadow mask had no bit for, and its passes' timestamps, a few
//! frames late.
//! Only the main thread submits and polls, and it never
//! waits on a map while it renders (ADR 0016): a readback whose slot is not
//! ready is skipped and counted; `flush` waits, at the end of a run. Metal
//! reads timestamps resolved before their pass's submission completed as 0
//! or stale (`docs/research/wgpu-capability-matrix.md`), so a frame's
//! timestamps are resolved only once its submission completed. A slot maps
//! once its copy completed and no sooner than a fixed number of frames after
//! it, so in the bench the frame that maps it, and allocates for it in wgpu,
//! does not depend on the GPU's timing.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

use crate::render::bend_pass::BEND_PASS;
use crate::render::blas_refit::BLAS_REFIT_PASS;
use crate::render::culling_pass::{CullingPhase, FrameDraw};
use crate::render::froxel_pass::{FROXEL_PASS, LightCounts};
use crate::render::main_pass::LIT_PASS;
use crate::render::{FRAMES_IN_FLIGHT, TIMED_PASSES};
use crate::stats::{FrameRecorder, GpuFrameStats};

/// Frames after its own by which a frame's submission has completed: those
/// in flight and the one being recorded. Its timestamps resolve then.
const COMPLETION_LAG: u64 = FRAMES_IN_FLIGHT as u64 + 1;

/// Frames after the frame copying a slot's timestamps when its map is
/// requested at the earliest, once that copy completed. The bench has
/// waited for that frame by then, so each frame requests one map whatever
/// the GPU's timing, and the frame's first submission maps it
/// [ref:wgpu-30-submit-maintains]. A map requested with the copy maps in
/// whichever submission or poll first sees the copy complete
/// [ref:wgpu-30-map-after-submission], and wgpu allocates for it there
/// [ref:wgpu-30-handle-mapping]: inside a frame or between two, as the GPU's
/// timing falls.
const MAP_LAG: u64 = FRAMES_IN_FLIGHT as u64;

/// A slot's timestamps are copied `COMPLETION_LAG` frames after its frame.
/// A window polls only through its submissions, so the copy may be seen
/// complete as late as the submissions `MAP_LAG` frames after it; its map is
/// then requested in the next frame and done by that frame's submissions,
/// and the slot is read, and reused, in the frame after.
const SLOT_COUNT: u64 = COMPLETION_LAG + MAP_LAG + 2;

/// A timestamp at the start and one at the end of each timed pass.
const QUERIES_PER_FRAME: usize = 2 * TIMED_PASSES.len();

const TIMESTAMP_BYTES: u64 = QUERIES_PER_FRAME as u64 * wgpu::QUERY_SIZE as u64;

/// Where a slot holds what the frame's lights counted (`LightCounts`), after
/// its draw's counts.
const LIGHT_COUNTS_OFFSET: u64 = size_of::<FrameDraw>() as u64;

/// Where a slot holds the count of the lights the frame's shadow mask had no
/// bit for, after its lights' counts.
const SHADOW_MASK_OVERFLOW_OFFSET: u64 = LIGHT_COUNTS_OFFSET + size_of::<LightCounts>() as u64;

/// Where a slot holds the frame's timestamps, after its counts.
const TIMESTAMP_OFFSET: u64 =
    (SHADOW_MASK_OVERFLOW_OFFSET + size_of::<u32>() as u64).next_multiple_of(8);

/// How far a slot's copy and map have come, as wgpu's callbacks report it.
const COPY_PENDING: u8 = 0;
const COPY_DONE: u8 = 1;
const MAP_PENDING: u8 = 2;
const MAP_DONE: u8 = 3;
const MAP_FAILED: u8 = 4;

/// The ring of readbacks of the frames' GPU stats.
pub struct GpuStatsReadback {
    /// `QUERIES_PER_FRAME` per slot, in one set: Apple GPUs allow 32
    /// counter sample buffers, and on Metal each set is one
    /// [ref:metal-feature-set-tables-sample-buffers].
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
    /// Set while the frames read back had lights their shadow mask had no
    /// bit for, which was warned about when it was set.
    warned_of_shadow_mask_overflow: bool,
    /// The last frame read back and the tick its last pass ended at, from
    /// which the next frame's build of the dynamic TLAS counts.
    last_pass_end: Option<(u64, u64)>,
}

struct Slot {
    /// Its frame's draw, then its timestamps.
    buffer: wgpu::Buffer,
    state: SlotState,
    progress: Arc<AtomicU8>,
    /// The Timed passes its frame ran.
    pass_log: PassLog,
}

/// Which of `TIMED_PASSES` a frame ran, each marked as its queries were
/// turned into its timestamp writes: a pass not marked reads as taking no
/// time, its queries holding an earlier frame's timestamps.
#[derive(Clone, Copy, Debug, Default)]
pub struct PassLog {
    passes: [Option<LoggedPass>; TIMED_PASSES.len()],
    /// The submission the passes marked now are recorded into.
    submission: u8,
    /// Where in `TIMED_PASSES` the last pass marked in that submission is,
    /// except a bracket.
    last_in_submission: Option<usize>,
}

/// A Timed pass a frame ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoggedPass {
    pub kind: PassKind,
    /// Which of the frame's submissions it was recorded into, numbered from
    /// 0 in the order they are submitted; a bracket, a command buffer of its
    /// own, is submitted just ahead of that submission.
    pub submission: u8,
}

/// How a Timed pass's queries were written: the timestamp writes its queries
/// were turned into.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PassKind {
    /// A compute pass, timed from its start to its end.
    Compute,
    /// A render pass, timed from its start to its end.
    Render,
    /// Work two compute passes bracket, in a command buffer of their own:
    /// the end of the first, the start of the second.
    Bracket,
    /// Work a compute pass ends at its start, from the end of the pass
    /// before it.
    Marker,
}

#[derive(Clone, Copy)]
enum SlotState {
    Free,
    /// The frame's counts are copied in; its timestamps wait for its
    /// submission to complete.
    Recorded {
        frame: u64,
    },
    /// The frame's timestamps are being copied in too, by the submission of
    /// frame `copied_in`; the buffer maps once the copy completed.
    Copying {
        frame: u64,
        copied_in: u64,
    },
    /// The frame's timestamps are copied in too, and the buffer maps.
    Mapping {
        frame: u64,
    },
}

/// Where a frame's passes write their timestamps, and the log of those that
/// do.
pub struct FrameTimestamps<'a> {
    query_set: &'a wgpu::QuerySet,
    first_query: u32,
    log: &'a mut PassLog,
}

/// The queries of one of `TIMED_PASSES`, which mark it run in its frame's
/// log as they are turned into its timestamp writes; dropped unturned, they
/// leave it unmarked.
pub struct PassQueries<'a, 'log> {
    query_set: &'a wgpu::QuerySet,
    first_query: u32,
    position: usize,
    log: &'log mut PassLog,
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
                    progress: Arc::new(AtomicU8::new(COPY_PENDING)),
                    pass_log: PassLog::default(),
                })
                .collect(),
            frames_completed: Arc::new(AtomicU64::new(0)),
            frame: 0,
            nanoseconds_per_tick: f64::from(nanoseconds_per_tick),
            read_back: Vec::with_capacity(SLOT_COUNT as usize),
            warned_of_overflow: false,
            warned_of_froxel_overflow: false,
            warned_of_shadow_mask_overflow: false,
            last_pass_end: None,
        }
    }

    /// Starts recording the next frame into `encoder`: requests the maps of
    /// the slots whose copies completed, at least `MAP_LAG` frames ago,
    /// reads back the slot it takes, resolves the timestamps of the frame
    /// `COMPLETION_LAG` before it, and returns where its passes write
    /// theirs, which logs those that do; `None` when its slot is still
    /// copying or mapping, a skip counted in `recorder`.
    pub fn begin_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        recorder: &mut FrameRecorder,
    ) -> Option<FrameTimestamps<'_>> {
        self.read_back.clear();
        if let Some(latest_copy) = self.frame.checked_sub(MAP_LAG) {
            self.request_maps(latest_copy);
        }
        if let Some(resolved) = self.frame.checked_sub(COMPLETION_LAG) {
            let index = slot_index(resolved);
            if let SlotState::Recorded { frame } = self.slots[index].state {
                if self.frames_completed.load(Ordering::Acquire) > frame {
                    self.resolve(encoder, index, frame);
                } else {
                    // Its timestamps may not be written yet.
                    self.slots[index].state = SlotState::Free;
                    recorder.counters.gpu_readbacks_skipped += 1;
                }
            }
        }

        let index = slot_index(self.frame);
        if !self.read(index) {
            recorder.counters.gpu_readbacks_skipped += 1;
            return None;
        }
        let slot = &mut self.slots[index];
        slot.state = SlotState::Recorded { frame: self.frame };
        slot.pass_log = PassLog::default();
        Some(FrameTimestamps {
            query_set: &self.query_set,
            first_query: (index * QUERIES_PER_FRAME) as u32,
            log: &mut slot.pass_log,
        })
    }

    /// Ends the frame `begin_frame` started: copies its draw's counts,
    /// `frame_draw`, what its lights counted, `light_counts`, and the lights
    /// its shadow mask had no bit for, `shadow_mask_lights_overflowed`, into
    /// its slot, and marks it completed once the GPU has run it.
    pub fn end_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_draw: &wgpu::Buffer,
        light_counts: &wgpu::Buffer,
        shadow_mask_lights_overflowed: &wgpu::Buffer,
    ) {
        let slot = &self.slots[slot_index(self.frame)];
        if matches!(slot.state, SlotState::Recorded { frame } if frame == self.frame) {
            let size = size_of::<FrameDraw>() as u64;
            encoder.copy_buffer_to_buffer(frame_draw, 0, &slot.buffer, 0, size);
            for (counts, offset, size) in [
                (light_counts, LIGHT_COUNTS_OFFSET, size_of::<LightCounts>()),
                (
                    shadow_mask_lights_overflowed,
                    SHADOW_MASK_OVERFLOW_OFFSET,
                    size_of::<u32>(),
                ),
            ] {
                encoder.copy_buffer_to_buffer(counts, 0, &slot.buffer, offset, size as u64);
            }
        }
        let (frame, frames_completed) = (self.frame, self.frames_completed.clone());
        // At most the one slot `begin_frame` resolved.
        let copying = self.copying_in(frame).next();
        encoder.on_submitted_work_done(move || {
            frames_completed.fetch_max(frame + 1, Ordering::Release);
            if let Some(progress) = copying {
                progress.store(COPY_DONE, Ordering::Release);
            }
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
        let copying: Vec<_> = self.copying_in(self.frame).collect();
        encoder.on_submitted_work_done(move || {
            for progress in copying {
                progress.store(COPY_DONE, Ordering::Release);
            }
        });
        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::PollType::wait_indefinitely())?;
        // Every copy submitted has completed, so every slot copied maps now.
        self.request_maps(self.frame);
        device.poll(wgpu::PollType::wait_indefinitely())?;
        for index in slots {
            self.read(index);
        }
        Ok(())
    }

    /// One past the last frame whose submission completed, as far as the
    /// submissions and polls since have seen.
    #[cfg(test)]
    pub(crate) fn frames_completed(&self) -> u64 {
        self.frames_completed.load(Ordering::Acquire)
    }

    /// The stats the last `begin_frame` or `flush` read back, by frame.
    pub fn read_back(&self) -> &[GpuFrameStats] {
        &self.read_back
    }

    /// Requests the maps of the slots whose copies, recorded in frame
    /// `latest_copy` or before it, have completed.
    fn request_maps(&mut self, latest_copy: u64) {
        for slot in &mut self.slots {
            if let SlotState::Copying { frame, copied_in } = slot.state
                && copied_in <= latest_copy
                && slot.progress.load(Ordering::Acquire) == COPY_DONE
            {
                slot.map(frame);
            }
        }
    }

    /// The progress of each slot whose timestamps frame `frame` copies, for
    /// its work-done callback to mark them copied.
    fn copying_in(&self, frame: u64) -> impl Iterator<Item = Arc<AtomicU8>> + '_ {
        self.slots
            .iter()
            .filter(move |slot| {
                matches!(slot.state, SlotState::Copying { copied_in, .. } if copied_in == frame)
            })
            .map(|slot| slot.progress.clone())
    }

    /// Records the copy of `frame`'s timestamps, from slot `index`'s
    /// queries, into the slot, which maps once the copy is done and
    /// `MAP_LAG` frames have passed.
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
        slot.progress.store(COPY_PENDING, Ordering::Relaxed);
        slot.state = SlotState::Copying {
            frame,
            copied_in: self.frame,
        };
    }

    /// Reads slot `index` back if it is mapped, and frees it. Returns
    /// whether it is free; one still copying or mapping is not. A frame
    /// whose draw had no room for some Instances, whose Froxel lists had
    /// none for some lights, or whose shadow mask had no bit for some, is
    /// warned about, once until a frame read back has room for all; a
    /// skipped readback neither warns nor clears it.
    fn read(&mut self, index: usize) -> bool {
        let slot = &mut self.slots[index];
        let SlotState::Mapping { frame } = slot.state else {
            return !matches!(slot.state, SlotState::Copying { .. });
        };
        match slot.progress.load(Ordering::Acquire) {
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
                        let light_counts: LightCounts = bytemuck::pod_read_unaligned(
                            &view[LIGHT_COUNTS_OFFSET as usize..][..size_of::<LightCounts>()],
                        );
                        let froxel_overflowed = light_counts.froxel_lights_overflowed > 0;
                        if froxel_overflowed && !self.warned_of_froxel_overflow {
                            tracing::warn!(
                                frame,
                                froxel_lights_overflowed = light_counts.froxel_lights_overflowed,
                                "The Froxel lists are full: lights past a list's capacity do not light its Froxel"
                            );
                        }
                        self.warned_of_froxel_overflow = froxel_overflowed;
                        let shadow_mask_lights_overflowed: u32 = bytemuck::pod_read_unaligned(
                            &view[SHADOW_MASK_OVERFLOW_OFFSET as usize..][..size_of::<u32>()],
                        );
                        let shadow_mask_overflowed = shadow_mask_lights_overflowed > 0;
                        if shadow_mask_overflowed && !self.warned_of_shadow_mask_overflow {
                            tracing::warn!(
                                frame,
                                shadow_mask_lights_overflowed,
                                "The shadow mask is full: lights past a pixel's bits cast no shadow there"
                            );
                        }
                        self.warned_of_shadow_mask_overflow = shadow_mask_overflowed;
                        let ticks: [u64; QUERIES_PER_FRAME] = bytemuck::pod_read_unaligned(
                            &view[TIMESTAMP_OFFSET as usize..][..TIMESTAMP_BYTES as usize],
                        );
                        // The dynamic TLAS's build runs alongside the frame
                        // before it, which hides most of it, so it counts
                        // from the end of that frame's last pass.
                        let previous_pass_end = self
                            .last_pass_end
                            .filter(|&(previous, _)| previous + 1 == frame)
                            .map_or(0, |(_, tick)| tick);
                        let FrameTimes {
                            pass_times,
                            frame_time,
                            gate_time,
                            lit_pass_wait,
                            last_pass_end,
                        } = frame_times(
                            &ticks,
                            &slot.pass_log,
                            previous_pass_end,
                            self.nanoseconds_per_tick,
                        );
                        self.last_pass_end = Some((frame, last_pass_end));
                        self.read_back.push(GpuFrameStats {
                            frame,
                            pass_log: slot.pass_log,
                            pass_times,
                            frame_time,
                            gate_time,
                            lit_pass_wait,
                            instances_drawn: draw.instances_drawn,
                            triangles_drawn: draw.triangles_drawn(),
                            phase_two_triangles_drawn: draw.phase_two_triangles_drawn(),
                            instances_overflowed: draw.instances_overflowed,
                            froxel_lights_overflowed: light_counts.froxel_lights_overflowed,
                            shadow_mask_lights_overflowed,
                            lit_pixels: light_counts.lit_pixels,
                            listed_point_lights: light_counts.listed_point_lights,
                            reaching_point_lights: light_counts.reaching_point_lights,
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

impl Slot {
    /// Requests the map of its buffer, whose copies completed, to read
    /// `frame`'s stats from.
    fn map(&mut self, frame: u64) {
        self.progress.store(MAP_PENDING, Ordering::Relaxed);
        let progress = self.progress.clone();
        self.buffer
            .map_async(wgpu::MapMode::Read, .., move |result| {
                let reported = if result.is_ok() { MAP_DONE } else { MAP_FAILED };
                progress.store(reported, Ordering::Release);
            });
        self.state = SlotState::Mapping { frame };
    }
}

impl PassLog {
    /// The pass at `position` in `TIMED_PASSES`, if the frame ran it.
    pub fn get(&self, position: usize) -> Option<LoggedPass> {
        self.passes[position]
    }

    /// Marks the pass at `position` run, as `kind`, in the submission being
    /// recorded.
    fn mark(&mut self, position: usize, kind: PassKind) {
        if kind != PassKind::Bracket {
            debug_assert!(
                self.last_in_submission.is_none_or(|last| last < position),
                "{} marked out of TIMED_PASSES order",
                TIMED_PASSES[position]
            );
            self.last_in_submission = Some(position);
        }
        self.passes[position] = Some(LoggedPass {
            kind,
            submission: self.submission,
        });
    }

    fn split_submission(&mut self) {
        self.submission += 1;
        self.last_in_submission = None;
    }

    /// Whether the frame ran the pass at `position` and it wrote both its
    /// timestamps, at its start and its end.
    fn wrote_both(&self, position: usize) -> bool {
        self.passes[position]
            .is_some_and(|pass| matches!(pass.kind, PassKind::Compute | PassKind::Render))
    }
}

impl<'a> FrameTimestamps<'a> {
    /// The queries of `pass`, of `TIMED_PASSES`.
    pub fn pass(&mut self, pass: &str) -> PassQueries<'a, '_> {
        let position = pass_position(pass);
        PassQueries {
            query_set: self.query_set,
            first_query: self.first_query + 2 * position as u32,
            position,
            log: self.log,
        }
    }

    /// Ends the frame's submission: the passes marked after it are submitted
    /// after those marked before it, apart.
    pub fn split_submission(&mut self) {
        self.log.split_submission();
    }
}

impl<'a> PassQueries<'a, '_> {
    /// The timestamp writes of the pass, a compute pass.
    pub fn compute(self) -> wgpu::ComputePassTimestampWrites<'a> {
        self.log.mark(self.position, PassKind::Compute);
        wgpu::ComputePassTimestampWrites {
            query_set: self.query_set,
            beginning_of_pass_write_index: Some(self.first_query),
            end_of_pass_write_index: Some(self.first_query + 1),
        }
    }

    /// The timestamp writes of the pass, a render pass.
    pub fn render(self) -> wgpu::RenderPassTimestampWrites<'a> {
        self.log.mark(self.position, PassKind::Render);
        wgpu::RenderPassTimestampWrites {
            query_set: self.query_set,
            beginning_of_pass_write_index: Some(self.first_query),
            end_of_pass_write_index: Some(self.first_query + 1),
        }
    }

    /// The timestamp writes that bracket the pass, in a command buffer of
    /// its own, with two compute passes around it: the end of the first,
    /// the start of the second.
    pub fn bracket(self) -> [wgpu::ComputePassTimestampWrites<'a>; 2] {
        self.log.mark(self.position, PassKind::Bracket);
        [
            wgpu::ComputePassTimestampWrites {
                query_set: self.query_set,
                beginning_of_pass_write_index: None,
                end_of_pass_write_index: Some(self.first_query),
            },
            wgpu::ComputePassTimestampWrites {
                query_set: self.query_set,
                beginning_of_pass_write_index: Some(self.first_query + 1),
                end_of_pass_write_index: None,
            },
        ]
    }

    /// The timestamp write of a marker that ends the pass at its start: the
    /// pass starts where the pass before it ends.
    #[cfg_attr(not(target_vendor = "apple"), allow(dead_code))]
    pub fn marker(self) -> wgpu::ComputePassTimestampWrites<'a> {
        self.log.mark(self.position, PassKind::Marker);
        wgpu::ComputePassTimestampWrites {
            query_set: self.query_set,
            beginning_of_pass_write_index: Some(self.first_query + 1),
            end_of_pass_write_index: None,
        }
    }
}

/// A frame's GPU times.
struct FrameTimes {
    pass_times: [(&'static str, Duration); TIMED_PASSES.len()],
    frame_time: Duration,
    gate_time: Duration,
    lit_pass_wait: Duration,
    /// The tick the frame's last pass ended at.
    last_pass_end: u64,
}

/// Each of `TIMED_PASSES`' time in a frame whose timestamps read `ticks`,
/// `nanoseconds_per_tick` apart, and which ran the passes `log` marked, a
/// render pass's past the passes before it that it waits on; the frame's
/// GPU time: the dynamic TLAS's build, plus the span of each submission's
/// passes, so neither the waits between submissions nor what a submission
/// records before its first pass counts; ADR 0019's gate, from the start of
/// culling phase one to the latest end of the passes up to the lit pass; the
/// lit pass's wait, from the latest end of the passes from culling phase one
/// up to it to its start; and the tick its last pass ended at. The build,
/// in a submission before the frame's passes, starts no earlier than
/// `build_floor`; a pass the frame did not run takes no time.
fn frame_times(
    ticks: &[u64; QUERIES_PER_FRAME],
    log: &PassLog,
    build_floor: u64,
    nanoseconds_per_tick: f64,
) -> FrameTimes {
    let time = |begin: u64, end: u64| {
        let ticks = end.saturating_sub(begin) as f64;
        Duration::from_nanos((ticks * nanoseconds_per_tick) as u64)
    };
    let ran = |pass: usize| log.get(pass).is_some();
    let (bend_pass, blas_refit) = (pass_position(BEND_PASS), pass_position(BLAS_REFIT_PASS));
    let refitted = ran(blas_refit);
    // An untimed refit leaves the build inside the span from the bend pass.
    let build_spanned = !refitted
        && log
            .get(bend_pass)
            .zip(log.get(0))
            .is_some_and(|(bend, build)| bend.submission < build.submission);
    let build = if ran(0) && !build_spanned {
        time(ticks[0].max(build_floor), ticks[1])
    } else {
        Duration::ZERO
    };
    // From the earliest start to the latest end of `passes` that write both
    // timestamps: Metal can run a pass alongside those recorded around it
    // when it reads none of their output, so the Froxel pass can start
    // before culling phase one, and a pass can end after the last.
    let bounds = |passes: std::ops::Range<usize>| {
        passes
            .filter(|&pass| log.wrote_both(pass))
            .fold((u64::MAX, 0), |(begin, end), pass| {
                (begin.min(ticks[2 * pass]), end.max(ticks[2 * pass + 1]))
            })
    };
    let span = |passes| {
        let (begin, end) = bounds(passes);
        time(begin, end)
    };
    // The refit runs from the end of the bend pass, and the frame's other
    // passes in a later submission. A frame that bends nothing records no
    // bend pass, whose timestamps are then an earlier frame's.
    let frame_time = if refitted {
        build
            + time(ticks[2 * bend_pass], ticks[2 * blas_refit + 1])
            + span(blas_refit + 1..TIMED_PASSES.len())
    } else {
        build + span(bend_pass..TIMED_PASSES.len())
    };
    // Apple GPUs can run a render pass's vertex stage while its fragment
    // stage waits for the passes before it [ref:apple-metal-fence-per-stage],
    // and the M4 Pro writes the pass's start timestamp as the vertex stage
    // starts: the lit pass's beside the shadow mask pass in some frames, the
    // tonemapping pass's beside the lit pass in every frame. So a render
    // pass counts from the latest end of the passes before it, if later than
    // its start: its time is what it adds past them. A compute pass keeps
    // its own time, as the Froxel pass runs beside culling phase one. The
    // lit pass is the first render pass to read the Froxel pass's lists, so
    // the depth passes, which Metal can run beside it and end first, do not
    // count from its end.
    let (froxel_pass, lit_pass) = (pass_position(FROXEL_PASS), pass_position(LIT_PASS));
    let counted_begin = |pass: usize| {
        let begin = ticks[2 * pass];
        if log
            .get(pass)
            .is_none_or(|logged| logged.kind != PassKind::Render)
        {
            return begin;
        }
        (bend_pass..pass)
            .filter(|&earlier| ran(earlier) && (earlier != froxel_pass || pass >= lit_pass))
            .map(|earlier| ticks[2 * earlier + 1])
            .fold(begin, u64::max)
    };
    let pass_times = std::array::from_fn(|pass| {
        let taken = match pass {
            0 => build,
            _ if pass == blas_refit && refitted => {
                time(ticks[2 * bend_pass + 1], ticks[2 * blas_refit + 1])
            }
            _ if !ran(pass) => Duration::ZERO,
            _ => time(counted_begin(pass), ticks[2 * pass + 1]),
        };
        (TIMED_PASSES[pass], taken)
    });
    let culling_phase_one = pass_position(CullingPhase::One.label());
    let (_, gate_end) = bounds(culling_phase_one..lit_pass + 1);
    let (_, before_lit_end) = bounds(culling_phase_one..lit_pass);
    FrameTimes {
        pass_times,
        frame_time,
        gate_time: time(ticks[2 * culling_phase_one], gate_end),
        lit_pass_wait: time(before_lit_end, ticks[2 * lit_pass]),
        last_pass_end: bounds(blas_refit + 1..TIMED_PASSES.len()).1,
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
pub(crate) mod tests {
    use std::io::Write;

    use super::*;
    use crate::capture::{FrameWait, render_frames};
    use crate::gpu::GpuContext;
    use crate::render::main_pass::{DEPTH_PREPASS, PHASE_TWO_DEPTH_PASS};
    use crate::render::shadow_mask_pass::SHADOW_MASK_PASS;
    use crate::render::tonemapping_pass::TONEMAPPING_PASS;
    use crate::render::{CrowdForm, RenderSwitches};
    use crate::scenes::ReferenceScene;

    /// How a frame bends Meshes: not at all, in its one submission, or in a
    /// submission ahead of its others, which the refit's marker ends if it
    /// is timed.
    #[derive(Clone, Copy, Debug)]
    pub(crate) enum Bending {
        Unbent,
        InFrame,
        Submitted { refit_timed: bool },
    }

    /// A frame as `render` records it: how it bends, and whether it
    /// brackets a build of the dynamic TLAS.
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct RecordedFrame {
        pub(crate) bending: Bending,
        pub(crate) built: bool,
    }

    impl RecordedFrame {
        pub(crate) const STILL: Self = Self {
            bending: Bending::Unbent,
            built: false,
        };

        /// The log of its passes, marked as `render` marks them, phase two's
        /// depth pass and the shadow mask pass among them if `shadow_rays`.
        pub(crate) fn log(self, shadow_rays: bool) -> PassLog {
            let mut log = PassLog::default();
            if !matches!(self.bending, Bending::Unbent) {
                log.mark(pass_position(BEND_PASS), PassKind::Compute);
            }
            if let Bending::Submitted { refit_timed } = self.bending {
                if refit_timed {
                    log.mark(pass_position(BLAS_REFIT_PASS), PassKind::Marker);
                }
                log.split_submission();
            }
            if self.built {
                log.mark(0, PassKind::Bracket);
            }
            let render_passes = [
                DEPTH_PREPASS,
                PHASE_TWO_DEPTH_PASS,
                LIT_PASS,
                TONEMAPPING_PASS,
            ];
            let shadow_ray_passes = [PHASE_TWO_DEPTH_PASS, SHADOW_MASK_PASS];
            let first_pass = pass_position(CullingPhase::One.label());
            for (position, pass) in TIMED_PASSES.iter().enumerate().skip(first_pass) {
                if shadow_rays || !shadow_ray_passes.contains(pass) {
                    let kind = if render_passes.contains(pass) {
                        PassKind::Render
                    } else {
                        PassKind::Compute
                    };
                    log.mark(position, kind);
                }
            }
            log
        }
    }

    /// Timestamps a microsecond apart, each pass starting 10 µs after the
    /// one before it and taking 4 µs, in submissions that start 100 µs
    /// apart: the build, then the bend pass and the refit, then the rest.
    /// The refit's start is not written, and it ends 7 µs after the bend.
    fn spread_ticks() -> [u64; QUERIES_PER_FRAME] {
        let (bend_pass, blas_refit) = (pass_position(BEND_PASS), pass_position(BLAS_REFIT_PASS));
        let mut ticks = [0; QUERIES_PER_FRAME];
        for pass in 0..TIMED_PASSES.len() {
            let submission = match pass {
                0 => 0,
                _ if pass <= blas_refit => 1,
                _ => 2,
            };
            ticks[2 * pass] = 1000 + 100 * submission + 10 * pass as u64;
            ticks[2 * pass + 1] = ticks[2 * pass] + 4;
        }
        ticks[2 * blas_refit] = 0;
        ticks[2 * blas_refit + 1] = ticks[2 * bend_pass + 1] + 7;
        ticks
    }

    /// Asserts `frame_times`' whole output, in microseconds, for the
    /// `spread_ticks` frame recorded as `frame` after a frame whose last
    /// pass ended 1 µs into this one's build.
    fn assert_spread_frame(
        frame: RecordedFrame,
        shadow_rays: bool,
        pass_microseconds: [u64; TIMED_PASSES.len()],
        frame_microseconds: u64,
        gate_microseconds: u64,
        lit_pass_wait_microseconds: u64,
    ) {
        let ticks = spread_ticks();
        let times = frame_times(&ticks, &frame.log(shadow_rays), ticks[0] + 1, 1000.0);
        let microseconds = Duration::from_micros;
        let expected_passes: [(&str, Duration); TIMED_PASSES.len()] =
            std::array::from_fn(|pass| (TIMED_PASSES[pass], microseconds(pass_microseconds[pass])));
        let kind = format!("{frame:?}, shadow rays {shadow_rays}");
        assert_eq!(times.pass_times, expected_passes, "{kind}");
        assert_eq!(times.frame_time, microseconds(frame_microseconds), "{kind}");
        assert_eq!(times.gate_time, microseconds(gate_microseconds), "{kind}");
        let lit_pass_wait = microseconds(lit_pass_wait_microseconds);
        assert_eq!(times.lit_pass_wait, lit_pass_wait, "{kind}");
        assert_eq!(times.last_pass_end, 1314, "{kind}");
    }

    #[test]
    fn a_frame_that_neither_builds_nor_bends_spans_its_last_submission() {
        let frame = RecordedFrame::STILL;
        let with_shadow_rays = [0, 0, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4];
        assert_spread_frame(frame, true, with_shadow_rays, 84, 74, 6);
        let without_shadow_rays = [0, 0, 0, 4, 4, 4, 4, 4, 0, 0, 4, 4];
        assert_spread_frame(frame, false, without_shadow_rays, 84, 74, 26);
    }

    #[test]
    fn a_frame_that_builds_but_does_not_bend_adds_its_build_from_the_floor() {
        let frame = RecordedFrame {
            built: true,
            ..RecordedFrame::STILL
        };
        let with_shadow_rays = [3, 0, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4];
        assert_spread_frame(frame, true, with_shadow_rays, 87, 74, 6);
        let without_shadow_rays = [3, 0, 0, 4, 4, 4, 4, 4, 0, 0, 4, 4];
        assert_spread_frame(frame, false, without_shadow_rays, 87, 74, 26);
    }

    /// A bent frame whose refit is absent and whose build is not bracketed,
    /// one whose bend pass is submitted ahead of the rest with no refit
    /// timed, the rebuild form's, and one whose refit went untimed beside a
    /// bracketed build, which its span from the bend pass holds.
    #[test]
    fn a_bent_frame_without_a_bracketed_build_spans_from_the_bend_pass() {
        let untimed_refit = Bending::Submitted { refit_timed: false };
        for (bending, built) in [
            (Bending::InFrame, false),
            (untimed_refit, false),
            (untimed_refit, true),
        ] {
            let frame = RecordedFrame { bending, built };
            let with_shadow_rays = [0, 4, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4];
            assert_spread_frame(frame, true, with_shadow_rays, 204, 74, 6);
            let without_shadow_rays = [0, 4, 0, 4, 4, 4, 4, 4, 0, 0, 4, 4];
            assert_spread_frame(frame, false, without_shadow_rays, 204, 74, 26);
        }
    }

    #[test]
    fn a_bent_frame_with_a_bracketed_build_and_no_refit_adds_the_build() {
        let frame = RecordedFrame {
            bending: Bending::InFrame,
            built: true,
        };
        let with_shadow_rays = [3, 4, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4];
        assert_spread_frame(frame, true, with_shadow_rays, 207, 74, 6);
        let without_shadow_rays = [3, 4, 0, 4, 4, 4, 4, 4, 0, 0, 4, 4];
        assert_spread_frame(frame, false, without_shadow_rays, 207, 74, 26);
    }

    /// No Moving Instance, so no build, beside a timed refit of a bent Mesh.
    #[test]
    fn a_refitted_frame_without_a_build_times_its_refit_from_the_bend() {
        let frame = RecordedFrame {
            bending: Bending::Submitted { refit_timed: true },
            built: false,
        };
        let with_shadow_rays = [0, 4, 7, 4, 4, 4, 4, 4, 4, 4, 4, 4];
        assert_spread_frame(frame, true, with_shadow_rays, 95, 74, 6);
        let without_shadow_rays = [0, 4, 7, 4, 4, 4, 4, 4, 0, 0, 4, 4];
        assert_spread_frame(frame, false, without_shadow_rays, 95, 74, 26);
    }

    #[test]
    fn a_refitted_frame_with_a_build_times_three_submissions_apart() {
        let frame = RecordedFrame {
            bending: Bending::Submitted { refit_timed: true },
            built: true,
        };
        let with_shadow_rays = [3, 4, 7, 4, 4, 4, 4, 4, 4, 4, 4, 4];
        assert_spread_frame(frame, true, with_shadow_rays, 98, 74, 6);
        let without_shadow_rays = [3, 4, 7, 4, 4, 4, 4, 4, 0, 0, 4, 4];
        assert_spread_frame(frame, false, without_shadow_rays, 98, 74, 26);
    }

    #[test]
    fn a_refitted_frame_times_its_submissions_apart_and_its_refit_from_the_bend() {
        let ticks = spread_ticks();
        let microseconds = Duration::from_micros;
        let refitted = RecordedFrame {
            bending: Bending::Submitted { refit_timed: true },
            built: true,
        };
        let FrameTimes {
            pass_times: passes,
            frame_time,
            ..
        } = frame_times(&ticks, &refitted.log(true), 0, 1000.0);
        let taken = |name| passes.iter().find(|&&(pass, _)| pass == name).unwrap().1;
        assert_eq!(taken(BLAS_REFIT_PASS), microseconds(7));
        assert_eq!(taken(BEND_PASS), microseconds(4));
        let rest = (pass_position(BLAS_REFIT_PASS) + 1) as u64..TIMED_PASSES.len() as u64;
        let rest_span = 10 * (rest.end - 1 - rest.start) + 4;
        // The build, the bend pass and the refit, and the rest of the frame.
        assert_eq!(
            frame_time,
            microseconds(4) + microseconds(4 + 7) + microseconds(rest_span)
        );

        let proxies = RecordedFrame {
            bending: Bending::InFrame,
            ..refitted
        };
        let FrameTimes {
            pass_times: passes,
            frame_time,
            ..
        } = frame_times(&ticks, &proxies.log(true), 0, 1000.0);
        assert_eq!(passes[pass_position(BLAS_REFIT_PASS)].1, Duration::ZERO);
        let bend_start = ticks[2 * pass_position(BEND_PASS)];
        let span = ticks[QUERIES_PER_FRAME - 1] - bend_start;
        assert_eq!(frame_time, microseconds(4) + microseconds(span));

        let still = RecordedFrame::STILL.log(true);
        let FrameTimes {
            pass_times: passes,
            frame_time,
            ..
        } = frame_times(&ticks, &still, 0, 1000.0);
        assert_eq!(passes[0].1, Duration::ZERO);
        assert_eq!(passes[pass_position(BEND_PASS)].1, Duration::ZERO);
        assert_eq!(frame_time, microseconds(rest_span));
    }

    /// Metal can start a compute pass before the passes recorded ahead of it
    /// when it reads none of their output, as the Froxel pass does culling
    /// phase one's, and end one after the last: the frame spans its passes'
    /// earliest start and latest end, so no pass outlasts it. An untimed
    /// pass's stale timestamps, and the unwritten start of the refit, span
    /// nothing.
    #[test]
    fn a_frame_spans_its_passes_earliest_start_and_latest_end() {
        let mut ticks = spread_ticks();
        let microseconds = Duration::from_micros;
        let (froxel_pass, lit_pass) = (pass_position(FROXEL_PASS), pass_position(LIT_PASS));
        let shadow_mask_pass = pass_position(SHADOW_MASK_PASS);
        let culling_start = ticks[2 * pass_position(CullingPhase::One.label())];
        ticks[2 * froxel_pass] = culling_start - 30;
        ticks[2 * lit_pass + 1] = ticks[QUERIES_PER_FRAME - 1] + 20;
        ticks[2 * shadow_mask_pass] = 1;
        ticks[2 * shadow_mask_pass + 1] = 1 << 40;
        let without_shadow_mask = |frame: RecordedFrame| {
            let mut log = frame.log(true);
            log.passes[shadow_mask_pass] = None;
            log
        };
        let rest_span = ticks[2 * lit_pass + 1] - (culling_start - 30);

        let still = without_shadow_mask(RecordedFrame::STILL);
        let FrameTimes {
            pass_times: passes,
            frame_time,
            ..
        } = frame_times(&ticks, &still, 0, 1000.0);
        assert_eq!(frame_time, microseconds(rest_span));
        assert!(passes.iter().all(|&(_, taken)| taken <= frame_time));

        let refitted = without_shadow_mask(RecordedFrame {
            bending: Bending::Submitted { refit_timed: true },
            built: true,
        });
        let frame_time = frame_times(&ticks, &refitted, 0, 1000.0).frame_time;
        assert_eq!(
            frame_time,
            microseconds(4) + microseconds(4 + 7) + microseconds(rest_span)
        );
    }

    /// ADR 0019's gate runs from the first culling dispatch to the latest
    /// end of the passes up to the lit pass, with neither the build nor the
    /// passes after the lit pass, even where Metal starts the Froxel pass
    /// earlier; the frame's last pass ends at the latest end of any.
    #[test]
    fn the_gate_spans_culling_phase_one_to_the_lit_pass() {
        let mut ticks = spread_ticks();
        let culling_start = ticks[2 * pass_position(CullingPhase::One.label())];
        let lit_end = ticks[2 * pass_position(LIT_PASS) + 1];
        let built = RecordedFrame {
            built: true,
            ..RecordedFrame::STILL
        }
        .log(false);
        let microseconds = Duration::from_micros;
        let times = frame_times(&ticks, &built, 0, 1000.0);
        assert_eq!(times.gate_time, microseconds(lit_end - culling_start));
        assert_eq!(times.last_pass_end, ticks[QUERIES_PER_FRAME - 1]);

        ticks[2 * pass_position(FROXEL_PASS)] = culling_start - 30;
        let times = frame_times(&ticks, &built, 0, 1000.0);
        assert_eq!(times.gate_time, microseconds(lit_end - culling_start));
    }

    /// The lit pass's wait runs from the latest end of the passes from
    /// culling phase one up to it to its start, so with the passes before
    /// it and its own time it makes up the gate; passes that did not run
    /// bound nothing, and a lit pass that starts beside the passes before
    /// it waits for none of them.
    #[test]
    fn the_lit_pass_waits_from_the_latest_end_before_it() {
        let mut ticks = spread_ticks();
        let microseconds = Duration::from_micros;
        let culling_start = ticks[2 * pass_position(CullingPhase::One.label())];
        let culling_two_end = ticks[2 * pass_position(CullingPhase::Two.label()) + 1];
        let lit_pass = pass_position(LIT_PASS);
        ticks[2 * lit_pass] = culling_two_end + 12_000;
        ticks[2 * lit_pass + 1] = culling_two_end + 12_500;
        // A stale end from an earlier frame's shadow mask pass.
        ticks[2 * pass_position(SHADOW_MASK_PASS) + 1] = culling_two_end + 20_000;
        let without_shadow_rays = RecordedFrame::STILL.log(false);

        let times = frame_times(&ticks, &without_shadow_rays, 0, 1000.0);
        assert_eq!(times.lit_pass_wait, microseconds(12_000));
        assert_eq!(
            times.gate_time,
            microseconds(culling_two_end - culling_start) + microseconds(12_000 + 500)
        );

        ticks[2 * lit_pass] = culling_two_end + 19_000;
        ticks[2 * lit_pass + 1] = culling_two_end + 21_000;
        let times = frame_times(&ticks, &RecordedFrame::STILL.log(true), 0, 1000.0);
        assert_eq!(times.lit_pass_wait, Duration::ZERO);
    }

    /// Metal starts the lit pass's vertex work beside the shadow mask pass
    /// in some frames, and the tonemapping pass's beside the lit pass in
    /// every frame, writing each start timestamp then: each counts from the
    /// end of the passes before it. The Froxel pass, a compute pass running
    /// beside culling phase one and outlasting it, keeps its own time; a
    /// pass that did not run, here the bend pass, bounds nothing with its
    /// stale end; and neither the frame nor the gate moves.
    #[test]
    fn a_render_pass_counts_from_the_end_of_the_passes_before_it() {
        let mut ticks = spread_ticks();
        let microseconds = Duration::from_micros;
        let culling_phase_one = pass_position(CullingPhase::One.label());
        let froxel_pass = pass_position(FROXEL_PASS);
        let shadow_mask_pass = pass_position(SHADOW_MASK_PASS);
        let (lit_pass, tonemapping_pass) =
            (pass_position(LIT_PASS), pass_position(TONEMAPPING_PASS));
        let culling_start = ticks[2 * culling_phase_one];
        ticks[2 * culling_phase_one + 1] = culling_start + 8;
        ticks[2 * froxel_pass] = culling_start - 3;
        ticks[2 * froxel_pass + 1] = culling_start + 9;
        ticks[2 * pass_position(BEND_PASS) + 1] = 1 << 40;
        let mask_start = ticks[2 * shadow_mask_pass];
        ticks[2 * shadow_mask_pass + 1] = mask_start + 2_000;
        ticks[2 * lit_pass] = mask_start + 1;
        ticks[2 * lit_pass + 1] = mask_start + 2_480;
        ticks[2 * tonemapping_pass] = mask_start + 12;
        ticks[2 * tonemapping_pass + 1] = mask_start + 2_800;

        let times = frame_times(&ticks, &RecordedFrame::STILL.log(true), 0, 1000.0);
        let taken = |pass: usize| times.pass_times[pass].1;
        assert_eq!(taken(lit_pass), microseconds(480));
        assert_eq!(taken(tonemapping_pass), microseconds(320));
        assert_eq!(taken(froxel_pass), microseconds(12));
        assert_eq!(taken(shadow_mask_pass), microseconds(2_000));
        assert_eq!(
            times.frame_time,
            microseconds(mask_start + 2_800 - (culling_start - 3))
        );
        assert_eq!(
            times.gate_time,
            microseconds(mask_start + 2_480 - culling_start)
        );
    }

    /// The depth passes read none of the Froxel pass's lists, so Metal can
    /// run them beside it and end them first: each counts from the passes
    /// it waits on, not from the Froxel pass's end, and reads its own time
    /// rather than 0. The lit pass reads the lists, so it counts from their
    /// end.
    #[test]
    fn a_depth_pass_counts_past_the_froxel_pass_it_does_not_read() {
        let mut ticks = spread_ticks();
        let microseconds = Duration::from_micros;
        let froxel_pass = pass_position(FROXEL_PASS);
        let lit_pass = pass_position(LIT_PASS);
        let lit_start = ticks[2 * lit_pass];
        ticks[2 * froxel_pass + 1] = lit_start + 2;

        let times = frame_times(&ticks, &RecordedFrame::STILL.log(true), 0, 1000.0);
        let taken = |pass| times.pass_times[pass_position(pass)].1;
        assert_eq!(taken(DEPTH_PREPASS), microseconds(4));
        assert_eq!(taken(PHASE_TWO_DEPTH_PASS), microseconds(4));
        assert_eq!(taken(LIT_PASS), microseconds(2));
        assert_eq!(taken(TONEMAPPING_PASS), microseconds(4));
    }

    /// The Timed passes `log` marks, in `TIMED_PASSES` order, with how each
    /// was timed and its submission.
    fn marked(log: PassLog) -> Vec<(&'static str, PassKind, u8)> {
        (0..TIMED_PASSES.len())
            .filter_map(|pass| log.get(pass).map(|logged| (pass, logged)))
            .map(|(pass, logged)| (TIMED_PASSES[pass], logged.kind, logged.submission))
            .collect()
    }

    /// `render` marks the Timed passes each kind of frame runs as the frames
    /// these tests time record them: in one submission, a frame that neither
    /// bends nor builds, with shadow rays and without, one that builds the
    /// dynamic TLAS, the orrery's, and one that bends the crowd and builds
    /// its proxies' TLAS; in two, one that rebuilds the crowd's BLASes,
    /// untimed, and on Metal one whose refit a marker ends, with the build
    /// bracketed ahead of the second.
    #[test]
    fn render_marks_the_passes_each_kind_of_frame_runs() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let defaults = RenderSwitches::default();
        let without_shadow_rays = RenderSwitches {
            shadow_rays: false,
            ..defaults
        };
        let crowd = |crowd_form| RenderSwitches {
            crowd_form,
            ..defaults
        };
        let frame = |bending, built| RecordedFrame { bending, built };
        let submitted = |refit_timed| Bending::Submitted { refit_timed };
        let mut frames = vec![
            (ReferenceScene::PbrSweep, defaults, RecordedFrame::STILL),
            (
                ReferenceScene::PbrSweep,
                without_shadow_rays,
                RecordedFrame::STILL,
            ),
            (
                ReferenceScene::Orrery,
                defaults,
                frame(Bending::Unbent, true),
            ),
            (
                ReferenceScene::Crowd,
                crowd(CrowdForm::Proxies),
                frame(Bending::InFrame, true),
            ),
            (
                ReferenceScene::Crowd,
                crowd(CrowdForm::Rebuild),
                frame(submitted(false), false),
            ),
        ];
        if context.backend() == wgpu::Backend::Metal {
            let refitted = frame(submitted(true), true);
            frames.push((ReferenceScene::Crowd, crowd(CrowdForm::Refit), refitted));
        }
        for (scene, switches, expected) in frames {
            let (_, frame_stats, _) = render_frames(
                &context,
                scene,
                glam::DVec3::ZERO,
                64,
                36,
                0..2,
                &[],
                scene.frame_delta(),
                switches,
                FrameWait::Block,
                ..,
            )
            .expect("the scene renders");
            for (frame, stats) in frame_stats.iter().enumerate() {
                let gpu = stats.gpu.expect("every frame read back its GPU stats");
                assert_eq!(
                    marked(gpu.pass_log),
                    marked(expected.log(switches.shadow_rays)),
                    "{scene:?} with {switches:?}, frame {frame}"
                );
            }
        }
    }

    /// Frames that never complete, as their encoders are never submitted:
    /// from `COMPLETION_LAG` frames on, each skips the timestamps of the
    /// frame that far back, counted, and none is read back. Then, in a ring
    /// whose frames complete, a slot whose copy never ran, as the encoder
    /// copying it was dropped, never maps and is skipped when the ring comes
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
        let light_counts = counts(
            "Readback Test Light Counts",
            size_of::<LightCounts>() as u64,
        );
        let shadow_mask_lights_overflowed = counts(
            "Readback Test Shadow Mask Overflow",
            size_of::<u32>() as u64,
        );
        // Records a frame, and submits it and waits for it if `completes`:
        // whether it is timed, the readbacks it skipped, the frames it read.
        let frame = |readback: &mut GpuStatsReadback, completes: bool| {
            let descriptor = wgpu::CommandEncoderDescriptor::default();
            let mut encoder = context.device.create_command_encoder(&descriptor);
            let mut recorder = FrameRecorder::new();
            let timed = readback.begin_frame(&mut encoder, &mut recorder).is_some();
            readback.end_frame(
                &mut encoder,
                &frame_draw,
                &light_counts,
                &shadow_mask_lights_overflowed,
            );
            if completes {
                context.queue.submit([encoder.finish()]);
                let wait = context.device.poll(wgpu::PollType::wait_indefinitely());
                wait.expect("the frame completes");
            }
            let read: Vec<u64> = readback.read_back().iter().map(|gpu| gpu.frame).collect();
            (timed, recorder.counters.gpu_readbacks_skipped, read)
        };

        let mut never_completes = new_ring();
        for number in 0..2 * SLOT_COUNT {
            let skipped = u32::from(number >= COMPLETION_LAG);
            let recorded = frame(&mut never_completes, false);
            assert_eq!(recorded, (true, skipped, vec![]), "frame {number}");
        }

        let mut ring = new_ring();
        for number in 0..SLOT_COUNT {
            // Frame 3 copies frame 0's timestamps, never to be mapped.
            let completes = number != COMPLETION_LAG;
            assert_eq!(
                frame(&mut ring, completes),
                (true, 0, vec![]),
                "frame {number}"
            );
        }
        let frame_number = SLOT_COUNT;
        assert_eq!(
            frame(&mut ring, true),
            (false, 1, vec![]),
            "frame {frame_number}"
        );
        let frame_number = SLOT_COUNT + 1;
        assert_eq!(
            frame(&mut ring, true),
            (true, 0, vec![1]),
            "frame {frame_number}"
        );
    }
}
