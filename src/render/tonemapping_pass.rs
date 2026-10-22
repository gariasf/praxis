//! The tonemapping pass: one triangle over the target that reads each pixel
//! of the HDR target and writes the pixel's display colour, sRGB-encoded and
//! dithered.

use crate::render::LoadStaging;
use crate::render::bind_groups::{TONEMAPPING_INPUT, pipeline_layout};
use crate::render::shaders;
use crate::render::targets::RenderTargets;
use crate::stats::texture_bytes;

/// The tonemapping pass's label, and its name in frame stats.
pub(crate) const TONEMAPPING_PASS: &str = "tonemapping_pass";

/// The side of the dither's mask, in texels: a power of two, so the shader
/// tiles the screen with it by a bit mask, where a modulo by the texture's
/// size cost about 38 µs at 1440p on the M4 Pro.
pub(crate) const DITHER_MASK_SIDE: u32 = 64;
const _: () = assert!(DITHER_MASK_SIDE.is_power_of_two());

/// The dither's blue-noise mask, rows top first: the red channel of
/// Peters's `assets/BlueNoise/LDR_LLL1_0.png` (CC0), compiled in, as a build
/// without `cook` links no PNG decoder and ADR 0018's cook would store it
/// lossy.
const DITHER_MASK: &[u8; (DITHER_MASK_SIDE * DITHER_MASK_SIDE) as usize] =
    include_bytes!("dither_mask.bin");

/// The pass's pipeline, which draws into views of one format, its dither
/// mask and its bind group.
pub struct TonemappingPass {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    dither_mask: wgpu::TextureView,
    /// Its group 3 (`TONEMAPPING_INPUT`), built from the render targets by
    /// `bind_render_targets`.
    input: wgpu::BindGroup,
}

impl TonemappingPass {
    /// The pass for views of `target_format`, which is not sRGB: the pass
    /// encodes sRGB itself, so its dither lands on 8-bit steps. It reads the
    /// HDR target of `render_targets` and loads its dither mask through
    /// `staging`.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        render_targets: &RenderTargets,
        staging: &mut LoadStaging,
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
        let dither_mask = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dither Mask"),
            size: wgpu::Extent3d {
                width: DITHER_MASK_SIDE,
                height: DITHER_MASK_SIDE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        staging.write_texture(&dither_mask, 0, DITHER_MASK);
        let dither_mask = dither_mask.create_view(&wgpu::TextureViewDescriptor::default());
        let input = Self::input_bind_group(device, &layout, render_targets, &dither_mask);
        TonemappingPass {
            pipeline,
            layout,
            dither_mask,
            input,
        }
    }

    /// Builds the bind group that reads `render_targets`: at creation, and
    /// after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.input =
            Self::input_bind_group(device, &self.layout, render_targets, &self.dither_mask);
    }

    /// The bind group that reads `render_targets`' HDR target and the dither
    /// mask.
    fn input_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        render_targets: &RenderTargets,
        dither_mask: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        TONEMAPPING_INPUT.bind_group(
            device,
            layout,
            [
                (
                    "hdr_target",
                    wgpu::BindingResource::TextureView(render_targets.hdr()),
                ),
                (
                    "dither_mask",
                    wgpu::BindingResource::TextureView(dither_mask),
                ),
            ],
        )
    }

    /// Bytes of its dither mask.
    pub fn gpu_memory_bytes(&self) -> u64 {
        texture_bytes(self.dither_mask.texture(), false)
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

    /// The mask the pass dithers with is the red channel of Peters's
    /// `assets/BlueNoise/LDR_LLL1_0.png` as committed, rows top first, and
    /// holds each of its 256 levels 16 times, so its values are uniform.
    #[test]
    fn the_dither_mask_is_the_red_channel_of_peterss_mask() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/BlueNoise/LDR_LLL1_0.png");
        let mask = image::open(path).expect("decode the mask").to_rgba8();
        let red: Vec<u8> = mask.pixels().map(|pixel| pixel.0[0]).collect();
        assert_eq!((mask.width(), mask.height()), (64, 64));
        assert!(red == DITHER_MASK, "the compiled mask is not the asset's");

        let mut counts = [0; 256];
        for &level in DITHER_MASK {
            counts[usize::from(level)] += 1;
        }
        assert!(counts.iter().all(|&count| count == 16), "{counts:?}");
    }

    /// The pass alone, after a lit pass that draws nothing and only clears,
    /// so the HDR target holds one exposed colour, weighted: every pixel of
    /// an 8-bit target shows it as GT7 maps it, the sRGB-encoded outputs of
    /// Polyphony's sample implementation between Rec. 709 and Rec. 2020 (as
    /// `tonemap`'s oracle takes them), within a step: the dither's, and fp16
    /// and a GPU's transcendentals moving a value near a step's edge, as
    /// lavapipe shows the last case's 132.65 as 132. Black, a grey in GT7's
    /// linear section, which keeps its exposure, the Reference scenes'
    /// background as in `tests/capture.rs`, red, and a red past the paper
    /// white, which whitens.
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
        let mut staging = LoadStaging::new(device.clone());
        let pass = TonemappingPass::new(device, format, &render_targets, &mut staging);
        staging.submit(&context.queue);
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

    /// The dither keeps a flat colour's mean: over the mask's 64 × 64 tile,
    /// a grey whose sRGB code GT7 and the encoding put at 153.32 shows 152,
    /// 153 and 154, averaging 153.32 within 0.02, where rounding alone would
    /// show 153 everywhere (exposed 0.32 in GT7's linear section).
    #[test]
    fn the_dither_spreads_a_flat_colour_over_three_steps_keeping_its_mean() {
        let Some(context) = gpu_context() else {
            return;
        };
        let device = &context.device;
        let size = wgpu::Extent3d {
            width: DITHER_MASK_SIDE,
            height: DITHER_MASK_SIDE,
            depth_or_array_layers: 1,
        };
        let render_targets = RenderTargets::new(device, size.width, size.height);
        let mut staging = LoadStaging::new(device.clone());
        let pass = TonemappingPass::new(device, TARGET_FORMAT, &render_targets, &mut staging);
        staging.submit(&context.queue);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Dither Test Target"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Dither Test Encoder"),
        });
        let grey = wgpu::Color {
            r: 0.32,
            g: 0.32,
            b: 0.32,
            a: 1.0,
        };
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Only Lit Pass"),
            color_attachments: &[Some(render_targets.color_attachment(grey))],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.record(
            &mut encoder,
            &target.create_view(&wgpu::TextureViewDescriptor::default()),
            None,
        );
        context.queue.submit(std::iter::once(encoder.finish()));
        let pixels = read_back(&context, &target).expect("read back failed");

        let reds: Vec<u8> = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| pixel[0])
            .collect();
        let mut codes: Vec<u8> = reds.clone();
        codes.sort_unstable();
        codes.dedup();
        let mean = reds.iter().map(|&code| f64::from(code)).sum::<f64>() / reds.len() as f64;
        assert_eq!(codes, [152, 153, 154]);
        assert!((mean - 153.32).abs() <= 0.02, "mean code {mean}");
    }
}
