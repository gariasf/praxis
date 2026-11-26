//! The Ray scene: what a ray is traced against, the static TLAS and the
//! dynamic TLAS in one frame whose origin is the corner of the static TLAS's
//! Anchor (ADR 0020), with the order of their steps in a frame and the bind
//! group a pass that traces sets.

use std::time::{Duration, Instant};

use crate::assets::MeshPool;
use crate::render::FrameData;
use crate::render::acceleration_structures::{DYNAMIC_TLAS_BUILD, DynamicTlas, StaticTlas};
use crate::render::bind_groups::RAY_SCENE;
use crate::render::readback::FrameTimestamps;
use crate::render::renderer::{CrowdForm, RenderCapacities, RenderSwitches};
use crate::stats::{FrameRecorder, GpuMemory};

/// The CPU stage that records the dynamic TLAS's build; with
/// `DYNAMIC_TLAS_SUBMIT`, its submit, the CPU cost ADR 0020 caps.
pub(crate) const DYNAMIC_TLAS_RECORDING: &str = "dynamic_tlas";

/// The CPU stage that hands Frame data to the static TLAS: its entry
/// changes, and the submit of a build its worker finished.
pub(crate) const STATIC_TLAS_UPDATE: &str = "static_tlas_update";

/// The CPU stage that submits the dynamic TLAS's build, on its own.
pub(crate) const DYNAMIC_TLAS_SUBMIT: &str = "dynamic_tlas_submit";

/// What the camera uniform takes from the Ray scene each frame.
pub struct RaySceneFrame {
    /// The Anchor whose corner is the origin a ray is composed from.
    pub anchor: [i32; 3],
    pub dynamic_entries: u32,
    /// A box that holds the dynamic TLAS's triangles, its lowest corner
    /// then its highest.
    pub dynamic_bounds: [[f32; 3]; 2],
}

/// Both TLAS and their steps in a frame: `begin_frame` once the frame's
/// loads have landed, `record_build` straight after the bend pass,
/// `submit_build` before the frame's own submit.
pub struct RayScene {
    static_tlas: StaticTlas,
    dynamic_tlas: DynamicTlas,
    wait_for_static_tlas: bool,
    /// Not with the crowd's rebuild form, whose rebuilds through wgpu would
    /// fail the first marker's query, as `DynamicTlas::mark` says.
    marks_dynamic_tlas_build: bool,
    layout: wgpu::BindGroupLayout,
    /// One per static TLAS, as `StaticTlas::bindings` orders them.
    bind_groups: [wgpu::BindGroup; 2],
    /// The dynamic TLAS's build `record_build` recorded, until
    /// `submit_build` submits it.
    build: Option<wgpu::CommandBuffer>,
}

impl RayScene {
    pub fn new(
        device: &wgpu::Device,
        memory: &GpuMemory,
        queue: &wgpu::Queue,
        backend: wgpu::Backend,
        switches: RenderSwitches,
        capacities: &RenderCapacities,
    ) -> Self {
        let static_tlas = StaticTlas::new(device, memory, queue, capacities.static_tlas_instances);
        let dynamic_tlas = DynamicTlas::new(device, memory, queue, backend, capacities);
        let layout = RAY_SCENE.layout(device);
        let bind_groups = static_tlas.bindings().map(|static_binding| {
            RAY_SCENE.bind_group(
                device,
                &layout,
                [
                    ("static_tlas", static_binding),
                    ("dynamic_tlas", dynamic_tlas.binding()),
                ],
            )
        });
        Self {
            static_tlas,
            dynamic_tlas,
            wait_for_static_tlas: switches.wait_for_static_tlas,
            marks_dynamic_tlas_build: switches.crowd_form != CrowdForm::Rebuild,
            layout,
            bind_groups,
            build: None,
        }
    }

    /// Hands `frame`'s static Instance changes to the static TLAS and fills
    /// the dynamic TLAS in its frame. Returns the frame's encoder, created
    /// after the one the dynamic TLAS's build records into, as
    /// `DynamicTlas::prepare_encoder` needs, and the time it staged.
    pub fn begin_frame(
        &mut self,
        frame: &FrameData,
        mesh_pool: &MeshPool,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        recorder: &mut FrameRecorder,
    ) -> (wgpu::CommandEncoder, RaySceneFrame, Duration) {
        debug_assert!(
            self.build.is_none(),
            "a dynamic TLAS build recorded and never submitted"
        );
        let preparing = Instant::now();
        self.dynamic_tlas.prepare_encoder(device);
        let prepare_time = recorder.stage(DYNAMIC_TLAS_RECORDING, preparing);
        let encoder = create_frame_encoder(device);
        let updating = Instant::now();
        let static_tlas_update =
            self.static_tlas
                .update(frame, mesh_pool, device, queue, self.wait_for_static_tlas);
        let update_time = recorder.stage(STATIC_TLAS_UPDATE, updating);
        recorder.counters.static_tlas_build_time =
            static_tlas_update.build_time.unwrap_or_default();
        recorder.counters.frames_until_traced = static_tlas_update.frames_until_traced;
        recorder.counters.uploaded_bytes += static_tlas_update.uploaded_bytes;
        recorder.counters.static_instances_left_out = self.static_tlas.instances_left_out();
        // After the update, which may have just swapped in a TLAS at a new
        // Anchor.
        let anchor = self.static_tlas.anchor();
        let filling = Instant::now();
        let dynamic_bounds = self.dynamic_tlas.fill(frame, mesh_pool, anchor);
        let fill_time = recorder.stage(DYNAMIC_TLAS_RECORDING, filling);
        let ray_scene_frame = RaySceneFrame {
            anchor,
            // Prepare lists no more than the dynamic TLAS holds.
            dynamic_entries: frame.moving_instances.len() as u32,
            dynamic_bounds,
        };
        (
            encoder,
            ray_scene_frame,
            prepare_time + update_time + fill_time,
        )
    }

