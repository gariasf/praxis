use std::time::Instant;

use crate::assets::{ARENA_CAPACITY_BYTES, MaterialPool, MeshPool};
use crate::gpu::GpuContext;
use crate::render::bind_groups::{FRAME, MATERIALS, SCENE_GEOMETRY};
use crate::render::culling_pass::{CullingPass, CullingPhase, DrawCapacity};
use crate::render::depth_pyramid_pass::{DEPTH_PYRAMID_PASS, DepthPyramidPass};
use crate::render::main_pass::{DEPTH_PREPASS, LIT_PASS, MainPass};
use crate::render::readback::GpuStatsReadback;
use crate::render::targets::RenderTargets;
use crate::render::{
    AccelerationStructureBytes, CameraUniform, FrameData, FrameStaging, INSTANCE_CAPACITY,
    InstanceData, LightUniform, LoadStaging, STATIC_TLAS_CAPACITY, StaticTlas,
};
use crate::stats::{FrameRecorder, GpuFrameStats};

/// The passes `render` records, in order, each timed from its start to its
/// end.
pub(crate) const TIMED_PASSES: [&str; 5] = [
    CullingPhase::One.label(),
    DEPTH_PREPASS,
    DEPTH_PYRAMID_PASS,
    CullingPhase::Two.label(),
    LIT_PASS,
];

/// The Render switches a Renderer is created with and keeps. The defaults are
/// what ships; capture and bench take others, for equivalence checks and
/// A/Bs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderSwitches {
    /// Whether the culling pass draws only the Instances whose Mesh bounds
    /// reach into the view frustum, rather than every Instance in use.
    pub culling: bool,
}

impl Default for RenderSwitches {
    fn default() -> Self {
        Self { culling: true }
    }
}

impl RenderSwitches {
    /// Each switch by the name of the WGSL `override` that every shader
    /// module declares for it, with its value as a pipeline sets it.
    pub(crate) fn constants(&self) -> [(&'static str, f64); 1] {
        [("CULLING", f64::from(u8::from(self.culling)))]
    }
}

/// Draws Frame data into a target view, the window's surface or an offscreen
/// texture, from GPU resources and Pools it owns; it reads no ECS World. It
/// holds the bind groups passes share, the render targets and the static
/// TLAS, and calls the passes in ADR 0019's order.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    target_format: wgpu::TextureFormat,
    aspect_ratio: f32,
    render_targets: RenderTargets,
    culling_pass: CullingPass,
    main_pass: MainPass,
    depth_pyramid_pass: DepthPyramidPass,
    camera_buffer: wgpu::Buffer,
    light_buffer: wgpu::Buffer,
    /// The camera and the lights (`FRAME`).
    frame_bind_group: wgpu::BindGroup,
    pub(crate) mesh_pool: MeshPool,
    pub(crate) material_pool: MaterialPool,
    pub(crate) load_staging: LoadStaging,
    instance_buffer: wgpu::Buffer,
    /// The Instances and the vertex Arena (`SCENE_GEOMETRY`).
    scene_geometry_bind_group: wgpu::BindGroup,
    static_tlas: StaticTlas,
    /// What its acceleration structures are estimated to take.
    acceleration_structure_bytes: AccelerationStructureBytes,
    /// Set when a frame's uploads failed after it queued Instance writes,
    /// which then never landed: the next prepare must write every Instance.
    lost_instance_writes: bool,
    /// The Instance slot count the culling pass reads, as last uploaded.
    uploaded_instance_slot_count: u32,
    transient_saves_memory: bool,
    frame_staging: FrameStaging,
    /// The frame's encoder from `update`, holding its upload copies, until
    /// `render` records the pass into it and submits it.
    frame_encoder: Option<wgpu::CommandEncoder>,
    gpu_stats: GpuStatsReadback,
}

impl Renderer {
    /// A Renderer with empty Pools that draws into `width` × `height` views of
    /// `target_format` textures, both non-zero, as `switches` set.
    pub fn new(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        switches: RenderSwitches,
    ) -> Renderer {
        Self::with_draw_capacity(
            context,
            target_format,
            width,
            height,
            switches,
            DrawCapacity::PROVISIONAL,
        )
    }

