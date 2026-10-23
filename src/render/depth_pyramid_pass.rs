//! The depth pyramid pass: a compute pass that builds the render targets'
//! depth pyramid from the depth the depth pre-pass stored, a level at a
//! time, each texel the minimum depth, the farthest, of the texels below
//! whose share of the screen overlaps its own (ADRs 0015, 0019).

use crate::render::bind_groups::{DEPTH_PYRAMID_LEVEL, pipeline_layout};
use crate::render::shaders;
use crate::render::targets::{RenderTargets, SAMPLE_COUNT};

/// The depth pyramid pass's label, and its name in frame stats.
pub(crate) const DEPTH_PYRAMID_PASS: &str = "depth_pyramid_pass";

/// Texels along each side of the tile of a level each workgroup reduces.
pub(crate) const DEPTH_PYRAMID_TILE_SIZE: u32 = 8;

// Level 0 loads each pixel's samples by index, as Metal's compiler does not
// unroll a loop over them (`depth_pyramid.wgsl`).
const _: () = assert!(SAMPLE_COUNT == 4);

/// The pass's pipelines, level 0's and the later levels', and a bind group
/// per level with its size.
pub struct DepthPyramidPass {
    reduce_depth: wgpu::ComputePipeline,
    reduce_level: wgpu::ComputePipeline,
    reduce_depth_layout: wgpu::BindGroupLayout,
    reduce_level_layout: wgpu::BindGroupLayout,
    /// Built from the render targets by `bind_render_targets`.
    levels: Vec<(wgpu::BindGroup, wgpu::Extent3d)>,
}

impl DepthPyramidPass {
    /// The pass, building the pyramid of `render_targets`.
    pub fn new(device: &wgpu::Device, render_targets: &RenderTargets) -> DepthPyramidPass {
        let shader = device.create_shader_module(shaders::DEPTH_PYRAMID.descriptor());
        // Level 0 reduces the depth target, each later level the one below.
        let reduce_depth_layout = DEPTH_PYRAMID_LEVEL.layout_of(device, &["level", "depth"]);
        let reduce_level_layout = DEPTH_PYRAMID_LEVEL.layout_of(device, &["level", "level_below"]);
        let pipeline = |entry_point, layout| {
            let label = "Depth Pyramid Pipeline Layout";
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout(
                    device,
                    label,
                    &[(&DEPTH_PYRAMID_LEVEL, layout)],
                )),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let mut pass = DepthPyramidPass {
            reduce_depth: pipeline("reduce_depth", &reduce_depth_layout),
            reduce_level: pipeline("reduce_level", &reduce_level_layout),
            reduce_depth_layout,
            reduce_level_layout,
            levels: Vec::new(),
        };
        pass.bind_render_targets(device, render_targets);
        pass
    }

