use std::io::Write;
use std::time::Duration;

use praxis::ReferenceScene;
use praxis::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, COOKED_CACHE_DIRECTORY, CookedCache,
};
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
    // Two rows of five spheres in one indirect draw.
    assert_eq!(last.draw_calls, 1);
    // Every frame writes the 96-byte camera and 192-byte light uniforms; only
    // the first also writes the ten 128-byte Instances and the 4-byte count
    // of Instance slots in use.
    assert_eq!(frames[0].uploaded_bytes, 10 * 128 + 96 + 192 + 4);
    assert_eq!(last.uploaded_bytes, 96 + 192);
    // Five 1×1 RGBA8 neutral Textures, one per Channel (the spheres sample
    // no other), the Mesh pool's five 16 MiB Arenas and its Mesh table of
    // 65,536 Meshes of 40 bytes, 256 Materials of 80 bytes, the camera and
    // light uniforms, 524,288 Instances of 128 bytes, the culling pass's
    // 4-byte Instance slot count and 8-byte offsets for each slot and block
    // of 256 slots, the frame's draw of 2^20 Meshlet records of 12 bytes,
    // 2^24 packed indices of 4 bytes and 20 bytes of indirect arguments, the
    // 160×90 Depth32Float texture, and three staging slots of the first
    // frame's 1572 uploaded bytes rounded up to 2048. The load's staging is
    // gone.
    assert_eq!(
        last.gpu_memory_bytes,
        5 * 4
            + 5 * 16 * MEBIBYTE
            + 65_536 * 40
            + 256 * 80
            + 96
            + 192
            + 524_288 * 128
            + 4
            + (524_288 + 2048) * 8
            + (1 << 20) * 12
            + (1 << 24) * 4
            + 20
            + 160 * 90 * 4
            + 3 * 2048
    );
    // One UV sphere in the Arenas: 48 × 24 quads of two triangles but one at
    // each pole, of three u32s, and 25 rings of 49 vertices of 28 bytes, less
    // the one vertex at each pole that only a dropped triangle used, which
    // the cook's weld drops; and its Meshlets as its Blob holds them, 16
    // bytes each, with a u32 per triangle and per Meshlet vertex.
    let triangles = 48 * 24 * 2 - 2 * 48;
    let sphere = CookedCache::new(COOKED_CACHE_DIRECTORY)
        .mesh(AssetId::new(
            AssetSource::Scene(ReferenceScene::PbrSweep.name()),
            AssetKind::Mesh,
            AssetName::Name("sphere"),
        ))
        .expect("capture cooked the sphere");
    let meshlets = || {
        sphere
            .primitives
            .iter()
            .flat_map(|primitive| &primitive.meshlets)
    };
    let meshlet_vertices: usize = meshlets().map(|meshlet| meshlet.vertices.len()).sum();
    assert_eq!(
        last.arena_bytes_in_use,
        (25 * 49 - 2) * 28
            + triangles * 3 * 4
            + meshlets().count() as u64 * 16
            + meshlet_vertices as u64 * 4
            + triangles * 4
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

/// On the Reference scenes whose camera moves every frame, the first frame
/// writes every Instance and the camera's motion writes none after it
/// (ADR 0003).
#[test]
fn camera_motion_writes_no_instances() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    for (scene, instances) in [(ReferenceScene::PbrSweep, 10), (ReferenceScene::UvGrid, 1)] {
        let (_, frames) =
            praxis::capture(&context, scene, glam::DVec3::ZERO, 160, 90).expect("capture failed");
        let written: Vec<u32> = frames.iter().map(|frame| frame.instances_written).collect();
        assert_eq!(written[0], instances, "{}", scene.name());
        assert!(
            written[1..].iter().all(|&count| count == 0),
            "{} wrote Instances after its first frame: {written:?}",
            scene.name()
        );
    }
}
