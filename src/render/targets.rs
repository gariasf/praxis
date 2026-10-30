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

/// The shadow mask's format: a word of two halves per pixel and surface, a
/// bit per light and the Froxel slice the bits follow.
pub const SHADOW_MASK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Uint;

/// The surfaces of a pixel the shadow mask holds a word for, a layer each:
/// the one its nearest sample shows, and the farthest other one where its
/// samples show more than one, as surface-based antialiasing stores a
/// small, fixed number of surfaces per pixel and shades each once
/// [ref:salvi-2012-sbaa-surfaces].
pub const SHADOW_MASK_SURFACES: u32 = 2;

/// The format of the shadow mask's samples: a bit per sample of a pixel,
/// set where the sample shows the surface of the mask's second layer.
pub const SHADOW_MASK_SAMPLES_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;

/// The surface normals' format: each sample's triangle normal, then the
/// normal interpolated over the triangle, as octahedral coordinates
/// (`octahedral_encode`, common.wgsl) of 8 bits each, mapped from [-1, 1].
pub const SURFACE_NORMALS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

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
    /// Which lights the shadow mask pass found hidden at each pixel, from
    /// each of its surfaces, which the lit pass reads.
    shadow_mask: wgpu::TextureView,
    /// Which samples of each pixel show the shadow mask's second surface.
    shadow_mask_samples: wgpu::TextureView,
    /// The shadow mask pass's list of the pixels whose samples show a second
    /// surface: a count, then an entry per pixel at most.
    second_surfaces: wgpu::Buffer,
    /// Prototype (#364): per pixel, the nearest surface's word, its second
    /// samples and the edge pass's extra sample count.
    shadow_edge_pixels: wgpu::Buffer,
    /// Prototype (#364): per pixel, the lights its nearest surface traced a
    /// ray toward, then, at edges, 2-bit counts of hidden extra samples per
    /// light as two bit planes.
    shadow_edge_coverage: wgpu::Buffer,
    /// Prototype (#364): the edge pixels' list, as `second_surfaces`.
    shadow_edges: wgpu::Buffer,
    /// Each sample's surface normals, which the depth passes write with
    /// shadow rays on and the shadow mask pass reads.
    surface_normals: wgpu::TextureView,
}

impl RenderTargets {
    /// Targets `width` × `height`, both non-zero.
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> RenderTargets {
        // The second surfaces' list holds a pixel's row in 14 bits
        // (`shadow_mask.wgsl`).
        assert!(height <= 1 << 14, "targets {height} pixels tall");
        let layered_target = |label, format, sample_count, layers, usage| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            });
            let dimension = if layers > 1 {
                wgpu::TextureViewDimension::D2Array
            } else {
                wgpu::TextureViewDimension::D2
            };
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(dimension),
                ..wgpu::TextureViewDescriptor::default()
            })
        };
        let target = |label, format, sample_count, usage| {
            layered_target(label, format, sample_count, 1, usage)
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
            shadow_mask: layered_target(
                "Shadow Mask",
                SHADOW_MASK_FORMAT,
                1,
                SHADOW_MASK_SURFACES,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            shadow_mask_samples: target(
                "Shadow Mask Samples",
                SHADOW_MASK_SAMPLES_FORMAT,
                1,
                wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            second_surfaces: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Second Surfaces Buffer"),
                size: (u64::from(width) * u64::from(height) + 1) * size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            shadow_edge_pixels: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Edge Pixels Buffer"),
                size: u64::from(width) * u64::from(height) * 16,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            shadow_edge_coverage: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Edge Coverage Buffer"),
                size: u64::from(width) * u64::from(height) * 16,
                usage: wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }),
            shadow_edges: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Shadow Edges Buffer"),
                size: (3 * u64::from(width) * u64::from(height) + 1) * size_of::<u32>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            surface_normals: target(
                "Surface Normals",
                SURFACE_NORMALS_FORMAT,
                SAMPLE_COUNT,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
        }
    }

    /// The surface normals.
    pub fn surface_normals(&self) -> &wgpu::TextureView {
        &self.surface_normals
    }

    /// The surface normals as a depth pass writes them: cleared where
    /// `clear`, loaded otherwise, and stored for the shadow mask pass.
    pub fn surface_normals_attachment(&self, clear: bool) -> wgpu::RenderPassColorAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view: &self.surface_normals,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
        }
    }

    /// The depth target, as passes read it.
    pub fn depth(&self) -> &wgpu::TextureView {
        &self.depth
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

    /// The shadow mask, every layer.
    pub fn shadow_mask(&self) -> &wgpu::TextureView {
        &self.shadow_mask
    }

    /// The samples of each pixel that show the shadow mask's second surface.
    pub fn shadow_mask_samples(&self) -> &wgpu::TextureView {
        &self.shadow_mask_samples
    }

    /// The shadow mask pass's list of second surfaces.
    pub fn second_surfaces(&self) -> &wgpu::Buffer {
        &self.second_surfaces
    }

    pub fn shadow_edge_pixels(&self) -> &wgpu::Buffer {
        &self.shadow_edge_pixels
    }

    pub fn shadow_edge_coverage(&self) -> &wgpu::Buffer {
        &self.shadow_edge_coverage
    }

    pub fn shadow_edges(&self) -> &wgpu::Buffer {
        &self.shadow_edges
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

    /// Bytes of the targets' textures and buffer.
    pub fn gpu_memory_bytes(&self, transient_saves_memory: bool) -> u64 {
        [
            &self.depth,
            &self.color,
            &self.hdr,
            &self.depth_pyramid_levels[0],
            &self.shadow_mask,
            &self.shadow_mask_samples,
            &self.surface_normals,
        ]
        .into_iter()
        .map(|view| texture_bytes(view.texture(), transient_saves_memory))
        .sum::<u64>()
            + self.second_surfaces.size()
            + self.shadow_edge_pixels.size()
            + self.shadow_edge_coverage.size()
            + self.shadow_edges.size()
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
