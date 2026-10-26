//! Each frame's GPU stats come back through a ring of buffers the CPU maps
//! for reading: the counts of the frame's draw, of its lights and of the
//! lights its shadow mask had no bit for, and its passes' timestamps, a few
//! frames late.
//! Only the main thread submits and polls, and it never
//! waits on a map while it renders (ADR 0016): a readback whose slot is not
//! ready is skipped and counted; `flush` waits, at the end of a run. Metal
//! reads timestamps resolved before their pass's submission completed as 0
//! or stale (`docs/research/wgpu-capability-matrix.md`), so a frame's
//! timestamps are resolved only once its submission completed.

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

use crate::render::bend_pass::BEND_PASS;
use crate::render::blas_refit::BLAS_REFIT_PASS;
use crate::render::culling_pass::FrameDraw;
use crate::render::froxel_pass::LightCounts;
use crate::render::shadow_mask_pass::SHADOW_MASK_PASS;
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

/// Where a slot holds what the frame's lights counted (`LightCounts`), after
/// its draw's counts.
const LIGHT_COUNTS_OFFSET: u64 = size_of::<FrameDraw>() as u64;

/// Where a slot holds the count of the lights the frame's shadow mask had no
/// bit for, after its lights' counts.
const SHADOW_MASK_OVERFLOW_OFFSET: u64 = LIGHT_COUNTS_OFFSET + size_of::<LightCounts>() as u64;

/// Where a slot holds the frame's timestamps, after its counts.
const TIMESTAMP_OFFSET: u64 =
    (SHADOW_MASK_OVERFLOW_OFFSET + size_of::<u32>() as u64).next_multiple_of(8);

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
    /// Set while the frames read back had lights their shadow mask had no
    /// bit for, which was warned about when it was set.
    warned_of_shadow_mask_overflow: bool,
    /// Which of `TIMED_PASSES` never run, so read as taking no time.
    untimed: [bool; TIMED_PASSES.len()],
    /// The last frame read back and the tick its shadow mask pass ended at,
    /// in frames that trace shadow rays: the next frame's build of the
    /// dynamic TLAS, which the rays read, waits for it.
    last_shadow_ray_end: Option<(u64, u64)>,
}

struct Slot {
    /// Its frame's draw, then its timestamps.
    buffer: wgpu::Buffer,
    state: SlotState,
    map_result: Arc<AtomicU8>,
    shape: FrameShape,
}

