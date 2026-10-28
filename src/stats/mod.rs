use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::render::TIMED_PASSES;

/// What one frame cost.
#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    /// Wall-clock time on the CPU of each ECS system and renderer stage the
    /// frame ran, in run order.
    pub cpu_times: Vec<(String, Duration)>,
    /// Wall-clock time from the start of `update` to the end of `render`.
    pub cpu_frame_time: Duration,
    /// What the GPU measured of the frame, read back a few frames after it:
    /// `None` until then, and for good if its readback was skipped.
    pub gpu: Option<GpuFrameStats>,
    pub draw_calls: u32,
    /// Bytes the frame's uploads wrote to GPU buffers: the Instances it wrote
    /// and the uniforms. Pools upload only while a Scene loads.
    pub uploaded_bytes: u64,
    /// Instances the frame wrote: those of new Renderables and of those whose
    /// World transform or Material changed (ADR 0003).
    pub instances_written: u32,
    /// Renderables the full Instance buffer refused a slot, so not drawn.
    pub renderables_refused: u32,
    /// Lights the light buffer had no room for, so not drawn.
    pub lights_overflowed: u32,
    /// Static Instances the static TLAS leaves out, their slots past its
    /// capacity, so they cast no shadow.
    pub static_instances_left_out: u32,
    /// Moving Instances the dynamic TLAS had no room for, so they cast no
    /// shadow.
    pub moving_instances_left_out: u32,
    /// Bends the bend pass had no room for, so their Meshes kept the shape
    /// they last took.
    pub bends_left_out: u32,
    /// Bytes of the GPU buffers and textures the State created, its Pools
    /// included, summed by the engine on every backend (ADR 0013), and of
    /// its acceleration structures, estimated. The render target and
    /// capture's own buffers are the caller's, and the GPU stats
    /// readback's are left out, since they measure the frame rather than
    /// hold it.
    pub gpu_memory_bytes: u64,
    /// Of `gpu_memory_bytes`, the acceleration structures', estimated from
    /// the bytes measured on the backend, since wgpu exposes no
    /// acceleration-structure size (ADR 0017).
    pub estimated_acceleration_structure_bytes: u64,
    /// Bytes of the Arenas that hold Meshes, which `gpu_memory_bytes`
    /// counts whole whatever they hold.
    pub arena_bytes_in_use: u64,
    /// wgpu's allocator report, a diagnostic only; Metal has none
    /// [ref:wgpu-hal-30-allocator-report-default].
    pub allocator_report_bytes: Option<u64>,
    /// Heap allocations and reallocations on the thread that ran the frame.
    pub heap_allocations: u64,
    /// Simulated time the frame skipped past its Tick cap (ADR 0022): the
    /// game slowed by this much.
    pub dropped_time: Duration,
    /// Readbacks of GPU stats the frame skipped, its own or an earlier
    /// frame's, as their ring slot was not ready and the main thread never
    /// waits on the GPU (ADR 0016).
    pub gpu_readbacks_skipped: u32,
}

