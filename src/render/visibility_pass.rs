//! The visibility pass: with the visibility pre-pass Render switch on, a
//! compute pass after the visibility ID pass that marks the triangles of the
//! frame's draw that own a sample of the visibility IDs, one bit each, or
//! with Meshlet granularity every triangle of their Meshlets, and compacts
//! the marked triangles' packed indices, in the frame's draw's order, into
//! the compacted draw, which the lit pass draws in place of the frame's
//! draw: a prototype of option B of
//! `docs/research/visibility-and-gpu-driven-main-pass.md`.

use crate::render::bind_groups::{VISIBILITY_BUILD, pipeline_layout};
use crate::render::culling_pass::{CullingPass, DrawCapacity, SCAN_BLOCK_SIZE};
use crate::render::renderer::VisibilityPrepass;
use crate::render::shaders;
use crate::render::targets::{RenderTargets, SAMPLE_COUNT};

/// The visibility pass's label, and its name in frame stats.
pub(crate) const VISIBILITY_PASS: &str = "visibility_pass";

/// Pixels along each side of the tile each workgroup marks.
pub(crate) const VISIBILITY_TILE_SIZE: u32 = 8;

/// The WGSL `override` that sets the pass's granularity (`visibility.wgsl`).
const MESHLET_GRANULARITY: &str = "MESHLET_GRANULARITY";

// The pass loads each pixel's samples by index (`visibility.wgsl`).
const _: () = assert!(SAMPLE_COUNT == 4);

/// The pass's pipelines, one per step, and the buffers it reads and writes.
pub struct VisibilityPass {
    mark_triangles: wgpu::ComputePipeline,
    offset_words: wgpu::ComputePipeline,
    offset_blocks: wgpu::ComputePipeline,
    compact_triangles: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
    buffers: VisibilityBuffers,
    /// Built from the render targets by `bind_render_targets`.
    bind_group: wgpu::BindGroup,
    /// The targets' size, in pixels.
    size: (u32, u32),
    /// Blocks of `SCAN_BLOCK_SIZE` words of triangle bits.
    block_count: u32,
}

/// The buffers of the pass's group 3: the culling pass's frame draw and
/// packed indices, kept to rebuild the group, and the pass's own.
struct VisibilityBuffers {
    frame_draw: wgpu::Buffer,
    packed_indices: wgpu::Buffer,
    /// A bit per Meshlet record the frame's draw can hold, set with Meshlet
    /// granularity where some sample shows one of its triangles; cleared
    /// before the pass.
    record_bits: wgpu::Buffer,
    /// A bit per triangle the frame's draw can hold, set where some sample
    /// shows it; cleared before the pass.
    triangle_bits: wgpu::Buffer,
    /// Each word's first compacted triangle within its block, then each
    /// block's within the draw.
    triangle_offsets: wgpu::Buffer,
    compacted_indices: wgpu::Buffer,
    /// The compacted draw's indirect arguments (`DrawArguments`).
    compacted_draw: wgpu::Buffer,
}

