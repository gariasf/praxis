//! Throwaway probe for #189, "State of the art: HDR, exposure and
//! tonemapping" (never merged). It makes the comparison images the survey
//! shows the owner and times the candidate passes on the M4 Pro.
//!
//! `cargo run --release -- images <directory of .f32 captures> <output directory>`
//! `cargo run --release -- bench`
//!
//! A `.f32` capture is the patched `praxis capture` output on this branch:
//! width and height as little-endian u32, then linear RGBA f32 texels.

use std::path::Path;

const OPERATORS: [(&str, u32); 9] = [
    ("clamp", 0),
    ("reinhard", 1),
    ("hable", 2),
    ("aces-narkowicz", 3),
    ("aces-hill", 4),
    ("agx", 5),
    ("pbr-neutral", 6),
    ("tony-mcmapface", 7),
    ("gt7", 8),
];
const PBR_NEUTRAL: u32 = 6;
const AGX: u32 = 5;

const BENCH_WIDTH: u32 = 2560;
const BENCH_HEIGHT: u32 = 1440;
const SUBMISSIONS: u32 = 45;
const WARM_UP_SUBMISSIONS: u32 = 5;
const REPETITIONS: u32 = 8;

const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const BLOOM_LEVELS: u32 = 6;

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    rg11b10_renderable: bool,
}

impl Gpu {
    async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await?;
        let info = adapter.get_info();
        eprintln!("adapter: {} ({:?}, {:?})", info.name, info.backend, info.device_type);
        let rg11b10_renderable = adapter
            .features()
            .contains(wgpu::Features::RG11B10UFLOAT_RENDERABLE);
        let mut required_features = wgpu::Features::TIMESTAMP_QUERY;
        if rg11b10_renderable {
            required_features |= wgpu::Features::RG11B10UFLOAT_RENDERABLE;
        }
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        Ok(Self {
            device,
            queue,
            rg11b10_renderable,
        })
    }

    fn texture(
        &self,
        label: &str,
        size: (u32, u32, u32),
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        sample_count: u32,
        dimension: wgpu::TextureDimension,
    ) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: size.2,
            },
            mip_level_count: 1,
            sample_count,
            dimension,
            format,
            usage,
            view_formats: &[],
        })
    }

    fn texture_2d(&self, label: &str, width: u32, height: u32, format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> wgpu::Texture {
        self.texture(label, (width, height, 1), format, usage, 1, wgpu::TextureDimension::D2)
    }

    fn wait(&self) -> anyhow::Result<()> {
        self.device.poll(wgpu::PollType::wait_indefinitely())?;
        Ok(())
    }

    fn read_buffer(&self, buffer: &wgpu::Buffer) -> anyhow::Result<Vec<u8>> {
        let (sender, receiver) = std::sync::mpsc::channel();
        buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            sender.send(result).ok();
        });
        self.wait()?;
        receiver.recv()??;
        let bytes = buffer.get_mapped_range(..)?.to_vec();
        buffer.unmap();
        Ok(bytes)
    }
}

/// Linear RGBA radiance, rows top first.
struct HdrImage {
    name: String,
    width: u32,
    height: u32,
    texels: Vec<[f32; 4]>,
}

impl HdrImage {
    fn load(path: &Path) -> anyhow::Result<Self> {
        let bytes = std::fs::read(path)?;
        anyhow::ensure!(bytes.len() >= 8, "{} is too short", path.display());
        let width = u32::from_le_bytes(bytes[0..4].try_into()?);
        let height = u32::from_le_bytes(bytes[4..8].try_into()?);
        let texels: Vec<[f32; 4]> = bytemuck::pod_collect_to_vec(&bytes[8..]);
        anyhow::ensure!(texels.len() == (width * height) as usize, "{} has the wrong size", path.display());
        Ok(Self {
            name: path.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
            width,
            height,
            texels,
        })
    }

    /// Fully saturated hues across, a neutral strip at the left, exposure
    /// down the rows: 0.18 times 2^10 at the top to 0.18 times 2^-6 at the
    /// bottom, linear Rec. 709.
    fn sweep(width: u32, height: u32) -> Self {
        let neutral_columns = width / 10;
        let mut texels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            let stops = 10.0 - 16.0 * y as f32 / (height - 1) as f32;
            let scale = 0.18 * stops.exp2();
            for x in 0..width {
                let color = if x < neutral_columns {
                    [1.0, 1.0, 1.0]
                } else {
                    hue((x - neutral_columns) as f32 / (width - neutral_columns) as f32)
                };
                texels.push([color[0] * scale, color[1] * scale, color[2] * scale, 1.0]);
            }
        }
        Self {
            name: "sweep".to_owned(),
            width,
            height,
            texels,
        }
    }

    /// A dim radial gradient, 0.004 to 0.016, where 8-bit output bands.
    fn dark_gradient(width: u32, height: u32) -> Self {
        let mut texels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let u = (x as f32 + 0.5) / width as f32 * 2.0 - 1.0;
                let v = (y as f32 + 0.5) / height as f32 * 2.0 - 1.0;
                let falloff = (1.0 - (u * u + v * v) * 0.5).max(0.0);
                let radiance = 0.004 + 0.012 * falloff;
                texels.push([radiance, radiance * 0.95, radiance * 0.9, 1.0]);
            }
        }
        Self {
            name: "dark-gradient".to_owned(),
            width,
            height,
            texels,
        }
    }

    fn upload(&self, gpu: &Gpu, format: wgpu::TextureFormat) -> wgpu::Texture {
        let texture = gpu.texture_2d(
            &self.name,
            self.width,
            self.height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let (bytes, texel_size): (Vec<u8>, u32) = match format {
            wgpu::TextureFormat::Rgba32Float => (bytemuck::cast_slice(&self.texels).to_vec(), 16),
            wgpu::TextureFormat::Rgba16Float => {
                let halves: Vec<u16> = self.texels.iter().flatten().map(|&value| f32_to_f16(value)).collect();
                (bytemuck::cast_slice(&halves).to_vec(), 8)
            }
            _ => unreachable!("HDR uploads are Rgba32Float or Rgba16Float"),
        };
        gpu.queue.write_texture(
            texture.as_image_copy(),
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width * texel_size),
                rows_per_image: Some(self.height),
            },
            texture.size(),
        );
        texture
    }
}

