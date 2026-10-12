use std::io::Write;

use praxis::ReferenceScene;
use praxis::gpu::GpuContext;

const MEBIBYTE: u64 = 1 << 20;

/// Frames per capture: one second at 60 Hz.
const FRAME_COUNT: usize = 60;

#[test]
fn capture_reports_the_pbr_sweep_frame_stats() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    let (_, frames) =
        praxis::capture(&context, ReferenceScene::PbrSweep, 160, 90).expect("capture failed");
    assert_eq!(frames.len(), FRAME_COUNT);
    let last = frames.last().expect("no frame stats");

    // Two rows of five spheres, one Primitive each.
    assert_eq!(last.draw_calls, 10);
    // Each frame writes ten 144-byte Instances and the 80-byte camera and
    // 192-byte light uniforms.
    assert_eq!(last.uploaded_bytes, 10 * 144 + 80 + 192);
    // 640 MiB of material arrays (5 Channels of 8 Layers, 2048² RGBA8), the
    // 1 MiB vertex and 256 KiB index buffers, 256 Materials of 80 bytes, the
    // camera and light uniforms, 16 Instances of 144 bytes, and the 160×90
    // Depth32Float texture.
    assert_eq!(
        last.gpu_memory_bytes,
        640 * MEBIBYTE + MEBIBYTE + 256 * 1024 + 256 * 80 + 80 + 192 + 16 * 144 + 160 * 90 * 4
    );

    // wgpu allocates for every command encoder, so a count of 0 would mean
    // the allocator is not counting.
    assert!(last.heap_allocations > 0, "no heap allocations counted");

    let printed = last.to_string();
    assert!(printed.lines().count() <= 5, "over 5 lines:\n{printed}");
}