/// Which of the timed work that runs in some frames only a frame ran.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameShape {
    /// Whether it built the dynamic TLAS, the first of `TIMED_PASSES`,
    /// between the markers its timestamps bracket: in a frame that does
    /// not, the build reads as taking no time.
    pub dynamic_tlas_built: bool,
    /// Whether it bent Meshes in the bend pass: in a frame that does not,
    /// the pass reads as taking no time, and the frame's passes start at the
    /// first after the BLAS refit.
    pub bent: bool,
    /// Whether it refitted the bent Meshes' BLASes after the bend pass, in a
    /// submission of their own, up to a marker its timestamps end them at:
    /// in a frame that does not, the refit reads as taking no time.
    pub blas_refitted: bool,
}

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
    /// The ring, empty; timestamps tick every `nanoseconds_per_tick`, and
    /// the passes of `TIMED_PASSES` that `untimed` names never run.
    pub fn new(device: &wgpu::Device, nanoseconds_per_tick: f32, untimed: &[&str]) -> Self {
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
                    shape: FrameShape::default(),
                })
                .collect(),
            frames_completed: Arc::new(AtomicU64::new(0)),
            frame: 0,
            nanoseconds_per_tick: f64::from(nanoseconds_per_tick),
            read_back: Vec::with_capacity(SLOT_COUNT as usize),
            warned_of_overflow: false,
            warned_of_froxel_overflow: false,
            warned_of_shadow_mask_overflow: false,
            untimed: untimed
                .iter()
                .fold([false; TIMED_PASSES.len()], |mut flags, pass| {
                    flags[pass_position(pass)] = true;
                    flags
                }),
            last_shadow_ray_end: None,
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
        self.slots[index].shape = FrameShape::default();
        Some(FrameTimestamps {
            query_set: &self.query_set,
            first_query: (index * QUERIES_PER_FRAME) as u32,
        })
    }

    /// Records which of the timed work that runs in some frames only the
    /// frame `begin_frame` started ran.
    pub fn shape(&mut self, shape: FrameShape) {
        let slot = &mut self.slots[slot_index(self.frame)];
        if matches!(slot.state, SlotState::Recorded { frame } if frame == self.frame) {
            slot.shape = shape;
        }
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
    /// for some Instances, whose Froxel lists had none for some lights, or
    /// whose shadow mask had no bit for some, is warned about, once until a
    /// frame read back has room for all; a skipped readback neither warns
    /// nor clears it.
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
                        // The marker before the dynamic TLAS's build can run
                        // while the frame before still traces rays against
                        // the TLAS, which the build waits for.
                        let previous_shadow_ray_end = self
                            .last_shadow_ray_end
                            .filter(|&(previous, _)| previous + 1 == frame)
                            .map_or(0, |(_, tick)| tick);
                        let shadow_mask_pass = pass_position(SHADOW_MASK_PASS);
                        self.last_shadow_ray_end = (!self.untimed[shadow_mask_pass])
                            .then(|| (frame, ticks[2 * shadow_mask_pass + 1]));
                        let (pass_times, frame_time) = frame_times(
                            &ticks,
                            slot.shape,
                            &self.untimed,
                            previous_shadow_ray_end,
                            self.nanoseconds_per_tick,
                        );
                        self.read_back.push(GpuFrameStats {
                            frame,
                            pass_times,
                            frame_time,
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

    /// The timestamp write of a marker that ends `pass`, of `TIMED_PASSES`,
    /// at its start: `pass` starts where the pass before it ends.
    pub fn marker_after(self, pass: &str) -> wgpu::ComputePassTimestampWrites<'a> {
        wgpu::ComputePassTimestampWrites {
            query_set: self.query_set,
            beginning_of_pass_write_index: Some(self.first_query_of(pass) + 1),
            end_of_pass_write_index: None,
        }
    }

    fn first_query_of(self, pass: &str) -> u32 {
        self.first_query + 2 * pass_position(pass) as u32
    }
}

/// Each of `TIMED_PASSES`' time in a frame of `shape` whose timestamps read
/// `ticks`, `nanoseconds_per_tick` apart, and the frame's GPU time: the
/// dynamic TLAS's build, plus the span of each submission's passes, so
/// neither the waits between submissions nor what a submission records
/// before its first pass counts. The build, in a submission before the
/// frame's passes, starts no earlier than `build_floor`; the passes
/// `untimed` names take no time.
fn frame_times(
    ticks: &[u64; QUERIES_PER_FRAME],
    shape: FrameShape,
    untimed: &[bool; TIMED_PASSES.len()],
    build_floor: u64,
    nanoseconds_per_tick: f64,
) -> ([(&'static str, Duration); TIMED_PASSES.len()], Duration) {
    let time = |begin: u64, end: u64| {
        let ticks = end.saturating_sub(begin) as f64;
        Duration::from_nanos((ticks * nanoseconds_per_tick) as u64)
    };
    let (bend_pass, blas_refit) = (pass_position(BEND_PASS), pass_position(BLAS_REFIT_PASS));
    let build = if shape.dynamic_tlas_built {
        time(ticks[0].max(build_floor), ticks[1])
    } else {
        Duration::ZERO
    };
    // From the earliest start to the latest end of `passes` that write both
    // timestamps: Metal can run a pass alongside those recorded around it
    // when it reads none of their output, so the Froxel pass can start
    // before culling phase one, and a pass can end after the last.
    let span = |passes: std::ops::Range<usize>| {
        let (begin, end) = passes
            .filter(|&pass| pass != blas_refit && !untimed[pass])
            .fold((u64::MAX, 0), |(begin, end), pass| {
                (begin.min(ticks[2 * pass]), end.max(ticks[2 * pass + 1]))
            });
        time(begin, end)
    };
    // The refit runs from the end of the bend pass, and the frame's other
    // passes in a later submission. A frame that bends nothing records no
    // bend pass, whose timestamps are then an earlier frame's.
    let frame_time = if shape.blas_refitted {
        build
            + time(ticks[2 * bend_pass], ticks[2 * blas_refit + 1])
            + span(blas_refit + 1..TIMED_PASSES.len())
    } else {
        let first_pass = if shape.bent {
            bend_pass
        } else {
            blas_refit + 1
        };
        build + span(first_pass..TIMED_PASSES.len())
    };
    let pass_times = std::array::from_fn(|pass| {
        let taken = match pass {
            0 => build,
            _ if pass == bend_pass && !shape.bent => Duration::ZERO,
            _ if pass == blas_refit && shape.blas_refitted => {
                time(ticks[2 * bend_pass + 1], ticks[2 * blas_refit + 1])
            }
            _ if pass == blas_refit => Duration::ZERO,
            _ if untimed[pass] => Duration::ZERO,
            _ => time(ticks[2 * pass], ticks[2 * pass + 1]),
        };
        (TIMED_PASSES[pass], taken)
    });
    (pass_times, frame_time)
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
    use crate::render::culling_pass::CullingPhase;
    use crate::render::froxel_pass::FROXEL_PASS;
    use crate::render::main_pass::LIT_PASS;

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

    #[test]
    fn a_refitted_frame_times_its_submissions_apart_and_its_refit_from_the_bend() {
        let ticks = spread_ticks();
        let untimed = [false; TIMED_PASSES.len()];
        let microseconds = Duration::from_micros;
        let refitted = FrameShape {
            dynamic_tlas_built: true,
            bent: true,
            blas_refitted: true,
        };
        let (passes, frame_time) = frame_times(&ticks, refitted, &untimed, 0, 1000.0);
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

        let proxies = FrameShape {
            blas_refitted: false,
            ..refitted
        };
        let (passes, frame_time) = frame_times(&ticks, proxies, &untimed, 0, 1000.0);
        assert_eq!(passes[pass_position(BLAS_REFIT_PASS)].1, Duration::ZERO);
        let bend_start = ticks[2 * pass_position(BEND_PASS)];
        let span = ticks[QUERIES_PER_FRAME - 1] - bend_start;
        assert_eq!(frame_time, microseconds(4) + microseconds(span));

        let still = FrameShape::default();
        let (passes, frame_time) = frame_times(&ticks, still, &untimed, 0, 1000.0);
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
        let mut untimed = [false; TIMED_PASSES.len()];
        untimed[shadow_mask_pass] = true;
        let rest_span = ticks[2 * lit_pass + 1] - (culling_start - 30);

        let (passes, frame_time) = frame_times(&ticks, FrameShape::default(), &untimed, 0, 1000.0);
        assert_eq!(frame_time, microseconds(rest_span));
        assert!(passes.iter().all(|&(_, taken)| taken <= frame_time));

        let refitted = FrameShape {
            dynamic_tlas_built: true,
            bent: true,
            blas_refitted: true,
        };
        let (_, frame_time) = frame_times(&ticks, refitted, &untimed, 0, 1000.0);
        assert_eq!(
            frame_time,
            microseconds(4) + microseconds(4 + 7) + microseconds(rest_span)
        );
    }

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
        let new_ring = || GpuStatsReadback::new(&context.device, 1.0, &[]);
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
