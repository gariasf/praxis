use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// What one frame cost.
#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    /// Wall-clock time on the CPU of each ECS system and renderer stage the
    /// frame ran, in run order.
    pub cpu_times: Vec<(String, Duration)>,
    /// Wall-clock time from the start of `update` to the end of `render`.
    pub cpu_frame_time: Duration,
    /// GPU time of each render pass, between timestamps at its boundaries;
    /// empty when the device has no timestamp queries.
    pub gpu_times: Vec<(&'static str, Duration)>,
    pub draw_calls: u32,
    /// Bytes the frame's uploads wrote to GPU buffers: the Instances it wrote
    /// and the uniforms. Pools upload only while a Scene loads.
    pub uploaded_bytes: u64,
    /// Instances the frame wrote: those of new Renderables and of those whose
    /// World transform or Material changed (ADR 0003).
    pub instances_written: u32,
    /// Bytes of the GPU buffers and textures the State created, its Pools
    /// included, summed by the engine on every backend (ADR 0013). The
    /// render target and capture's own buffers are the caller's.
    pub gpu_memory_bytes: u64,
    /// wgpu's allocator report, a diagnostic only; Metal has none.
    pub allocator_report_bytes: Option<u64>,
    /// Heap allocations and reallocations on the thread that ran the frame.
    pub heap_allocations: u64,
    /// Simulated time the frame skipped past its Tick cap (ADR 0022): the
    /// game slowed by this much.
    pub dropped_time: Duration,
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
        if !self.gpu_times.is_empty() {
            write!(formatter, "gpu_us")?;
            for (pass, time) in &self.gpu_times {
                write!(formatter, " {pass}={:.1}", microseconds(time))?;
            }
            writeln!(formatter)?;
        }
        writeln!(
            formatter,
            "counts draw_calls={} uploaded_bytes={} instances_written={} heap_allocations={} dropped_us={:.1}",
            self.draw_calls,
            self.uploaded_bytes,
            self.instances_written,
            self.heap_allocations,
            microseconds(&self.dropped_time)
        )?;
        let allocator_report_bytes = self
            .allocator_report_bytes
            .map_or("none".to_owned(), |bytes| bytes.to_string());
        writeln!(
            formatter,
            "gpu_memory bytes={} allocator_report_bytes={allocator_report_bytes}",
            self.gpu_memory_bytes
        )
    }
}

/// Bytes of `texture`'s mip chain at every sample, as ADR 0013 counts GPU
/// memory. A transient attachment counts 0 where the adapter keeps it in tile
/// memory (memoryless on Apple GPUs; owner decision on #54).
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
    pub(crate) dropped_time: Duration,
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
            dropped_time: Duration::ZERO,
        }
    }

    /// Starts measuring a frame.
    pub(crate) fn begin(&mut self) {
        self.systems.reset();
        self.stages.clear();
        self.draw_calls = 0;
        self.uploaded_bytes = 0;
        self.instances_written = 0;
        self.dropped_time = Duration::ZERO;
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
            heap_allocations: self.heap_allocations,
            dropped_time: self.dropped_time,
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
