use std::time::Duration;

use crate::camera::Camera;
use crate::gpu::GpuContext;
use crate::render::{FRAMES_IN_FLIGHT, RenderSwitches};
use crate::scenes::ReferenceScene;
use crate::state::State;
use crate::stats::FrameStats;

/// 8-bit UNORM like the window's surface: the tonemapping pass writes both
/// sRGB-encoded. RGBA order makes the read-back bytes the image's pixels.
pub(crate) const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Where the Goldens live, relative to the repository root: one directory
/// per `golden_key`, one PNG per scene.
pub const GOLDEN_DIRECTORY: &str = "tests/goldens";

/// The Golden directory's name for an adapter: its backend, plus `-cpu` on
/// a software rasterizer, which rounds differently from hardware on the
/// same API.
pub fn golden_key(backend: wgpu::Backend, device_type: wgpu::DeviceType) -> String {
    match device_type {
        wgpu::DeviceType::Cpu => format!("{}-cpu", backend.to_str()),
        _ => backend.to_str().to_owned(),
    }
}

/// The largest channel difference, in 8-bit steps, that still matches.
/// Measured noise on the M4 Pro (Metal) and on CI's lavapipe: 20 captures of
/// each of the first two Golden scenes, `pbr-sweep` and `uv-grid`, each in
/// its own process, were byte-identical, so the noise is 0.
/// One step above it absorbs a driver rounding a value on the other side of
/// .5; any larger difference in any pixel fails.
pub const GOLDEN_TOLERANCE: u8 = 1;

/// How two captures of equal size differ.
#[derive(Debug, PartialEq)]
pub struct ImageDifference {
    /// Pixels with a channel that differs by more than `GOLDEN_TOLERANCE`.
    pub differing_pixels: usize,
    pub max_channel_delta: u8,
}

/// Compares two RGBA8 images of equal size channel by channel.
pub fn compare_images(expected: &[u8], actual: &[u8]) -> ImageDifference {
    assert_eq!(expected.len(), actual.len(), "images differ in size");
    let mut difference = ImageDifference {
        differing_pixels: 0,
        max_channel_delta: 0,
    };
    for (expected_pixel, actual_pixel) in expected
        .as_chunks::<4>()
        .0
        .iter()
        .zip(actual.as_chunks::<4>().0)
    {
        let pixel_delta = channel_delta(expected_pixel, actual_pixel);
        difference.max_channel_delta = difference.max_channel_delta.max(pixel_delta);
        if pixel_delta > GOLDEN_TOLERANCE {
            difference.differing_pixels += 1;
        }
    }
    difference
}

/// `actual`, an RGBA8 image the size of `expected`, dimmed to a third, with
/// every pixel beyond `GOLDEN_TOLERANCE` in opaque red.
pub fn difference_image(expected: &[u8], actual: &[u8]) -> Vec<u8> {
    assert_eq!(expected.len(), actual.len(), "images differ in size");
    expected
        .as_chunks::<4>()
        .0
        .iter()
        .zip(actual.as_chunks::<4>().0)
        .flat_map(|(expected_pixel, actual_pixel)| {
            if channel_delta(expected_pixel, actual_pixel) > GOLDEN_TOLERANCE {
                [255, 0, 0, 255]
            } else {
                [
                    actual_pixel[0] / 3,
                    actual_pixel[1] / 3,
                    actual_pixel[2] / 3,
                    255,
                ]
            }
        })
        .collect()
}

fn channel_delta(expected: &[u8], actual: &[u8]) -> u8 {
    expected
        .iter()
        .zip(actual)
        .map(|(expected_channel, actual_channel)| expected_channel.abs_diff(*actual_channel))
        .max()
        .unwrap_or(0)
}

/// Frames rendered per capture; the last one is read back.
pub(crate) const FRAME_COUNT: u32 = 60;

/// Renders `scene`, moved with its camera by `shift`, into a `width` ×
/// `height` offscreen texture for `FRAME_COUNT` frames, as `switches` set,
/// and returns the last one's RGBA8 pixels, rows top first, with every
/// frame's stats, its GPU stats included.
/// Time and camera are injected (the scene's frame delta and its camera path
/// by frame) and nothing reads the wall clock or input, so equal arguments
/// give equal images on one machine.
pub fn capture(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    width: u32,
    height: u32,
    switches: RenderSwitches,
) -> anyhow::Result<(Vec<u8>, Vec<FrameStats>)> {
    let last_frame = FRAME_COUNT - 1;
    let (mut pixels, frame_stats, _) = render_frames(
        context,
        scene,
        shift,
        &|frame| scene.camera_at(frame, shift),
        width,
        height,
        0..FRAME_COUNT,
        &[last_frame],
        scene.frame_delta(),
        RenderSwitches {
            wait_for_static_tlas: true,
            ..switches
        },
        FrameWait::Block,
        ..,
    )?;
    Ok((pixels.remove(0), frame_stats))
}

