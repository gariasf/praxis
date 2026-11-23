use crate::render::acceleration_structures::{DynamicTlas, StaticTlas};
use crate::render::bind_groups::{
    PROBE_RAY_TRACE, SHADOW_RAY_SCENE, compute_pipeline, pipeline_layout,
};
use crate::render::renderer::RenderCapacities;
use crate::render::shaders;

pub(crate) const PROBE_RAY_PASS: &str = "probe_ray_pass";

// WGSL declares it as `PROBE_GRID`'s product.
const PROBES: u32 = 32 * 8 * 32;

// WGSL declares it as `PROBE_RAY_WORKGROUP_SIZE`.
const WORKGROUP_SIZE: u32 = 64;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ProbeRaySwitches {
    rays_per_probe: u32,
    ray_length_metres: f32,
    unchecked: bool,
    repeats: u32,
    verify_hits: bool,
    block_frames: Option<u32>,
    direct_dispatch: bool,
}

impl ProbeRaySwitches {
    pub(crate) fn from_environment() -> Option<Self> {
        let rays_per_probe = number("PRAXIS_PROBE_RAYS_PER_PROBE")?;
        Some(Self {
            rays_per_probe,
            ray_length_metres: number("PRAXIS_PROBE_RAY_METRES").unwrap_or(256.0),
            unchecked: std::env::var_os("PRAXIS_PROBE_UNCHECKED").is_some(),
            repeats: number("PRAXIS_PROBE_REPEATS").unwrap_or(1),
            verify_hits: std::env::var_os("PRAXIS_PROBE_VERIFY").is_some(),
            block_frames: number("PRAXIS_PROBE_BLOCK_FRAMES"),
            direct_dispatch: std::env::var_os("PRAXIS_PROBE_DIRECT_DISPATCH").is_some(),
        })
    }
}

fn number<T: std::str::FromStr>(variable: &str) -> Option<T> {
    let value = std::env::var(variable).ok()?;
    match value.parse() {
        Ok(number) => Some(number),
        Err(_) => panic!("{variable}={value} is not a number"),
    }
}

pub(crate) const HELD_MEASURED_FRAMES: u32 = 600;

const ALONE_REPEATS: u32 = 8;
const ALONE_SUBMISSIONS: u32 = 45;
const ALONE_WARM_UP_SUBMISSIONS: u32 = 5;

// ponytail: a global, as one Renderer times the pass alone in a process; carry it in the frame stats if this leaves the prototype.
static ALONE_NANOSECONDS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn alone_microseconds() -> Option<f64> {
    match ALONE_NANOSECONDS.load(std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        nanoseconds => Some(nanoseconds as f64 / 1e3),
    }
}

pub(crate) fn held_camera_position() -> Option<glam::DVec3> {
    let value = std::env::var("PRAXIS_PROBE_CAMERA").ok()?;
    let coordinates: Vec<f64> = value
        .split(',')
        .map(|coordinate| coordinate.parse().expect("PRAXIS_PROBE_CAMERA is x,y,z"))
        .collect();
    Some(glam::DVec3::from_slice(&coordinates))
}

pub(crate) fn town_crowd() -> Option<bool> {
    match std::env::var("PRAXIS_PROBE_MOVING").ok()?.as_str() {
        "empty" => Some(false),
        "crowd" => Some(true),
        other => panic!("PRAXIS_PROBE_MOVING={other} is neither empty nor crowd"),
    }
}

pub struct ProbeRayPass {
    pipeline: wgpu::ComputePipeline,
    ray_scenes: [wgpu::BindGroup; 2],
    bind_group: wgpu::BindGroup,
    results: wgpu::Buffer,
    // Metal can run a queued frame's passes one after another unless its ray-query compute pass dispatches indirectly.
    dispatch: wgpu::Buffer,
    switches: ProbeRaySwitches,
    frames: u32,
}