impl VisibilityPass {
    /// The pass, reading `culling_pass`'s frame draw and packed indices, of
    /// `capacity`, and the visibility IDs of `render_targets`, compacting
    /// at `granularity`.
    pub fn new(
        device: &wgpu::Device,
        culling_pass: &CullingPass,
        capacity: DrawCapacity,
        render_targets: &RenderTargets,
        granularity: VisibilityPrepass,
    ) -> VisibilityPass {
        let word_count = (capacity.packed_indices / 3).div_ceil(u32::BITS);
        let block_count = word_count.div_ceil(SCAN_BLOCK_SIZE);
        let buffer = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let buffers = VisibilityBuffers {
            frame_draw: culling_pass.frame_draw().clone(),
            packed_indices: culling_pass.packed_indices().clone(),
            record_bits: buffer(
                "Record Bit Buffer",
                u64::from(capacity.meshlet_records.div_ceil(u32::BITS)) * size_of::<u32>() as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            ),
            // Cleared before the pass, so COPY_DST.
            triangle_bits: buffer(
                "Triangle Bit Buffer",
                u64::from(word_count) * size_of::<u32>() as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            ),
            triangle_offsets: buffer(
                "Triangle Offset Buffer",
                u64::from(word_count + block_count) * size_of::<u32>() as u64,
                wgpu::BufferUsages::STORAGE,
            ),
            compacted_indices: buffer(
                "Compacted Index Buffer",
                u64::from(capacity.packed_indices) * size_of::<u32>() as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
            ),
            compacted_draw: buffer(
                "Compacted Draw Buffer",
                size_of::<wgpu::util::DrawIndexedIndirectArgs>() as u64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT,
            ),
        };
        let layout = VISIBILITY_BUILD.layout(device);
        let bind_group = bind_group(device, &layout, &buffers, render_targets);
        let shader = device.create_shader_module(shaders::VISIBILITY.descriptor());
        let pipeline_layout = pipeline_layout(
            device,
            "Visibility Pass Pipeline Layout",
            &[(&VISIBILITY_BUILD, &layout)],
        );
        let constants = [(
            MESHLET_GRANULARITY,
            f64::from(u8::from(granularity == VisibilityPrepass::Meshlets)),
        )];
        let pipeline = |entry_point| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry_point),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry_point),
                compilation_options: wgpu::PipelineCompilationOptions {
                    constants: &constants,
                    ..wgpu::PipelineCompilationOptions::default()
                },
                cache: None,
            })
        };
        let (width, height) = render_targets.size();
        VisibilityPass {
            mark_triangles: pipeline("mark_triangles"),
            offset_words: pipeline("offset_words"),
            offset_blocks: pipeline("offset_blocks"),
            compact_triangles: pipeline("compact_triangles"),
            layout,
            buffers,
            bind_group,
            size: (width, height),
            block_count,
        }
    }

    /// Builds the bind group that reads `render_targets` again, as `new`
    /// builds it: after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.bind_group = bind_group(device, &self.layout, &self.buffers, render_targets);
        self.size = render_targets.size();
    }

    /// Records the pass into `encoder`: clears the marks, marks the
    /// triangles the visibility IDs show, or their Meshlets, a workgroup per
    /// tile of pixels, then sums the marks by word and by block and compacts
    /// the marked triangles' indices, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        encoder.clear_buffer(&self.buffers.triangle_bits, 0, None);
        encoder.clear_buffer(&self.buffers.record_bits, 0, None);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(VISIBILITY_PASS),
            timestamp_writes,
        });
        pass.set_bind_group(VISIBILITY_BUILD.index, &self.bind_group, &[]);
        pass.set_pipeline(&self.mark_triangles);
        let (width, height) = self.size;
        pass.dispatch_workgroups(
            width.div_ceil(VISIBILITY_TILE_SIZE),
            height.div_ceil(VISIBILITY_TILE_SIZE),
            1,
        );
        pass.set_pipeline(&self.offset_words);
        pass.dispatch_workgroups(self.block_count, 1, 1);
        pass.set_pipeline(&self.offset_blocks);
        pass.dispatch_workgroups(1, 1, 1);
        pass.set_pipeline(&self.compact_triangles);
        pass.dispatch_workgroups(self.block_count, 1, 1);
    }

    /// The compacted draw's index buffer.
    pub fn compacted_indices(&self) -> &wgpu::Buffer {
        &self.buffers.compacted_indices
    }

    /// The compacted draw's indirect arguments.
    pub fn compacted_draw(&self) -> &wgpu::Buffer {
        &self.buffers.compacted_draw
    }

    /// Bytes of the pass's own buffers; the visibility IDs are a render
    /// target, and the frame's draw the culling pass's.
    pub fn gpu_memory_bytes(&self) -> u64 {
        [
            &self.buffers.triangle_bits,
            &self.buffers.record_bits,
            &self.buffers.triangle_offsets,
            &self.buffers.compacted_indices,
            &self.buffers.compacted_draw,
        ]
        .iter()
        .map(|buffer| buffer.size())
        .sum()
    }
}

/// The pass's group 3 (`VISIBILITY_BUILD`) of `layout`: `buffers`, and the
/// visibility IDs of `render_targets`, which it has with the
/// `visibility_prepass` Render switch on, as this pass exists only then.
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffers: &VisibilityBuffers,
    render_targets: &RenderTargets,
) -> wgpu::BindGroup {
    let visibility_ids = render_targets
        .visibility_ids()
        .expect("render targets created with the visibility_prepass switch hold visibility IDs");
    VISIBILITY_BUILD.bind_group(
        device,
        layout,
        [
            (
                "visibility_ids",
                wgpu::BindingResource::TextureView(visibility_ids),
            ),
            ("frame_draw", buffers.frame_draw.as_entire_binding()),
            ("packed_indices", buffers.packed_indices.as_entire_binding()),
            ("triangle_bits", buffers.triangle_bits.as_entire_binding()),
            ("record_bits", buffers.record_bits.as_entire_binding()),
            (
                "triangle_offsets",
                buffers.triangle_offsets.as_entire_binding(),
            ),
            (
                "compacted_indices",
                buffers.compacted_indices.as_entire_binding(),
            ),
            ("compacted_draw", buffers.compacted_draw.as_entire_binding()),
        ],
    )
}
