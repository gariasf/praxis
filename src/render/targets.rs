//! The targets whose size is the frame's, and the depth and colour
//! conventions every pass follows. Depth is reverse-Z with an infinite far
//! plane (ADR 0015): it clears to 0, the far plane, nearer is greater, and a
//! Hi-Z pyramid keeps the minimum, the farthest. The main pass draws at 4x
//! MSAA (ADR 0019) and resolves into the HDR target, which holds exposed
//! colour weighted by 1/(1 + luminance), so the resolve averages it in
//! display space (`resolve_weighted`, hdr.wgsl).

use crate::stats::texture_bytes;

/// The depth target's format.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Samples per pixel of the depth target and the multisampled colour, which
/// every pipeline that draws into them matches.
pub const SAMPLE_COUNT: u32 = 4;

/// The depth pyramid's format: a depth per texel.
pub const DEPTH_PYRAMID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

/// The HDR target's format, and the multisampled colour's.
pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The shadow mask's format: a word of two halves per pixel, a bit per light
/// and the Froxel slice the bits follow.
pub const SHADOW_MASK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Uint;

/// The visibility IDs' format, with the `visibility_prepass` Render switch
/// on: a 32-bit triangle ID per sample in two halves, since WebGPU's
/// guaranteed format features multisample Rg16Uint but not R32Uint.
pub const VISIBILITY_ID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Uint;

/// The visibility IDs' clear value: no triangle, every bit set.
const NO_TRIANGLE: wgpu::Color = wgpu::Color {
    r: 65535.0,
    g: 65535.0,
    b: 0.0,
    a: 0.0,
};

/// The luminance of a linear colour in Rec. 709's primaries, the scene's, per
/// unit of each: ITU-R BT.709-6's luma weights (item 3.2), which are the Y
/// row of its linear RGB-to-XYZ matrix as ITU-R BT.2087-0 prints it
/// [ref:bt709-6-luma] [ref:bt2087-rec709-to-xyz]; WGSL declares them as
/// `LUMINANCE_COEFFICIENTS`.
pub const LUMINANCE_COEFFICIENTS: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// Every size-dependent target, created together by `new`, which a resize
/// calls again. Bind groups are snapshots, not pointers: a pass builds the
/// bind groups that read targets in one function, which it calls at
/// creation and after every resize.
pub struct RenderTargets {
    depth: wgpu::TextureView,
    /// A transient attachment, memoryless where the adapter allows: the pass
    /// that clears it resolves it into `hdr` before it ends.
    color: wgpu::TextureView,
    /// The lit pass's colour, resolved, which the tonemapping pass reads.
    hdr: wgpu::TextureView,
    /// The depth pyramid, every level, as phase two of the culling pass reads
    /// it.
    depth_pyramid: wgpu::TextureView,
    /// A view of each level of the depth pyramid, the Hi-Z pyramid that
    /// holds the minimum depth over each share of the screen: level 0 half
    /// the targets' size, rounded down, and each level half the one below,
    /// to 1 × 1, the rows and columns of each level sharing the screen
    /// evenly.
    depth_pyramid_levels: Vec<wgpu::TextureView>,
    /// Which lights the shadow mask pass found hidden at each pixel, which
    /// the lit pass reads.
    shadow_mask: wgpu::TextureView,
    /// With the `visibility_prepass` Render switch on, the triangle each
    /// sample of the depth target shows, which the visibility ID pass writes
    /// and the visibility pass reads.
    visibility_ids: Option<wgpu::TextureView>,
}

