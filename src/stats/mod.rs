use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// What one frame cost.
#[derive(Clone, Debug, Default)]
pub struct FrameStats {
    pub draw_calls: u32,
    /// Bytes the frame's uploads wrote to GPU buffers: Instances and uniforms.
    /// Pools upload only while a Scene loads.
    pub uploaded_bytes: u64,
    /// Bytes of the GPU buffers and textures the State created, its Pools
    /// included, summed by the engine on every backend (ADR 0013). The
    /// render target and capture's own buffers are the caller's.
    pub gpu_memory_bytes: u64,
    /// wgpu's allocator report, a diagnostic only; Metal has none.
    pub allocator_report_bytes: Option<u64>,
    /// Heap allocations and reallocations on the thread that ran the frame.
    pub heap_allocations: u64,
}

/// One line per group, each a label and then `key=value` pairs, so the text
/// is both compact and machine-readable.
impl std::fmt::Display for FrameStats {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            formatter,
            "counts draw_calls={} uploaded_bytes={} heap_allocations={}",
            self.draw_calls, self.uploaded_bytes, self.heap_allocations
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

/// Measures the frame in progress for `State`.
#[derive(Default)]
pub(crate) struct FrameRecorder {
    allocations_at_start: u64,
    heap_allocations: u64,
    pub(crate) draw_calls: u32,
    pub(crate) uploaded_bytes: u64,
}

impl FrameRecorder {
    /// Starts measuring a frame.
    pub(crate) fn begin(&mut self) {
        self.draw_calls = 0;
        self.uploaded_bytes = 0;
        self.allocations_at_start = thread_allocations();
    }

    /// Ends the frame `begin` started.
    pub(crate) fn end(&mut self) {
        self.heap_allocations = thread_allocations() - self.allocations_at_start;
    }

    /// The stats of the last frame measured.
    pub(crate) fn stats(&self) -> FrameStats {
        FrameStats {
            draw_calls: self.draw_calls,
            uploaded_bytes: self.uploaded_bytes,
            heap_allocations: self.heap_allocations,
            ..FrameStats::default()
        }
    }
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
