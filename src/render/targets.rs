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

/// Every size-dependent target, created together by `new`, which a resize
/// calls again. Bind groups are snapshots, not pointers: a pass builds the
/// bind groups that read targets in one function, which it calls at
/// creation and after every resize.
pub struct RenderTargets {
    depth: wgpu::TextureView,
    /// A transient attachment, memoryless where the adapter allows: the pass
    /// that clears it resolves it into the target before it ends.
    color: wgpu::TextureView,
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
        RenderTargets {
            depth: attachment(
                "Depth Texture",
                DEPTH_FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            ),
            color: attachment(
                "Multisampled Colour Texture",
                target_format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TRANSIENT_ATTACHMENT,
            ),
        }
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

    /// The depth target as the attachment of a pass that only tests against
    /// it: read-only, so wgpu refuses any pipeline of the pass that writes it.
    pub fn read_only_depth_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth,
            depth_ops: None,
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
        [&self.depth, &self.color]
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