impl RenderTargets {
    /// Targets `width` × `height`, both non-zero, with visibility IDs where
    /// `visibility_prepass` is set.
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        visibility_prepass: bool,
    ) -> RenderTargets {
        let target = |label, format, sample_count, usage| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            });
            texture.create_view(&wgpu::TextureViewDescriptor::default())
        };
        let pyramid_size = wgpu::Extent3d {
            width: (width / 2).max(1),
            height: (height / 2).max(1),
            depth_or_array_layers: 1,
        };
        let depth_pyramid = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Pyramid"),
            size: pyramid_size,
            mip_level_count: pyramid_size.max_mips(wgpu::TextureDimension::D2),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_PYRAMID_FORMAT,
            // Each level is written as storage, then read to reduce the next.
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        RenderTargets {
            // Read by the depth pyramid pass.
            depth: target(
                "Depth Texture",
                DEPTH_FORMAT,
                SAMPLE_COUNT,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            color: target(
                "Multisampled Colour Texture",
                HDR_FORMAT,
                SAMPLE_COUNT,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TRANSIENT_ATTACHMENT,
            ),
            hdr: target(
                "HDR Texture",
                HDR_FORMAT,
                1,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            depth_pyramid: depth_pyramid.create_view(&wgpu::TextureViewDescriptor::default()),
            depth_pyramid_levels: (0..depth_pyramid.mip_level_count())
                .map(|level| {
                    depth_pyramid.create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: level,
                        mip_level_count: Some(1),
                        ..wgpu::TextureViewDescriptor::default()
                    })
                })
                .collect(),
            // Written as storage, then read by the lit pass.
            shadow_mask: target(
                "Shadow Mask",
                SHADOW_MASK_FORMAT,
                1,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            // Drawn by the visibility ID pass, then read by the visibility
            // pass.
            visibility_ids: visibility_prepass.then(|| {
                target(
                    "Visibility IDs",
                    VISIBILITY_ID_FORMAT,
                    SAMPLE_COUNT,
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                )
            }),
        }
    }

    /// The visibility IDs, with the `visibility_prepass` Render switch on.
    pub fn visibility_ids(&self) -> Option<&wgpu::TextureView> {
        self.visibility_ids.as_ref()
    }

    /// The visibility IDs as the visibility ID pass's colour attachment,
    /// with the `visibility_prepass` Render switch on: cleared to no
    /// triangle, and stored for the visibility pass.
    pub fn visibility_ids_attachment(&self) -> Option<wgpu::RenderPassColorAttachment<'_>> {
        self.visibility_ids
            .as_ref()
            .map(|view| wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(NO_TRIANGLE),
                    store: wgpu::StoreOp::Store,
                },
            })
    }

    /// The depth target as the attachment of a pass that reads it at depth
    /// equal and writes nothing: read-only, so the pass stores nothing back.
    pub fn read_only_depth_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: None,
            stencil_ops: None,
        }
    }

    /// The depth target, as passes read it.
    pub fn depth(&self) -> &wgpu::TextureView {
        &self.depth
    }

    /// The targets' size, in pixels.
    pub fn size(&self) -> (u32, u32) {
        let size = self.depth.texture().size();
        (size.width, size.height)
    }

    /// The HDR target, as the tonemapping pass reads it.
    pub fn hdr(&self) -> &wgpu::TextureView {
        &self.hdr
    }

    /// The depth pyramid, every level.
    pub fn depth_pyramid(&self) -> &wgpu::TextureView {
        &self.depth_pyramid
    }

    /// A view of each level of the depth pyramid, from level 0.
    pub fn depth_pyramid_levels(&self) -> &[wgpu::TextureView] {
        &self.depth_pyramid_levels
    }

    /// The shadow mask.
    pub fn shadow_mask(&self) -> &wgpu::TextureView {
        &self.shadow_mask
    }

    /// The depth target as the attachment of the frame's first pass that
    /// draws depth: cleared to 0, the far plane, and stored.
    pub fn depth_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(0.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }

    /// The depth target as the attachment of a pass that draws more depth
    /// for a later pass to read: loaded, and stored.
    pub fn loaded_depth_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }

    /// The depth target as the attachment of the frame's last pass that
    /// draws depth: loaded, and discarded unless `kept` for a later pass to
    /// read. A pass that writes depth and stores it writes the whole target
    /// back, about 50 µs at 1440p on the M4 Pro.
    pub fn final_depth_attachment(&self, kept: bool) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: if kept {
                    wgpu::StoreOp::Store
                } else {
                    wgpu::StoreOp::Discard
                },
            }),
            stencil_ops: None,
        }
    }

    /// The multisampled colour as a pass's attachment: cleared to `clear`,
    /// exposed colour, clamped and weighted as the lit pass clamps and weights
    /// what it draws (`resolve_weighted`, hdr.wgsl), and resolved at the
    /// pass's end into the HDR target; its samples are discarded, as a
    /// transient attachment's must be.
    pub fn color_attachment(&self, clear: wgpu::Color) -> wgpu::RenderPassColorAttachment<'_> {
        // fp16's largest finite value, so no backend stores infinity
        // (ADR 0023).
        let largest_half = 65504.0;
        let [red, green, blue] =
            [clear.r, clear.g, clear.b].map(|channel| channel.min(largest_half));
        let coefficients = LUMINANCE_COEFFICIENTS.map(f64::from);
        let luminance = coefficients[0] * red + coefficients[1] * green + coefficients[2] * blue;
        let weight = 1.0 / (1.0 + luminance);
        wgpu::RenderPassColorAttachment {
            view: &self.color,
            resolve_target: Some(&self.hdr),
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: red * weight,
                    g: green * weight,
                    b: blue * weight,
                    a: clear.a,
                }),
                store: wgpu::StoreOp::Discard,
            },
        }
    }

    /// Bytes of the targets' textures.
    pub fn gpu_memory_bytes(&self, transient_saves_memory: bool) -> u64 {
        [
            &self.depth,
            &self.color,
            &self.hdr,
            &self.depth_pyramid_levels[0],
            &self.shadow_mask,
        ]
        .into_iter()
        .chain(self.visibility_ids.as_ref())
        .map(|view| texture_bytes(view.texture(), transient_saves_memory))
        .sum()
    }
}

/// The depth state of a pipeline that keeps the nearest depth: it passes
/// `Greater`, nearer being greater, and writes.
pub fn depth_stencil_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: Some(true),
        depth_compare: Some(wgpu::CompareFunction::Greater),
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

/// The depth state of a pipeline that draws only what a depth pre-pass kept:
/// it passes `Equal` and writes nothing.
pub fn depth_equal_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        depth_write_enabled: Some(false),
        depth_compare: Some(wgpu::CompareFunction::Equal),
        ..depth_stencil_state()
    }
}