    /// Records the dynamic TLAS's build into a command buffer of its own,
    /// which it keeps for `submit_build`, timed into `timestamps`.
    pub fn record_build(
        &mut self,
        device: &wgpu::Device,
        timestamps: &mut Option<FrameTimestamps<'_>>,
        recorder: &mut FrameRecorder,
    ) {
        let started = Instant::now();
        let build_queries = timestamps
            .as_mut()
            .filter(|_| self.marks_dynamic_tlas_build)
            .map(|timestamps| timestamps.pass(DYNAMIC_TLAS_BUILD));
        self.build =
            self.dynamic_tlas
                .record(device, build_queries)
                .map(|(commands, uploaded_bytes)| {
                    recorder.counters.uploaded_bytes += uploaded_bytes;
                    commands
                });
        recorder.stage(DYNAMIC_TLAS_RECORDING, started);
    }

    /// Submits the build `record_build` recorded, if any.
    pub fn submit_build(&mut self, queue: &wgpu::Queue, recorder: &mut FrameRecorder) {
        let started = Instant::now();
        if let Some(commands) = self.build.take() {
            queue.submit(std::iter::once(commands));
        }
        recorder.stage(DYNAMIC_TLAS_SUBMIT, started);
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    /// The group of the static TLAS rays trace this frame.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_groups[self.static_tlas.bound_index()]
    }

    #[cfg(test)]
    pub(crate) fn anchor(&self) -> [i32; 3] {
        self.static_tlas.anchor()
    }
}