/// What the GPU measured of one frame, read back a few frames after it.
#[derive(Clone, Copy, Debug)]
pub struct GpuFrameStats {
    /// The frame's number: how many frames its Renderer rendered before it.
    pub frame: u64,
    /// GPU time of each pass, between timestamps at its start and its end,
    /// in the order the passes ran, after the dynamic TLAS's build, between
    /// the markers around it: no time in a frame that does not build it, as
    /// for the bend pass in a frame that bends nothing.
    pub pass_times: [(&'static str, Duration); TIMED_PASSES.len()],
    /// The dynamic TLAS's build, from the end of the frame before's last
    /// pass, plus the span from the start of the first pass to the end of
    /// the last.
    pub frame_time: Duration,
    /// ADR 0019's gate: from the start of culling phase one to the latest
    /// end of the passes up to the lit pass.
    pub gate_time: Duration,
    /// The Instances the frame's draw held, and their triangles.
    pub instances_drawn: u32,
    pub triangles_drawn: u32,
    /// Of those triangles, phase two's draw's: none where phase one drew
    /// every Instance phase two would find visible.
    pub phase_two_triangles_drawn: u32,
    /// Instances in use that the frame's draw had no room for, so not drawn.
    pub instances_overflowed: u32,
    /// Lights the Froxel lists had no room for, summed over Froxels: each
    /// lights nothing in a Froxel whose list it is missing from.
    pub froxel_lights_overflowed: u32,
    /// Lights that reach a pixel past the shadow mask's bits there, summed
    /// over pixels: each casts no shadow at those pixels.
    pub shadow_mask_lights_overflowed: u32,
    /// With the `count_point_lights` Render switch on, the pixels where the
    /// frame drew a surface at the first sample, and the point lights their
    /// Froxels list and whose range reaches that surface, summed over those
    /// pixels; 0 with it off.
    pub lit_pixels: u32,
    pub listed_point_lights: u32,
    pub reaching_point_lights: u32,
}

/// One line per group, each a label and then `key=value` pairs, so the text
/// is both compact and machine-readable.
impl std::fmt::Display for FrameStats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let microseconds = |time: &Duration| time.as_secs_f64() * 1e6;
        write!(
            formatter,
            "cpu_us frame={:.1}",
            microseconds(&self.cpu_frame_time)
        )?;
        for (name, time) in &self.cpu_times {
            write!(formatter, " {name}={:.1}", microseconds(time))?;
        }
        writeln!(formatter)?;
        if let Some(gpu) = &self.gpu {
            write!(
                formatter,
                "gpu_us frame={:.1}",
                microseconds(&gpu.frame_time)
            )?;
            for (pass, time) in &gpu.pass_times {
                write!(formatter, " {pass}={:.1}", microseconds(time))?;
            }
            writeln!(formatter)?;
        }
        write!(formatter, "counts draw_calls={}", self.draw_calls)?;
        if let Some(gpu) = &self.gpu {
            write!(
                formatter,
                " instances_drawn={} triangles_drawn={} instances_overflowed={} froxel_lights_overflowed={} shadow_mask_lights_overflowed={}",
                gpu.instances_drawn,
                gpu.triangles_drawn,
                gpu.instances_overflowed,
                gpu.froxel_lights_overflowed,
                gpu.shadow_mask_lights_overflowed
            )?;
            // Counted only with the `count_point_lights` Render switch on.
            if gpu.lit_pixels > 0 {
                let per_lit_pixel = |lights: u32| f64::from(lights) / f64::from(gpu.lit_pixels);
                write!(
                    formatter,
                    " lit_pixels={} listed_point_lights_per_lit_pixel={:.2} reaching_point_lights_per_lit_pixel={:.2}",
                    gpu.lit_pixels,
                    per_lit_pixel(gpu.listed_point_lights),
                    per_lit_pixel(gpu.reaching_point_lights)
                )?;
            }
        }
        writeln!(
            formatter,
            " uploaded_bytes={} instances_written={} renderables_refused={} lights_overflowed={} static_instances_left_out={} moving_instances_left_out={} bends_left_out={} heap_allocations={} dropped_us={:.1} gpu_readbacks_skipped={}",
            self.uploaded_bytes,
            self.instances_written,
            self.renderables_refused,
            self.lights_overflowed,
            self.static_instances_left_out,
            self.moving_instances_left_out,
            self.bends_left_out,
            self.heap_allocations,
            microseconds(&self.dropped_time),
            self.gpu_readbacks_skipped
        )?;
        let allocator_report_bytes = self
            .allocator_report_bytes
            .map_or("none".to_owned(), |bytes| bytes.to_string());
        writeln!(
            formatter,
            "gpu_memory bytes={} estimated_acceleration_structure_bytes={} arena_bytes_in_use={} allocator_report_bytes={allocator_report_bytes}",
            self.gpu_memory_bytes,
            self.estimated_acceleration_structure_bytes,
            self.arena_bytes_in_use
        )
    }
}