/// A fully saturated Rec. 709 hue, `position` 0 to 1 around the hexagon.
fn hue(position: f32) -> [f32; 3] {
    let sector = position * 6.0;
    let fraction = sector.fract();
    match sector as u32 % 6 {
        0 => [1.0, fraction, 0.0],
        1 => [1.0 - fraction, 1.0, 0.0],
        2 => [0.0, 1.0, fraction],
        3 => [0.0, 1.0 - fraction, 1.0],
        4 => [fraction, 0.0, 1.0],
        _ => [1.0, 0.0, 1.0 - fraction],
    }
}

/// IEEE 754 binary16 from binary32, rounding to nearest even; values past
/// 65504 become infinity, as the conversion rules give.
fn f32_to_f16(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x7f_ffff;
    if exponent == 0xff {
        return sign | 0x7c00 | if mantissa != 0 { 0x200 } else { 0 };
    }
    let half_exponent = exponent - 127 + 15;
    if half_exponent >= 0x1f {
        return sign | 0x7c00;
    }
    if half_exponent <= 0 {
        if half_exponent < -10 {
            return sign;
        }
        let full = mantissa | 0x80_0000;
        let shift = (14 - half_exponent) as u32;
        let half = full >> shift;
        let remainder = full & ((1 << shift) - 1);
        let halfway = 1 << (shift - 1);
        let rounded = if remainder > halfway || (remainder == halfway && half & 1 == 1) { half + 1 } else { half };
        return sign | rounded as u16;
    }
    let half = ((half_exponent as u32) << 10) | (mantissa >> 13);
    let remainder = mantissa & 0x1fff;
    let rounded = if remainder > 0x1000 || (remainder == 0x1000 && half & 1 == 1) { half + 1 } else { half };
    sign | rounded as u16
}

/// Post variants: the override constants of `post.wgsl`.
#[derive(Clone, Copy)]
struct PostVariant {
    operator: u32,
    dither: bool,
    invert_weighting: bool,
    bloom: bool,
    supersample: u32,
    display_average: bool,
}

impl PostVariant {
    fn operator(operator: u32) -> Self {
        Self {
            operator,
            dither: false,
            invert_weighting: false,
            bloom: false,
            supersample: 1,
            display_average: false,
        }
    }
}

struct Post {
    module: wgpu::ShaderModule,
    layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    tony_view: wgpu::TextureView,
    blank_view: wgpu::TextureView,
}

impl Post {
    fn new(gpu: &Gpu, tony_lut_path: &Path) -> anyhow::Result<Self> {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::include_wgsl!("post.wgsl"));
        let texture_entry = |binding, view_dimension, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post"),
            entries: &[
                texture_entry(0, wgpu::TextureViewDimension::D2, false),
                texture_entry(1, wgpu::TextureViewDimension::D3, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture_entry(4, wgpu::TextureViewDimension::D2, true),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Post"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Linear Clamp"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // The LUT ships as a DDS: 128 header bytes, a 20-byte DX10 header
        // naming R9G9B9E5_SHAREDEXP (67) and a 3D texture, then 48^3 texels.
        let dds = std::fs::read(tony_lut_path)?;
        anyhow::ensure!(&dds[0..4] == b"DDS " && dds.len() == 148 + 48 * 48 * 48 * 4, "unexpected LUT file");
        anyhow::ensure!(u32::from_le_bytes(dds[128..132].try_into()?) == 67, "LUT is not R9G9B9E5");
        let tony = gpu.texture(
            "Tony McMapface",
            (48, 48, 48),
            wgpu::TextureFormat::Rgb9e5Ufloat,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            1,
            wgpu::TextureDimension::D3,
        );
        gpu.queue.write_texture(
            tony.as_image_copy(),
            &dds[148..],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(48 * 4),
                rows_per_image: Some(48),
            },
            tony.size(),
        );
        let blank = gpu.texture_2d(
            "Blank",
            1,
            1,
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );
        Ok(Self {
            module,
            layout,
            pipeline_layout,
            uniform,
            sampler,
            tony_view: tony.create_view(&Default::default()),
            blank_view: blank.create_view(&Default::default()),
        })
    }

    fn pipeline(&self, gpu: &Gpu, variant: PostVariant) -> wgpu::RenderPipeline {
        let constants = [
            ("OPERATOR", f64::from(variant.operator)),
            ("DITHER", f64::from(u8::from(variant.dither))),
            ("INVERT_WEIGHTING", f64::from(u8::from(variant.invert_weighting))),
            ("BLOOM", f64::from(u8::from(variant.bloom))),
            ("SUPERSAMPLE", f64::from(variant.supersample)),
            ("DISPLAY_AVERAGE", f64::from(u8::from(variant.display_average))),
        ];
        fullscreen_pipeline(gpu, &self.module, &self.pipeline_layout, "fs_post", &constants, OUTPUT_FORMAT, None)
    }