pub(super) fn create_frame_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Frame Encoder"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::pools::tests::read_storage;
    use crate::gpu::GpuContext;
    use crate::render::acceleration_structures::tests::Triangle;
    use crate::render::bind_groups::{
        BindGroupDefinition, compute_pipeline, pipeline_layout, read_only_storage, storage,
    };
    use crate::render::instance::ANCHOR_CELL_SIZE;
    use crate::render::prepare::{InstanceWrite, MovingInstance};
    use crate::render::{CameraUniform, InstanceData};
    use crate::stats::gpu_memory::tests::gpu_memory;

    const CAMERA_ANCHOR: [i32; 3] = [1562, 0, -3];

    const TRACE_DOWN: BindGroupDefinition = BindGroupDefinition {
        index: 3,
        label: "Trace Down Bind Group",
        bindings: &[
            read_only_storage("points", "array<vec4<f32>>", wgpu::ShaderStages::COMPUTE),
            storage("tags", "array<u32>"),
        ],
    };

    /// What a ray down -Z from 1 m above each of `points`, in the Ray
    /// scene's frame, hits within 2 m, in the static TLAS or else in the
    /// dynamic TLAS: the tag of the Instance it hits.
    fn trace_down(
        context: &GpuContext,
        ray_scene: &RayScene,
        points: &[[f32; 3]],
    ) -> Vec<Option<u32>> {
        let device = &context.device;
        let source = format!(
            "enable wgpu_ray_query;
            {}{}
            @compute @workgroup_size(1)
            fn trace_down(@builtin(global_invocation_id) invocation: vec3<u32>) {{
                let origin = points[invocation.x].xyz + vec3(0.0, 0.0, 1.0);
                let down = vec3(0.0, 0.0, -1.0);
                let ray = RayDesc(RAY_FLAG_NONE, 0xffu, 0.0, 2.0, origin, down);
                var query: ray_query;
                rayQueryInitialize(&query, static_tlas, ray);
                while rayQueryProceed(&query) {{}}
                var hit = rayQueryGetCommittedIntersection(&query);
                if hit.kind == RAY_QUERY_INTERSECTION_NONE {{
                    rayQueryInitialize(&query, dynamic_tlas, ray);
                    while rayQueryProceed(&query) {{}}
                    hit = rayQueryGetCommittedIntersection(&query);
                }}
                let missed = hit.kind == RAY_QUERY_INTERSECTION_NONE;
                tags[invocation.x] = select(hit.instance_custom_data, 0xffffffffu, missed);
            }}",
            RAY_SCENE.declarations(),
            TRACE_DOWN.declarations()
        );
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Trace Down"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let layout = TRACE_DOWN.layout(device);
        let groups = [(&RAY_SCENE, ray_scene.layout()), (&TRACE_DOWN, &layout)];
        let pipeline_layout = pipeline_layout(device, "Trace Down Pipeline Layout", &groups);
        let pipeline = compute_pipeline(device, &pipeline_layout, &module, "trace_down", &[]);
        let padded: Vec<[f32; 4]> = points.iter().map(|&[x, y, z]| [x, y, z, 0.0]).collect();
        let point_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("Trace Down Points"),
                contents: bytemuck::cast_slice(&padded),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let tags = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Trace Down Tags"),
            size: (points.len() * size_of::<u32>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let bind_group = TRACE_DOWN.bind_group(
            device,
            &layout,
            [
                ("points", point_buffer.as_entire_binding()),
                ("tags", tags.as_entire_binding()),
            ],
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(RAY_SCENE.index, ray_scene.bind_group(), &[]);
            pass.set_bind_group(TRACE_DOWN.index, &bind_group, &[]);
            pass.dispatch_workgroups(points.len() as u32, 1, 1);
        }
        context.queue.submit(std::iter::once(encoder.finish()));
        read_storage::<u32>(context, &tags, points.len())
            .into_iter()
            .map(|tag| (tag != u32::MAX).then_some(tag))
            .collect()
    }

    /// A static Instance and a Moving Instance, each in another Anchor cell
    /// than the camera's, are hit by rays composed from the corner of the
    /// Anchor `begin_frame` returns and traced through `bind_group()`. A
    /// further static Instance starts a build: once it is bound,
    /// `bind_group()` is the other group and rays through it hit all three.
    #[test]
    fn rays_through_its_bind_group_hit_both_tlas_before_and_after_the_bound_one_swaps() {
        let Some(triangle) = Triangle::new() else {
            return;
        };
        let context = &triangle.context;
        let switches = RenderSwitches {
            wait_for_static_tlas: true,
            ..RenderSwitches::default()
        };
        let capacities = RenderCapacities {
            static_tlas_instances: 8,
            ..RenderCapacities::checked_in()
        };
        let mut ray_scene = RayScene::new(
            &context.device,
            &gpu_memory(context),
            &context.queue,
            context.backend(),
            switches,
            &capacities,
        );
        let [x, y, z] = CAMERA_ANCHOR;
        let at = |anchor, offset_x| {
            triangle.instance(anchor, [offset_x, 0.0, 0.0], glam::Mat3::IDENTITY)
        };
        let instances = vec![
            at([x + 1, y, z], 5.0),
            at([x - 1, y, z], 0.0),
            at(CAMERA_ANCHOR, -4.0),
        ];
        let write = |slot: usize, moving| InstanceWrite {
            first_slot: slot as u32,
            instances: slot..slot + 1,
            moving,
        };
        let mut frame = FrameData {
            camera: CameraUniform {
                anchor: CAMERA_ANCHOR,
                ..CameraUniform::default()
            },
            instances: instances.clone(),
            moving_instances: vec![MovingInstance {
                slot: 1,
                instance: 1,
            }],
            ..FrameData::default()
        };
        // The frame, then the frame again without its Instance writes, which
        // binds the build its changes started.
        let mut trace_after = |ray_scene: &mut RayScene, writes: Vec<InstanceWrite>| {
            frame.instance_writes = writes;
            let mut anchor = [0; 3];
            for _ in 0..2 {
                let mut recorder = FrameRecorder::new();
                let (_, ray_scene_frame, _) = ray_scene.begin_frame(
                    &frame,
                    &triangle.mesh_pool,
                    &context.device,
                    &context.queue,
                    &mut recorder,
                );
                ray_scene.record_build(&context.device, &mut None, &mut recorder);
                ray_scene.submit_build(&context.queue, &mut recorder);
                anchor = ray_scene_frame.anchor;
                frame.instance_writes.clear();
            }
            let points: Vec<[f32; 3]> = instances
                .iter()
                .map(|instance: &InstanceData| {
                    let cells = glam::IVec3::from(instance.anchor) - glam::IVec3::from(anchor);
                    let corner = cells.as_vec3() * ANCHOR_CELL_SIZE as f32;
                    (corner + glam::Vec3::from(instance.offset) + glam::vec3(0.8, 0.0, 0.0))
                        .to_array()
                })
                .collect();
            trace_down(context, ray_scene, &points)
        };

        let traced = trace_after(&mut ray_scene, vec![write(0, false), write(1, true)]);
        assert_eq!(traced, [Some(0), Some(1), None]);
        let bound_first = ray_scene.bind_group().clone();

        let traced = trace_after(&mut ray_scene, vec![write(2, false)]);
        assert!(ray_scene.bind_group() != &bound_first);
        assert_eq!(traced, [Some(0), Some(1), Some(2)]);
    }
}
