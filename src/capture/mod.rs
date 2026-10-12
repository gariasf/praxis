use crate::gpu::GpuContext;
use crate::scenes::ReferenceScene;
use crate::state::State;

/// 8-bit sRGB like the window's surface, which takes the first sRGB format
/// the surface offers (`Bgra8UnormSrgb` on Metal). RGBA order makes the
/// read-back bytes the image's pixels.
const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Frames rendered per capture, one second at `FRAME_DELTA_TIME`; the last
/// one is read back.
const FRAME_COUNT: u32 = 60;

/// The injected time step in seconds, 60 Hz.
const FRAME_DELTA_TIME: f32 = 1.0 / 60.0;

/// Renders `scene` into a `width` × `height` offscreen texture for
/// `FRAME_COUNT` frames and returns the last one. Time and camera are
/// injected (a fixed step, the scene's camera path by frame) and nothing
/// reads the wall clock or input, so equal arguments give equal images on one
/// machine.
pub fn capture(
    context: &GpuContext,
    scene: ReferenceScene,
    width: u32,
    height: u32,
) -> anyhow::Result<image::RgbaImage> {
    let max_dimension = context.device.limits().max_texture_dimension_2d;
    anyhow::ensure!(
        (1..=max_dimension).contains(&width) && (1..=max_dimension).contains(&height),
        "capture size {width}x{height} is outside 1 to {max_dimension} per side"
    );

    let mut state = State::new(context, TARGET_FORMAT, width, height, scene)?;
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
    for frame in 0..FRAME_COUNT {
        state.world.insert_resource(scene.camera_at(frame));
        state.update(FRAME_DELTA_TIME);
        state.render(&target_view);
    }

    read_back(context, &target)
}

/// Copies an RGBA8 `texture` to the CPU, without the row padding the copy
/// needs.
fn read_back(context: &GpuContext, texture: &wgpu::Texture) -> anyhow::Result<image::RgbaImage> {
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
    context.queue.submit(std::iter::once(encoder.finish()));

    let (sender, receiver) = std::sync::mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        sender.send(result).ok();
    });
    context.device.poll(wgpu::PollType::wait_indefinitely())?;
    receiver.recv()??;

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