    fn set_uniform(&self, gpu: &Gpu, exposure: f32, bloom_strength: f32) {
        gpu.queue
            .write_buffer(&self.uniform, 0, bytemuck::cast_slice(&[exposure, bloom_strength, 0.0, 0.0]));
    }

    fn bind_group(&self, gpu: &Gpu, hdr: &wgpu::TextureView, bloom: Option<&wgpu::TextureView>) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(hdr),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.tony_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(bloom.unwrap_or(&self.blank_view)),
                },
            ],
        })
    }
}

fn fullscreen_pipeline(
    gpu: &Gpu,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    fragment_entry: &str,
    constants: &[(&str, f64)],
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    let options = wgpu::PipelineCompilationOptions {
        constants,
        zero_initialize_workgroup_memory: true,
    };
    gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(fragment_entry),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("vs_fullscreen"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(fragment_entry),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: options,
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn color_attachment<'a>(
    view: &'a wgpu::TextureView,
    resolve_target: Option<&'a wgpu::TextureView>,
    load: wgpu::LoadOp<wgpu::Color>,
    store: wgpu::StoreOp,
) -> Option<wgpu::RenderPassColorAttachment<'a>> {
    Some(wgpu::RenderPassColorAttachment {
        view,
        resolve_target,
        depth_slice: None,
        ops: wgpu::Operations { load, store },
    })
}

fn render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    color: Option<wgpu::RenderPassColorAttachment<'a>>,
    depth: Option<wgpu::RenderPassDepthStencilAttachment<'a>>,
    timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'a>>,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[color],
        depth_stencil_attachment: depth,
        occlusion_query_set: None,
        timestamp_writes,
        multiview_mask: None,
    })
}

fn run_post(gpu: &Gpu, pipeline: &wgpu::RenderPipeline, bind_group: &wgpu::BindGroup, target: &wgpu::TextureView) {
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    {
        let mut pass = render_pass(
            &mut encoder,
            color_attachment(target, None, wgpu::LoadOp::Clear(wgpu::Color::BLACK), wgpu::StoreOp::Store),
            None,
            None,
        );
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
}

fn save_rgba8(gpu: &Gpu, texture: &wgpu::Texture, path: &Path) -> anyhow::Result<()> {
    let (width, height) = (texture.width(), texture.height());
    let row_bytes = width * 4;
    let padded = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Read-back"),
        size: u64::from(padded) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: None,
            },
        },
        texture.size(),
    );
    gpu.queue.submit([encoder.finish()]);
    let bytes = gpu.read_buffer(&buffer)?;
    let mut pixels = Vec::with_capacity((row_bytes * height) as usize);
    for row in bytes.chunks_exact(padded as usize) {
        pixels.extend_from_slice(&row[..row_bytes as usize]);
    }
    image::save_buffer_with_format(path, &pixels, width, height, image::ExtendedColorType::Rgba8, image::ImageFormat::Png)?;
    Ok(())
}

fn output_target(gpu: &Gpu, width: u32, height: u32) -> wgpu::Texture {
    gpu.texture_2d(
        "Output",
        width,
        height,
        OUTPUT_FORMAT,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
    )
}

/// Flat-shaded triangles in clip space, each vertex position then radiance.
struct SceneMesh {
    vertices: Vec<[f32; 5]>,
}

impl SceneMesh {
    fn push_quad(&mut self, corners: [[f32; 2]; 4], radiance: [f32; 3]) {
        for index in [0, 1, 2, 0, 2, 3] {
            let [x, y] = corners[index];
            self.vertices.push([x, y, radiance[0], radiance[1], radiance[2]]);
        }
    }

    /// A bar from `start` to `end` in pixels, `width` pixels wide.
    fn push_bar(&mut self, size: (f32, f32), start: [f32; 2], end: [f32; 2], width: f32, radiance: [f32; 3]) {
        let direction = [end[0] - start[0], end[1] - start[1]];
        let length = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
        let normal = [-direction[1] / length * width * 0.5, direction[0] / length * width * 0.5];
        let corners = [
            [start[0] + normal[0], start[1] + normal[1]],
            [start[0] - normal[0], start[1] - normal[1]],
            [end[0] - normal[0], end[1] - normal[1]],
            [end[0] + normal[0], end[1] + normal[1]],
        ]
        .map(|pixel| to_clip(size, pixel));
        self.push_quad(corners, radiance);
    }

    fn push_disc(&mut self, size: (f32, f32), center: [f32; 2], radius: f32, radiance: [f32; 3]) {
        let segments = 64;
        for segment in 0..segments {
            let angle = |index: u32| index as f32 / segments as f32 * std::f32::consts::TAU;
            let point = |index: u32| to_clip(size, [center[0] + radius * angle(index).cos(), center[1] + radius * angle(index).sin()]);
            for [x, y] in [to_clip(size, center), point(segment + 1), point(segment)] {
                self.vertices.push([x, y, radiance[0], radiance[1], radiance[2]]);
            }
        }
    }