/// The pixels of each of `frames`, in ascending order, of `scene` rendered
/// as `capture` renders its last frame. Only frame 0, whose camera anchors
/// the static TLAS, and each of `frames` and the frame before it are drawn,
/// so each frame's culling reads the depth of the frame before, as in an
/// unbroken run; the World runs through the others undrawn, so a frame far
/// along the camera path costs its Ticks, not its draws.
pub fn capture_frames(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    width: u32,
    height: u32,
    frames: &[u32],
    switches: RenderSwitches,
) -> anyhow::Result<Vec<Vec<u8>>> {
    anyhow::ensure!(
        frames.is_sorted_by(|earlier, later| earlier < later),
        "frames {frames:?} are not in ascending order"
    );
    let mut drawn: Vec<u32> = std::iter::once(0)
        .chain(
            frames
                .iter()
                .flat_map(|&frame| [frame.saturating_sub(1), frame]),
        )
        .collect();
    drawn.dedup();
    let (pixels, _, _) = render_frames(
        context,
        scene,
        shift,
        &|frame| scene.camera_at(frame, shift),
        width,
        height,
        drawn,
        frames,
        scene.frame_delta(),
        RenderSwitches {
            wait_for_static_tlas: true,
            ..switches
        },
        FrameWait::Block,
        ..,
    )?;
    Ok(pixels)
}

/// Renders `scene` as `capture` does, with its camera at `camera_at` each
/// frame of its path, drawing each of `drawn`, in ascending order, after the
/// World runs undrawn through the frames before it, each frame advancing by
/// `frame_delta`. While the Chunk streamer holds after the camera arrives,
/// the path pauses: its frame is drawn again until the hold ends. Returns
/// the pixels of each of `read_back`, which `drawn` holds, as the hold left
/// them, with the stats of each frame rendered, held ones included, and the
/// scene's content hash, from the Blobs it loaded. Up to `FRAMES_IN_FLIGHT`
/// frames are in flight, as a window's swapchain allows: the GPU stays busy,
/// and the CPU never runs further ahead than a window would. The CPU waits
/// for the oldest frame as `wait` says, and on each drawn frame `settled`
/// holds for the Chunk streamer to report it Settled; a load that fails on
/// such a frame fails the render. A dev launch cooks the Chunks that the
/// other drawn frames reach before the first frame.
#[expect(clippy::too_many_arguments)]
pub(crate) fn render_frames(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    camera_at: &dyn Fn(u32) -> Camera,
    width: u32,
    height: u32,
    drawn: impl IntoIterator<Item = u32>,
    read_back: &[u32],
    frame_delta: Duration,
    switches: RenderSwitches,
    wait: FrameWait,
    settled: impl std::ops::RangeBounds<u32>,
) -> anyhow::Result<(Vec<Vec<u8>>, Vec<FrameStats>, String)> {
    let max_dimension = context.device.limits().max_texture_dimension_2d;
    anyhow::ensure!(
        (1..=max_dimension).contains(&width) && (1..=max_dimension).contains(&height),
        "capture size {width}x{height} is outside 1 to {max_dimension} per side"
    );

    let mut state = State::new(
        context,
        TARGET_FORMAT,
        width,
        height,
        scene,
        shift,
        switches,
    )?;
    let target = context.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Capture Target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TARGET_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    let mut pixels = Vec::with_capacity(read_back.len());
    let drawn: Vec<u32> = drawn.into_iter().collect();
    let streamed = drawn.iter().filter(|&&frame| !settled.contains(&frame));
    (state.streaming).cook_ahead(streamed.map(|&frame| camera_at(frame).position))?;
    let mut frame_stats = Vec::with_capacity(drawn.len());
    let mut in_flight = std::collections::VecDeque::with_capacity(FRAMES_IN_FLIGHT);
    let mut next_frame = 0;
    for frame in drawn {
        for undrawn in next_frame..frame {
            place_camera(&mut state.simulation.world, camera_at(undrawn));
            state.simulation.simulate(frame_delta);
        }
        next_frame = frame + 1;
        place_camera(&mut state.simulation.world, camera_at(frame));
        for held_frames in 0.. {
            anyhow::ensure!(
                held_frames < MAX_HELD_FRAMES,
                "the Chunk streamer held frame {frame} of the camera path for {MAX_HELD_FRAMES} frames"
            );
            if in_flight.len() == FRAMES_IN_FLIGHT {
                let oldest = in_flight.pop_front();
                let timeout = match wait {
                    FrameWait::Block => None,
                    FrameWait::Spin => Some(Duration::ZERO),
                };
                loop {
                    match context.device.poll(wgpu::PollType::Wait {
                        submission_index: oldest.clone(),
                        timeout,
                    }) {
                        Ok(_) => break,
                        Err(wgpu::PollError::Timeout) => std::hint::spin_loop(),
                        Err(error) => return Err(error.into()),
                    }
                }
            }
            if settled.contains(&frame) {
                state.update_settled(frame_delta)?;
            } else {
                state.update(frame_delta);
            }
            in_flight.push_back(state.render(&target_view));
            frame_stats.push(state.frame_stats());
            add_gpu_stats(&mut frame_stats, &state);
            if !frame_stats.last().is_some_and(|stats| stats.holding) {
                break;
            }
        }
        if read_back.contains(&frame) {
            pixels.push(self::read_back(context, &target)?);
        }
    }
    anyhow::ensure!(
        pixels.len() == read_back.len(),
        "frames {read_back:?} to read back are not all drawn"
    );
    state.flush_gpu_frame_stats()?;
    add_gpu_stats(&mut frame_stats, &state);
    Ok((pixels, frame_stats, state.scene_content_hash()))
}

