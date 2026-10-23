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
        praxis::RenderSwitches::default(),
    )
    .expect("capture failed");
    assert_eq!(frames.len(), FRAME_COUNT);
    let last = frames.last().expect("no frame stats");

    // 60 Hz frames run one or two Ticks each, under the cap.
    assert!(frames.iter().all(|frame| frame.dropped_time.is_zero()));
    // Two rows of five spheres in one indirect draw, which the depth pre-pass
    // draws with one call, and the lit pass with two, phase two's into depth
    // first; then the tonemapping pass's one triangle.
    assert_eq!(last.draw_calls, 4);
    // Every frame writes the 112-byte camera and 32-byte light uniforms and
    // the sun and four point lights, 48 bytes each; only the first also
    // writes the ten 128-byte Instances and the 4-byte count of Instance
    // slots in use.
    assert_eq!(frames[0].uploaded_bytes, 10 * 128 + 112 + 32 + 5 * 48 + 4);
    assert_eq!(last.uploaded_bytes, 112 + 32 + 5 * 48);
    // The sphere's triangles: 48 × 24 quads of two triangles but one at each
    // pole.
    let triangles = 48 * 24 * 2 - 2 * 48;
    // Its compacted BLAS, and the static TLAS's 2^18 Instances, by the bytes
    // measured on this backend: estimates (#202).
    let (compacted_blas_bytes_per_triangle, tlas_bytes_per_instance) = match context.backend() {
        wgpu::Backend::Vulkan => (104.0, 240.0),
        _ => (45.5, 288.5),
    };
    let acceleration_structure_bytes = (triangles as f64 * compacted_blas_bytes_per_triangle
        + (1 << 18) as f64 * tlas_bytes_per_instance)
        .round() as u64;
    assert_eq!(
        last.estimated_acceleration_structure_bytes,
        acceleration_structure_bytes
    );
    // The acceleration structures, five 1×1 RGBA8 neutral Textures, one per
    // Channel (the spheres sample no other), the Mesh pool's five 16 MiB
    // Arenas and its Mesh table of 65,536 Meshes of 40 bytes, 256 Materials
    // of 80 bytes, the camera and light uniforms, the light buffer of 4,096
    // lights of 48 bytes, 524,288 Instances of 128
    // bytes, the culling pass's 4-byte Instance slot count and 8-byte
    // offsets for each slot and block of 256 slots and visibility bit for
    // each slot, the frame's draw of 2^20 Meshlet records of 12 bytes, 2^24
    // packed indices of 4 bytes and 56 bytes of both phases' indirect
    // arguments, counts and phase two's offsets, the 160×90 Depth32Float
    // texture at 4 samples, its 4x RGBA16F colour, which Apple GPUs keep in
    // tile memory, so it counts nothing on Metal, the RGBA16F HDR target it
    // resolves into, the R32Float depth pyramid from 80×45 down to 1×1
    // (80×45, 40×22, 20×11, 10×5, 5×2, 2×1, 1×1), three staging slots of the
    // first frame's 1668 uploaded bytes rounded up to 2048, the GPU stats
    // readback: its resolve buffer of a begin and an end timestamp of 8
    // bytes for each of the seven passes, and six slots of 176 bytes, the
    // tonemapping pass's 64×64 R8 dither mask, and the Froxel pass's lists,
    // 32 × 18 × 32 of 128 u32s, and its 4-byte overflow count. The load's
    // staging is gone.
    let multisampled_color = match context.backend() {
        wgpu::Backend::Metal => 0,
        _ => 160 * 90 * 4 * 8,
    };
    assert_eq!(
        last.gpu_memory_bytes,
        acceleration_structure_bytes
            + 5 * 4
            + 5 * 16 * MEBIBYTE
            + 65_536 * 40
            + 256 * 80
            + 112
            + 32
            + 4096 * 48
            + 524_288 * 128
            + 4
            + (524_288 + 2048) * 8
            + 524_288 / 8
            + (1 << 20) * 12
            + (1 << 24) * 4
            + 56
            + 160 * 90 * 4 * 4
            + multisampled_color
            + 160 * 90 * 8
            + (3600 + 880 + 220 + 50 + 10 + 2 + 1) * 4
            + 3 * 2048
            + 7 * 2 * 8
            + 6 * 176
            + 64 * 64
            + 32 * 18 * 32 * 128 * 4
            + 4
    );
    // One UV sphere in the Arenas: its triangles of three u32s, and 25 rings
    // of 49 vertices of 28 bytes, less the one vertex at each pole that only
    // a dropped triangle used, which the cook's weld drops; and its Meshlets
    // as its Blob holds them, 16 bytes each, with a u32 per triangle and per
    // Meshlet vertex.
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

    // Timestamp queries are in the Capability floor, so every frame read
    // back its passes' GPU time, none skipped: Metal reads a timestamp
    // resolved before its pass's submission completed as 0. The frame spans
    // its passes.
    for (frame, stats) in frames.iter().enumerate() {
        assert_eq!(stats.gpu_readbacks_skipped, 0, "frame {frame}");
        let Some(gpu) = stats.gpu else {
            panic!("frame {frame} read back no GPU stats");
        };
        let passes: Vec<&str> = gpu.pass_times.iter().map(|&(pass, _)| pass).collect();
        assert_eq!(
            passes,
            [
                "culling_phase_one",
                "froxel_pass",
                "depth_prepass",
                "depth_pyramid_pass",
                "culling_phase_two",
                "lit_pass",
                "tonemapping_pass"
            ],
            "frame {frame}"
        );
        for (pass, time) in gpu.pass_times {
            assert!(
                Duration::ZERO < time
                    && time <= gpu.frame_time
                    && gpu.frame_time < Duration::from_secs(1),
                "frame {frame}: {pass} took {time:?} of {:?} on the GPU",
                gpu.frame_time
            );
        }
    }

    // wgpu allocates for every command encoder, so a count of 0 would mean
    // the allocator is not counting.
    assert!(last.heap_allocations > 0, "no heap allocations counted");

    let printed = last.to_string();
    assert!(printed.lines().count() <= 5, "over 5 lines:\n{printed}");
}