impl ProbeRayPass {
    pub(crate) fn new(
        device: &wgpu::Device,
        static_tlas: &StaticTlas,
        dynamic_tlas: &DynamicTlas,
        buffers: [(&'static str, &wgpu::Buffer); 7],
        switches: ProbeRaySwitches,
        capacities: &RenderCapacities,
    ) -> Self {
        let ray_scene_layout = SHADOW_RAY_SCENE.layout(device);
        let ray_scenes = static_tlas.bindings().map(|static_binding| {
            SHADOW_RAY_SCENE.bind_group(
                device,
                &ray_scene_layout,
                [
                    ("static_tlas", static_binding),
                    ("dynamic_tlas", dynamic_tlas.binding()),
                ],
            )
        });
        let layout = PROBE_RAY_TRACE.layout(device);
        let results = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Probe Ray Results"),
            size: u64::from(PROBES * switches.rays_per_probe) * size_of::<u32>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let dispatch = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Probe Ray Dispatch"),
                contents: wgpu::util::DispatchIndirectArgs {
                    x: PROBES * switches.rays_per_probe / WORKGROUP_SIZE,
                    y: 1,
                    z: 1,
                }
                .as_bytes(),
                usage: wgpu::BufferUsages::INDIRECT,
            },
        );
        let bind_group = PROBE_RAY_TRACE.bind_group(
            device,
            &layout,
            buffers
                .iter()
                .map(|(name, buffer)| (*name, buffer.as_entire_binding()))
                .chain([("probe_ray_results", results.as_entire_binding())]),
        );
        let descriptor = shaders::PROBE_RAYS.descriptor(capacities);
        let shader = if switches.unchecked {
            // SAFETY: every index comes from a hit in a TLAS built over the buffers read, in scenes that free no Instance slot.
            unsafe {
                device.create_shader_module_trusted(
                    descriptor,
                    wgpu::ShaderRuntimeChecks::unchecked(),
                )
            }
        } else {
            device.create_shader_module(descriptor)
        };
        let pipeline_layout = pipeline_layout(
            device,
            "Probe Ray Pipeline Layout",
            &[
                (&SHADOW_RAY_SCENE, &ray_scene_layout),
                (&PROBE_RAY_TRACE, &layout),
            ],
        );
        let constants = [
            ("RAYS_PER_PROBE", f64::from(switches.rays_per_probe)),
            ("RAY_LENGTH_METRES", f64::from(switches.ray_length_metres)),
            ("VERIFY_HITS", f64::from(u8::from(switches.verify_hits))),
        ];
        let pipeline = compute_pipeline(
            device,
            &pipeline_layout,
            &shader,
            "trace_probe_rays",
            &constants,
        );
        tracing::info!(?switches, "probe ray pass on");
        Self {
            pipeline,
            ray_scenes,
            bind_group,
            results,
            dispatch,
            switches,
            frames: 0,
        }
    }