/// The frames a hold may take, a minute at 60 Hz, before the render fails.
const MAX_HELD_FRAMES: u32 = 3600;

/// Places `camera` in `world`, still teleported if the camera it replaces
/// teleported on a frame Chunk streaming has not run since.
fn place_camera(world: &mut bevy_ecs::world::World, mut camera: Camera) {
    camera.teleported |= world.resource::<Camera>().teleported;
    world.insert_resource(camera);
}

/// How `render_frames` waits for the oldest frame in flight.
#[derive(Clone, Copy)]
pub(crate) enum FrameWait {
    /// Blocks, as a window does in `get_current_texture`.
    Block,
    /// Polls until the frame is done. Behind a GPU-bound frame, the M4 Pro
    /// runs a main thread that blocks markedly slower for the whole next
    /// frame (measured: `docs/performance.md`, Measurements behind the
    /// code), so its CPU times would measure the clock, not the work. Apple
    /// asks games not to busy-wait, which holds a P-core and spends thermal
    /// headroom [ref:apple-tech-talk-110147-busy-waits], so only the bench
    /// spins, on a hardware adapter.
    Spin,
}

/// Gives each frame of `frame_stats`, numbered from 0 as `state`'s drawn
/// frames are, the GPU stats `state` read back for it.
fn add_gpu_stats(frame_stats: &mut [FrameStats], state: &State) {
    for gpu in state.gpu_frame_stats() {
        frame_stats[gpu.frame as usize].gpu = Some(*gpu);
    }
}

/// Copies an RGBA8 `texture`'s pixels to the CPU, rows top first, without the
/// row padding the copy needs.
pub(crate) fn read_back(context: &GpuContext, texture: &wgpu::Texture) -> anyhow::Result<Vec<u8>> {
    let (width, height) = (texture.width(), texture.height());
    let row_bytes = width * 4;
    let padded_row_bytes = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Capture Read-back Buffer"),
        size: u64::from(padded_row_bytes) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Capture Read-back Encoder"),
        });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: None,
            },
        },
        texture.size(),
    );
    submit_and_map(context, encoder, &buffer)?;

    let mut pixels = Vec::with_capacity((row_bytes * height) as usize);
    for row in buffer
        .get_mapped_range(..)?
        .chunks_exact(padded_row_bytes as usize)
    {
        pixels.extend_from_slice(&row[..row_bytes as usize]);
    }
    Ok(pixels)
}

