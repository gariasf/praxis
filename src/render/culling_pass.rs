//! The culling pass: a compute pass that builds the frame's draw from the
//! Instances, a Meshlet record per Meshlet of each Instance and a packed
//! index per triangle corner, in slot order within each phase, which the
//! main pass draws with one indirect call per phase and depth pipeline, and
//! one of both phases for the lit pipeline (ADR 0019). It runs in two
//! phases: phase one draws the Instances visible the frame before, phase
//! two, after the depth pyramid, the others in view that the pyramid does
//! not hide, and it records each Instance's visibility for the next frame. With the `culling` Render switch on, it
//! draws only the Instances whose Mesh bounds reach into the view frustum;
//! off, phase one draws every Instance in use. Each phase sums its
//! Instances' offsets over the slots in use, writes their Meshlet records,
//! then their packed indices with a workgroup per Meshlet record, dispatched
//! indirectly, all in one compute pass.

use crate::cooked::MESHLET_MAX_VERTICES;
use crate::render::INSTANCE_CAPACITY;
use crate::render::bind_groups::{BindGroupDefinition, FRAME_DRAW, pipeline_layout};
use crate::render::prepare::FrameData;
use crate::render::renderer::RenderSwitches;
use crate::render::shaders;
use crate::render::targets::RenderTargets;

/// A phase of the culling pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CullingPhase {
    /// Draws the Instances visible the frame before into the depth pre-pass.
    One,
    /// After the depth pyramid, draws the other Instances in view that it
    /// does not hide, and records which are visible for the next frame.
    Two,
}

impl CullingPhase {
    /// The phase's label, and its name in frame stats.
    pub(crate) const fn label(self) -> &'static str {
        match self {
            CullingPhase::One => "culling_phase_one",
            CullingPhase::Two => "culling_phase_two",
        }
    }
}

/// Slots each workgroup of the pass's scans sums: the scans of
/// `INSTANCE_CAPACITY` slots take one workgroup of blocks, 8 to an
/// invocation.
pub(crate) const SCAN_BLOCK_SIZE: u32 = 256;

/// The workgroups in each dimension of a dispatch the Capability floor
/// guarantees, wgpu's default (ADR 0021)
/// [ref:wgpu-types-30-default-workgroups-per-dimension]: `append_triangles`
/// is dispatched in rows of these.
pub(crate) const MAX_WORKGROUPS_PER_DIMENSION: u32 = 65_535;

/// What the frame's draw holds, provisional until ADR 0013's split of the 8 GB
/// Budget sizes it.
#[derive(Clone, Copy)]
pub struct DrawCapacity {
    pub meshlet_records: u32,
    pub packed_indices: u32,
}

impl DrawCapacity {
    /// 2^25 packed indices, 128 MiB, the most one storage binding holds at
    /// the Capability floor (ADR 0021), draw 11.2 million triangles: all of
    /// M3's dense scene with culling off, as its Goldens compare.
    /// 2^20 Meshlet records, 12 MiB, are as many Meshlets of 10.7 triangles
    /// on average.
    pub const PROVISIONAL: DrawCapacity = DrawCapacity {
        meshlet_records: 1 << 20,
        packed_indices: 1 << 25,
    };
}

/// The frame's draw as `culling.wgsl` lays it out, which the GPU stats
/// readback reads: the indirect arguments of the indexed draw of both
/// phases, its index count first, then the Instances it holds and those it
/// has no room for, then phase two's own draw's arguments, where phase
/// two's Meshlet records and packed indices start, and the phase's dispatch
/// of `append_triangles`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FrameDraw {
    pub(crate) draw: wgpu::util::DrawIndexedIndirectArgs,
    pub(crate) instances_drawn: u32,
    pub(crate) instances_overflowed: u32,
    pub(crate) phase_two_draw: wgpu::util::DrawIndexedIndirectArgs,
    phase_two_offsets: [u32; 2],
    append_dispatch: wgpu::util::DispatchIndirectArgs,
    /// To WGSL's size: `phase_two_offsets` aligns the struct to 8 bytes.
    _padding: u32,
}