    /// The resolve comparison: bright wires of decreasing width at several
    /// angles, a lamp, a sun glint and a mid-grey panel, over a dark ground.
    fn resolve_test(width: u32, height: u32) -> Self {
        let size = (width as f32, height as f32);
        let mut mesh = Self { vertices: Vec::new() };
        let panel = [
            [0.55 * size.0, 0.05 * size.1],
            [0.95 * size.0, 0.05 * size.1],
            [0.95 * size.0, 0.95 * size.1],
            [0.55 * size.0, 0.95 * size.1],
        ]
        .map(|pixel| to_clip(size, pixel));
        // Clockwise in pixels (y down) is counter-clockwise in clip space.
        mesh.push_quad([panel[0], panel[3], panel[2], panel[1]], [0.18, 0.18, 0.18]);
        let wire = [40.0, 38.0, 34.0];
        for (row, width_in_pixels) in [2.0, 1.0, 0.5, 0.25].into_iter().enumerate() {
            for (column, angle_degrees) in [3.0f32, 12.0, 30.0, 60.0].into_iter().enumerate() {
                let origin = [8.0 + 100.0 * column as f32, 12.0 + 55.0 * row as f32];
                let angle = angle_degrees.to_radians();
                let end = [origin[0] + 90.0 * angle.cos(), origin[1] + 40.0 * angle.sin()];
                mesh.push_bar(size, origin, end, width_in_pixels, wire);
            }
        }
        mesh.push_disc(size, [0.75 * size.0, 0.3 * size.1], 18.0, [24.0, 20.0, 12.0]);
        mesh.push_disc(size, [0.66 * size.0, 0.7 * size.1], 1.2, [600.0, 560.0, 500.0]);
        for (index, width_in_pixels) in [1.5, 0.75, 0.4].into_iter().enumerate() {
            let x = 0.6 * size.0 + 18.0 * index as f32;
            mesh.push_bar(size, [x, 0.45 * size.1], [x + 60.0, 0.9 * size.1], width_in_pixels, [8.0, 7.0, 6.0]);
        }
        mesh
    }

    /// The bench's stand-in for the lit pass: a screen of jittered cells
    /// split into triangles, so most pixels near a cell edge are partly
    /// covered, with radiance up to 64.
    fn edge_field(width: u32, height: u32) -> Self {
        let size = (width as f32, height as f32);
        let cell = 10.37;
        let columns = (size.0 / cell) as u32 + 1;
        let rows = (size.1 / cell) as u32 + 1;
        let mut mesh = Self { vertices: Vec::new() };
        let mut state = 0x9e37_79b9u32;
        let mut random = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as f32 / u32::MAX as f32
        };
        for row in 0..rows {
            for column in 0..columns {
                let x = column as f32 * cell;
                let y = row as f32 * cell;
                let brightness = 64.0f32.powf(random());
                let radiance = [brightness * random(), brightness * random(), brightness * random()];
                let corners = [[x, y], [x, y + cell], [x + cell, y + cell], [x + cell, y]].map(|pixel| to_clip(size, pixel));
                mesh.push_quad(corners, radiance);
            }
        }
        mesh
    }

    fn buffer(&self, gpu: &Gpu) -> wgpu::Buffer {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene"),
            size: (self.vertices.len() * 20) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&self.vertices));
        buffer
    }
}

fn to_clip(size: (f32, f32), pixel: [f32; 2]) -> [f32; 2] {
    [pixel[0] / size.0 * 2.0 - 1.0, 1.0 - pixel[1] / size.1 * 2.0]
}

struct Scene {
    module: wgpu::ShaderModule,
    layout: wgpu::PipelineLayout,
}

impl Scene {
    fn new(gpu: &Gpu) -> Self {
        Self {
            module: gpu.device.create_shader_module(wgpu::include_wgsl!("scene.wgsl")),
            layout: gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Scene"),
                bind_group_layouts: &[],
                immediate_size: 0,
            }),
        }
    }

    fn pipeline(&self, gpu: &Gpu, format: wgpu::TextureFormat, sample_count: u32, weighting: bool) -> wgpu::RenderPipeline {
        let constants = [("WEIGHTING", f64::from(u8::from(weighting)))];
        gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Scene"),
            layout: Some(&self.layout),
            vertex: wgpu::VertexState {
                module: &self.module,
                entry_point: Some("vs_scene"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 20,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.module,
                entry_point: Some("fs_scene"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &constants,
                    zero_initialize_workgroup_memory: true,
                },
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }
}

/// Render targets for one scene configuration: a 4x MSAA colour and depth
/// that live only in tile memory (memoryless on Apple GPUs), resolved by the
/// hardware into a 1x texture; or 1x targets stored directly.
struct SceneTargets {
    color: wgpu::TextureView,
    depth: wgpu::TextureView,
    resolved: Option<wgpu::TextureView>,
    resolved_texture: wgpu::Texture,
}

impl SceneTargets {
    fn new(gpu: &Gpu, width: u32, height: u32, format: wgpu::TextureFormat, sample_count: u32) -> Self {
        let sampled = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let transient = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TRANSIENT_ATTACHMENT;
        let resolved_texture = gpu.texture_2d("Resolved", width, height, format, sampled);
        let depth = gpu.texture(
            "Depth",
            (width, height, 1),
            wgpu::TextureFormat::Depth32Float,
            transient,
            sample_count,
            wgpu::TextureDimension::D2,
        );
        if sample_count == 1 {
            return Self {
                color: resolved_texture.create_view(&Default::default()),
                depth: depth.create_view(&Default::default()),
                resolved: None,
                resolved_texture,
            };
        }
        let color = gpu.texture("Color 4x", (width, height, 1), format, transient, sample_count, wgpu::TextureDimension::D2);
        Self {
            color: color.create_view(&Default::default()),
            depth: depth.create_view(&Default::default()),
            resolved: Some(resolved_texture.create_view(&Default::default())),
            resolved_texture,
        }
    }

    fn draw<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        pipeline: &wgpu::RenderPipeline,
        vertices: &wgpu::Buffer,
        vertex_count: u32,
        background: f64,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'a>>,
    ) {
        let store = if self.resolved.is_some() { wgpu::StoreOp::Discard } else { wgpu::StoreOp::Store };
        let clear = wgpu::Color {
            r: background,
            g: background,
            b: background,
            a: 1.0,
        };
        let mut pass = render_pass(
            encoder,
            color_attachment(&self.color, self.resolved.as_ref(), wgpu::LoadOp::Clear(clear), store),
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes,
        );
        pass.set_pipeline(pipeline);
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.draw(0..vertex_count, 0..1);
    }
}