/// Bytes of `texture`'s mip chain at every sample, as ADR 0013 counts GPU
/// memory. A transient attachment counts 0 where the adapter keeps it in tile
/// memory (memoryless on Apple GPUs).
pub(crate) fn texture_bytes(texture: &wgpu::Texture, transient_saves_memory: bool) -> u64 {
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

/// Measures the frame in progress for `State`, without allocating once the
/// first frames have sized its lists.
pub(crate) struct FrameRecorder {
    pub(crate) systems: SystemTimer,
    started: Instant,
    allocations_at_start: u64,
    stages: Vec<(&'static str, Duration)>,
    frame_time: Duration,
    heap_allocations: u64,
    pub(crate) draw_calls: u32,
    pub(crate) uploaded_bytes: u64,
    pub(crate) instances_written: u32,
    pub(crate) renderables_refused: u32,
    pub(crate) lights_overflowed: u32,
    pub(crate) static_instances_left_out: u32,
    pub(crate) moving_instances_left_out: u32,
    pub(crate) bends_left_out: u32,
    pub(crate) dropped_time: Duration,
    pub(crate) gpu_readbacks_skipped: u32,
}

impl FrameRecorder {
    pub(crate) fn new() -> Self {
        Self {
            systems: SystemTimer::default(),
            started: Instant::now(),
            allocations_at_start: 0,
            stages: Vec::new(),
            frame_time: Duration::ZERO,
            heap_allocations: 0,
            draw_calls: 0,
            uploaded_bytes: 0,
            instances_written: 0,
            renderables_refused: 0,
            lights_overflowed: 0,
            static_instances_left_out: 0,
            moving_instances_left_out: 0,
            bends_left_out: 0,
            dropped_time: Duration::ZERO,
            gpu_readbacks_skipped: 0,
        }
    }

    /// Starts measuring a frame.
    pub(crate) fn begin(&mut self) {
        self.systems.reset();
        self.stages.clear();
        self.draw_calls = 0;
        self.uploaded_bytes = 0;
        self.instances_written = 0;
        self.renderables_refused = 0;
        self.lights_overflowed = 0;
        self.static_instances_left_out = 0;
        self.moving_instances_left_out = 0;
        self.bends_left_out = 0;
        self.dropped_time = Duration::ZERO;
        self.gpu_readbacks_skipped = 0;
        self.allocations_at_start = thread_allocations();
        self.started = Instant::now();
    }

    /// Records the CPU time of `stage`, which began at `started`.
    pub(crate) fn stage(&mut self, stage: &'static str, started: Instant) {
        self.stages.push((stage, started.elapsed()));
    }

    /// Ends the frame `begin` started.
    pub(crate) fn end(&mut self) {
        self.frame_time = self.started.elapsed();
        self.heap_allocations = thread_allocations() - self.allocations_at_start;
    }

    /// The stats of the last frame measured.
    pub(crate) fn stats(&self) -> FrameStats {
        let stage_times = self
            .stages
            .iter()
            .map(|&(stage, time)| (stage.to_owned(), time));
        FrameStats {
            cpu_times: self
                .systems
                .times()
                .into_iter()
                .chain(stage_times)
                .collect(),
            cpu_frame_time: self.frame_time,
            draw_calls: self.draw_calls,
            uploaded_bytes: self.uploaded_bytes,
            instances_written: self.instances_written,
            renderables_refused: self.renderables_refused,
            lights_overflowed: self.lights_overflowed,
            static_instances_left_out: self.static_instances_left_out,
            moving_instances_left_out: self.moving_instances_left_out,
            bends_left_out: self.bends_left_out,
            heap_allocations: self.heap_allocations,
            dropped_time: self.dropped_time,
            gpu_readbacks_skipped: self.gpu_readbacks_skipped,
            ..FrameStats::default()
        }
    }
}

/// CPU time per ECS system, from the span that bevy_ecs's `trace` feature
/// enters around every system run. A span reports to the subscriber that was
/// the default when it was created, so the systems created inside
/// [`Self::scope`] report here, whatever the global subscriber filters.
#[derive(Clone, Default)]
pub(crate) struct SystemTimer(Arc<Mutex<SystemSpans>>);

#[derive(Default)]
struct SystemSpans {
    /// By span ID − 1.
    systems: Vec<SystemSpan>,
    /// The runs since `reset`, in run order: span index and CPU time.
    runs: Vec<(usize, Duration)>,
}

struct SystemSpan {
    name: String,
    entered: Option<Instant>,
}

impl SystemTimer {
    /// Calls `create_systems` with this timer as the default subscriber;
    /// events logged meanwhile are dropped.
    pub(crate) fn scope<T>(&self, create_systems: impl FnOnce() -> T) -> T {
        tracing::subscriber::with_default(self.clone(), create_systems)
    }

    fn spans(&self) -> MutexGuard<'_, SystemSpans> {
        // Only timings are behind the lock; a panic elsewhere leaves them valid.
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn reset(&self) {
        self.spans().runs.clear();
    }

    fn times(&self) -> Vec<(String, Duration)> {
        let spans = self.spans();
        spans
            .runs
            .iter()
            .map(|&(index, time)| (spans.systems[index].name.clone(), time))
            .collect()
    }
}

impl tracing::Subscriber for SystemTimer {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        metadata.name() == "system" && metadata.target().starts_with("bevy_ecs")
    }

    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::INFO)
    }

    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let mut name = SystemName(String::new());
        span.record(&mut name);
        let mut spans = self.spans();
        spans.systems.push(SystemSpan {
            name: name.0,
            entered: None,
        });
        tracing::span::Id::from_u64(spans.systems.len() as u64)
    }

    fn record(&self, _: &tracing::span::Id, _: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _: &tracing::span::Id, _: &tracing::span::Id) {}

    fn event(&self, _: &tracing::Event<'_>) {}

    fn enter(&self, span: &tracing::span::Id) {
        self.spans().systems[span_index(span)].entered = Some(Instant::now());
    }

    fn exit(&self, span: &tracing::span::Id) {
        let mut spans = self.spans();
        let index = span_index(span);
        if let Some(entered) = spans.systems[index].entered.take() {
            spans.runs.push((index, entered.elapsed()));
        }
    }
}