/// Where the frame's draw holds the indirect arguments of phase two's draw.
pub(crate) const PHASE_TWO_DRAW_OFFSET: u64 =
    std::mem::offset_of!(FrameDraw, phase_two_draw) as u64;

/// Where the frame's draw holds the phase's dispatch of `append_triangles`.
const APPEND_DISPATCH_OFFSET: u64 = std::mem::offset_of!(FrameDraw, append_dispatch) as u64;

impl FrameDraw {
    /// Its index count over 3: the drawn Instances' indices fill the draw
    /// from index 0 with no gap, in slot order within each phase.
    pub(crate) fn triangles_drawn(&self) -> u32 {
        self.draw.index_count / 3
    }

    /// Phase two's draw's index count over 3.
    pub(crate) fn phase_two_triangles_drawn(&self) -> u32 {
        self.phase_two_draw.index_count / 3
    }
}

/// The pipelines of one phase of the culling pass, one per step.
struct PhasePipelines {
    offset_slots: wgpu::ComputePipeline,
    offset_blocks: wgpu::ComputePipeline,
    append_records: wgpu::ComputePipeline,
    append_triangles: wgpu::ComputePipeline,
}

/// The culling pass's pipelines, each phase's, and the frame's draw it
/// writes.
pub struct CullingPass {
    phase_one: PhasePipelines,
    phase_two: PhasePipelines,
    buffers: FrameDrawBuffers,
    frame_draw_layout: wgpu::BindGroupLayout,
    /// Its group 3 (`FRAME_DRAW`), and the group `append_triangles` runs
    /// with, both built by `frame_draw_bind_groups`.
    bind_group: wgpu::BindGroup,
    append_bind_group: wgpu::BindGroup,
}

/// The buffers of the pass's group 3, which holds the depth pyramid besides.
struct FrameDrawBuffers {
    /// The Instance slots in use, which the CPU writes when they change.
    instance_slot_count: wgpu::Buffer,
    /// Each slot's first Meshlet record and packed index within its block of
    /// `SCAN_BLOCK_SIZE` slots, then each block's.
    instance_offsets: wgpu::Buffer,
    meshlet_records: wgpu::Buffer,
    packed_indices: wgpu::Buffer,
    /// The draw's indirect arguments and counts (`FrameDraw`).
    frame_draw: wgpu::Buffer,
    /// What `append_triangles`'s group binds in `frame_draw`'s place, which
    /// it does not read: a dispatch cannot take its arguments from a buffer
    /// its bindings write, as they and its indirect buffer are one usage
    /// scope [ref:wgpu-core-30-dispatch-usage-scope].
    frame_draw_stand_in: wgpu::Buffer,
    /// A bit per Instance slot, set where phase two found its Instance
    /// visible, which phase one reads the next frame.
    instance_visibility: wgpu::Buffer,
}