fn images(gpu: &Gpu, input: &Path, output: &Path, tony_lut: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(output)?;
    let post = Post::new(gpu, tony_lut)?;
    post.set_uniform(gpu, 1.0, 0.0);
    let pipelines: Vec<_> = OPERATORS
        .iter()
        .map(|&(name, operator)| (name, post.pipeline(gpu, PostVariant::operator(operator))))
        .collect();

    let mut inputs = vec![HdrImage::sweep(1280, 720)];
    let mut paths: Vec<_> = std::fs::read_dir(input)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "f32"))
        .collect();
    paths.sort();
    for path in paths {
        inputs.push(HdrImage::load(&path)?);
    }
    for image in &inputs {
        let texture = image.upload(gpu, wgpu::TextureFormat::Rgba32Float);
        let bind_group = post.bind_group(gpu, &texture.create_view(&Default::default()), None);
        let target = output_target(gpu, image.width, image.height);
        let target_view = target.create_view(&Default::default());
        for (name, pipeline) in &pipelines {
            run_post(gpu, pipeline, &bind_group, &target_view);
            save_rgba8(gpu, &target, &output.join(format!("{}-{name}.png", image.name)))?;
        }
    }

    // Banding: a dim gradient through PBR Neutral, without and with the
    // static triangular dither.
    let gradient = HdrImage::dark_gradient(1280, 720);
    let texture = gradient.upload(gpu, wgpu::TextureFormat::Rgba32Float);
    let bind_group = post.bind_group(gpu, &texture.create_view(&Default::default()), None);
    let target = output_target(gpu, gradient.width, gradient.height);
    let target_view = target.create_view(&Default::default());
    for (suffix, dither) in [("", false), ("-dither", true)] {
        let pipeline = post.pipeline(
            gpu,
            PostVariant {
                dither,
                ..PostVariant::operator(PBR_NEUTRAL)
            },
        );
        run_post(gpu, &pipeline, &bind_group, &target_view);
        save_rgba8(gpu, &target, &output.join(format!("dark-gradient-pbr-neutral{suffix}.png")))?;
    }

    resolve_images(gpu, &post, output)?;
    Ok(())
}

/// The resolve comparison through one operator: the hardware box resolve of
/// 4x MSAA, the weighted resolve, and two 64-sample references made by
/// rendering at 4x4 the size with 4x MSAA, averaged in scene-linear space
/// (what a box resolve converges to) or in display-linear space.
fn resolve_images(gpu: &Gpu, post: &Post, output: &Path) -> anyhow::Result<()> {
    let (width, height) = (400, 240);
    let scene = Scene::new(gpu);
    let background = 0.02;
    for (operator_name, operator) in [("pbr-neutral", PBR_NEUTRAL), ("agx", AGX)] {
        let target = output_target(gpu, width, height);
        let target_view = target.create_view(&Default::default());
        let cases = [
            ("box", 1u32, false, PostVariant::operator(operator)),
            (
                "weighted",
                1,
                true,
                PostVariant {
                    invert_weighting: true,
                    ..PostVariant::operator(operator)
                },
            ),
            (
                "reference-linear",
                4,
                false,
                PostVariant {
                    supersample: 4,
                    ..PostVariant::operator(operator)
                },
            ),
            (
                "reference-display",
                4,
                false,
                PostVariant {
                    supersample: 4,
                    display_average: true,
                    ..PostVariant::operator(operator)
                },
            ),
        ];
        for (name, scale, weighting, variant) in cases {
            // Laid out in output pixels; clip space is the same at any scale.
            let mesh = SceneMesh::resolve_test(width, height);
            let vertices = mesh.buffer(gpu);
            let targets = SceneTargets::new(gpu, width * scale, height * scale, wgpu::TextureFormat::Rgba16Float, 4);
            let pipeline = scene.pipeline(gpu, wgpu::TextureFormat::Rgba16Float, 4, weighting);
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            let weighted_background = if weighting { background / (1.0 + background) } else { background };
            targets.draw(&mut encoder, &pipeline, &vertices, mesh.vertices.len() as u32, weighted_background, None);
            gpu.queue.submit([encoder.finish()]);
            let bind_group = post.bind_group(gpu, &targets.resolved_texture.create_view(&Default::default()), None);
            run_post(gpu, &post.pipeline(gpu, variant), &bind_group, &target_view);
            save_rgba8(gpu, &target, &output.join(format!("resolve-{operator_name}-{name}.png")))?;
        }
    }
    Ok(())
}

/// Timestamp pairs for every repetition of one configuration. Metal reads a
/// pass timestamp correctly only when it is resolved after the pass's
/// submission completed, so every submission is waited on and the set is
/// resolved once at the end (`docs/research/wgpu-capability-matrix.md`).
struct Timer {
    query_set: wgpu::QuerySet,
}