    /// Builds the bind groups that read and write `render_targets`: at
    /// creation, and after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        let levels = render_targets.depth_pyramid_levels();
        let bind = |index: usize, layout, reduced: (&'static str, &wgpu::TextureView)| {
            let level = &levels[index];
            let bind_group = DEPTH_PYRAMID_LEVEL.bind_group(
                device,
                layout,
                [
                    ("level", wgpu::BindingResource::TextureView(level)),
                    (reduced.0, wgpu::BindingResource::TextureView(reduced.1)),
                ],
            );
            let size = level
                .texture()
                .size()
                .mip_level_size(index as u32, wgpu::TextureDimension::D2);
            (bind_group, size)
        };
        let reduce_depth = bind(
            0,
            &self.reduce_depth_layout,
            ("depth", render_targets.depth()),
        );
        self.levels = std::iter::once(reduce_depth)
            .chain((1..levels.len()).map(|index| {
                bind(
                    index,
                    &self.reduce_level_layout,
                    ("level_below", &levels[index - 1]),
                )
            }))
            .collect();
    }

    /// Records the pass into `encoder`, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(DEPTH_PYRAMID_PASS),
            timestamp_writes,
        });
        // Level 0 reduces the depth target, each later level the one below.
        for (index, (bind_group, size)) in self.levels.iter().enumerate() {
            pass.set_pipeline(if index == 0 {
                &self.reduce_depth
            } else {
                &self.reduce_level
            });
            pass.set_bind_group(DEPTH_PYRAMID_LEVEL.index, bind_group, &[]);
            pass.dispatch_workgroups(
                size.width.div_ceil(DEPTH_PYRAMID_TILE_SIZE),
                size.height.div_ceil(DEPTH_PYRAMID_TILE_SIZE),
                1,
            );
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::assets::pools::tests::{gpu_context, read_storage};
    use crate::gpu::GpuContext;
    use crate::render::targets;

    /// The pass, rebound to render targets created again at an odd size, so
    /// each level rounds down and some texels cover three below them: after
    /// triangles at several depths over one that covers most of the screen,
    /// each texel of each level holds the minimum depth of the texels below
    /// whose share of the screen overlaps its own, against a CPU reduction of
    /// every sample of the depth target, read back.
    #[test]
    fn each_depth_pyramid_texel_holds_the_minimum_depth_of_its_share_of_the_screen() {
        let Some(context) = gpu_context() else {
            return;
        };
        let device = &context.device;
        let mut pass = DepthPyramidPass::new(device, &RenderTargets::new(device, 16, 16));
        let (width, height) = (37, 23);
        let render_targets = RenderTargets::new(device, width, height);
        pass.bind_render_targets(device, &render_targets);
        // Each triangle's centre, half its width and its depth, in clip space.
        let triangles = [
            ([0.0, 0.0], 0.9, 0.2),
            ([-0.4, 0.3], 0.3, 0.7),
            ([0.3, -0.2], 0.3, 0.45),
            ([0.5, 0.4], 0.3, 0.9),
            ([-0.6, -0.5], 0.3, 0.3),
        ]
        .map(|([x, y], half_width, depth)| {
            [[-1.0, -1.0], [1.0, -1.0], [0.0, 1.0]]
                .map(|[across, up]| [x + across * half_width, y + up * half_width, depth, 1.0])
        });
        draw_depth(&context, &render_targets, &triangles);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        pass.record(&mut encoder, None);
        context.queue.submit([encoder.finish()]);

        let samples = read_depth_samples(&context, render_targets.depth());
        let expected = minimum_depth_pyramid(&samples, width, height);
        let levels = render_targets.depth_pyramid_levels();
        assert_eq!(levels.len(), expected.len());
        for ((level, view), expected) in (0..).zip(levels).zip(&expected) {
            let actual = read_texels(&context, view, level);
            let differing = actual
                .iter()
                .zip(expected)
                .filter(|(actual, expected)| actual != expected)
                .count();
            assert!(
                actual.len() == expected.len() && differing == 0,
                "level {level}: {differing} of {} texels differ\n{actual:?}\n{expected:?}",
                expected.len()
            );
        }
        // Not all background: the large triangle covers whole texels.
        assert!(expected.iter().flatten().any(|&depth| depth > 0.0));
    }

    /// Draws `triangles`, each three clip-space corners, into the depth
    /// target of `render_targets`, as the depth pre-pass draws: cleared to
    /// 0, keeping the nearest.
    pub(crate) fn draw_depth(
        context: &GpuContext,
        render_targets: &RenderTargets,
        triangles: &[[[f32; 4]; 3]],
    ) {
        let device = &context.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Test Depth"),
            source: wgpu::ShaderSource::Wgsl(
                "@group(0) @binding(0) var<storage, read> corners: array<vec4<f32>>;
                @vertex
                fn corner(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                    return corners[index];
                }"
                .into(),
            ),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Depth"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("corner"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(targets::depth_stencil_state()),
            multisample: wgpu::MultisampleState {
                count: targets::SAMPLE_COUNT,
                ..wgpu::MultisampleState::default()
            },
            multiview_mask: None,
            cache: None,
        });
        let corners = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Test Depth Corners"),
            size: size_of_val(triangles) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        context
            .queue
            .write_buffer(&corners, 0, bytemuck::cast_slice(triangles));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Test Depth"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: corners.as_entire_binding(),
            }],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Test Depth"),
            color_attachments: &[],
            depth_stencil_attachment: Some(render_targets.depth_attachment()),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        render_pass.set_pipeline(&pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3 * triangles.len() as u32, 0..1);
        drop(render_pass);
        context.queue.submit([encoder.finish()]);
    }

    /// Each level of a pyramid of the minimum of `samples`, those of a
    /// `width` × `height` depth target, each pixel's together, rows top
    /// first: from the pixels, each level half the size of the one below,
    /// rounded down but at least 1, to 1 × 1, each texel the minimum of the
    /// texels below whose share of the screen overlaps its own, the rows and
    /// columns of every level sharing it evenly.
    fn minimum_depth_pyramid(samples: &[f32], width: u32, height: u32) -> Vec<Vec<f32>> {
        let samples_per_pixel = samples.len() / (width * height) as usize;
        let mut below: Vec<f32> = samples
            .chunks(samples_per_pixel)
            .map(|pixel| pixel.iter().copied().fold(f32::INFINITY, f32::min))
            .collect();
        let (mut below_width, mut below_height) = (width, height);
        // Whether texel `below_texel` of a row `below_size` long and texel
        // `texel` of one `size` long share part of the screen.
        let overlap = |texel: u32, size: u32, below_texel: u32, below_size: u32| {
            below_texel * size < (texel + 1) * below_size
                && (below_texel + 1) * size > texel * below_size
        };
        let mut levels = Vec::new();
        loop {
            let (level_width, level_height) = ((below_width / 2).max(1), (below_height / 2).max(1));
            let mut level = Vec::new();
            for y in 0..level_height {
                for x in 0..level_width {
                    let mut minimum = f32::INFINITY;
                    for below_y in 0..below_height {
                        for below_x in 0..below_width {
                            if overlap(y, level_height, below_y, below_height)
                                && overlap(x, level_width, below_x, below_width)
                            {
                                minimum =
                                    minimum.min(below[(below_y * below_width + below_x) as usize]);
                            }
                        }
                    }
                    level.push(minimum);
                }
            }
            levels.push(level.clone());
            if (level_width, level_height) == (1, 1) {
                return levels;
            }
            (below, below_width, below_height) = (level, level_width, level_height);
        }
    }

    /// Every sample of a multisampled depth target, each pixel's together,
    /// rows top first.
    pub(crate) fn read_depth_samples(context: &GpuContext, depth: &wgpu::TextureView) -> Vec<f32> {
        let texture = depth.texture();
        read_texture(
            context,
            "@group(0) @binding(0) var source: texture_depth_multisampled_2d;
            @group(0) @binding(1) var<storage, read_write> values: array<f32>;
            @compute @workgroup_size(1)
            fn copy(@builtin(global_invocation_id) pixel: vec3<u32>) {
                let samples = textureNumSamples(source);
                let first = (pixel.y * textureDimensions(source).x + pixel.x) * samples;
                for (var sample = 0u; sample < samples; sample++) {
                    values[first + sample] = textureLoad(source, pixel.xy, sample);
                }
            }",
            depth,
            [texture.width(), texture.height()],
            texture.sample_count(),
        )
    }

    /// The texels of `level`, a view of level `index` alone of an R32Float
    /// texture, rows top first.
    fn read_texels(context: &GpuContext, level: &wgpu::TextureView, index: u32) -> Vec<f32> {
        let size = level
            .texture()
            .size()
            .mip_level_size(index, wgpu::TextureDimension::D2);
        read_texture(
            context,
            "@group(0) @binding(0) var source: texture_2d<f32>;
            @group(0) @binding(1) var<storage, read_write> values: array<f32>;
            @compute @workgroup_size(1)
            fn copy(@builtin(global_invocation_id) texel: vec3<u32>) {
                values[texel.y * textureDimensions(source).x + texel.x] =
                    textureLoad(source, texel.xy, 0).r;
            }",
            level,
            [size.width, size.height],
            1,
        )
    }

    /// The f32s `kernel` copies from `view`, with one invocation of its `copy`
    /// entry point per texel of its `size`, each writing `values_per_texel`.
    pub(crate) fn read_texture(
        context: &GpuContext,
        kernel: &str,
        view: &wgpu::TextureView,
        size: [u32; 2],
        values_per_texel: u32,
    ) -> Vec<f32> {
        let device = &context.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Texture Read-back"),
            source: wgpu::ShaderSource::Wgsl(kernel.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Texture Read-back"),
            layout: None,
            module: &module,
            entry_point: Some("copy"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let [width, height] = size;
        let count = width * height * values_per_texel;
        let values = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Texture Read-back Values"),
            size: u64::from(count) * size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Read-back"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: values.as_entire_binding(),
                },
            ],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(width, height, 1);
        drop(pass);
        context.queue.submit([encoder.finish()]);
        read_storage(context, &values, count as usize)
    }
}
