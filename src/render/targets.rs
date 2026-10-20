//! The targets whose size is the frame's, and the depth conventions every
//! pass follows. Depth is reverse-Z with an infinite far plane (ADR 0015):
//! it clears to 0, the far plane, nearer is greater, and a Hi-Z pyramid
//! keeps the minimum, the farthest.

use crate::stats::texture_bytes;

/// The depth target's format.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Samples per pixel of the targets passes draw into.
pub const SAMPLE_COUNT: u32 = 1;

/// Every size-dependent target, created together by `new`, which a resize
/// calls again. Bind groups are snapshots, not pointers: a pass builds the
/// bind groups that read targets in one function, which it calls at
/// creation and after every resize.
pub struct RenderTargets {
    depth: wgpu::TextureView,
}

impl RenderTargets {
    /// Targets `width` × `height`, both non-zero.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> RenderTargets {
        let depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: SAMPLE_COUNT,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        RenderTargets {
            depth: depth.create_view(&wgpu::TextureViewDescriptor::default()),
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

    /// Bytes of the targets' textures.
    pub fn gpu_memory_bytes(&self, transient_saves_memory: bool) -> u64 {
        texture_bytes(self.depth.texture(), transient_saves_memory)
    }
}

/// The depth state of a pipeline that keeps the nearest surface: it passes
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
