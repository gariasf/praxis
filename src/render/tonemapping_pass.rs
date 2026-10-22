//! The tonemapping pass: one triangle over the target that reads each pixel
//! of the HDR target and writes the pixel's display colour, sRGB-encoded.

use crate::render::bind_groups::{TONEMAPPING_INPUT, pipeline_layout};
use crate::render::shaders;
use crate::render::targets::RenderTargets;

/// The tonemapping pass's label, and its name in frame stats.
pub(crate) const TONEMAPPING_PASS: &str = "tonemapping_pass";

/// The pass's pipeline, which draws into views of one format, and its bind
/// group.
pub struct TonemappingPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    /// Its group 3 (`TONEMAPPING_INPUT`), built from the render targets by
    /// `bind_render_targets`.
    input: wgpu::BindGroup,
}

impl TonemappingPass {
    /// The pass for views of `target_format`, which is not sRGB: the pass
    /// encodes sRGB itself. It reads the HDR target of `render_targets`.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        render_targets: &RenderTargets,
    ) -> TonemappingPass {
        debug_assert!(
            !target_format.is_srgb(),
            "the tonemapping pass encodes sRGB itself, so writes no sRGB view"
        );
        let shader = device.create_shader_module(shaders::TONEMAPPING.descriptor());
        let layout = TONEMAPPING_INPUT.layout(device);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tonemapping Pipeline"),
            layout: Some(&pipeline_layout(
                device,
                "Tonemapping Pipeline Layout",
                &[(&TONEMAPPING_INPUT, &layout)],
            )),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_full_screen"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_tonemap"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let input = Self::input_bind_group(device, &layout, render_targets);
        TonemappingPass {
            pipeline,
            layout,
            input,
        }
    }

    /// Builds the bind group that reads `render_targets`: at creation, and
    /// after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.input = Self::input_bind_group(device, &self.layout, render_targets);
    }

    /// The bind group that reads `render_targets`' HDR target.
    fn input_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        render_targets: &RenderTargets,
    ) -> wgpu::BindGroup {
        TONEMAPPING_INPUT.bind_group(
            device,
            layout,
            [(
                "hdr_target",
                wgpu::BindingResource::TextureView(render_targets.hdr()),
            )],
        )
    }

    /// Records the pass into `encoder`: every pixel of `target`, a view in
    /// this pass's format and the render targets' size, takes its display
    /// colour, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(TONEMAPPING_PASS),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                depth_slice: None,
                // The triangle covers every pixel; a clear spares the load.
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(TONEMAPPING_INPUT.index, &self.input, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::pools::tests::gpu_context;
    use crate::capture::{TARGET_FORMAT, read_back};

    /// The pass alone, after a lit pass that draws nothing and only clears,
    /// so the HDR target holds one exposed colour, weighted: every pixel of
    /// an 8-bit target shows it as GT7 maps it, the sRGB-encoded outputs
    /// of Polyphony's sample implementation between Rec. 709 and Rec. 2020
    /// (as `tonemap`'s oracle takes them), within a step, as fp16 and a
    /// GPU's transcendentals move a value near a step's edge: lavapipe shows
    /// the last case's 132.65 as 132. Black stays black, a grey in GT7's
    /// linear section keeps its exposure, the Reference scenes' background
    /// shows as in `tests/capture.rs`, red stays red, and a red past the
    /// paper white whitens.
    #[test]
    fn the_pass_shows_each_pixel_of_the_hdr_target_as_gt7_maps_it() {
        let Some(context) = gpu_context() else {
            return;
        };
        let device = &context.device;
        let format = TARGET_FORMAT;
        let size = wgpu::Extent3d {
            width: 5,
            height: 3,
            depth_or_array_layers: 1,
        };
        let render_targets = RenderTargets::new(device, size.width, size.height);
        let pass = TonemappingPass::new(device, format, &render_targets);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Tonemapping Test Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        // Exposed colours and their sRGB codes: 0, 127.8, (87.6, 123.4,
        // 148.6), 249.1, and (255.0, 188.0, 132.7).
        let cases = [
            ([0.0, 0.0, 0.0], [0, 0, 0]),
            ([0.2152; 3], [128; 3]),
            ([0.1, 0.2, 0.3], [88, 123, 149]),
            ([1.0, 0.0, 0.0], [249, 0, 0]),
            ([8.0, 0.0, 0.0], [255, 188, 133]),
        ];

        for ([red, green, blue], expected) in cases {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Tonemapping Test Encoder"),
            });
            let clear = wgpu::Color {
                r: red,
                g: green,
                b: blue,
                a: 1.0,
            };
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Only Lit Pass"),
                color_attachments: &[Some(render_targets.color_attachment(clear))],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            pass.record(&mut encoder, &view, None);
            context.queue.submit(std::iter::once(encoder.finish()));
            let pixels = read_back(&context, &target).expect("read back failed");

            for pixel in pixels.as_chunks::<4>().0 {
                assert!(
                    pixel
                        .iter()
                        .zip(expected)
                        .all(|(&channel, expected)| { channel.abs_diff(expected) <= 1 }),
                    "exposed ({red}, {green}, {blue}) shows {pixel:?}"
                );
            }
        }
    }
}