impl Timer {
    fn new(gpu: &Gpu) -> Self {
        Self {
            query_set: gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamps"),
                ty: wgpu::QueryType::Timestamp,
                count: 2 * SUBMISSIONS,
            }),
        }
    }

    /// Runs `record` `REPETITIONS` times per submission, with a timestamp
    /// before the first repetition's first pass and after the last
    /// repetition's last pass, and returns each submission's milliseconds
    /// per repetition after the warm-up submissions. Timing whole
    /// submissions keeps the overlap of consecutive passes on a tile-based
    /// GPU out of the figure.
    fn measure(
        &self,
        gpu: &Gpu,
        record: &mut dyn FnMut(&mut wgpu::CommandEncoder, &wgpu::QuerySet, Option<u32>, Option<u32>),
    ) -> anyhow::Result<Vec<f64>> {
        for submission in 0..SUBMISSIONS {
            let mut encoder = gpu.device.create_command_encoder(&Default::default());
            for repetition in 0..REPETITIONS {
                let begin = (repetition == 0).then_some(2 * submission);
                let end = (repetition == REPETITIONS - 1).then_some(2 * submission + 1);
                record(&mut encoder, &self.query_set, begin, end);
            }
            let index = gpu.queue.submit([encoder.finish()]);
            gpu.device.poll(wgpu::PollType::Wait {
                submission_index: Some(index),
                timeout: None,
            })?;
        }
        let count = 2 * SUBMISSIONS;
        let size = u64::from(count) * u64::from(wgpu::QUERY_SIZE);
        let resolve = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Resolve"),
            size,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Read"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        encoder.resolve_query_set(&self.query_set, 0..count, &resolve, 0);
        encoder.copy_buffer_to_buffer(&resolve, 0, &read, 0, size);
        gpu.queue.submit([encoder.finish()]);
        let ticks: Vec<u64> = bytemuck::pod_collect_to_vec(&gpu.read_buffer(&read)?);
        let nanoseconds_per_tick = f64::from(gpu.queue.get_timestamp_period());
        Ok((WARM_UP_SUBMISSIONS..SUBMISSIONS)
            .map(|submission| {
                let index = 2 * submission as usize;
                let elapsed = ticks[index + 1].saturating_sub(ticks[index]);
                elapsed as f64 * nanoseconds_per_tick / 1e6 / f64::from(REPETITIONS)
            })
            .collect())
    }
}

fn report(name: &str, mut milliseconds: Vec<f64>) {
    milliseconds.sort_by(f64::total_cmp);
    let at = |fraction: f64| milliseconds[((milliseconds.len() - 1) as f64 * fraction).round() as usize];
    let zeros = milliseconds.iter().filter(|&&value| value == 0.0).count();
    println!(
        "{name:<44} median {:7.3} ms  p10 {:7.3}  p90 {:7.3}  n {}  zero {}",
        at(0.5),
        at(0.1),
        at(0.9),
        milliseconds.len(),
        zeros
    );
}

fn render_timestamps(query_set: &wgpu::QuerySet, begin: Option<u32>, end: Option<u32>) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
    (begin.is_some() || end.is_some()).then_some(wgpu::RenderPassTimestampWrites {
        query_set,
        beginning_of_pass_write_index: begin,
        end_of_pass_write_index: end,
    })
}