/// On every Reference scene, each frame reads back from the GPU the
/// Instances and triangles its draw held, which equal the scene's
/// closed-form counts, none overflowed: every Instance in use is drawn.
#[test]
fn each_reference_scene_draws_its_closed_form_instances_and_triangles() {
    let context = match pollster::block_on(GpuContext::headless()) {
        Ok(context) => context,
        Err(error) if error.is::<wgpu::RequestAdapterError>() => {
            // Not `eprintln!`, which libtest captures from passing tests.
            writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
            return;
        }
        Err(error) => panic!("headless GPU context failed: {error:#}"),
    };

    // A sphere: 48 × 24 quads of two triangles but one at each pole. A
    // ground plane or a quad: two triangles. SciFiHelmet's and
    // NormalTangentTest's cooked Meshes: 70,074 and 23,322 indices.
    let (sphere_triangles, plane_triangles) = (48 * 24 * 2 - 2 * 48, 2);
    let (helmet_triangles, normal_tangent_triangles) = (23_358, 7_774);
    for scene in ReferenceScene::ALL {
        if scene == ReferenceScene::Helmet && !std::path::Path::new(praxis::DEFAULT_MODEL).exists()
        {
            writeln!(std::io::stderr(), "skipped helmet: assets not fetched").ok();
            continue;
        }
        let (_, frames) = praxis::capture(
            &context,
            scene,
            glam::DVec3::ZERO,
            160,
            90,
            praxis::RenderSwitches::default(),
        )
        .expect("capture failed");
        let (instances, triangles) = match scene {
            // Ten sweep helmets and the textured one.
            ReferenceScene::Helmet => (11, helmet_triangles),
            ReferenceScene::PbrSweep => (10, sphere_triangles),
            ReferenceScene::UvGrid | ReferenceScene::Minification => (1, plane_triangles),
            ReferenceScene::NormalTangent => (1, normal_tangent_triangles),
            ReferenceScene::FarCoplanar => (2, plane_triangles),
            ReferenceScene::Orrery => (5, sphere_triangles),
            ReferenceScene::TextureWall => (100, plane_triangles),
            // One Mesh of a ground plane and a sphere.
            ReferenceScene::Shadows => (1, plane_triangles + sphere_triangles),
        };
        for (frame, stats) in frames.iter().enumerate() {
            let Some(gpu) = stats.gpu else {
                panic!("{} frame {frame} read back no GPU stats", scene.name());
            };
            assert_eq!(
                (
                    gpu.instances_drawn,
                    gpu.triangles_drawn,
                    gpu.instances_overflowed
                ),
                (instances, instances * triangles, 0),
                "{} frame {frame}",
                scene.name()
            );
        }
    }
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
        let (_, frames) = praxis::capture(
            &context,
            scene,
            glam::DVec3::ZERO,
            160,
            90,
            praxis::RenderSwitches::default(),
        )
        .expect("capture failed");
        let written: Vec<u32> = frames.iter().map(|frame| frame.instances_written).collect();
        assert_eq!(written[0], instances, "{}", scene.name());
        assert!(
            written[1..].iter().all(|&count| count == 0),
            "{} wrote Instances after its first frame: {written:?}",
            scene.name()
        );
    }
}
