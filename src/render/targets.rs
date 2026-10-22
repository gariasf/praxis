//! The targets whose size is the frame's, and the depth conventions every
//! pass follows. Depth is reverse-Z with an infinite far plane (ADR 0015):
//! it clears to 0, the far plane, nearer is greater, and a Hi-Z pyramid
//! keeps the minimum, the farthest. The main pass draws at 4x MSAA
//! (ADR 0019).

use crate::stats::texture_bytes;

/// The depth target's format.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Samples per pixel of the depth target and the multisampled colour, which
/// every pipeline that draws into them matches.
pub const SAMPLE_COUNT: u32 = 4;

/// The depth pyramid's format: a depth per texel.
pub const DEPTH_PYRAMID_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

/// Every size-dependent target, created together by `new`, which a resize
/// calls again. Bind groups are snapshots, not pointers: a pass builds the
/// bind groups that read targets in one function, which it calls at
/// creation and after every resize.
pub struct RenderTargets {
    depth: wgpu::TextureView,
    /// A transient attachment, memoryless where the adapter allows: the pass
    /// that clears it resolves it into the target before it ends.
    color: wgpu::TextureView,
    /// The depth pyramid, every level, as phase two of the culling pass reads
    /// it.
    depth_pyramid: wgpu::TextureView,
    /// A view of each level of the depth pyramid, the Hi-Z pyramid that
    /// holds the minimum depth over each share of the screen: level 0 half
    /// the targets' size, rounded down, and each level half the one below,
    /// to 1 × 1, the rows and columns of each level sharing the screen
    /// evenly.
    depth_pyramid_levels: Vec<wgpu::TextureView>,
}

impl RenderTargets {
    /// Targets `width` × `height`, both non-zero, whose colour resolves into
    /// views of `target_format`.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> RenderTargets {
        let attachment = |label, format, usage| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
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
            depth: attachment(
                "Depth Texture",
                DEPTH_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            color: attachment(
                "Multisampled Colour Texture",
                target_format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TRANSIENT_ATTACHMENT,
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
        }
    }

    /// The depth target, as passes read it.
    pub fn depth(&self) -> &wgpu::TextureView {
        &self.depth
    }

    /// The depth pyramid, every level.
    pub fn depth_pyramid(&self) -> &wgpu::TextureView {
        &self.depth_pyramid
    }

    /// A view of each level of the depth pyramid, from level 0.
    pub fn depth_pyramid_levels(&self) -> &[wgpu::TextureView] {
        &self.depth_pyramid_levels
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

    /// The depth target as the attachment of the frame's last pass that
    /// draws depth: loaded, and discarded, since no later pass reads it. A
    /// pass that writes depth and stores it writes the whole target back,
    /// about 50 µs at 1440p on the M4 Pro.
    pub fn final_depth_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
        }
    }

    /// The multisampled colour as a pass's attachment: cleared to `clear`,
    /// and resolved at the pass's end into `target`, a view of the targets'
    /// size and colour format; its samples are discarded, as a transient
    /// attachment's must be.
    pub fn color_attachment<'a>(
        &'a self,
        target: &'a wgpu::TextureView,
        clear: wgpu::Color,
    ) -> wgpu::RenderPassColorAttachment<'a> {
        wgpu::RenderPassColorAttachment {
            view: &self.color,
            resolve_target: Some(target),
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(clear),
                store: wgpu::StoreOp::Discard,
            },
        }
    }

    /// Bytes of the targets' textures.
    pub fn gpu_memory_bytes(&self, transient_saves_memory: bool) -> u64 {
        [&self.depth, &self.color, &self.depth_pyramid_levels[0]]
            .into_iter()
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
