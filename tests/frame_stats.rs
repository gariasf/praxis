use std::io::Write;
use std::time::Duration;

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

    let (_, frames) = praxis::capture(
        &context,
        ReferenceScene::PbrSweep,
        glam::DVec3::ZERO,
        160,
        90,
    )
    .expect("capture failed");
    assert_eq!(frames.len(), FRAME_COUNT);
    let last = frames.last().expect("no frame stats");

    // 60 Hz frames run one or two Ticks each, under the cap.
    assert!(frames.iter().all(|frame| frame.dropped_time.is_zero()));
    // Two rows of five spheres, one Primitive each.
    assert_eq!(last.draw_calls, 10);
    // Each frame writes ten 128-byte Instances and the 96-byte camera and
    // 192-byte light uniforms.
    assert_eq!(last.uploaded_bytes, 10 * 128 + 96 + 192);
    // 640 MiB of material arrays (5 Channels of 8 Layers, 2048² RGBA8), the
    // 1 MiB vertex and 256 KiB index buffers, 256 Materials of 80 bytes, the
    // camera and light uniforms, 16 Instances of 128 bytes, the 160×90
    // Depth32Float texture, and three staging slots of the frame's 1568
    // uploaded bytes rounded up to 2048.
    assert_eq!(
        last.gpu_memory_bytes,
        640 * MEBIBYTE
            + MEBIBYTE
            + 256 * 1024
            + 256 * 80
            + 96
            + 192
            + 16 * 128
            + 160 * 90 * 4
            + 3 * 2048
    );

    let timed: Vec<&str> = last
        .cpu_times
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    for name in [
        "fly_camera",
        "propagate_world_transforms",
        "clear_just_pressed",
        "prepare_frame",
        "upload",
        "encode",
        "submit",
    ] {
        assert!(timed.contains(&name), "no CPU time for {name}: {timed:?}");
    }
    // The systems and stages run one after another inside the frame.
    let summed: Duration = last.cpu_times.iter().map(|(_, time)| *time).sum();
    assert!(
        summed <= last.cpu_frame_time,
        "{summed:?} of system and stage time in a {:?} frame",
        last.cpu_frame_time
    );

    // Timestamp queries are in the Capability floor, so every frame's pass
    // took GPU time: Metal reads a timestamp resolved before its pass's
    // submission completed as 0.
    for (frame, stats) in frames.iter().enumerate() {
        match stats.gpu_times.as_slice() {
            [("main_pass", pass_time)] => assert!(
                Duration::ZERO < *pass_time && *pass_time < Duration::from_secs(1),
                "frame {frame} took {pass_time:?} on the GPU"
            ),
            other => panic!("frame {frame} has GPU times {other:?}"),
        }
    }

    // wgpu allocates for every command encoder, so a count of 0 would mean
    // the allocator is not counting.
    assert!(last.heap_allocations > 0, "no heap allocations counted");

    let printed = last.to_string();
    assert!(printed.lines().count() <= 5, "over 5 lines:\n{printed}");
}