impl CullingPass {
    /// The pass, given the layouts of the shared bind groups its shader
    /// module declares, with its draw of `capacity`, as `switches` set,
    /// testing Instances against the depth pyramid of `render_targets`.
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
        capacity: DrawCapacity,
        switches: RenderSwitches,
        render_targets: &RenderTargets,
    ) -> CullingPass {
        // A record above its Meshlet vertex's bits, short of the all-ones
        // index.
        assert!(capacity.meshlet_records < 1 << (u32::BITS - MESHLET_MAX_VERTICES.ilog2()));
        assert!(
            device.limits().max_compute_workgroups_per_dimension >= MAX_WORKGROUPS_PER_DIMENSION
        );
        let buffer = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let instance_slot_count = buffer(
            "Instance Slot Count Buffer",
            size_of::<u32>() as u64,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let instance_offsets = buffer(
            "Instance Offset Buffer",
            u64::from(INSTANCE_CAPACITY + INSTANCE_CAPACITY / SCAN_BLOCK_SIZE + 1)
                * size_of::<[u32; 2]>() as u64,
            wgpu::BufferUsages::STORAGE,
        );
        // A record is three u32s: Instance slot, Meshlet and Material ID.
        let meshlet_records = buffer(
            "Meshlet Record Buffer",
            u64::from(capacity.meshlet_records) * size_of::<[u32; 3]>() as u64,
            wgpu::BufferUsages::STORAGE,
        );
        let packed_indices = buffer(
            "Packed Index Buffer",
            u64::from(capacity.packed_indices) * size_of::<u32>() as u64,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
        );
        // Cleared before phase one, so COPY_DST, and copied to the GPU stats
        // readback after it, so COPY_SRC.
        let frame_draw = buffer(
            "Frame Draw Buffer",
            size_of::<FrameDraw>() as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );
        let frame_draw_stand_in = buffer(
            "Frame Draw Stand-in Buffer",
            size_of::<FrameDraw>() as u64,
            wgpu::BufferUsages::STORAGE,
        );
        // Zeroed at creation: no Instance was visible before the first frame.
        let instance_visibility = buffer(
            "Instance Visibility Buffer",
            u64::from(INSTANCE_CAPACITY / u32::BITS) * size_of::<u32>() as u64,
            wgpu::BufferUsages::STORAGE,
        );
        let buffers = FrameDrawBuffers {
            instance_slot_count,
            instance_offsets,
            meshlet_records,
            packed_indices,
            frame_draw,
            frame_draw_stand_in,
            instance_visibility,
        };
        let frame_draw_layout = FRAME_DRAW.layout(device);
        let [bind_group, append_bind_group] =
            frame_draw_bind_groups(device, &frame_draw_layout, &buffers, render_targets);

        let shader = device.create_shader_module(shaders::CULLING.descriptor());
        let mut layouts = bind_group_layouts.to_vec();
        layouts.push((&FRAME_DRAW, &frame_draw_layout));
        let layout = pipeline_layout(device, "Culling Pass Pipeline Layout", &layouts);
        let phase_pipelines = |phase| {
            let phase_two = phase == CullingPhase::Two;
            let constants: Vec<(&str, f64)> = switches
                .constants()
                .into_iter()
                .chain([("PHASE_TWO", f64::from(u8::from(phase_two)))])
                .collect();
            let pipeline = |entry_point| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some(entry_point),
                    layout: Some(&layout),
                    module: &shader,
                    entry_point: Some(entry_point),
                    compilation_options: wgpu::PipelineCompilationOptions {
                        constants: &constants,
                        ..wgpu::PipelineCompilationOptions::default()
                    },
                    cache: None,
                })
            };
            PhasePipelines {
                offset_slots: pipeline("offset_slots"),
                offset_blocks: pipeline("offset_blocks"),
                append_records: pipeline("append_records"),
                append_triangles: pipeline("append_triangles"),
            }
        };
        CullingPass {
            phase_one: phase_pipelines(CullingPhase::One),
            phase_two: phase_pipelines(CullingPhase::Two),
            buffers,
            frame_draw_layout,
            bind_group,
            append_bind_group,
        }
    }

    /// Records `phase` of the pass into `encoder`: phase one clears the
    /// frame's draw first. It appends the Meshlets of the Instances the phase
    /// draws among `frame`'s slots in use, with `bind_groups` set, timed into
    /// `timestamp_writes`. Where they need more Meshlet records or packed
    /// indices than the draw holds, it draws the Instances that fit and
    /// counts the others as overflowed, which the GPU stats readback warns
    /// about.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        frame: &FrameData,
        phase: CullingPhase,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        let pipelines = match phase {
            CullingPhase::One => {
                encoder.clear_buffer(&self.buffers.frame_draw, 0, None);
                &self.phase_one
            }
            CullingPhase::Two => &self.phase_two,
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(phase.label()),
            timestamp_writes,
        });
        for &(definition, bind_group) in bind_groups {
            pass.set_bind_group(definition.index, bind_group, &[]);
        }
        pass.set_bind_group(FRAME_DRAW.index, &self.bind_group, &[]);
        let slots = frame.instance_slot_count;
        if slots > 0 {
            // Each slot's offsets are the sum of the earlier slots' Meshlet
            // records and packed indices: summed by block, then the blocks.
            pass.set_pipeline(&pipelines.offset_slots);
            pass.dispatch_workgroups(slots.div_ceil(SCAN_BLOCK_SIZE), 1, 1);
            pass.set_pipeline(&pipelines.offset_blocks);
            pass.dispatch_workgroups(1, 1, 1);
            // An invocation per slot.
            pass.set_pipeline(&pipelines.append_records);
            pass.dispatch_workgroups(slots.div_ceil(SCAN_BLOCK_SIZE), 1, 1);
            // A workgroup per Meshlet record the phase appends.
            pass.set_bind_group(FRAME_DRAW.index, &self.append_bind_group, &[]);
            pass.set_pipeline(&pipelines.append_triangles);
            pass.dispatch_workgroups_indirect(&self.buffers.frame_draw, APPEND_DISPATCH_OFFSET);
        }
    }

    /// Builds the bind groups that read `render_targets` again, as `new`
    /// builds them: after every resize, which creates them again.
    pub fn bind_render_targets(&mut self, device: &wgpu::Device, render_targets: &RenderTargets) {
        [self.bind_group, self.append_bind_group] = frame_draw_bind_groups(
            device,
            &self.frame_draw_layout,
            &self.buffers,
            render_targets,
        );
    }

    /// The Instance slots in use, a u32 the CPU writes.
    pub fn instance_slot_count(&self) -> &wgpu::Buffer {
        &self.buffers.instance_slot_count
    }

    /// The frame's Meshlet records, which the main pass's vertex shader reads.
    pub fn meshlet_records(&self) -> &wgpu::Buffer {
        &self.buffers.meshlet_records
    }

    /// The frame's draw's index buffer.
    pub fn packed_indices(&self) -> &wgpu::Buffer {
        &self.buffers.packed_indices
    }

    /// The frame's draw's indirect arguments and counts (`FrameDraw`).
    pub fn frame_draw(&self) -> &wgpu::Buffer {
        &self.buffers.frame_draw
    }

    /// Bytes of the frame's draw and its stand-in, and of the Instance slot
    /// count, offsets and visibility.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.buffers
            .named()
            .iter()
            .map(|(_, buffer)| buffer.size())
            .sum::<u64>()
            + self.buffers.frame_draw_stand_in.size()
    }
}