/// Submits `encoder` and waits until `buffer`, which it writes, is mapped for
/// reading.
pub(crate) fn submit_and_map(
    context: &GpuContext,
    encoder: wgpu::CommandEncoder,
    buffer: &wgpu::Buffer,
) -> anyhow::Result<()> {
    context.queue.submit(std::iter::once(encoder.finish()));
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        sender.send(result).ok();
    });
    context.device.poll(wgpu::PollType::wait_indefinitely())?;
    receiver.recv()??;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    /// A frame allocates as often whenever the GPU finished the frames
    /// before it. wgpu retires a finished submission, and maps the buffers
    /// that waited on it, in the first submission or poll that finds it
    /// finished, which may fall in a frame or between two. The orrery,
    /// whose Moving Instances build the dynamic TLAS every frame, renders
    /// twice: once with the GPU's work retired by a poll between frames,
    /// once with it left finished for the next frame to retire, and once left
    /// finished with each frame but two after the warm-up pausing before its
    /// last submission, as when the CPU runs other threads mid-frame, so the
    /// GPU finishes the frame's dynamic TLAS build before that submission
    /// retires it.
    /// The dynamic TLAS build's own allocations are left out of the
    /// comparison: on Vulkan and DX12 its allocator grows a map in a frame
    /// its random hash seed picks, whenever the GPU finished.
    /// A run whose GPU had not finished a frame when the next began, as when
    /// other work holds the GPU, is not left finished, so it runs again
    /// with the wait doubled.
    #[test]
    fn a_frames_allocations_do_not_depend_on_when_the_gpu_finished() {
        const FRAMES: u32 = 24;
        const WARM_UP_FRAMES: usize = 8;
        // Far longer than a hardware GPU takes to render a frame of 64 × 64.
        const GPU_FINISH_WAIT: Duration = Duration::from_millis(100);
        // Up to 1.6 s a frame.
        const LEFT_FINISHED_ATTEMPTS: u32 = 5;
        // Far longer than a hardware GPU takes to build the orrery's dynamic TLAS.
        const SUBMIT_PAUSE: Duration = Duration::from_millis(20);
        // Unpaused after paused frames, the next frame's encoders swap roles anew.
        const UNPAUSED_FRAMES: std::ops::Range<u32> =
            WARM_UP_FRAMES as u32 + 4..WARM_UP_FRAMES as u32 + 6;
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        // A software rasterizer shares the test's CPU, and on CI's runner
        // can take longer than the longest wait to finish a frame.
        if context.adapter.get_info().device_type == wgpu::DeviceType::Cpu {
            writeln!(std::io::stderr(), "skipped: needs a hardware adapter").ok();
            return;
        }
        let scene = ReferenceScene::Orrery;
        // Each frame's work retired by a poll before the next frame, or, given
        // a wait, left finished for the next frame to retire after that wait;
        // `None` when the next frame did not retire it; with `pausing`, each
        // frame but `UNPAUSED_FRAMES` pauses before its last submission.
        let allocations = |left_finished_after: Option<Duration>, pausing: bool| {
            let mut state = State::new(
                &context,
                TARGET_FORMAT,
                64,
                64,
                scene,
                glam::DVec3::ZERO,
                RenderSwitches::default(),
            )
            .expect("the orrery loads");
            let target = context.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Allocation Test Target"),
                size: wgpu::Extent3d {
                    width: 64,
                    height: 64,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: TARGET_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());
            let mut allocations_per_frame = Vec::new();
            for frame in 0..FRAMES {
                match left_finished_after {
                    None => {
                        let wait = context.device.poll(wgpu::PollType::wait_indefinitely());
                        wait.expect("the frames before finish");
                    }
                    Some(finish_wait) => std::thread::sleep(finish_wait),
                }
                let pause = if pausing && !UNPAUSED_FRAMES.contains(&frame) {
                    SUBMIT_PAUSE
                } else {
                    Duration::ZERO
                };
                state.pause_before_frame_submit(pause);
                state
                    .simulation
                    .world
                    .insert_resource(scene.camera_at(frame, glam::DVec3::ZERO));
                state.update(scene.frame_delta());
                state.render(&view);
                // The frame before was retired by this one's poll or
                // submissions, so the GPU had finished it before they ran.
                if state.frames_completed() < u64::from(frame) {
                    writeln!(
                        std::io::stderr(),
                        "frame {frame} began before the GPU finished the frame before it"
                    )
                    .ok();
                    return None;
                }
                let stats = state.frame_stats();
                allocations_per_frame.push(stats.heap_allocations - stats.tlas_build_allocations);
            }
            Some(allocations_per_frame)
        };

        let retired = allocations(None, false).expect("a poll waits for every frame");
        let left_finished_allocations = |pausing| {
            (0..LEFT_FINISHED_ATTEMPTS)
                .find_map(|attempt| allocations(Some(GPU_FINISH_WAIT * 2u32.pow(attempt)), pausing))
                .expect("the GPU finishes each frame before the next within the longest wait")
        };
        let left_finished = left_finished_allocations(false);
        let paused = left_finished_allocations(true);
        assert_eq!(
            retired[WARM_UP_FRAMES..],
            left_finished[WARM_UP_FRAMES..],
            "heap allocations per frame but the dynamic TLAS build's, retired between frames {retired:?}, left finished {left_finished:?}"
        );
        assert_eq!(
            retired[WARM_UP_FRAMES..],
            paused[WARM_UP_FRAMES..],
            "heap allocations per frame but the dynamic TLAS build's, retired between frames {retired:?}, left finished with pauses before submitting {paused:?}"
        );
    }

    /// The camera jumps 4 km away from the scene's one Chunk, which
    /// unloads, then jumps back: a load that does not wait finishes on a
    /// later frame, so the hold lasts a frame at least.
    #[test]
    fn a_fast_travel_pauses_the_camera_path_until_its_hold_ends() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let scene = ReferenceScene::UvGrid;
        let path_frames_asked = std::cell::RefCell::new(Vec::new());
        let camera_at = |frame: u32| {
            path_frames_asked.borrow_mut().push(frame);
            let camera = scene.camera_at(frame, glam::DVec3::ZERO);
            let away = glam::DVec3::X * if frame == 1 { 4096.0 } else { 0.0 };
            Camera {
                position: camera.position + away,
                teleported: frame == 1 || frame == 2,
                ..camera
            }
        };
        let (_, frames, _) = render_frames(
            &context,
            scene,
            glam::DVec3::ZERO,
            &camera_at,
            64,
            36,
            0..4,
            &[],
            scene.frame_delta(),
            RenderSwitches::default(),
            FrameWait::Block,
            ..1,
        )
        .expect("the scene renders");
        let path_frames_asked = path_frames_asked.into_inner();
        assert_eq!(
            path_frames_asked[path_frames_asked.len() - 4..],
            [0, 1, 2, 3]
        );
        let holding: Vec<bool> = frames.iter().map(|stats| stats.holding).collect();
        let held = holding.iter().filter(|&&holding| holding).count();
        assert!(held > 0);
        let mut expected = vec![false; 4 + held];
        expected[2..2 + held].fill(true);
        assert_eq!(holding, expected);
        let holds_ended: Vec<usize> = (frames.iter().enumerate())
            .filter(|(_, stats)| stats.fast_travel_hold.is_some())
            .map(|(frame, _)| frame)
            .collect();
        assert_eq!(holds_ended, [1, 2 + held]);
    }

    /// The camera starts 4 km from the scene's one Chunk and jumps back to
    /// it on an undrawn frame: the next drawn frame still arrives, so it
    /// holds and counts no Late Chunk.
    #[test]
    fn a_fast_travel_on_an_undrawn_frame_arrives_on_the_next_drawn_frame() {
        let Some(context) = crate::assets::pools::tests::gpu_context() else {
            return;
        };
        let scene = ReferenceScene::UvGrid;
        let camera_at = |frame: u32| {
            let camera = scene.camera_at(frame, glam::DVec3::ZERO);
            let away = glam::DVec3::X * if frame == 0 { 4096.0 } else { 0.0 };
            Camera {
                position: camera.position + away,
                teleported: frame == 1,
                ..camera
            }
        };
        let (_, frames, _) = render_frames(
            &context,
            scene,
            glam::DVec3::ZERO,
            &camera_at,
            64,
            36,
            [0, 2, 3],
            &[],
            scene.frame_delta(),
            RenderSwitches::default(),
            FrameWait::Block,
            ..1,
        )
        .expect("the scene renders");
        assert!(frames[1].holding);
        let late_chunks: u32 = frames.iter().map(|stats| stats.late_chunks).sum();
        assert_eq!(late_chunks, 0);
    }

    #[test]
    fn compare_counts_pixels_beyond_tolerance_and_marks_them_red() {
        let expected = [90, 90, 90, 255].repeat(3);
        let mut actual = expected.clone();
        actual[4] = 90 + GOLDEN_TOLERANCE;
        actual[9] = 70;

        assert_eq!(
            compare_images(&expected, &actual),
            ImageDifference {
                differing_pixels: 1,
                max_channel_delta: 20,
            }
        );
        assert_eq!(
            difference_image(&expected, &actual),
            [[30, 30, 30, 255], [30, 30, 30, 255], [255, 0, 0, 255]].concat()
        );
    }
}