fn bench(gpu: &Gpu, tony_lut: &Path) -> anyhow::Result<()> {
    let (width, height) = (BENCH_WIDTH, BENCH_HEIGHT);
    let timer = Timer::new(gpu);
    let post = Post::new(gpu, tony_lut)?;
    post.set_uniform(gpu, 1.0, 0.04);
    let hdr = HdrImage::sweep(width, height).upload(gpu, wgpu::TextureFormat::Rgba16Float);
    let hdr_view = hdr.create_view(&Default::default());
    let target = output_target(gpu, width, height);
    let target_view = target.create_view(&Default::default());

    let bloom_levels = |format: wgpu::TextureFormat| -> Vec<wgpu::TextureView> {
        (1..=BLOOM_LEVELS)
            .map(|level| {
                gpu.texture_2d(
                    "Bloom",
                    width >> level,
                    height >> level,
                    format,
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                )
                .create_view(&Default::default())
            })
            .collect()
    };
    let bloom_16 = bloom_levels(wgpu::TextureFormat::Rgba16Float);

    // Final pass, one operator at a time, then with each optional stage.
    let mut variants: Vec<(String, PostVariant)> = OPERATORS
        .iter()
        .map(|&(name, operator)| (format!("post {name}"), PostVariant::operator(operator)))
        .collect();
    variants.push((
        "post agx + static dither".to_owned(),
        PostVariant {
            dither: true,
            ..PostVariant::operator(AGX)
        },
    ));
    variants.push((
        "post agx + inverse weighting".to_owned(),
        PostVariant {
            invert_weighting: true,
            ..PostVariant::operator(AGX)
        },
    ));
    variants.push((
        "post agx + bloom composite".to_owned(),
        PostVariant {
            bloom: true,
            ..PostVariant::operator(AGX)
        },
    ));
    let plain_bind_group = post.bind_group(gpu, &hdr_view, None);
    let bloom_bind_group = post.bind_group(gpu, &hdr_view, Some(&bloom_16[0]));
    // Bring the GPU's clock up before the first measurement.
    let warm_up = post.pipeline(gpu, PostVariant::operator(0));
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_secs(2) {
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        for _ in 0..REPETITIONS {
            let mut pass = render_pass(
                &mut encoder,
                color_attachment(&target_view, None, wgpu::LoadOp::Clear(wgpu::Color::BLACK), wgpu::StoreOp::Store),
                None,
                None,
            );
            pass.set_pipeline(&warm_up);
            pass.set_bind_group(0, &plain_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        let index = gpu.queue.submit([encoder.finish()]);
        gpu.device.poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: None,
        })?;
    }
    for (name, variant) in &variants {
        let pipeline = post.pipeline(gpu, *variant);
        let bind_group = if variant.bloom { &bloom_bind_group } else { &plain_bind_group };
        let milliseconds = timer.measure(gpu, &mut |encoder, query_set, begin, end| {
            let mut pass = render_pass(
                encoder,
                color_attachment(&target_view, None, wgpu::LoadOp::Clear(wgpu::Color::BLACK), wgpu::StoreOp::Store),
                None,
                render_timestamps(query_set, begin, end),
            );
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.draw(0..3, 0..1);
        })?;
        report(name, milliseconds);
    }

    // Histogram exposure: build, then average and adapt, two dispatches.
    {
        let module = gpu.device.create_shader_module(wgpu::include_wgsl!("histogram.wgsl"));
        let storage = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Histogram"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(1),
                storage(2),
            ],
        });
        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Histogram"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let compute = |entry_point| {
            gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(entry_point),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let build = compute("build_histogram");
        let average = compute("average_histogram");
        let histogram = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Histogram"),
            size: 256 * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let exposure = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Exposure"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&exposure, 0, bytemuck::cast_slice(&[0.0f32, 1.0, 0.05, 0.0]));
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Histogram"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: histogram.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: exposure.as_entire_binding(),
                },
            ],
        });
        let milliseconds = timer.measure(gpu, &mut |encoder, query_set, begin, end| {
            for (pipeline, workgroups, pass_begin, pass_end) in [
                (&build, (width.div_ceil(16), height.div_ceil(16)), begin, None),
                (&average, (1, 1), None, end),
            ] {
                let timestamp_writes = (pass_begin.is_some() || pass_end.is_some()).then_some(wgpu::ComputePassTimestampWrites {
                    query_set,
                    beginning_of_pass_write_index: pass_begin,
                    end_of_pass_write_index: pass_end,
                });
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: None,
                    timestamp_writes,
                });
                pass.set_pipeline(pipeline);
                pass.set_bind_group(0, &bind_group, &[]);
                pass.dispatch_workgroups(workgroups.0, workgroups.1, 1);
            }
        })?;
        report("histogram exposure (build + average)", milliseconds);
    }

    // Bloom chain: six 13-tap downsamples, the first with the Karis
    // average, then five additive tent upsamples.
    {
        let module = gpu.device.create_shader_module(wgpu::include_wgsl!("bloom.wgsl"));
        let layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let mut formats = vec![("bloom chain Rgba16Float", wgpu::TextureFormat::Rgba16Float, bloom_16.clone())];
        if gpu.rg11b10_renderable {
            formats.push(("bloom chain Rg11b10Ufloat", wgpu::TextureFormat::Rg11b10Ufloat, bloom_levels(wgpu::TextureFormat::Rg11b10Ufloat)));
        }
        for (name, format, levels) in formats {
            let additive = wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::REPLACE,
            };
            let first_downsample = fullscreen_pipeline(gpu, &module, &pipeline_layout, "fs_downsample", &[("KARIS_AVERAGE", 1.0)], format, None);
            let downsample = fullscreen_pipeline(gpu, &module, &pipeline_layout, "fs_downsample", &[], format, None);
            let upsample = fullscreen_pipeline(gpu, &module, &pipeline_layout, "fs_upsample", &[], format, Some(additive));
            let bind = |view: &wgpu::TextureView| {
                gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bloom"),
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&post.sampler),
                        },
                    ],
                })
            };
            let source_groups: Vec<_> = std::iter::once(bind(&hdr_view)).chain(levels.iter().map(&bind)).collect();
            // Passes: (pipeline, source group, target level, load).
            let mut passes = Vec::new();
            for level in 0..BLOOM_LEVELS as usize {
                let pipeline = if level == 0 { &first_downsample } else { &downsample };
                passes.push((pipeline, level, level, wgpu::LoadOp::Clear(wgpu::Color::BLACK)));
            }
            for level in (0..BLOOM_LEVELS as usize - 1).rev() {
                passes.push((&upsample, level + 2, level, wgpu::LoadOp::Load));
            }
            let pass_count = passes.len();
            let milliseconds = timer.measure(gpu, &mut |encoder, query_set, begin, end| {
                for (pass_index, (pipeline, source, target_level, load)) in passes.iter().enumerate() {
                    let pass_begin = begin.filter(|_| pass_index == 0);
                    let pass_end = end.filter(|_| pass_index == pass_count - 1);
                    let mut pass = render_pass(
                        encoder,
                        color_attachment(&levels[*target_level], None, *load, wgpu::StoreOp::Store),
                        None,
                        render_timestamps(query_set, pass_begin, pass_end),
                    );
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, &source_groups[*source], &[]);
                    pass.draw(0..3, 0..1);
                }
            })?;
            report(name, milliseconds);
        }
    }

    // Lit-pass stand-in: flat triangles into 4x memoryless colour and depth,
    // hardware-resolved, by format and with the reversible weighting.
    {
        let scene = Scene::new(gpu);
        let mesh = SceneMesh::edge_field(width, height);
        let vertices = mesh.buffer(gpu);
        let vertex_count = mesh.vertices.len() as u32;
        let mut cases = vec![
            ("scene 1x Rgba16Float, stored", wgpu::TextureFormat::Rgba16Float, 1, false),
            ("scene 4x Rgba16Float, memoryless, resolved", wgpu::TextureFormat::Rgba16Float, 4, false),
            ("scene 4x Rgba16Float + weighting, resolved", wgpu::TextureFormat::Rgba16Float, 4, true),
            ("scene 4x Rgba8Unorm, memoryless, resolved", wgpu::TextureFormat::Rgba8Unorm, 4, false),
        ];
        if gpu.rg11b10_renderable {
            cases.push(("scene 4x Rg11b10Ufloat, memoryless, resolved", wgpu::TextureFormat::Rg11b10Ufloat, 4, false));
        }
        for (name, format, sample_count, weighting) in cases {
            let targets = SceneTargets::new(gpu, width, height, format, sample_count);
            let pipeline = scene.pipeline(gpu, format, sample_count, weighting);
            let milliseconds = timer.measure(gpu, &mut |encoder, query_set, begin, end| {
                targets.draw(encoder, &pipeline, &vertices, vertex_count, 0.02, render_timestamps(query_set, begin, end));
            })?;
            report(name, milliseconds);
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let tony_lut = std::env::var("TONY_LUT").unwrap_or_else(|_| "tony_mc_mapface.dds".to_owned());
    let gpu = pollster::block_on(Gpu::new())?;
    match arguments.as_slice() {
        [command, input, output] if command == "images" => images(&gpu, Path::new(input), Path::new(output), Path::new(&tony_lut)),
        [command] if command == "bench" => bench(&gpu, Path::new(&tony_lut)),
        [command] if command == "overflow" => overflow(&gpu),
        _ => anyhow::bail!("usage: images <directory of .f32 captures> <output directory> | bench | overflow"),
    }
}

/// IEEE 754 binary16 to f32.
fn f16_to_f32(bits: u16) -> f32 {
    let sign = if bits & 0x8000 != 0 { -1.0f32 } else { 1.0 };
    let exponent = i32::from((bits >> 10) & 0x1f);
    let mantissa = f32::from(bits & 0x3ff);
    match exponent {
        0 => sign * mantissa * 2f32.powi(-24),
        0x1f if mantissa == 0.0 => sign * f32::INFINITY,
        0x1f => f32::NAN,
        _ => sign * (1.0 + mantissa / 1024.0) * 2f32.powi(exponent - 15),
    }
}

/// What an `Rgba16Float` attachment stores for values past 65504, at 1x and
/// as one of four samples under the hardware resolve.
fn overflow(gpu: &Gpu) -> anyhow::Result<()> {
    let module = gpu.device.create_shader_module(wgpu::include_wgsl!("overflow.wgsl"));
    let layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Overflow"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Overflow"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = |vertex_entry: &str, sample_count: u32| {
        gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Overflow"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some(vertex_entry),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_value"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    };
    let written = [65504.0f32, 65519.0, 65520.0, 70000.0, 1.0e6, 3.0e9, f32::INFINITY, -1.0e6];
    let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Values"),
        size: 8 * 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Overflow"),
        layout: &layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });
    let read_texels = |texture: &wgpu::Texture| -> anyhow::Result<Vec<f32>> {
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Read-back"),
            size: 256,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256),
                    rows_per_image: None,
                },
            },
            texture.size(),
        );
        gpu.queue.submit([encoder.finish()]);
        let bytes = gpu.read_buffer(&buffer)?;
        let halves: Vec<u16> = bytemuck::pod_collect_to_vec(&bytes);
        Ok(halves.iter().step_by(4).take(texture.width() as usize).map(|&bits| f16_to_f32(bits)).collect())
    };
    let target_usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC;

    // 1x: pixel x writes written[x] to every channel.
    gpu.queue.write_buffer(&uniform, 0, bytemuck::cast_slice(&written.map(|value| [value; 4])));
    let target = gpu.texture_2d("Overflow 1x", 8, 1, wgpu::TextureFormat::Rgba16Float, target_usage);
    let view = target.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    {
        let mut pass = render_pass(
            &mut encoder,
            color_attachment(&view, None, wgpu::LoadOp::Clear(wgpu::Color::BLACK), wgpu::StoreOp::Store),
            None,
            None,
        );
        pass.set_pipeline(&pipeline("vs_triangle", 1));
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    gpu.queue.submit([encoder.finish()]);
    for (value, stored) in written.iter().zip(read_texels(&target)?) {
        println!("1x Rgba16Float: wrote {value:e}, stored {stored:e}");
    }

    // 4x: the clear covers every sample with 0.5; one sample gets the value;
    // the hardware resolves into a 1x texture.
    let multisampled = gpu.texture(
        "Overflow 4x",
        (1, 1, 1),
        wgpu::TextureFormat::Rgba16Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TRANSIENT_ATTACHMENT,
        4,
        wgpu::TextureDimension::D2,
    );
    let multisampled_view = multisampled.create_view(&Default::default());
    let resolved = gpu.texture_2d("Overflow resolved", 1, 1, wgpu::TextureFormat::Rgba16Float, target_usage);
    let resolved_view = resolved.create_view(&Default::default());
    let sample_pipeline = pipeline("vs_sample_zero", 4);
    for value in [100.0f32, 65504.0, 1.0e6, f32::INFINITY] {
        gpu.queue.write_buffer(&uniform, 0, bytemuck::cast_slice(&[[value; 4]; 8]));
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        {
            let clear = wgpu::Color {
                r: 0.5,
                g: 0.5,
                b: 0.5,
                a: 0.5,
            };
            let mut pass = render_pass(
                &mut encoder,
                color_attachment(&multisampled_view, Some(&resolved_view), wgpu::LoadOp::Clear(clear), wgpu::StoreOp::Discard),
                None,
                None,
            );
            pass.set_pipeline(&sample_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        gpu.queue.submit([encoder.finish()]);
        let stored = read_texels(&resolved)?[0];
        println!("4x Rgba16Float, one sample {value:e} over three of 0.5: resolved {stored:e}");
    }
    Ok(())
}
