//! The culling pass: a compute pass that builds the frame's draw from the
//! Instances, a Meshlet record per Meshlet of each Instance and a packed
//! index per triangle corner, in slot order, which the main pass draws with
//! one indirect call (ADR 0019). With the `culling` Render switch on, it
//! draws only the Instances whose Mesh bounds reach into the view frustum.

use crate::cooked::MESHLET_MAX_VERTICES;
use crate::render::INSTANCE_CAPACITY;
use crate::render::bind_groups::{BindGroupDefinition, FRAME_DRAW, pipeline_layout};
use crate::render::prepare::FrameData;
use crate::render::renderer::RenderSwitches;
use crate::render::shaders;

/// The culling pass's label, and its name in frame stats.
pub(crate) const CULLING_PASS: &str = "culling_pass";

/// Slots each workgroup of the pass's scans sums: the scans of
/// `INSTANCE_CAPACITY` slots take one workgroup of blocks, 8 to an
/// invocation.
pub(crate) const SCAN_BLOCK_SIZE: u32 = 256;

/// What the frame's draw holds, provisional until M4's Budget split sizes it.
#[derive(Clone, Copy)]
pub struct DrawCapacity {
    pub meshlet_records: u32,
    pub packed_indices: u32,
}

impl DrawCapacity {
    /// 2^24 packed indices, 64 MiB, draw 5.6 million triangles: M3's dense
    /// scene draws at least 5 million. 2^20 Meshlet records, 12 MiB, are as
    /// many Meshlets of 5.3 triangles on average.
    pub const PROVISIONAL: DrawCapacity = DrawCapacity {
        meshlet_records: 1 << 20,
        packed_indices: 1 << 24,
    };
}

/// The frame's draw as `culling.wgsl` lays it out, which the GPU stats
/// readback reads: the indirect arguments of one indexed draw, its index
/// count first, then the Instances it holds and those it has no room for.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct FrameDraw {
    arguments: [u32; 5],
    pub(crate) instances_drawn: u32,
    pub(crate) instances_overflowed: u32,
}

impl FrameDraw {
    /// Its index count over 3: the drawn Instances' indices fill the draw
    /// from index 0 with no gap, in slot order.
    pub(crate) fn triangles_drawn(&self) -> u32 {
        self.arguments[0] / 3
    }
}

/// The culling pass's pipelines, one per step, and the frame's draw it
/// writes.
pub struct CullingPass {
    offset_slots: wgpu::ComputePipeline,
    offset_blocks: wgpu::ComputePipeline,
    append_meshlets: wgpu::ComputePipeline,
    /// The Instance slots in use, which the CPU writes when they change.
    instance_slot_count: wgpu::Buffer,
    /// Each slot's first Meshlet record and packed index within its block of
    /// `SCAN_BLOCK_SIZE` slots, then each block's.
    instance_offsets: wgpu::Buffer,
    meshlet_records: wgpu::Buffer,
    packed_indices: wgpu::Buffer,
    /// The draw's indirect arguments and counts (`FrameDraw`).
    frame_draw: wgpu::Buffer,
    /// Its group 3 (`FRAME_DRAW`).
    bind_group: wgpu::BindGroup,
    capacity: DrawCapacity,
    max_workgroups_per_dimension: u32,
    /// Set while the Instances in use need more of the draw than it holds,
    /// which was warned about when it was set.
    warned_of_overflow: bool,
}