    /// As `new`, with a frame's draw of `draw_capacity`.
    fn with_draw_capacity(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        switches: RenderSwitches,
        draw_capacity: DrawCapacity,
    ) -> Renderer {
        let _span = tracing::info_span!("renderer_init").entered();

        let device = context.device.clone();
        let queue = context.queue.clone();
        let frame_bind_group_layout = FRAME.layout(&device);
        let scene_geometry_bind_group_layout = SCENE_GEOMETRY.layout(&device);

        // Material-as-ID: one bind group (SSBO + per-channel texture arrays +
        // sampler) serves every material. Created before the main pass
        // because its pipeline layout references its bind group layout.
        let mut load_staging = LoadStaging::new(device.clone());
        let material_pool = MaterialPool::new(&device, &mut load_staging);

        let render_targets = RenderTargets::new(&device, target_format, width, height);
        let culling_pass = CullingPass::new(
            &device,
            &[
                (&FRAME, &frame_bind_group_layout),
                (&SCENE_GEOMETRY, &scene_geometry_bind_group_layout),
            ],
            draw_capacity,
            switches,
            &render_targets,
        );
        let main_pass = MainPass::new(
            &device,
            target_format,
            &[
                (&FRAME, &frame_bind_group_layout),
                (&MATERIALS, &material_pool.bind_group_layout),
                (&SCENE_GEOMETRY, &scene_geometry_bind_group_layout),
            ],
            culling_pass.meshlet_records(),
        );

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: std::mem::size_of::<LightUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let frame_bind_group = FRAME.bind_group(
            &device,
            &frame_bind_group_layout,
            [
                ("camera", camera_buffer.as_entire_binding()),
                ("light", light_buffer.as_entire_binding()),
            ],
        );

        // Created at its fixed capacity; it never grows.
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: u64::from(INSTANCE_CAPACITY) * std::mem::size_of::<InstanceData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // The most a frame uploads: every buffer `upload` writes, in full.
        let frame_staging = FrameStaging::new(
            [
                &camera_buffer,
                &light_buffer,
                &instance_buffer,
                culling_pass.instance_slot_count(),
            ]
            .iter()
            .map(|buffer| buffer.size())
            .sum(),
        );

        let mesh_pool = MeshPool::new(&device, ARENA_CAPACITY_BYTES);
        let scene_geometry_bind_group = SCENE_GEOMETRY.bind_group(
            &device,
            &scene_geometry_bind_group_layout,
            [
                ("instances", instance_buffer.as_entire_binding()),
                ("vertices", mesh_pool.vertex_arena().as_entire_binding()),
                ("meshes", mesh_pool.mesh_table().as_entire_binding()),
                ("meshlets", mesh_pool.meshlet_arena().as_entire_binding()),
                (
                    "meshlet_vertices",
                    mesh_pool.meshlet_vertex_arena().as_entire_binding(),
                ),
                (
                    "meshlet_triangles",
                    mesh_pool.meshlet_triangle_arena().as_entire_binding(),
                ),
            ],
        );

        let depth_pyramid_pass = DepthPyramidPass::new(&device, &render_targets);
        let gpu_stats = GpuStatsReadback::new(&device, queue.get_timestamp_period());
        let static_tlas = StaticTlas::new(&device, STATIC_TLAS_CAPACITY);

        Self {
            mesh_pool,
            material_pool,
            load_staging,
            device,
            queue,
            target_format,
            aspect_ratio: width as f32 / height as f32,
            render_targets,
            culling_pass,
            main_pass,
            depth_pyramid_pass,
            camera_buffer,
            light_buffer,
            frame_bind_group,
            instance_buffer,
            scene_geometry_bind_group,
            static_tlas,
            acceleration_structure_bytes: AccelerationStructureBytes::estimate(context.backend()),
            lost_instance_writes: false,
            uploaded_instance_slot_count: 0,
            transient_saves_memory: context.adapter.get_info().transient_saves_memory == Some(true),
            frame_staging,
            frame_encoder: None,
            gpu_stats,
        }
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        // A pass whose bind groups read a target rebuilds them here, after
        // the render targets.
        self.render_targets = RenderTargets::new(&self.device, self.target_format, width, height);
        self.depth_pyramid_pass
            .bind_render_targets(&self.device, &self.render_targets);
        self.culling_pass
            .bind_render_targets(&self.device, &self.render_targets);
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// The width of the target over its height.
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Records the copies of `frame`'s uniforms, the Instances it writes and
    /// its Instance slot count, if it changed, and the static TLAS's rebuild,
    /// if they change it, into the frame's encoder, which `render` submits;
    /// every upload is followed by a render. First it submits the loads
    /// staged since the last frame, and, when they loaded Meshes, waits for
    /// the GPU to build their BLASes and compacts them.
    pub fn upload(&mut self, frame: &FrameData, recorder: &mut FrameRecorder) {
        let started = Instant::now();
        debug_assert!(
            self.frame_encoder.is_none(),
            "an upload whose frame was not rendered"
        );
        // Loads staged since the last frame land before its copies and
        // draws, and their BLASes are compacted before the static TLAS
        // takes them.
        self.load_staging.submit(&self.queue);
        self.mesh_pool.compact_blases(&self.queue);
        let mut encoder = create_frame_encoder(&self.device);
        let instance_size = std::mem::size_of::<InstanceData>() as wgpu::BufferAddress;
        let instance_uploads = frame.instance_writes.iter().map(|write| {
            (
                &self.instance_buffer,
                u64::from(write.first_slot) * instance_size,
                bytemuck::cast_slice(&frame.instances[write.instances.clone()]),
            )
        });
        let slot_count_upload = (frame.instance_slot_count != self.uploaded_instance_slot_count)
            .then(|| {
                let buffer = self.culling_pass.instance_slot_count();
                (buffer, 0, bytemuck::bytes_of(&frame.instance_slot_count))
            });
        let uploads = [
            (&self.camera_buffer, 0, bytemuck::bytes_of(&frame.camera)),
            (&self.light_buffer, 0, bytemuck::bytes_of(&frame.light)),
        ]
        .into_iter()
        .chain(instance_uploads)
        .chain(slot_count_upload);
        let uploaded_bytes = self
            .frame_staging
            .write(&self.device, &mut encoder, uploads);
        recorder.uploaded_bytes += uploaded_bytes;
        // 0 bytes: the staging slot failed to map, as logged, and the
        // frame's writes never land.
        if uploaded_bytes == 0 {
            self.lost_instance_writes |= !frame.instances.is_empty();
        } else {
            recorder.instances_written += frame.instances.len() as u32;
            self.uploaded_instance_slot_count = frame.instance_slot_count;
        }
        self.static_tlas
            .update(frame, &self.mesh_pool, &mut encoder);
        recorder.static_instances_left_out = self.static_tlas.instances_left_out();
        self.frame_encoder = Some(encoder);
        recorder.stage("upload", started);
    }

    /// Draws `frame`, uploaded by `upload`, into `target`, a view in the
    /// format and size this Renderer renders to, timing its passes for
    /// `gpu_frame_stats`, which reads them back a few frames later. Returns
    /// the frame's submission.
    pub fn render(
        &mut self,
        frame: &FrameData,
        target: &wgpu::TextureView,
        recorder: &mut FrameRecorder,
    ) -> wgpu::SubmissionIndex {
        let started = Instant::now();
        let mut encoder = self
            .frame_encoder
            .take()
            .unwrap_or_else(|| create_frame_encoder(&self.device));
        self.material_pool.refresh_bind_group(&self.device);

        let timestamps = self.gpu_stats.begin_frame(&mut encoder, recorder);
        let culling_bind_groups = [
            (&FRAME, &self.frame_bind_group),
            (&SCENE_GEOMETRY, &self.scene_geometry_bind_group),
        ];
        self.culling_pass.record(
            &mut encoder,
            &culling_bind_groups,
            frame,
            CullingPhase::One,
            timestamps.map(|timestamps| timestamps.compute_pass(CullingPhase::One.label())),
        );
        let bind_groups = [
            (&FRAME, &self.frame_bind_group),
            (&MATERIALS, &self.material_pool.bind_group),
            (&SCENE_GEOMETRY, &self.scene_geometry_bind_group),
        ];
        self.main_pass.record_depth_prepass(
            &mut encoder,
            &self.render_targets,
            &bind_groups,
            self.culling_pass.packed_indices(),
            self.culling_pass.frame_draw(),
            timestamps.map(|timestamps| timestamps.render_pass(DEPTH_PREPASS)),
        );
        self.depth_pyramid_pass.record(
            &mut encoder,
            timestamps.map(|timestamps| timestamps.compute_pass(DEPTH_PYRAMID_PASS)),
        );
        self.culling_pass.record(
            &mut encoder,
            &culling_bind_groups,
            frame,
            CullingPhase::Two,
            timestamps.map(|timestamps| timestamps.compute_pass(CullingPhase::Two.label())),
        );
        self.main_pass.record_lit_pass(
            &mut encoder,
            target,
            &self.render_targets,
            &bind_groups,
            self.culling_pass.packed_indices(),
            self.culling_pass.frame_draw(),
            timestamps.map(|timestamps| timestamps.render_pass(LIT_PASS)),
        );
        // The depth pre-pass draws phase one's draw, and the lit pass phase
        // two's into depth, then both phases', each with one indirect call.
        recorder.draw_calls += 3;
        self.gpu_stats
            .end_frame(&mut encoder, self.culling_pass.frame_draw());

        let command_buffer = encoder.finish();
        recorder.stage("encode", started);

        let started = Instant::now();
        // submit will accept anything that implements IntoIter
        let submission = self.queue.submit(std::iter::once(command_buffer));
        recorder.stage("submit", started);
        submission
    }

    /// The GPU stats of earlier frames that the last `render` or
    /// `flush_gpu_frame_stats` read back, by frame number, counted from 0.
    pub fn gpu_frame_stats(&self) -> &[GpuFrameStats] {
        self.gpu_stats.read_back()
    }

    /// Waits until the GPU stats of every frame rendered are read back,
    /// for `gpu_frame_stats`: the end of a run.
    pub fn flush_gpu_frame_stats(&mut self) -> anyhow::Result<()> {
        self.gpu_stats.flush(&self.device, &self.queue)
    }

    /// Whether Instance writes were lost since the last call; if so, the
    /// next prepare must write every Instance again.
    pub fn take_lost_instance_writes(&mut self) -> bool {
        std::mem::take(&mut self.lost_instance_writes)
    }

    /// wgpu's allocator report total, a diagnostic only; Metal has none.
    pub fn allocator_report_bytes(&self) -> Option<u64> {
        self.device
            .generate_allocator_report()
            .map(|report| report.total_allocated_bytes)
    }

    /// Bytes of the buffers and textures this Renderer created, its Pools
    /// included, and of its acceleration structures, estimated; the render
    /// target is the caller's.
    pub fn gpu_memory_bytes(&self) -> u64 {
        let buffers = [
            &self.camera_buffer,
            &self.light_buffer,
            &self.instance_buffer,
        ];
        buffers.iter().map(|buffer| buffer.size()).sum::<u64>()
            + self
                .render_targets
                .gpu_memory_bytes(self.transient_saves_memory)
            + self.culling_pass.gpu_memory_bytes()
            + self.mesh_pool.gpu_memory_bytes()
            + self.estimated_acceleration_structure_bytes()
            + self
                .material_pool
                .gpu_memory_bytes(self.transient_saves_memory)
            + self.frame_staging.gpu_memory_bytes()
            + self.gpu_stats.gpu_memory_bytes()
    }

    /// Bytes of its acceleration structures, estimated from the bytes
    /// measured on its backend, since wgpu exposes no acceleration-structure
    /// size (ADR 0017).
    pub fn estimated_acceleration_structure_bytes(&self) -> u64 {
        let bytes = &self.acceleration_structure_bytes;
        self.mesh_pool.estimated_blas_bytes(bytes) + self.static_tlas.estimated_bytes(bytes)
    }
}

fn create_frame_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Frame Encoder"),
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    use super::*;
    use crate::assets::material::MaterialData;
    use crate::assets::pools::tests::{logged, read_storage};
    use crate::assets::{MaterialHandle, MeshHandle};
    use crate::camera::{Camera, projection_matrix};
    use crate::capture::read_back;
    use crate::components::{MeshRef, Transform};
    use crate::cooked::{CookedMesh, CookedMeshlet, CookedPrimitive, CookedVertex};
    use crate::render::culling_pass::FrameDraw;
    use crate::render::instance::{anchor_and_offset, padded_columns};
    use crate::render::prepare::InstanceWrite;
    use crate::render::{FramePreparer, NO_MATERIAL, NO_MESH};
    use crate::time::FixedTime;
    use crate::transform::propagate_world_transforms;

    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    const SIZE: u32 = 16;
    const RED: [u8; 4] = [255, 0, 0, 255];
    const GREEN: [u8; 4] = [0, 255, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];