impl FrameDrawBuffers {
    /// Each buffer by the name of its binding.
    fn named(&self) -> [(&'static str, &wgpu::Buffer); 6] {
        [
            ("instance_slot_count", &self.instance_slot_count),
            ("instance_offsets", &self.instance_offsets),
            ("meshlet_records", &self.meshlet_records),
            ("packed_indices", &self.packed_indices),
            ("frame_draw", &self.frame_draw),
            ("instance_visibility", &self.instance_visibility),
        ]
    }
}

/// The pass's group 3 (`FRAME_DRAW`) of `layout`: `buffers`, and the depth
/// pyramid of `render_targets`; then the same with the frame's draw's
/// stand-in in its place, which `append_triangles` runs with.
fn frame_draw_bind_groups(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffers: &FrameDrawBuffers,
    render_targets: &RenderTargets,
) -> [wgpu::BindGroup; 2] {
    [&buffers.frame_draw, &buffers.frame_draw_stand_in].map(|frame_draw| {
        let buffers = buffers.named().map(|(name, buffer)| {
            let buffer = if name == "frame_draw" {
                frame_draw
            } else {
                buffer
            };
            (name, buffer.as_entire_binding())
        });
        let depth_pyramid = wgpu::BindingResource::TextureView(render_targets.depth_pyramid());
        FRAME_DRAW.bind_group(
            device,
            layout,
            buffers
                .into_iter()
                .chain([("depth_pyramid", depth_pyramid)]),
        )
    })
}