impl CullingPass {
    /// The pass, given the layouts of the shared bind groups its shader
    /// module declares, with its draw of `capacity`, as `switches` set.
    pub fn new(
        device: &wgpu::Device,
        bind_group_layouts: &[(&BindGroupDefinition, &wgpu::BindGroupLayout)],
        capacity: DrawCapacity,
        switches: RenderSwitches,
    ) -> CullingPass {
        // A record above its Meshlet vertex's bits, short of the all-ones
        // index.
        assert!(capacity.meshlet_records < 1 << (u32::BITS - MESHLET_MAX_VERTICES.ilog2()));
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
            u64::from(INSTANCE_CAPACITY + INSTANCE_CAPACITY / SCAN_BLOCK_SIZE)
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
        // Cleared before each pass, so COPY_DST, and copied to the GPU stats
        // readback after it, so COPY_SRC.
        let frame_draw = buffer(
            "Frame Draw Buffer",
            size_of::<FrameDraw>() as u64,
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        );
        let frame_draw_layout = FRAME_DRAW.layout(device);
        let bind_group = FRAME_DRAW.bind_group(
            device,
            &frame_draw_layout,
            [
                (
                    "instance_slot_count",
                    instance_slot_count.as_entire_binding(),
                ),
                ("instance_offsets", instance_offsets.as_entire_binding()),
                ("meshlet_records", meshlet_records.as_entire_binding()),
                ("packed_indices", packed_indices.as_entire_binding()),
                ("frame_draw", frame_draw.as_entire_binding()),
            ],
        );

        let shader = device.create_shader_module(shaders::CULLING.descriptor());
        let mut layouts = bind_group_layouts.to_vec();
        layouts.push((&FRAME_DRAW, &frame_draw_layout));
        let layout = pipeline_layout(device, "Culling Pass Pipeline Layout", &layouts);
        let constants = switches.constants();
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
        CullingPass {
            offset_slots: pipeline("offset_slots"),
            offset_blocks: pipeline("offset_blocks"),
            append_meshlets: pipeline("append_meshlets"),
            instance_slot_count,
            instance_offsets,
            meshlet_records,
            packed_indices,
            frame_draw,
            bind_group,
            capacity,
            max_workgroups_per_dimension: device.limits().max_compute_workgroups_per_dimension,
            warned_of_overflow: false,
        }
    }

    /// Records the pass into `encoder`: clears the frame's draw, then
    /// appends the Meshlets of the Instances it draws among `frame`'s slots
    /// in use, with `bind_groups` set, timed into `timestamp_writes`. Where
    /// they need more Meshlet records or packed indices than the draw holds,
    /// it draws the Instances that fit. It warns, once until they fit again,
    /// where every Instance in use, culled or not, would need more.
    pub fn record(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &[(&BindGroupDefinition, &wgpu::BindGroup)],
        frame: &FrameData,
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) {
        let overflows = frame.meshlet_count > u64::from(self.capacity.meshlet_records)
            || frame.index_count > u64::from(self.capacity.packed_indices);
        if overflows && !self.warned_of_overflow {
            tracing::warn!(
                meshlet_records = frame.meshlet_count,
                packed_indices = frame.index_count,
                meshlet_record_capacity = self.capacity.meshlet_records,
                packed_index_capacity = self.capacity.packed_indices,
                "The frame's draw is full: Instances whose Meshlets do not fit are not drawn"
            );
        }
        self.warned_of_overflow = overflows;

        // Each slot's offsets are the sum of the earlier slots' Meshlet
        // records and packed indices: summed by block, then the blocks.
        encoder.clear_buffer(&self.frame_draw, 0, None);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(CULLING_PASS),
            timestamp_writes,
        });
        for &(definition, bind_group) in bind_groups {
            pass.set_bind_group(definition.index, bind_group, &[]);
        }
        pass.set_bind_group(FRAME_DRAW.index, &self.bind_group, &[]);
        let slots = frame.instance_slot_count;
        if slots > 0 {
            pass.set_pipeline(&self.offset_slots);
            pass.dispatch_workgroups(slots.div_ceil(SCAN_BLOCK_SIZE), 1, 1);
            pass.set_pipeline(&self.offset_blocks);
            pass.dispatch_workgroups(1, 1, 1);
            // A workgroup per slot, in rows each dimension's limit holds.
            pass.set_pipeline(&self.append_meshlets);
            let rows = slots.div_ceil(self.max_workgroups_per_dimension);
            pass.dispatch_workgroups(slots.div_ceil(rows), rows, 1);
        }
    }

    /// The Instance slots in use, a u32 the CPU writes.
    pub fn instance_slot_count(&self) -> &wgpu::Buffer {
        &self.instance_slot_count
    }

    /// The frame's Meshlet records, which the main pass's vertex shader reads.
    pub fn meshlet_records(&self) -> &wgpu::Buffer {
        &self.meshlet_records
    }

    /// The frame's draw's index buffer.
    pub fn packed_indices(&self) -> &wgpu::Buffer {
        &self.packed_indices
    }

    /// The frame's draw's indirect arguments and counts (`FrameDraw`).
    pub fn frame_draw(&self) -> &wgpu::Buffer {
        &self.frame_draw
    }

    /// Bytes of the frame's draw, and of the Instance slot count and offsets.
    pub fn gpu_memory_bytes(&self) -> u64 {
        [
            &self.instance_slot_count,
            &self.instance_offsets,
            &self.meshlet_records,
            &self.packed_indices,
            &self.frame_draw,
        ]
        .iter()
        .map(|buffer| buffer.size())
        .sum()
    }
}
