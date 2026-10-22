use std::time::Duration;

use crate::gpu::GpuContext;
use crate::render::{FRAMES_IN_FLIGHT, RenderSwitches};
use crate::scenes::ReferenceScene;
use crate::state::State;
use crate::stats::FrameStats;

/// 8-bit UNORM like the window's surface: the tonemapping pass writes both
/// sRGB-encoded. RGBA order makes the read-back bytes the image's pixels.
pub(crate) const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// Frames rendered per capture; the last one is read back.
const FRAME_COUNT: u32 = 60;

/// Renders `scene`, moved with its camera by `shift` metres, into a `width` ×
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
    let (target, frame_stats) = render_frames(
        context,
        scene,
        shift,
        width,
        height,
        FRAME_COUNT,
        scene.frame_delta(),
        switches,
    )?;
    Ok((read_back(context, &target)?, frame_stats))
}

/// Renders `frame_count` frames of `scene` as `capture` does, each advancing
/// by `frame_delta`, and returns the target, holding the last frame, with
/// every frame's stats. Up to `FRAMES_IN_FLIGHT` frames are in flight, as a
/// window's swapchain allows: the GPU stays busy, and the CPU never runs
/// further ahead than a window would.
#[expect(clippy::too_many_arguments)]
pub(crate) fn render_frames(
    context: &GpuContext,
    scene: ReferenceScene,
    shift: glam::DVec3,
    width: u32,
    height: u32,
    frame_count: u32,
    frame_delta: Duration,
    switches: RenderSwitches,
) -> anyhow::Result<(wgpu::Texture, Vec<FrameStats>)> {
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
        in_flight.push_back(state.render(&target_view));
        frame_stats.push(state.frame_stats());
        add_gpu_stats(&mut frame_stats, &state);
    }
    state.flush_gpu_frame_stats()?;
    add_gpu_stats(&mut frame_stats, &state);
    Ok((target, frame_stats))
}

/// Gives each frame of `frame_stats`, numbered from 0 as `state`'s frames
/// are, the GPU stats `state` read back for it.
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
