//! The material pass: with the `visibility_buffer` Render switch on, a
//! prototype of option D of ADR 0019's research note, a compute pass that
//! shades each pixel from the visibility IDs the depth passes wrote, in
//! place of the lit pass, and writes the HDR target. It rebuilds each
//! triangle a sample shows from the frame's draw, interpolates its
//! attributes at the pixel's centre and one pixel right and down, samples
//! every Material with those gradients in one shader, and averages the
//! triangles' colours by the samples each covers, as the lit pass's resolve
//! does (`material_pass.wgsl`).

use crate::render::bind_groups::{
    MATERIAL_PASS_FRAME, MATERIAL_PASS_GEOMETRY, MATERIAL_PASS_INPUTS, MATERIALS, pipeline_layout,
};
use crate::render::culling_pass::{CullingPass, PHASE_TWO_DRAW_OFFSET};
use crate::render::renderer::RenderSwitches;
use crate::render::shaders;
use crate::render::targets::{RenderTargets, SAMPLE_COUNT};

/// The pass's label, and its name in frame stats.
pub(crate) const MATERIAL_PASS: &str = "material_pass";

/// Pixels along each side of the tile each workgroup shades.
pub(crate) const MATERIAL_TILE_SIZE: u32 = 8;

// The shader reads each pixel's samples into one vector.
const _: () = assert!(SAMPLE_COUNT == 4);

/// The pass's uniform, as `material_pass.wgsl` declares it: the clear
/// colour, exposed colour and alpha, which the CPU writes each frame, then
/// the first index of phase two's draw, which the GPU copies from the
/// frame's draw.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MaterialPassUniform {
    pub(crate) background: [f32; 4],
    pub(crate) phase_two_first_index: u32,
    /// To WGSL's size: `background` aligns the struct to 16 bytes.
    pub(crate) _padding: [u32; 3],
}

/// Where the frame's draw holds the first index of phase two's draw.
const PHASE_TWO_FIRST_INDEX_OFFSET: u64 = PHASE_TWO_DRAW_OFFSET
    + std::mem::offset_of!(wgpu::util::DrawIndexedIndirectArgs, first_index) as u64;

/// Where the uniform holds it.
const UNIFORM_FIRST_INDEX_OFFSET: u64 =
    std::mem::offset_of!(MaterialPassUniform, phase_two_first_index) as u64;

/// The pass's pipeline and bind groups.
pub struct MaterialPass {
    pipeline: wgpu::ComputePipeline,
    /// Its groups 0 and 2 (`MATERIAL_PASS_FRAME`, `MATERIAL_PASS_GEOMETRY`),
    /// built once, since their buffers never change.
    frame: wgpu::BindGroup,
    geometry: wgpu::BindGroup,
    inputs_layout: wgpu::BindGroupLayout,
    buffers: MaterialPassBuffers,
    /// Its group 3 (`MATERIAL_PASS_INPUTS`), built from the render targets
    /// by `bind_render_targets`.
    inputs: wgpu::BindGroup,
    /// The HDR target's size.
    size: wgpu::Extent3d,
}

/// The buffers of the pass's group 3, kept to rebuild it: the culling
/// pass's frame's draw, and the pass's uniform.
struct MaterialPassBuffers {
    meshlet_records: wgpu::Buffer,
    packed_indices: wgpu::Buffer,
    frame_draw: wgpu::Buffer,
    uniform: wgpu::Buffer,
}

