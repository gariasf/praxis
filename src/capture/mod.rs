use std::time::Duration;

use crate::gpu::GpuContext;
use crate::render::MAIN_PASS;
use crate::scenes::ReferenceScene;
use crate::state::State;
use crate::stats::FrameStats;

/// 8-bit sRGB like the window's surface, which takes the first sRGB format
/// the surface offers (`Bgra8UnormSrgb` on Metal). RGBA order makes the
/// read-back bytes the image's pixels.
pub(crate) const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Frames rendered per capture; the last one is read back.
const FRAME_COUNT: u32 = 60;

/// Frames submitted but not finished on the GPU before the next one starts,
/// as a window's swapchain allows: the GPU stays busy, and the CPU never runs
/// further ahead than a window would.
const FRAMES_IN_FLIGHT: usize = 2;

/// Renders `scene`, moved with its camera by `shift` metres, into a `width` ×
/// `height` offscreen texture for `FRAME_COUNT` frames and returns the last
/// one, with every frame's stats.
/// Time and camera are injected (the scene's frame delta and its camera path
/// by frame) and nothing reads the wall clock or input, so equal arguments
/// give equal images on one machine.
pub fn capture(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    width: u32,
    height: u32,
) -> anyhow::Result<(image::RgbaImage, Vec<FrameStats>)> {
    let (target, frame_stats) = render_frames(
        context,
        scene,
        shift,
        width,
        height,
        FRAME_COUNT,
        scene.frame_delta(),
    )?;
    Ok((read_back(context, &target)?, frame_stats))
}

/// Renders `frame_count` frames of `scene` as `capture` does, each advancing
/// by `frame_delta`, and returns the target, holding the last frame, with
/// every frame's stats.
pub(crate) fn render_frames(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    width: u32,
    height: u32,
    frame_count: u32,
    frame_delta: Duration,
) -> anyhow::Result<(wgpu::Texture, Vec<FrameStats>)> {
    let max_dimension = context.device.limits().max_texture_dimension_2d;
    anyhow::ensure!(
        (1..=max_dimension).contains(&width) && (1..=max_dimension).contains(&height),
        "capture size {width}x{height} is outside 1 to {max_dimension} per side"
    );
    anyhow::ensure!(
        (1..=PassTimer::MAX_FRAMES).contains(&frame_count),
        "{frame_count} frames is outside 1 to {} per run",
        PassTimer::MAX_FRAMES
    );

    let mut state = State::new(context, TARGET_FORMAT, width, height, scene, shift)?;
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
    let pass_timer = PassTimer::new(&context.device, frame_count);
    let mut frame_stats = Vec::with_capacity(frame_count as usize);
    let mut in_flight = std::collections::VecDeque::with_capacity(FRAMES_IN_FLIGHT);
    for frame in 0..frame_count {
        if in_flight.len() == FRAMES_IN_FLIGHT {
            let oldest = in_flight.pop_front();
            context.device.poll(wgpu::PollType::Wait {
                submission_index: oldest,
                timeout: None,
            })?;
        }
        state.world.insert_resource(scene.camera_at(frame, shift));
        state.update(frame_delta);
        let submission = state.render(&target_view, Some(pass_timer.writes(frame)));
        in_flight.push_back(submission);
        frame_stats.push(state.frame_stats());
    }

    for (stats, pass_time) in frame_stats.iter_mut().zip(pass_timer.read(context)?) {
        stats.gpu_times.push((MAIN_PASS, pass_time));
    }
    Ok((target, frame_stats))
}

/// GPU time of the State's pass in each frame of a capture, from timestamps
/// at the pass's boundaries (Apple GPUs have none inside passes). Metal reads
/// a timestamp as 0 or stale unless it is resolved after its pass's submission
/// completed (`docs/research/wgpu-capability-matrix.md`), so `read` resolves
/// every frame at once, after the last, and one set bounds a run's length.
/// Runs without end (the window) need a ring of sets instead, each resolved
/// once its frame's submission completes.
struct PassTimer {
    query_set: wgpu::QuerySet,
}

impl PassTimer {
    /// One set holds a begin and an end timestamp per frame.
    const MAX_FRAMES: u32 = wgpu::QUERY_SET_MAX_QUERIES / 2;

    /// A timer for `frame_count` frames. Timestamp queries are in the
    /// Capability floor (ADR 0021).
    fn new(device: &wgpu::Device, frame_count: u32) -> Self {
        Self {
            query_set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Pass Timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 2 * frame_count,
            }),
        }
    }

    fn writes(&self, frame: u32) -> wgpu::RenderPassTimestampWrites<'_> {
        wgpu::RenderPassTimestampWrites {
            query_set: &self.query_set,
            beginning_of_pass_write_index: Some(2 * frame),
            end_of_pass_write_index: Some(2 * frame + 1),
        }
    }

    /// Each frame's pass time, in frame order.
    fn read(&self, context: &GpuContext) -> anyhow::Result<Vec<Duration>> {
        let query_count = self.query_set.count();
        let size = u64::from(query_count) * u64::from(wgpu::QUERY_SIZE);
        let resolve_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pass Timestamp Resolve Buffer"),
            size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read_back_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pass Timestamp Read-back Buffer"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Every frame's submission completes before the resolve is recorded.
        context.device.poll(wgpu::PollType::wait_indefinitely())?;
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Pass Timestamp Resolve Encoder"),
            });
        encoder.resolve_query_set(&self.query_set, 0..query_count, &resolve_buffer, 0);
        encoder.copy_buffer_to_buffer(&resolve_buffer, 0, &read_back_buffer, 0, size);
        submit_and_map(context, encoder, &read_back_buffer)?;

        let nanoseconds_per_tick = f64::from(context.queue.get_timestamp_period());
        let ticks: Vec<u64> = bytemuck::pod_collect_to_vec(&read_back_buffer.get_mapped_range(..)?);
        Ok(ticks
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&[begin, end]| {
                let pass_ticks = end.saturating_sub(begin);
                Duration::from_nanos((pass_ticks as f64 * nanoseconds_per_tick) as u64)
            })
            .collect())
    }
}

/// Copies an RGBA8 `texture` to the CPU, without the row padding the copy
/// needs.
pub(crate) fn read_back(
    context: &GpuContext,
    texture: &wgpu::Texture,
) -> anyhow::Result<image::RgbaImage> {
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
    Ok(image::RgbaImage::from_raw(width, height, pixels)
        .expect("the read-back holds width × height RGBA8 pixels"))
}

/// Submits `encoder` and waits until `buffer`, which it writes, is mapped for
/// reading.
fn submit_and_map(
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