    #[test]
    fn renders_hand_built_frame_data_without_a_world() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut renderer = Renderer::new(
            &context,
            TARGET_FORMAT,
            SIZE,
            SIZE,
            RenderSwitches::default(),
        );
        let frame = triangle_frame(&mut renderer);
        let (image, recorder, _) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);

        assert_eq!(recorder.draw_calls, 3);
        assert_eq!(image.get_pixel(SIZE / 2, SIZE / 2).0, [255, 0, 0, 255]);
        // The clear colour, linear (0.1, 0.2, 0.3), sRGB-encoded as in
        // tests/capture.rs; the GPU may round its green either way.
        let corner = image.get_pixel(0, 0).0;
        assert!(
            corner[0] == 89 && corner[1].abs_diff(124) <= 1 && corner[2] == 149,
            "corner {corner:?}"
        );
    }

    #[test]
    fn a_resized_renderer_draws_as_one_created_at_its_new_size() {
        let Some(context) = gpu_context() else {
            return;
        };
        // Wider than tall, so swapped sides would show.
        let (width, height) = (24, 20);
        let mut resized = Renderer::new(
            &context,
            TARGET_FORMAT,
            SIZE,
            SIZE,
            RenderSwitches::default(),
        );
        let frame = triangle_frame(&mut resized);
        // A frame at the first size before the resize, as a window draws.
        render_frame(&context, &mut resized, &frame, SIZE, SIZE);
        resized.resize(width, height);
        let (resized_image, ..) = render_frame(&context, &mut resized, &frame, width, height);

        let mut created = Renderer::new(
            &context,
            TARGET_FORMAT,
            width,
            height,
            RenderSwitches::default(),
        );
        let frame = triangle_frame(&mut created);
        let (created_image, ..) = render_frame(&context, &mut created, &frame, width, height);

        assert_eq!(resized.aspect_ratio(), 1.2);
        assert_eq!(
            resized_image.get_pixel(width / 2, height / 2).0,
            [255, 0, 0, 255]
        );
        assert!(
            resized_image == created_image,
            "the resized Renderer drew another image than one created at its size"
        );
    }

    /// Two Instances of a Mesh whose Primitives name a red and a green
    /// Material, the upper one drawn with them, the lower one overriding them
    /// with blue: the Mesh pool's tables give each Primitive's Meshlets its
    /// Material ID, unless its Instance overrides it.
    #[test]
    fn each_primitive_draws_with_its_own_material_unless_its_instance_overrides_it() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut renderer = Renderer::new(
            &context,
            TARGET_FORMAT,
            SIZE,
            SIZE,
            RenderSwitches::default(),
        );
        let [red, green, blue] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            .map(|emissive| emissive_material(&mut renderer, emissive));
        let mesh = load_triangles(
            &mut renderer,
            &[[-0.5, 0.0], [0.5, 0.0]],
            &[Some(red), Some(green)],
        );
        let frame = frame_of(vec![
            instance(mesh, NO_MATERIAL, [0.0, 0.5]),
            instance(mesh, blue.index(), [0.0, -0.5]),
        ]);
        let (image, recorder, _) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);

        assert_eq!(recorder.draw_calls, 3);
        let quarters = [(4, 4), (12, 4), (4, 12), (12, 12)].map(|(x, y)| image.get_pixel(x, y).0);
        assert_eq!(quarters, [RED, GREEN, BLUE, BLUE]);
    }

    /// Twice the Instances of a two-triangle Mesh draw twice the Instances
    /// and triangles, read back from the GPU, in the same draw calls: one in
    /// the depth pre-pass and two in the lit pass.
    #[test]
    fn doubling_the_instances_doubles_instances_and_triangles_drawn_not_draw_calls() {
        let Some(context) = gpu_context() else {
            return;
        };
        let switches = RenderSwitches::default();
        let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE, switches);
        let red = emissive_material(&mut renderer, [1.0, 0.0, 0.0]);
        let mesh = load_triangles(&mut renderer, &[[-0.5, 0.0], [0.5, 0.0]], &[Some(red); 2]);
        let mut drawn = |instances: u32| {
            let frame = frame_of(vec![
                instance(mesh, NO_MATERIAL, [0.0, 0.0]);
                instances as usize
            ]);
            let (_, recorder, gpu) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);
            (
                gpu.instances_drawn,
                gpu.triangles_drawn,
                recorder.draw_calls,
            )
        };

        assert_eq!(drawn(3), (3, 6, 3));
        assert_eq!(drawn(6), (6, 12, 3));
    }

    /// Instances past a block of the culling pass's scans, and past a row of
    /// its dispatches, where the draw holds the indices of all but the last.
    /// None culled, each draws from its own offsets until the draw is full,
    /// then none do, with a warning, and the last is counted as overflowed.
    /// Culled, the off-screen ones leave room for the last two, so none
    /// overflows and nothing warns, though the Instances in use would not
    /// fit.
    #[test]
    fn a_frame_past_the_draws_capacity_draws_what_fits_and_warns_only_of_drawn_overflow() {
        let Some(context) = gpu_context() else {
            return;
        };
        const INSTANCES: u32 = 70_000;
        let capacity = DrawCapacity {
            meshlet_records: INSTANCES,
            packed_indices: 3 * (INSTANCES - 1),
        };
        for switches in [RenderSwitches { culling: false }, RenderSwitches::default()] {
            let mut renderer = Renderer::with_draw_capacity(
                &context,
                TARGET_FORMAT,
                SIZE,
                SIZE,
                switches,
                capacity,
            );
            let red = emissive_material(&mut renderer, [1.0, 0.0, 0.0]);
            let mesh = load_triangles(&mut renderer, &[[0.0, 0.0]], &[Some(red)]);
            // Every Instance but the last two is outside the target; those
            // two are drawn above and below its centre.
            let instances = (0..INSTANCES)
                .map(|slot| {
                    let offset = match INSTANCES - slot {
                        2 => [0.0, 0.5],
                        1 => [0.0, -0.5],
                        _ => [5.0, 5.0],
                    };
                    instance(mesh, NO_MATERIAL, offset)
                })
                .collect();
            let frame = frame_of(instances);
            let ((image, _, gpu), log) =
                logged(|| render_frame(&context, &mut renderer, &frame, SIZE, SIZE));

            let culled = switches.culling;
            assert_eq!(image.get_pixel(SIZE / 2, 4).0, RED);
            assert_eq!(image.get_pixel(SIZE / 2, 12).0 == RED, culled);
            assert_eq!(log.contains("The frame's draw is full"), !culled, "{log}");
            let drawn = (
                gpu.instances_drawn,
                gpu.triangles_drawn,
                gpu.instances_overflowed,
            );
            let expected = if culled {
                (2, 2, 0)
            } else {
                (INSTANCES - 1, INSTANCES - 1, 1)
            };
            assert_eq!(drawn, expected, "{switches:?}");
        }
    }

    /// Quads seen from the origin: a near one over the left half of the view,
    /// one wholly behind it, one partly behind it and one beside it. Phase
    /// two draws all four in the first frame, against an empty pyramid; from
    /// the third, it culls the hidden one, which Frame stats count out, and
    /// draws the one partly behind; when the near one moves behind the
    /// camera, phase two draws the hidden one again in that frame. Every
    /// frame draws the image culling off draws. The Renderer is created at
    /// another size and resized first, so a pyramid, or a culling bind
    /// group, left at the old size would cull wrongly.
    #[test]
    fn an_instance_hidden_behind_a_nearer_one_is_culled_by_phase_two_until_it_reappears() {
        let Some(context) = gpu_context() else {
            return;
        };
        let camera = CameraUniform {
            view_proj: (projection_matrix(1.0) * Camera::new().view_matrix()).to_cols_array_2d(),
            ..CameraUniform::default()
        };
        // Each frame's image and Instances drawn.
        let frames = |switches| {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, 24, 24, switches);
            renderer.resize(SIZE, SIZE);
            let [red, green, blue] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
                .map(|emissive| emissive_material(&mut renderer, emissive));
            let quad = load_quad(&mut renderer);
            // A quad of `width` × `height` metres centred at `centre`.
            let placed = |material: MaterialHandle, centre: [f32; 3], [width, height]: [f32; 2]| {
                InstanceData {
                    rotation_scale: padded_columns(glam::Mat3::from_diagonal(glam::vec3(
                        width, height, 1.0,
                    ))),
                    offset: centre,
                    ..instance(quad, material.index(), [0.0, 0.0])
                }
            };
            let hidden = placed(green, [-1.0, 0.0, -6.0], [0.8, 0.8]);
            let partly_hidden = placed(blue, [0.0, 0.6, -6.0], [0.5, 0.5]);
            let beside = placed(red, [1.2, -0.6, -6.0], [0.5, 0.5]);
            [-3.0, -3.0, -3.0, 5.0].map(|near_z| {
                let near = placed(red, [-1.0, 0.0, near_z], [2.0, 4.0]);
                let frame = FrameData {
                    camera,
                    ..frame_of(vec![near, hidden, partly_hidden, beside])
                };
                let (image, _, gpu) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);
                (image, gpu.instances_drawn)
            })
        };
        let culled = frames(RenderSwitches::default());
        let unculled = frames(RenderSwitches { culling: false });

        let drawn = |frames: &[(image::RgbaImage, u32); 4]| frames.clone().map(|(_, drawn)| drawn);
        // The near one is behind the camera in the last frame.
        assert_eq!(drawn(&culled), [4, 4, 3, 3]);
        assert_eq!(drawn(&unculled), [4, 4, 4, 4]);
        // A pixel the hidden quad covers: the near quad's red, then its green.
        assert_eq!(culled[2].0.get_pixel(5, 8).0, RED);
        assert_eq!(culled[3].0.get_pixel(5, 8).0, GREEN);
        for (frame, (culled, unculled)) in culled.iter().zip(&unculled).enumerate() {
            assert!(
                culled.0 == unculled.0,
                "frame {frame} differs with culling off"
            );
        }
    }

    /// Two Instances, the lower one out of view, then moved into it: in the
    /// frame it enters, phase one draws the upper one, visible the frame
    /// before, and phase two the lower one, into depth before the lit pass
    /// shades both, which draws the image culling off draws; in the next
    /// frame, phase one draws both and phase two none.
    #[test]
    fn an_instance_entering_the_view_is_drawn_by_phase_two_in_the_frame_it_enters() {
        let Some(context) = gpu_context() else {
            return;
        };
        // Each frame's image, and its draw's index count and phase two's
        // first index and count.
        let frames = |switches| {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE, switches);
            let red = emissive_material(&mut renderer, [1.0, 0.0, 0.0]);
            let mesh = load_triangles(&mut renderer, &[[0.0, 0.0]], &[Some(red)]);
            let upper = instance(mesh, NO_MATERIAL, [0.0, 0.5]);
            [[5.0, -0.5], [0.0, -0.5], [0.0, -0.5]].map(|lower| {
                let frame = frame_of(vec![upper, instance(mesh, NO_MATERIAL, lower)]);
                let (image, ..) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);
                let [draw] =
                    read_storage::<FrameDraw>(&context, renderer.culling_pass.frame_draw(), 1)[..]
                else {
                    unreachable!("one frame's draw");
                };
                let phase_two = draw.phase_two_draw;
                (
                    image,
                    draw.draw.index_count,
                    phase_two.first_index,
                    phase_two.index_count,
                )
            })
        };
        let culled = frames(RenderSwitches::default());
        let unculled = frames(RenderSwitches { culling: false });

        let [
            (_, out_of_view, ..),
            (entering, both, entering_first, entering_count),
            (_, _, _, after),
        ] = &culled;
        assert_eq!((*out_of_view, *both), (3, 6));
        assert_eq!((*entering_first, *entering_count, *after), (3, 3, 0));
        assert_eq!(entering.get_pixel(SIZE / 2, 12).0, RED);
        for (frame, (culled, unculled)) in culled.iter().zip(&unculled).enumerate() {
            assert!(
                culled.0 == unculled.0,
                "frame {frame} differs with culling off"
            );
        }
    }

    /// Two Renderables, the right one then despawned: its slot, which still
    /// holds its last Instance, draws nothing from the next frame.
    #[test]
    fn a_despawned_renderables_freed_slot_draws_nothing() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut renderer = Renderer::new(
            &context,
            TARGET_FORMAT,
            SIZE,
            SIZE,
            RenderSwitches::default(),
        );
        let red = emissive_material(&mut renderer, [1.0, 0.0, 0.0]);
        // A triangle facing the camera, 2 m ahead of it.
        let mesh = load_triangles(&mut renderer, &[[0.0, 0.0]], &[Some(red)]);
        let mut world = World::new();
        world.insert_resource(Camera::new());
        world.insert_resource(FixedTime::default());
        let mut preparer = FramePreparer::new(&mut world);
        let [_, right] = [-0.5, 0.5].map(|x| {
            let transform = Transform::from_translation(glam::dvec3(x, 0.0, 0.0));
            world.spawn((transform, MeshRef(mesh))).id()
        });
        // The red pixels of each half of the frame `prepare` makes.
        let mut red_pixels = |world: &mut World, renderer: &mut Renderer| -> [usize; 2] {
            world.run_system_once(propagate_world_transforms).unwrap();
            let frame = preparer.prepare(world, &renderer.mesh_pool, 1.0);
            let (image, ..) = render_frame(&context, renderer, frame, SIZE, SIZE);
            [0..SIZE / 2, SIZE / 2..SIZE].map(|columns| {
                image
                    .enumerate_pixels()
                    .filter(|(x, _, pixel)| columns.contains(x) && pixel.0 == RED)
                    .count()
            })
        };

        let [left, before] = red_pixels(&mut world, &mut renderer);
        assert!(left > 0 && before > 0, "{left} and {before} red pixels");
        world.despawn(right);
        assert_eq!(red_pixels(&mut world, &mut renderer), [left, 0]);
    }

    /// The cooked bounds of every Mesh `load_triangles` loads: a box that
    /// holds its triangles, off their pivot, its three sides unequal, so a
    /// culling test reads each axis apart.
    const TRIANGLE_BOUNDS: [[f32; 3]; 2] = [[-1.0, -1.5, -0.5], [2.0, 1.0, 1.5]];

    /// Instances about a camera 100 km out, turned and scaled: inside the
    /// view frustum, beyond each of its planes alone, across them, around
    /// the camera, and freed, over blocks of the culling pass's scans. With
    /// culling on, the frame's draw holds the Meshlets of exactly those
    /// whose Mesh bounds reach into the frustum by their corners in f64, in
    /// slot order, and Frame stats count only those drawn; with it off, of
    /// every Instance in use.
    #[test]
    fn the_culling_pass_draws_the_instances_whose_bounds_reach_into_the_frustum() {
        let Some(context) = gpu_context() else {
            return;
        };
        let camera = Camera {
            position: glam::dvec3(100_000.3, 20.0, -70_000.6),
            yaw: 0.4,
            pitch: -0.2,
            ..Camera::new()
        };
        let view_proj = projection_matrix(1.0) * camera.view_matrix();
        let [forward, right] = [camera.forward(), camera.right()].map(glam::Vec3::as_dvec3);
        // Per plane of the frustum, x ≥ −w, x ≤ w, y ≥ −w, y ≤ w and the near
        // plane z ≤ w, with no far plane (ADR 0015), how far inside it in
        // clip space the box's corner farthest inside lies: the box is
        // outside where one is negative.
        let planes = [
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, -1.0],
        ];
        let reaches = |pivot: glam::DVec3, rotation_scale: glam::Mat3| {
            planes.map(|[x, y, z]| {
                (0..8)
                    .map(|corner: usize| {
                        let bound = |axis: usize| TRIANGLE_BOUNDS[(corner >> axis) & 1][axis];
                        let local = glam::vec3(bound(0), bound(1), bound(2)).as_dvec3();
                        let position = rotation_scale.as_dmat3() * local + pivot - camera.position;
                        glam::dvec4(x, y, z, 1.0).dot(view_proj.as_dmat4() * position.extend(1.0))
                    })
                    .fold(f64::MIN, f64::max)
            })
        };
        // Uniform in [0, 1), from a fixed-seed generator.
        let mut seed: u64 = 0x5eed;
        let mut random = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        let placed: Vec<(glam::DVec3, glam::Mat3, [f64; 5])> = [
            // Within the near plane alone, 5 cm ahead.
            (camera.position + forward * 0.05, 0.01),
            // Around the camera, its centre behind it.
            (camera.position - forward, 8.0),
        ]
        .map(|(pivot, scale)| (pivot, glam::Mat3::from_diagonal(glam::Vec3::splat(scale))))
        .into_iter()
        .chain(std::iter::from_fn(|| {
            let distance = 46.0 * random() - 6.0;
            let [across, along_up] =
                [(); 2].map(|_| (0.7 * distance.abs() + 3.0) * (2.0 * random() - 1.0));
            let pivot = camera.position
                + forward * distance
                + right * across
                + right.cross(forward) * along_up;
            let [axis, scale] =
                [(); 2].map(|_| glam::Vec3::from_array([(); 3].map(|_| random() as f32)));
            let rotation = glam::Quat::from_axis_angle(
                (2.0 * axis - 1.0).try_normalize().unwrap_or(glam::Vec3::Y),
                std::f32::consts::TAU * random() as f32,
            );
            Some((
                pivot,
                glam::Mat3::from_quat(rotation) * glam::Mat3::from_diagonal(0.2 + 2.8 * scale),
            ))
        }))
        .map(|(pivot, rotation_scale)| (pivot, rotation_scale, reaches(pivot, rotation_scale)))
        // At least 1e-3 from every plane, past what f32 rounding moves, so
        // no verdict is a tie.
        .filter(|(_, _, reaches)| reaches.iter().all(|reach| reach.abs() > 1e-3))
        .take(1000)
        .collect();
        // Every 97th slot freed.
        let freed = |slot: usize| slot % 97 == 50;
        let culled_by_each_plane_alone = (0..5).map(|plane| {
            (0..placed.len())
                .filter(|&slot| {
                    !freed(slot)
                        && (0..5).all(|other| (placed[slot].2[other] < 0.0) == (other == plane))
                })
                .count()
        });
        assert!(
            culled_by_each_plane_alone.clone().all(|count| count > 0),
            "{:?}",
            culled_by_each_plane_alone.collect::<Vec<_>>()
        );

        let (anchor, offset) = anchor_and_offset(camera.position);
        for switches in [RenderSwitches::default(), RenderSwitches { culling: false }] {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE, switches);
            let red = emissive_material(&mut renderer, [1.0, 0.0, 0.0]);
            let mesh = load_triangles(&mut renderer, &[[0.0, 0.0]], &[Some(red)]);
            let instances = (0..placed.len())
                .map(|slot| {
                    let (pivot, rotation_scale, _) = placed[slot];
                    let (anchor, offset) = anchor_and_offset(pivot);
                    InstanceData {
                        rotation_scale: padded_columns(rotation_scale),
                        anchor,
                        offset,
                        mesh: if freed(slot) { NO_MESH } else { mesh.0 },
                        ..instance(mesh, NO_MATERIAL, [0.0, 0.0])
                    }
                })
                .collect();
            let frame = FrameData {
                camera: CameraUniform {
                    view_proj: view_proj.to_cols_array_2d(),
                    anchor,
                    offset,
                    ..CameraUniform::default()
                },
                ..frame_of(instances)
            };
            let (_, _, gpu) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);

            // The draw's index count, its first indirect argument, holds a
            // triangle per Instance drawn, each a Meshlet record naming its
            // slot.
            let drawn = read_storage::<u32>(&context, renderer.culling_pass.frame_draw(), 1)[0] / 3;
            let records = read_storage::<[u32; 3]>(
                &context,
                renderer.culling_pass.meshlet_records(),
                drawn as usize,
            );
            let expected: Vec<u32> = (0..placed.len())
                .filter(|&slot| {
                    !freed(slot)
                        && (!switches.culling || placed[slot].2.iter().all(|&reach| reach > 0.0))
                })
                .map(|slot| slot as u32)
                .collect();
            // Frame stats count the Instances drawn, so those culled show.
            assert_eq!(gpu.instances_drawn as usize, expected.len(), "{switches:?}");
            let drawn_slots: Vec<u32> = records.iter().map(|record| record[0]).collect();
            assert_eq!(drawn_slots, expected, "{switches:?}");
        }
    }

    /// A headless context, or `None` after saying so on a machine without a
    /// GPU adapter, where these tests skip.
    fn gpu_context() -> Option<GpuContext> {
        match pollster::block_on(GpuContext::headless()) {
            Ok(context) => Some(context),
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                None
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        }
    }

    /// Frame data that draws a triangle around the target's centre, black,
    /// unlit and red emissive, loaded into `renderer`'s Pools after a
    /// triangle outside the target, so it is drawn only from past the first
    /// entry of every table.
    fn triangle_frame(renderer: &mut Renderer) -> FrameData {
        load_triangles(renderer, &[[3.0, 3.0]], &[None]);
        let mesh = load_triangles(renderer, &[[0.0, 0.0]], &[None]);
        let red = emissive_material(renderer, [1.0, 0.0, 0.0]);
        frame_of(vec![instance(mesh, red.index(), [0.0, 0.0])])
    }

    /// A Material that is black and unlit, so only its `emissive` colour
    /// reaches the target.
    fn emissive_material(renderer: &mut Renderer, emissive: [f32; 3]) -> MaterialHandle {
        let [red, green, blue] = emissive;
        renderer.material_pool.insert(
            &mut renderer.load_staging,
            MaterialData {
                base_color: [0.0, 0.0, 0.0, 1.0],
                metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                emissive: [red, green, blue, 1.0],
                texture_indices: [MaterialPool::DEFAULT_LAYER; 4],
                extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
            },
        )
    }

    /// Loads a Mesh of a triangle 0.6 wide about each of `centres`, each its
    /// own Primitive drawn with its Material of `materials`, counter-clockwise
    /// towards -Z; its QTangents are the identity, so its normals are +Z.
    fn load_triangles(
        renderer: &mut Renderer,
        centres: &[[f32; 2]],
        materials: &[Option<MaterialHandle>],
    ) -> MeshHandle {
        let vertex = |x: f32, y: f32| CookedVertex {
            position: [x, y, 0.5],
            uv: [0.0, 0.0],
            qtangent: [0, 0, 0, i16::MAX],
        };
        let vertices = centres
            .iter()
            .flat_map(|&[x, y]| {
                [
                    vertex(x - 0.3, y - 0.3),
                    vertex(x + 0.3, y - 0.3),
                    vertex(x, y + 0.3),
                ]
            })
            .collect();
        let primitives = (0..centres.len() as u32)
            .map(|primitive| CookedPrimitive {
                first_index: 3 * primitive,
                index_count: 3,
                material: None,
                meshlets: vec![CookedMeshlet {
                    vertices: (3 * primitive..3 * primitive + 3).collect(),
                    triangles: vec![[0, 1, 2]],
                }],
            })
            .collect();
        let mesh = CookedMesh {
            vertices,
            indices: (0..3 * centres.len() as u32).collect(),
            primitives,
            bounds: TRIANGLE_BOUNDS,
        };
        renderer
            .mesh_pool
            .load(&mut renderer.load_staging, &mesh, materials)
            .expect("the triangles fit")
    }

    /// Loads a Mesh of a 1 m square, its Material given by its Instances,
    /// facing +Z, whose bounds are its own.
    fn load_quad(renderer: &mut Renderer) -> MeshHandle {
        let vertices = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
            .map(|[x, y]| CookedVertex {
                position: [x, y, 0.0],
                uv: [0.0, 0.0],
                qtangent: [0, 0, 0, i16::MAX],
            })
            .to_vec();
        let mesh = CookedMesh {
            vertices,
            indices: vec![0, 1, 2, 0, 2, 3],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: 6,
                material: None,
                meshlets: vec![CookedMeshlet {
                    vertices: vec![0, 1, 2, 3],
                    triangles: vec![[0, 1, 2], [0, 2, 3]],
                }],
            }],
            bounds: [[-0.5, -0.5, 0.0], [0.5, 0.5, 0.0]],
        };
        renderer
            .mesh_pool
            .load(&mut renderer.load_staging, &mesh, &[None])
            .expect("the quad fits")
    }

    /// An Instance of `mesh` moved by `offset` in x and y.
    fn instance(mesh: MeshHandle, material_id: u32, offset: [f32; 2]) -> InstanceData {
        let identity = padded_columns(glam::Mat3::IDENTITY);
        let [x, y] = offset;
        InstanceData {
            rotation_scale: identity,
            normal_matrix: identity,
            anchor: [0; 3],
            material_id,
            offset: [x, y, 0.0],
            mesh: mesh.0,
        }
    }

    /// Frame data that writes `instances` into the first slots, seen with an
    /// identity view-projection.
    fn frame_of(instances: Vec<InstanceData>) -> FrameData {
        let slots = instances.len();
        FrameData {
            camera: CameraUniform {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                ..CameraUniform::default()
            },
            light: LightUniform::default(),
            instance_slot_count: slots as u32,
            instances,
            instance_writes: vec![InstanceWrite {
                first_slot: 0,
                instances: 0..slots,
                moving: false,
            }],
        }
    }

    /// Uploads and renders `frame` into a new `width` × `height` target, and
    /// reads it back, with what the frame recorded and its GPU stats.
    fn render_frame(
        context: &GpuContext,
        renderer: &mut Renderer,
        frame: &FrameData,
        width: u32,
        height: u32,
    ) -> (image::RgbaImage, FrameRecorder, GpuFrameStats) {
        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame Data Test Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TARGET_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let mut recorder = FrameRecorder::new();
        renderer.upload(frame, &mut recorder);
        renderer.render(
            frame,
            &target.create_view(&wgpu::TextureViewDescriptor::default()),
            &mut recorder,
        );
        let pixels = read_back(context, &target).expect("read back failed");
        let image =
            image::RgbaImage::from_raw(width, height, pixels).expect("width × height pixels");
        renderer.flush_gpu_frame_stats().expect("GPU stats flushed");
        let gpu = *renderer
            .gpu_frame_stats()
            .last()
            .expect("the frame's GPU stats");
        (image, recorder, gpu)
    }
}