impl MaterialPass {
    /// The pass, given the Materials' bind group layout, which shades the
    /// frame's draw of `culling_pass` from the visibility IDs of
    /// `render_targets` into their HDR target, with the Renderer's `frame`
    /// buffers, `FRAME`'s by name, and its `geometry` buffers,
    /// `SCENE_GEOMETRY`'s, lit as `switches` set.
    pub fn new(
        device: &wgpu::Device,
        materials_layout: &wgpu::BindGroupLayout,
        frame: [(&'static str, &wgpu::Buffer); 4],
        geometry: [(&'static str, &wgpu::Buffer); 4],
        culling_pass: &CullingPass,
        render_targets: &RenderTargets,
        switches: RenderSwitches,
    ) -> MaterialPass {
        let frame_layout = MATERIAL_PASS_FRAME.layout(device);
        let geometry_layout = MATERIAL_PASS_GEOMETRY.layout(device);
        let inputs_layout = MATERIAL_PASS_INPUTS.layout(device);
        let shader = device.create_shader_module(shaders::MATERIAL.descriptor());
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("shade_pixels"),
            layout: Some(&pipeline_layout(
                device,
                "Material Pass Pipeline Layout",
                &[
                    (&MATERIAL_PASS_FRAME, &frame_layout),
                    (&MATERIALS, materials_layout),
                    (&MATERIAL_PASS_GEOMETRY, &geometry_layout),
                    (&MATERIAL_PASS_INPUTS, &inputs_layout),
                ],
            )),
            module: &shader,
            entry_point: Some("shade_pixels"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &switches.constants(),
                ..wgpu::PipelineCompilationOptions::default()
            },
            cache: None,
        });
        let frame = MATERIAL_PASS_FRAME.bind_group(
            device,
            &frame_layout,
            frame.map(|(name, buffer)| (name, buffer.as_entire_binding())),
        );
        let geometry = MATERIAL_PASS_GEOMETRY.bind_group(
            device,
            &geometry_layout,
            geometry.map(|(name, buffer)| (name, buffer.as_entire_binding())),
        );
        let buffers = MaterialPassBuffers {
            meshlet_records: culling_pass.meshlet_records().clone(),
            packed_indices: culling_pass.packed_indices().clone(),
            frame_draw: culling_pass.frame_draw().clone(),
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Material Pass Uniform Buffer"),
                size: std::mem::size_of::<MaterialPassUniform>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        };
        let inputs = inputs_bind_group(device, &inputs_layout, &buffers, render_targets);
        MaterialPass {
            pipeline,
            frame,
            geometry,
            inputs_layout,
            buffers,
            inputs,
            size: render_targets.hdr().texture().size(),
        }
    }

    /// Builds the bind group that reads `render_targets` again, as `new`
    /// builds it: after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        self.inputs = inputs_bind_group(device, &self.inputs_layout, &self.buffers, render_targets);
        self.size = render_targets.hdr().texture().size();
    }

    /// The pass's uniform buffer, whose clear colour the CPU writes each
    /// frame, from its start.
    pub fn uniform(&self) -> &wgpu::Buffer {
        &self.buffers.uniform
    }

    /// Bytes of the pass's own buffer.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.buffers.uniform.size()
    }

    /// Records the pass into `encoder`, after phase two of the culling pass,
    /// with the Materials' bind group `materials` set: copies where phase
    /// two's draw starts into its uniform, then dispatches a workgroup per
    /// tile of the HDR target, timed into `timestamp_writes`.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        materials: &wgpu::BindGroup,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        encoder.copy_buffer_to_buffer(
            &self.buffers.frame_draw,
            PHASE_TWO_FIRST_INDEX_OFFSET,
            &self.buffers.uniform,
            UNIFORM_FIRST_INDEX_OFFSET,
            Some(std::mem::size_of::<u32>() as u64),
        );
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(MATERIAL_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(MATERIAL_PASS_FRAME.index, &self.frame, &[]);
        pass.set_bind_group(MATERIALS.index, materials, &[]);
        pass.set_bind_group(MATERIAL_PASS_GEOMETRY.index, &self.geometry, &[]);
        pass.set_bind_group(MATERIAL_PASS_INPUTS.index, &self.inputs, &[]);
        pass.dispatch_workgroups(
            self.size.width.div_ceil(MATERIAL_TILE_SIZE),
            self.size.height.div_ceil(MATERIAL_TILE_SIZE),
            1,
        );
    }
}

/// The pass's group 3 (`MATERIAL_PASS_INPUTS`) of `layout`: `buffers`, and
/// the shadow mask, visibility IDs and HDR target of `render_targets`.
fn inputs_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffers: &MaterialPassBuffers,
    render_targets: &RenderTargets,
) -> wgpu::BindGroup {
    let Some(visibility_ids) = render_targets.visibility_ids() else {
        panic!("the material pass reads visibility IDs, which these targets lack");
    };
    MATERIAL_PASS_INPUTS.bind_group(
        device,
        layout,
        [
            (
                "meshlet_records",
                buffers.meshlet_records.as_entire_binding(),
            ),
            ("packed_indices", buffers.packed_indices.as_entire_binding()),
            (
                "shadow_mask",
                wgpu::BindingResource::TextureView(render_targets.shadow_mask()),
            ),
            (
                "visibility_ids",
                wgpu::BindingResource::TextureView(visibility_ids),
            ),
            ("material_pass", buffers.uniform.as_entire_binding()),
            (
                "hdr_target",
                wgpu::BindingResource::TextureView(render_targets.hdr()),
            ),
        ],
    )
}