    pub(crate) fn next_frame_traces(&mut self) -> bool {
        let frame = self.frames;
        self.frames += 1;
        (self.switches.block_frames)
            .is_none_or(|block_frames| (frame / block_frames).is_multiple_of(2))
    }

    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        static_tlas_index: usize,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        let mut timestamp_writes = timestamp_writes;
        for _ in 0..self.switches.repeats {
            self.record_once(encoder, static_tlas_index, timestamp_writes.take());
        }
    }

    fn record_once(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        static_tlas_index: usize,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(PROBE_RAY_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(
            SHADOW_RAY_SCENE.index,
            &self.ray_scenes[static_tlas_index],
            &[],
        );
        pass.set_bind_group(PROBE_RAY_TRACE.index, &self.bind_group, &[]);
        if self.switches.direct_dispatch {
            pass.dispatch_workgroups(PROBES * self.switches.rays_per_probe / WORKGROUP_SIZE, 1, 1);
        } else {
            pass.dispatch_workgroups_indirect(&self.dispatch, 0);
        }
    }

    pub(crate) fn time_alone(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        static_tlas_index: usize,
    ) {
        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Probe Ray Alone Queries"),
            ty: wgpu::QueryType::Timestamp,
            count: 2 * ALONE_SUBMISSIONS,
        });
        let buffer = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let timestamp_bytes = 2 * u64::from(ALONE_SUBMISSIONS) * u64::from(wgpu::QUERY_SIZE);
        let resolved = buffer(
            "Probe Ray Alone Resolve",
            timestamp_bytes,
            wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
        );
        let read = buffer(
            "Probe Ray Alone Read",
            timestamp_bytes,
            wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        );
        let wait = || device.poll(wgpu::PollType::wait_indefinitely());
        for submission in 0..ALONE_SUBMISSIONS {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            for repeat in 0..ALONE_REPEATS {
                let (first, last) = (repeat == 0, repeat + 1 == ALONE_REPEATS);
                let timestamp_writes =
                    (first || last).then_some(wgpu::ComputePassTimestampWrites {
                        query_set: &query_set,
                        beginning_of_pass_write_index: first.then_some(2 * submission),
                        end_of_pass_write_index: last.then_some(2 * submission + 1),
                    });
                self.record_once(&mut encoder, static_tlas_index, timestamp_writes);
            }
            queue.submit(std::iter::once(encoder.finish()));
            if let Err(error) = wait() {
                tracing::error!(%error, "probe rays not timed alone");
                return;
            }
        }
        // Metal reads a timestamp resolved before its submission completed as 0 or stale.
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.resolve_query_set(&query_set, 0..2 * ALONE_SUBMISSIONS, &resolved, 0);
        encoder.copy_buffer_to_buffer(&resolved, 0, &read, 0, timestamp_bytes);
        queue.submit(std::iter::once(encoder.finish()));
        read.map_async(wgpu::MapMode::Read, .., |_| {});
        let view = match wait().map(|_| read.get_mapped_range(..)) {
            Ok(Ok(view)) => view,
            _ => {
                tracing::error!("probe rays' times alone not read");
                return;
            }
        };
        let ticks: &[u64] = bytemuck::cast_slice(&view);
        let nanoseconds_per_tick = f64::from(queue.get_timestamp_period());
        let mut microseconds: Vec<f64> = ticks
            .chunks_exact(2)
            .skip(ALONE_WARM_UP_SUBMISSIONS as usize)
            .map(|pair| {
                pair[1].saturating_sub(pair[0]) as f64 * nanoseconds_per_tick
                    / (1e3 * f64::from(ALONE_REPEATS))
            })
            .collect();
        microseconds.sort_by(f64::total_cmp);
        if microseconds[0] <= 0.0 {
            tracing::error!("a submission of probe ray passes read no time");
            return;
        }
        let ranked =
            |fraction: f64| microseconds[(fraction * (microseconds.len() - 1) as f64) as usize];
        let median = ranked(0.5);
        ALONE_NANOSECONDS.store((median * 1e3) as u64, std::sync::atomic::Ordering::Relaxed);
        tracing::info!(
            median_microseconds = median,
            tenth_percentile_microseconds = ranked(0.1),
            ninetieth_percentile_microseconds = ranked(0.9),
            "probe ray pass alone"
        );
    }

    pub(crate) fn log_hits(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Probe Ray Results Read"),
            size: self.results.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_buffer_to_buffer(&self.results, 0, &read, 0, None);
        queue.submit(std::iter::once(encoder.finish()));
        read.map_async(wgpu::MapMode::Read, .., |_| {});
        if let Err(error) = device.poll(wgpu::PollType::wait_indefinitely()) {
            tracing::error!(%error, "probe ray results not read");
            return;
        }
        let Ok(view) = read.get_mapped_range(..) else {
            tracing::error!("probe ray results not mapped");
            return;
        };
        let words: &[u32] = bytemuck::cast_slice(&view);
        let hits = words.iter().filter(|&&word| word & 1 != 0).count();
        let lit = words.iter().filter(|&&word| word & 3 == 3).count();
        let on_their_triangles = words.iter().filter(|&&word| word & 5 == 5).count();
        let hit_metres: u64 = words.iter().map(|&word| u64::from(word >> 16)).sum();
        tracing::info!(
            rays = words.len(),
            hits,
            lit,
            on_their_triangles,
            mean_hit_metres = hit_metres as f64 / hits.max(1) as f64,
            "probe rays"
        );
    }
}