fn span_index(span: &tracing::span::Id) -> usize {
    span.into_u64() as usize - 1
}

/// A system span's `name` field without its module path or generic
/// arguments: `fly_camera`, not `praxis::camera::fly_camera`.
struct SystemName(String);

impl tracing::field::Visit for SystemName {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "name" {
            let path = value.split('<').next().unwrap_or(value);
            self.0 = path.rsplit("::").next().unwrap_or(path).to_owned();
        }
    }

    fn record_debug(&mut self, _: &tracing::field::Field, _: &dyn std::fmt::Debug) {}
}

thread_local! {
    static THREAD_ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

/// Heap allocations and reallocations made so far on this thread.
fn thread_allocations() -> u64 {
    THREAD_ALLOCATIONS.with(Cell::get)
}

/// The system allocator, counting allocations per thread, so a frame's count
/// is the difference between two readings on the thread that runs it.
struct CountingAllocator;

impl CountingAllocator {
    fn count(&self) {
        // `try_with`, as an allocator must not panic. The counter has no
        // destructor, so it is never gone while its thread allocates.
        let _ = THREAD_ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
    }
}

// SAFETY: every method passes its arguments unchanged to `System`, which
// upholds the `GlobalAlloc` contract; counting reads no allocated memory.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.count();
        // SAFETY: the caller meets `alloc`'s contract, which `System` shares.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.count();
        // SAFETY: as for `alloc`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.count();
        // SAFETY: the caller meets `realloc`'s contract, which `System` shares.
        unsafe { System.realloc(pointer, layout, new_size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the caller meets `dealloc`'s contract, which `System` shares.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
