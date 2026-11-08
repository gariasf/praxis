use std::io::Write;
use std::time::Duration;

use praxis::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, COOKED_CACHE_DIRECTORY, CookedCache,
};
use praxis::gpu::GpuContext;
use praxis::{GpuMemoryKind, ReferenceScene};

const MEBIBYTE: u64 = 1 << 20;

/// town's Instances and their triangles, as its generator places them
/// (`src/scenes/town.rs`).
const TOWN_INSTANCES: u32 = 254_319;
const TOWN_TRIANGLES: u32 = 9_994_316;
const WORLD_INSTANCES: u32 = 164_912;
const WORLD_TRIANGLES: u32 = 1_645_574;

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
    // Every frame writes the 144-byte camera and 32-byte light uniforms and
    // the sun and four point lights, 48 bytes each; only the first also
    // writes the ten 128-byte Instances and the 4-byte count of Instance
    // slots in use, and streams the Chunks: the sphere Mesh's 76,720 bytes
    // and ten 80-byte Materials.
    let streamed_bytes = 76_720 + 10 * 80;
    assert_eq!(
        frames[0].uploaded_bytes,
        10 * 128 + 144 + 32 + 5 * 48 + 4 + streamed_bytes
    );
    assert_eq!(last.uploaded_bytes, 144 + 32 + 5 * 48);
    // The sphere's triangles: 48 × 24 quads of two triangles but one at each
    // pole.
    let triangles = 48 * 24 * 2 - 2 * 48;
    // Its BLAS, of the 25 rings of 49 vertices less the two the cook's weld
    // drops (below), at the adapter's compacted fraction of the bytes its
    // driver gives, rounded to a whole byte; the two static TLASes' 2^18
    // Instances each and the dynamic TLAS's 10,000 at theirs.
    let input = context.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("Frame Stats Size Query Input"),
        size: 64,
        usage: wgpu::BufferUsages::TLAS_INPUT,
        mapped_at_creation: false,
    });
    let sphere_blas_bytes = praxis::blas_bytes(
        context.device(),
        &input,
        wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE
            | wgpu::AccelerationStructureFlags::ALLOW_COMPACTION,
        &wgpu::BlasTriangleGeometrySizeDescriptor {
            vertex_format: wgpu::VertexFormat::Float32x3,
            vertex_count: 25 * 49 - 2,
            index_format: Some(wgpu::IndexFormat::Uint32),
            index_count: Some(triangles as u32 * 3),
            flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
        },
    );
    let tlas_bytes = |max_instances| {
        praxis::tlas_bytes(
            context.device(),
            &input,
            &wgpu::CreateTlasDescriptor {
                label: None,
                max_instances,
                flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            },
        )
    };
    let compacted_blas_fraction = praxis::compacted_blas_fraction(&context.adapter_info());
    let acceleration_structure_bytes = (sphere_blas_bytes as f64 * compacted_blas_fraction).round()
        as u64
        + 2 * tlas_bytes(1 << 18)
        + tlas_bytes(10_000);
    // GPU memory by kind. The geometry Arenas: the Mesh pool's five of
    // 16 MiB. The Resident tails: five 1×1 neutral Textures, one per
    // Channel (the spheres sample no other), RGBA8 but the normal Channel's
    // RGBA16F, each a mip of 64 KiB or less; so nothing in the texture pool.
    let geometry_arenas = 5 * 16 * MEBIBYTE;
    let resident_tails = 4 * 4 + 8;
    // The render targets: the 160×90 Depth32Float texture at 4 samples, its
    // 4x RGBA16F colour, which Apple GPUs keep in tile memory, so it counts
    // nothing on Metal, the RGBA16F HDR target it resolves into, the
    // R32Float depth pyramid from 80×45 down to 1×1 (80×45, 40×22, 20×11,
    // 10×5, 5×2, 2×1, 1×1), the shadow mask of two u32s a pixel for each of
    // two surfaces, a u32 a pixel of the samples that show the second, the
    // list of second surfaces, a u32 a pixel after its count, the
    // directional verdicts, a u32 a pixel, the list of shadow edges, as the
    // second surfaces', and the surface normals' 4 bytes at each of 4
    // samples a pixel.
    let multisampled_color = match context.backend() {
        wgpu::Backend::Metal => 0,
        _ => 160 * 90 * 4 * 8,
    };
    let render_targets = 160 * 90 * 4 * 4
        + multisampled_color
        + 160 * 90 * 8
        + (3600 + 880 + 220 + 50 + 10 + 2 + 1) * 4
        + 2 * 160 * 90 * 8
        + 160 * 90 * 4
        + (160 * 90 + 1) * 4
        + 160 * 90 * 4
        + (160 * 90 + 1) * 4
        + 160 * 90 * 4 * 4;
    // Staging: three slots of the first frame's 1700 uploaded bytes rounded
    // up to 2048. The load's staging is gone.
    let staging = 3 * 2048;
    // The scene and pass buffers: the Mesh table of 65,536 Meshes of 40
    // bytes, 256 Materials of 80 bytes, the camera and light uniforms, the
    // light buffer of 4,096 lights of 48 bytes, 524,288 Instances of 128
    // bytes, the culling pass's 4-byte Instance slot count and 8-byte
    // offsets for each slot and block of 256 slots and of the phase's
    // records, and visibility bit for each slot, the frame's draw of 2^20
    // Meshlet records of 12 bytes, 2^25 packed indices of 4 bytes and 72
    // bytes of both phases' indirect arguments, counts, phase two's offsets
    // and the phase's dispatch, and its 72-byte stand-in, the shadow mask
    // pass's 4-byte overflow count and its two 12-byte dispatches and
    // stand-in, the tonemapping pass's 64×64 R8 dither mask, the Froxel
    // pass's lists, 32 × 18 × 32 of 128 u32s, and its 16 bytes of light
    // counts, the dynamic TLAS's markers' 4-byte result, and the bend pass's
    // 4,096 bends of 20 bytes.
    let scene_and_pass_buffers = 65_536 * 40
        + 256 * 80
        + 144
        + 32
        + 4096 * 48
        + 524_288 * 128
        + 4
        + (524_288 + 2048 + 1) * 8
        + 524_288 / 8
        + (1 << 20) * 12
        + (1 << 25) * 4
        + 72
        + 72
        + 4
        + 12
        + 12
        + 12
        + 64 * 64
        + 32 * 18 * 32 * 128 * 4
        + 16
        + 4
        + 4096 * 20;
    let by_kind = [
        (GpuMemoryKind::GeometryArenas, geometry_arenas),
        (GpuMemoryKind::TexturePool, 0),
        (GpuMemoryKind::ResidentTails, resident_tails),
        (GpuMemoryKind::RenderTargets, render_targets),
        (
            GpuMemoryKind::AccelerationStructures,
            acceleration_structure_bytes,
        ),
        (GpuMemoryKind::Staging, staging),
        (GpuMemoryKind::SceneAndPassBuffers, scene_and_pass_buffers),
    ];
    for (kind, bytes) in by_kind {
        assert_eq!(
            last.gpu_memory_bytes_by_kind[kind as usize],
            bytes,
            "{}",
            kind.name()
        );
    }
    assert_eq!(
        last.gpu_memory_bytes,
        by_kind.iter().map(|&(_, bytes)| bytes).sum::<u64>()
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
        "dynamic_tlas",
        "encode",
        "dynamic_tlas_submit",
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
    // its passes; nothing moves and nothing bends, so no frame builds the
    // dynamic TLAS, runs the bend pass or refits BLASes, which take no time.
    // Phase two's depth pass draws nothing in a frame whose phase one drew
    // every visible Instance, and NVIDIA times an empty pass at 0.
    for (frame, stats) in frames.iter().enumerate() {
        assert_eq!(stats.gpu_readbacks_skipped, 0, "frame {frame}");
        let Some(gpu) = stats.gpu else {
            panic!("frame {frame} read back no GPU stats");
        };
        let passes: Vec<&str> = gpu.pass_times.iter().map(|&(pass, _)| pass).collect();
        assert_eq!(
            passes,
            [
                "dynamic_tlas_build",
                "bend_pass",
                "blas_refit",
                "culling_phase_one",
                "froxel_pass",
                "depth_prepass",
                "depth_pyramid_pass",
                "culling_phase_two",
                "phase_two_depth_pass",
                "shadow_mask_pass",
                "lit_pass",
                "tonemapping_pass"
            ],
            "frame {frame}"
        );
        for (pass, time) in gpu.pass_times {
            let runs = !["dynamic_tlas_build", "bend_pass", "blas_refit"].contains(&pass);
            let may_be_empty = pass == "phase_two_depth_pass" && gpu.phase_two_triangles_drawn == 0;
            assert!(
                (Duration::ZERO < time || may_be_empty) == runs
                    && time <= gpu.frame_time
                    // Under a second from frame 1 on: frame 0 can time
                    // first-use work such as shader compilation. On lavapipe
                    // it read 5.8 s, its compute passes seconds each, in the
                    // first of twenty runs on a fresh CI runner, and under
                    // 0.2 s in the other nineteen. A wrong tick period would
                    // misread every frame alike.
                    && (frame == 0 || gpu.frame_time < Duration::from_secs(1)),
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
/// far-coplanar's tiles, town and the world's settlement lie partly out of
/// any one view, so all three are drawn with culling off, which the first
/// two's Goldens compare with culling on: the frame's draw holds every
/// tile, the whole town and every Instance within the world's Load radius.
/// After the
/// first frame, each frame writes the Instances of what moves alone, town's
/// 10,000 birds among them, and leaves no Moving Instance out of the
/// dynamic TLAS; town's birds fill it, each uploading its entry every
/// frame.
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

    // A sphere: 48 × 24 quads of two triangles but one at each pole; a crowd
    // figure, 40 × 65 of them. A ground plane or a quad: two triangles.
    // SciFiHelmet's and NormalTangentTest's cooked Meshes: 70,074 and
    // 23,322 indices.
    let (sphere_triangles, plane_triangles) = (48 * 24 * 2 - 2 * 48, 2);
    let figure_triangles = 40 * 65 * 2 - 2 * 40;
    let (helmet_triangles, normal_tangent_triangles) = (23_358, 7_774);
    for scene in ReferenceScene::ALL {
        if scene == ReferenceScene::Helmet && !std::path::Path::new(praxis::DEFAULT_MODEL).exists()
        {
            writeln!(std::io::stderr(), "skipped helmet: assets not fetched").ok();
            continue;
        }
        let switches = praxis::RenderSwitches {
            culling: !matches!(
                scene,
                ReferenceScene::Town | ReferenceScene::FarCoplanar | ReferenceScene::World
            ),
            ..praxis::RenderSwitches::default()
        };
        let (_, frames) = praxis::capture(&context, scene, glam::DVec3::ZERO, 160, 90, switches)
            .expect("capture failed");
        let (instances, triangles, moving) = match scene {
            // Ten sweep helmets and the textured one.
            ReferenceScene::Helmet => (11, 11 * helmet_triangles, 0),
            ReferenceScene::PbrSweep => (10, 10 * sphere_triangles, 0),
            ReferenceScene::UvGrid | ReferenceScene::Minification => (1, plane_triangles, 0),
            ReferenceScene::NormalTangent => (1, normal_tangent_triangles, 0),
            // Two planes of 32 × 32 tiles.
            ReferenceScene::FarCoplanar => (2_048, 2_048 * plane_triangles, 0),
            ReferenceScene::Orrery => (5, 5 * sphere_triangles, 5),
            ReferenceScene::TextureWall => (100, 100 * plane_triangles, 0),
            // One Mesh of a ground plane and a sphere.
            ReferenceScene::Shadows => (1, plane_triangles + sphere_triangles, 0),
            // The ground and 100 figures; their shadow proxies, 16 a figure,
            // move with the sway but are not drawn.
            ReferenceScene::Crowd => (101, plane_triangles + 100 * figure_triangles, 1_600),
            // Every Instance its generator places, and their triangles; its
            // birds circle.
            ReferenceScene::Town => (TOWN_INSTANCES, TOWN_TRIANGLES, 10_000),
            // Every Instance of the Chunks within its Load radius of the
            // camera, as its generator places them, and their triangles.
            ReferenceScene::World => (WORLD_INSTANCES, WORLD_TRIANGLES, 0),
        };
        // The 144-byte camera and 32-byte light uniforms, the sun and the
        // 1,024 lamps' lights of 48 bytes, and each bird's 128-byte Instance
        // and its dynamic TLAS entry, as its backend's instance record.
        let entry_bytes = match context.backend() {
            wgpu::Backend::Metal => 72,
            _ => 64,
        };
        let town_upload = 144 + 32 + 1025 * 48 + 10_000 * (128 + entry_bytes);
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
                (instances, triangles, 0),
                "{} frame {frame}",
                scene.name()
            );
            if frame > 0 {
                assert_eq!(
                    (stats.instances_written, stats.moving_instances_left_out),
                    (moving, 0),
                    "{} frame {frame}: Instances written and Moving Instances left out",
                    scene.name()
                );
                if scene == ReferenceScene::Town {
                    assert_eq!(stats.uploaded_bytes, town_upload, "town frame {frame}");
                }
            }
        }
    }
}

/// The orrery's five spheres are Moving Instances: every frame writes their
/// Instances and uploads their dynamic TLAS entries, each as its backend's
/// instance record, which wgpu-core converts it to, and leaves none of them
/// out.
#[test]
fn the_orrerys_moving_instances_upload_their_dynamic_tlas_entries_every_frame() {
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
        ReferenceScene::Orrery,
        glam::DVec3::ZERO,
        160,
        90,
        praxis::RenderSwitches::default(),
    )
    .expect("capture failed");
    let last = frames.last().expect("no frame stats");
    // `MTLIndirectAccelerationStructureInstanceDescriptor`, or
    // `VkAccelerationStructureInstanceKHR`.
    let entry_bytes = match context.backend() {
        wgpu::Backend::Metal => 72,
        _ => 64,
    };
    assert_eq!(last.moving_instances_left_out, 0);
    // Every frame builds it: its entries are uploaded, its recording and its
    // submit are timed on the CPU, and its build on the GPU, within the
    // frame. The build's time starts at the end of the frame before, so a
    // build that finishes while that frame's passes still run reads 0, as
    // the Baseline's does.
    for (frame, stats) in frames.iter().enumerate() {
        // The 144-byte camera and 32-byte light uniforms, the sun and four
        // point lights, 48 bytes each, the five 128-byte Instances and their
        // entries, and on the first frame the 4-byte Instance slot count and
        // the grid Texture's move to its whole chain's class: its Resident tail, mips
        // 3 on of 2,048², in 16-byte blocks, 64², 32², … 2² of them, then 1
        // at each of the last three mips.
        let first_frame = if frame == 0 {
            4 + 16 * (4096 + 1024 + 256 + 64 + 16 + 4 + 3)
        } else {
            0
        };
        assert_eq!(
            stats.uploaded_bytes,
            144 + 32 + 5 * 48 + 5 * 128 + 5 * entry_bytes + first_frame,
            "frame {frame}"
        );
        let timed: Vec<&str> = stats
            .cpu_times
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        for stage in ["dynamic_tlas", "dynamic_tlas_submit"] {
            assert!(
                timed.contains(&stage),
                "frame {frame}: no CPU time for {stage}"
            );
        }
        let Some(gpu) = stats.gpu else {
            panic!("frame {frame} read back no GPU stats");
        };
        let (pass, build) = gpu.pass_times[0];
        assert_eq!(pass, "dynamic_tlas_build", "frame {frame}");
        assert!(
            build <= gpu.frame_time,
            "frame {frame}: the build took {build:?} of {:?}",
            gpu.frame_time
        );
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

/// The pbr sweep loads its Chunk in its first frame, whose static Instances
/// the static TLAS's worker builds into the spare, which a capture waits
/// for: that frame submits the build and reports the worker's time on it,
/// and the next binds it, a frame after the Chunk loaded. No other frame
/// builds or binds one.
#[test]
fn a_chunk_loaded_in_the_first_frame_is_traced_from_the_second() {
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
    let built: Vec<usize> = (0..frames.len())
        .filter(|&frame| !frames[frame].static_tlas_build_time.is_zero())
        .collect();
    assert_eq!(built, [0]);
    let traced: Vec<(usize, u32)> = frames
        .iter()
        .enumerate()
        .filter_map(|(frame, stats)| Some((frame, stats.frames_until_traced?)))
        .collect();
    assert_eq!(traced, [(1, 1)]);
}
