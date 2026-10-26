use std::time::Instant;

use crate::assets::{ARENA_CAPACITY_BYTES, MaterialPool, MeshPool};
use crate::gpu::GpuContext;
use crate::render::acceleration_structures::DYNAMIC_TLAS_BUILD;
use crate::render::bend_pass::{BEND_PASS, BendPass};
use crate::render::bind_groups::{FRAME, MATERIALS, SCENE_GEOMETRY};
use crate::render::blas_refit::{BLAS_REFIT_PASS, BlasRefit};
use crate::render::culling_pass::{CullingPass, CullingPhase, DrawCapacity};
use crate::render::depth_pyramid_pass::{DEPTH_PYRAMID_PASS, DepthPyramidPass};
use crate::render::froxel_pass::{FROXEL_PASS, FroxelPass};
use crate::render::light_count_pass::LightCountPass;
use crate::render::main_pass::{DEPTH_PREPASS, LIT_PASS, MainPass, PHASE_TWO_DEPTH_PASS};
use crate::render::readback::{FrameShape, GpuStatsReadback};
use crate::render::shadow_mask_pass::{SHADOW_MASK_PASS, ShadowMaskPass};
use crate::render::targets::RenderTargets;
use crate::render::tonemapping_pass::{TONEMAPPING_PASS, TonemappingPass};
use crate::render::{
    AccelerationStructureBytes, CameraUniform, DynamicTlas, FrameData, FrameStaging,
    INSTANCE_CAPACITY, InstanceData, LIGHT_CAPACITY, LightData, LightUniform, LoadStaging,
    STATIC_TLAS_CAPACITY, StaticTlas,
};
use crate::stats::{FrameRecorder, GpuFrameStats};

/// The passes `render` records, in order, each timed from its start to its
/// end, after the dynamic TLAS's build, timed from the end of the marker
/// before it to the start of the marker after it in frames that build it.
/// The bend pass runs in frames that bend Meshes, and the BLAS refit, timed
/// from the end of the bend pass to the start of the marker after it, in
/// those frames with the crowd's refit form; phase two's depth pass and the
/// shadow mask pass only with shadow rays on (`SHADOW_RAY_PASSES`). The
/// rebuild form, an oracle, times neither its rebuild nor its build.
pub(crate) const TIMED_PASSES: [&str; 12] = [
    DYNAMIC_TLAS_BUILD,
    BEND_PASS,
    BLAS_REFIT_PASS,
    CullingPhase::One.label(),
    FROXEL_PASS,
    DEPTH_PREPASS,
    DEPTH_PYRAMID_PASS,
    CullingPhase::Two.label(),
    PHASE_TWO_DEPTH_PASS,
    SHADOW_MASK_PASS,
    LIT_PASS,
    TONEMAPPING_PASS,
];

/// The passes only shadow rays run: with them off, each reads as taking no
/// time.
pub(crate) const SHADOW_RAY_PASSES: [&str; 2] = [PHASE_TWO_DEPTH_PASS, SHADOW_MASK_PASS];

/// The CPU stages of the dynamic TLAS's build: its recording and its
/// submit, the CPU cost ADR 0020 caps.
pub(crate) const DYNAMIC_TLAS_RECORDING: &str = "dynamic_tlas";

/// The CPU stage that records the frame's BLAS updates, with the crowd's
/// refit or rebuild form, and submits them with the frame's commands before
/// them.
pub(crate) const BLAS_REFIT: &str = "blas_refit";
pub(crate) const DYNAMIC_TLAS_SUBMIT: &str = "dynamic_tlas_submit";

/// The Render switches a Renderer is created with and keeps. The defaults are
/// what ships; capture and bench take others, for equivalence checks and
/// A/Bs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderSwitches {
    /// Whether the culling pass draws only the Instances whose Mesh bounds
    /// reach into the view frustum, rather than every Instance in use.
    pub culling: bool,
    /// Whether the lit pass shades each pixel with the point lights its
    /// Froxel's list holds, rather than every point light (brute force).
    pub froxel_lights: bool,
    /// Whether the shadow mask pass traces a hard shadow ray at each pixel
    /// toward each light that reaches it, against the static and dynamic
    /// TLAS (ADR 0020), for the lit pass to read, rather than light every
    /// pixel the light faces.
    pub shadow_rays: bool,
    /// Whether a compute pass after the frame's others counts into the GPU
    /// stats the pixels where the frame drew a surface at the first sample,
    /// with the point lights the surface's Froxel lists and those whose
    /// range reaches it, from the depth the main pass then keeps. Off, as it
    /// ships, nothing counts them.
    pub count_point_lights: bool,
    /// The form the `crowd` scene's figures, which the GPU bends each frame,
    /// take in the TLAS.
    pub crowd_form: CrowdForm,
}

/// How the shapes the GPU bends Meshes to each frame reach the TLAS (ADR
/// 0020): the forms M3's `crowd` scene measures for M7 to pick from.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CrowdForm {
    /// The bent Meshes have no BLAS: rigid shadow proxies, one per bone,
    /// cast their shadows as Moving Instances.
    #[default]
    Proxies,
    /// Each bent Mesh's BLAS is refitted to its bent shape each frame
    /// through wgpu-hal, and its Instance is a Moving Instance. Metal only:
    /// wgpu-core rebuilds BLASes but never updates them.
    Refit,
    /// As `Refit`, each BLAS rebuilt through wgpu instead: what refit must
    /// match, on any backend.
    Rebuild,
}

impl CrowdForm {
    pub const ALL: [Self; 3] = [Self::Proxies, Self::Refit, Self::Rebuild];

    /// Its name on the command line.
    pub fn name(self) -> &'static str {
        match self {
            Self::Proxies => "proxies",
            Self::Refit => "refit",
            Self::Rebuild => "rebuild",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|form| form.name() == name)
    }

    /// Why it cannot run on `backend`, if it cannot.
    pub fn unavailable_on(self, backend: wgpu::Backend) -> Option<&'static str> {
        (self == CrowdForm::Refit && backend != wgpu::Backend::Metal)
            .then_some("the crowd's refit form updates BLASes through wgpu-hal on Metal only")
    }
}

impl Default for RenderSwitches {
    fn default() -> Self {
        Self {
            culling: true,
            froxel_lights: true,
            shadow_rays: true,
            count_point_lights: false,
            crowd_form: CrowdForm::Proxies,
        }
    }
}

impl RenderSwitches {
    /// Each switch by the name of the WGSL `override` that every shader
    /// module declares for it, with its value as a pipeline sets it.
    pub(crate) fn constants(&self) -> [(&'static str, f64); 3] {
        [
            ("CULLING", f64::from(u8::from(self.culling))),
            ("FROXEL_LIGHTS", f64::from(u8::from(self.froxel_lights))),
            ("SHADOW_RAYS", f64::from(u8::from(self.shadow_rays))),
        ]
    }
}

/// Draws Frame data into a target view, the window's surface or an offscreen
/// texture, from GPU resources and Pools it owns; it reads no ECS World. It
/// holds the bind groups passes share, the render targets and both TLAS, and
/// calls the passes in ADR 0019's order.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    aspect_ratio: f32,
    render_targets: RenderTargets,
    bend_pass: BendPass,
    froxel_pass: FroxelPass,
    culling_pass: CullingPass,
    main_pass: MainPass,
    shadow_mask_pass: ShadowMaskPass,
    /// Whether the frame traces shadow rays, which the shadow mask pass and
    /// phase two's depth pass run for.
    shadow_rays: bool,
    /// With the crowd's refit or rebuild form.
    blas_refit: Option<BlasRefit>,
    /// Whether the dynamic TLAS's build is timed between markers: not with
    /// the crowd's rebuild form, whose rebuilds through wgpu would fail the
    /// first marker's query, as `DynamicTlas::mark` says.
    marks_dynamic_tlas_build: bool,
    depth_pyramid_pass: DepthPyramidPass,
    tonemapping_pass: TonemappingPass,
    /// With the `count_point_lights` Render switch on.
    light_count_pass: Option<LightCountPass>,
    camera_buffer: wgpu::Buffer,
    light_uniform_buffer: wgpu::Buffer,
    /// Created at its fixed capacity; it never grows.
    light_buffer: wgpu::Buffer,
    /// The camera, the lights and the Froxel lists (`FRAME`).
    frame_bind_group: wgpu::BindGroup,
    pub(crate) mesh_pool: MeshPool,
    pub(crate) material_pool: MaterialPool,
    pub(crate) load_staging: LoadStaging,
    instance_buffer: wgpu::Buffer,
    /// The Instances, the vertex Arena, the Mesh pool's tables and both
    /// TLAS (`SCENE_GEOMETRY`).
    scene_geometry_bind_group: wgpu::BindGroup,
    static_tlas: StaticTlas,
    dynamic_tlas: DynamicTlas,
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
    /// `target_format`, both non-zero, as `switches` set. The format is not
    /// sRGB: the tonemapping pass encodes sRGB itself.
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
        // samplers) serves every material. Created before the main pass
        // because its pipeline layout references its bind group layout.
        let mut load_staging = LoadStaging::new(device.clone());
        let material_pool = MaterialPool::new(&device, &mut load_staging);

        let render_targets = RenderTargets::new(&device, width, height);
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
            &[
                (&FRAME, &frame_bind_group_layout),
                (&MATERIALS, &material_pool.bind_group_layout),
                (&SCENE_GEOMETRY, &scene_geometry_bind_group_layout),
            ],
            culling_pass.meshlet_records(),
            &render_targets,
            switches,
        );

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Uniform Buffer"),
            size: std::mem::size_of::<LightUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: u64::from(LIGHT_CAPACITY) * std::mem::size_of::<LightData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let froxel_pass = FroxelPass::new(
            &device,
            &camera_buffer,
            &light_uniform_buffer,
            &light_buffer,
        );
        let light_count_pass = switches.count_point_lights.then(|| {
            LightCountPass::new(
                &device,
                &camera_buffer,
                &light_buffer,
                &froxel_pass,
                &render_targets,
            )
        });
        let frame_bind_group = FRAME.bind_group(
            &device,
            &frame_bind_group_layout,
            [
                ("camera", camera_buffer.as_entire_binding()),
                ("light", light_uniform_buffer.as_entire_binding()),
                ("lights", light_buffer.as_entire_binding()),
                (
                    "froxel_lists",
                    froxel_pass.froxel_lists().as_entire_binding(),
                ),
            ],
        );

        // Created at its fixed capacity; it never grows.
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: u64::from(INSTANCE_CAPACITY) * std::mem::size_of::<InstanceData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mesh_pool = MeshPool::new(&device, ARENA_CAPACITY_BYTES);
        let bend_pass = BendPass::new(&device, mesh_pool.vertex_arena());
        // The most a frame uploads: every buffer `upload` writes, in full.
        let frame_staging = FrameStaging::new(
            [
                &camera_buffer,
                &light_uniform_buffer,
                &light_buffer,
                &instance_buffer,
                culling_pass.instance_slot_count(),
                bend_pass.bends(),
            ]
            .iter()
            .map(|buffer| buffer.size())
            .sum(),
        );

        let static_tlas = StaticTlas::new(&device, STATIC_TLAS_CAPACITY);
        let dynamic_tlas = DynamicTlas::new(&device, &queue, context.backend());
        let blas_refit = BlasRefit::new(&device, &queue, context.backend(), switches.crowd_form);
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
                ("static_tlas", static_tlas.binding()),
                ("dynamic_tlas", dynamic_tlas.binding()),
            ],
        );

        let shadow_mask_pass = ShadowMaskPass::new(
            &device,
            &scene_geometry_bind_group_layout,
            [
                ("camera", &camera_buffer),
                ("light", &light_uniform_buffer),
                ("lights", &light_buffer),
                ("froxel_lists", froxel_pass.froxel_lists()),
            ],
            &render_targets,
            switches,
        );
        let depth_pyramid_pass = DepthPyramidPass::new(&device, &render_targets);
        let tonemapping_pass =
            TonemappingPass::new(&device, target_format, &render_targets, &mut load_staging);
        let gpu_stats = GpuStatsReadback::new(
            &device,
            queue.get_timestamp_period(),
            if switches.shadow_rays {
                &[]
            } else {
                &SHADOW_RAY_PASSES
            },
        );

        let adapter_info = context.adapter.get_info();
        Self {
            mesh_pool,
            material_pool,
            load_staging,
            device,
            queue,
            aspect_ratio: width as f32 / height as f32,
            render_targets,
            bend_pass,
            froxel_pass,
            culling_pass,
            main_pass,
            shadow_mask_pass,
            shadow_rays: switches.shadow_rays,
            blas_refit,
            marks_dynamic_tlas_build: switches.crowd_form != CrowdForm::Rebuild,
            depth_pyramid_pass,
            tonemapping_pass,
            light_count_pass,
            camera_buffer,
            light_uniform_buffer,
            light_buffer,
            frame_bind_group,
            instance_buffer,
            scene_geometry_bind_group,
            static_tlas,
            dynamic_tlas,
            acceleration_structure_bytes: AccelerationStructureBytes::estimate(&adapter_info),
            lost_instance_writes: false,
            uploaded_instance_slot_count: 0,
            transient_saves_memory: adapter_info.transient_saves_memory == Some(true),
            frame_staging,
            frame_encoder: None,
            gpu_stats,
        }
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        // A pass whose bind groups read a target rebuilds them here, after
        // the render targets.
        self.render_targets = RenderTargets::new(&self.device, width, height);
        self.depth_pyramid_pass
            .bind_render_targets(&self.device, &self.render_targets);
        self.culling_pass
            .bind_render_targets(&self.device, &self.render_targets);
        self.main_pass
            .bind_render_targets(&self.device, &self.render_targets);
        self.shadow_mask_pass
            .bind_render_targets(&self.device, &self.render_targets);
        self.tonemapping_pass
            .bind_render_targets(&self.device, &self.render_targets);
        if let Some(light_count_pass) = &mut self.light_count_pass {
            light_count_pass.bind_render_targets(&self.device, &self.render_targets);
        }
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// The width of the target over its height.
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Records the static TLAS's rebuild, if `frame` changes it, and the
    /// copies of `frame`'s uniforms, the camera's with the static TLAS's
    /// Anchor, the Instances it writes, its bends and its Instance slot
    /// count, if it changed, into the frame's encoder, which `render` submits; every
    /// upload is followed by a render. First it submits the loads staged
    /// since the last frame, and, when they loaded Meshes, waits for the GPU
    /// to build their BLASes and to compact them.
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
        self.static_tlas
            .update(frame, &self.mesh_pool, &mut encoder);
        recorder.static_instances_left_out = self.static_tlas.instances_left_out();
        // Shadow rays start in the static TLAS's frame, whose Anchor the
        // first Frame data's camera set, and the dynamic TLAS shares it.
        let tlas_anchor = self.static_tlas.anchor();
        let camera = CameraUniform {
            tlas_anchor,
            // Prepare lists no more than the dynamic TLAS holds.
            dynamic_tlas_entries: frame.moving_instances.len() as u32,
            ..frame.camera
        };
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
        let light_upload = (!frame.lights.is_empty())
            .then(|| (&self.light_buffer, 0, bytemuck::cast_slice(&frame.lights)));
        let bend_upload = (!frame.bends.is_empty()).then(|| {
            (
                self.bend_pass.bends(),
                0,
                bytemuck::cast_slice(&frame.bends),
            )
        });
        let uploads = [
            (&self.camera_buffer, 0, bytemuck::bytes_of(&camera)),
            (
                &self.light_uniform_buffer,
                0,
                bytemuck::bytes_of(&frame.light),
            ),
        ]
        .into_iter()
        .chain(light_upload)
        .chain(bend_upload)
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
        self.frame_encoder = Some(encoder);
        recorder.stage("upload", started);
    }

    /// Draws `frame`, uploaded by `upload`, into `target`, a view in the
    /// format and size this Renderer renders to, timing its passes for
    /// `gpu_frame_stats`, which reads them back a few frames later, and
    /// submits it after the dynamic TLAS's build, if it has one, which it
    /// records into a command buffer of its own; with BLAS updates, the
    /// frame's commands up to its bend pass and the updates are submitted
    /// before the build. Returns the frame's submission.
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
        // Bent Meshes take their shapes before any pass reads their vertices.
        let bend_timestamps = timestamps.map(|timestamps| timestamps.compute_pass(BEND_PASS));
        let bent = self
            .bend_pass
            .record(&mut encoder, &frame.bends, bend_timestamps);
        let encoding = started.elapsed();
        // Their BLASes take them too, in a submission of their own, which
        // the dynamic TLAS's build, recorded after it, follows.
        let mut blas_refitted = false;
        if let Some(blas_refit) = self.blas_refit.as_mut().filter(|_| bent) {
            let started = Instant::now();
            let head = std::mem::replace(&mut encoder, create_frame_encoder(&self.device));
            let refit_end = timestamps.map(|timestamps| timestamps.marker_after(BLAS_REFIT_PASS));
            blas_refitted =
                blas_refit.submit(&self.device, &self.queue, head, &self.mesh_pool, refit_end);
            recorder.stage(BLAS_REFIT, started);
        }
        // The dynamic TLAS's build, its recording timed apart from the
        // frame's: `encode` resumes after it.
        let started = Instant::now();
        let bracket = timestamps
            .filter(|_| self.marks_dynamic_tlas_build)
            .map(|timestamps| timestamps.bracket(DYNAMIC_TLAS_BUILD));
        let tlas_anchor = self.static_tlas.anchor();
        let dynamic_tlas_commands = self
            .dynamic_tlas
            .record(&self.device, frame, &self.mesh_pool, tlas_anchor, bracket)
            .map(|(commands, uploaded_bytes)| {
                recorder.uploaded_bytes += uploaded_bytes;
                commands
            });
        recorder.stage(DYNAMIC_TLAS_RECORDING, started);
        let started = Instant::now() - encoding;
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
        // After the first culling dispatch, so ADR 0019's gate, which spans
        // from it to the end of the lit pass, counts the lights' culling.
        self.froxel_pass.record(
            &mut encoder,
            timestamps.map(|timestamps| timestamps.compute_pass(FROXEL_PASS)),
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
        if self.shadow_rays {
            self.main_pass.record_phase_two_depth(
                &mut encoder,
                &self.render_targets,
                &bind_groups,
                self.culling_pass.packed_indices(),
                self.culling_pass.frame_draw(),
                timestamps.map(|timestamps| timestamps.render_pass(PHASE_TWO_DEPTH_PASS)),
            );
            self.shadow_mask_pass.record(
                &mut encoder,
                &self.scene_geometry_bind_group,
                timestamps.map(|timestamps| timestamps.compute_pass(SHADOW_MASK_PASS)),
            );
        }
        self.main_pass.record_lit_pass(
            &mut encoder,
            &self.render_targets,
            &bind_groups,
            self.culling_pass.packed_indices(),
            self.culling_pass.frame_draw(),
            frame.background,
            timestamps.map(|timestamps| timestamps.render_pass(LIT_PASS)),
        );
        self.tonemapping_pass.record(
            &mut encoder,
            target,
            timestamps.map(|timestamps| timestamps.render_pass(TONEMAPPING_PASS)),
        );
        // Untimed, after the passes the frame's GPU time spans.
        if let Some(light_count_pass) = &self.light_count_pass {
            light_count_pass.record(&mut encoder);
        }
        // The depth pre-pass draws phase one's draw, and phase two's depth
        // pass or the lit pass phase two's into depth, then the lit pass both
        // phases', each with one indirect call; the tonemapping pass draws
        // one triangle.
        recorder.draw_calls += 4;
        // A frame whose refit went untimed spans its build's submission
        // from the bend pass, so its build is not counted apart.
        let refit_untimed = bent && self.blas_refit.is_some() && !blas_refitted;
        self.gpu_stats.shape(FrameShape {
            dynamic_tlas_built: dynamic_tlas_commands.is_some()
                && self.marks_dynamic_tlas_build
                && !refit_untimed,
            bent,
            blas_refitted,
        });
        self.gpu_stats.end_frame(
            &mut encoder,
            self.culling_pass.frame_draw(),
            self.froxel_pass.light_counts(),
            self.shadow_mask_pass.shadow_mask_lights_overflowed(),
        );

        let command_buffer = encoder.finish();
        recorder.stage("encode", started);

        // Its own submit, timed on its own: the dynamic build's CPU cost is
        // its recording and its submit (ADR 0020).
        let started = Instant::now();
        if let Some(commands) = dynamic_tlas_commands {
            self.queue.submit(std::iter::once(commands));
        }
        recorder.stage(DYNAMIC_TLAS_SUBMIT, started);

        let started = Instant::now();
        // submit will accept anything that implements IntoIter
        let submission = self.queue.submit(std::iter::once(command_buffer));
        recorder.stage("submit", started);
        submission
    }

    /// The GPU stats of earlier frames that the last `upload` or
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
            &self.light_uniform_buffer,
            &self.light_buffer,
            &self.instance_buffer,
        ];
        buffers.iter().map(|buffer| buffer.size()).sum::<u64>()
            + self
                .render_targets
                .gpu_memory_bytes(self.transient_saves_memory)
            + self.bend_pass.gpu_memory_bytes()
            + self
                .blas_refit
                .as_ref()
                .map_or(0, BlasRefit::gpu_memory_bytes)
            + self.froxel_pass.gpu_memory_bytes()
            + self.shadow_mask_pass.gpu_memory_bytes()
            + self.culling_pass.gpu_memory_bytes()
            + self.mesh_pool.gpu_memory_bytes()
            + self.estimated_acceleration_structure_bytes()
            + self.dynamic_tlas.gpu_memory_bytes()
            + self
                .material_pool
                .gpu_memory_bytes(self.transient_saves_memory)
            + self.frame_staging.gpu_memory_bytes()
            + self.gpu_stats.gpu_memory_bytes()
            + self.tonemapping_pass.gpu_memory_bytes()
    }

    /// Bytes of its acceleration structures, estimated from the bytes
    /// measured on its backend, since wgpu exposes no acceleration-structure
    /// size (ADR 0017).
    pub fn estimated_acceleration_structure_bytes(&self) -> u64 {
        let bytes = &self.acceleration_structure_bytes;
        self.mesh_pool.estimated_blas_bytes(bytes)
            + self.static_tlas.estimated_bytes(bytes)
            + self.dynamic_tlas.estimated_bytes(bytes)
    }
}

fn create_frame_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Frame Encoder"),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::Write;

    use bevy_ecs::prelude::*;
    use bevy_ecs::system::RunSystemOnce;

    use super::*;
    use crate::assets::material::MaterialData;
    use crate::assets::pools::tests::{logged, read_storage};
    use crate::assets::{MaterialHandle, MeshHandle};
    use crate::camera::{Camera, ev100_for_exposure, projection_matrix};
    use crate::capture::read_back;
    use crate::components::{MeshRef, Transform};
    use crate::cooked::{CookedMesh, CookedMeshlet, CookedPrimitive, CookedVertex};
    use crate::environment::Environment;
    use crate::render::culling_pass::FrameDraw;
    use crate::render::depth_pyramid_pass::tests::read_texture;
    use crate::render::froxel_pass::{
        FIRST_FROXEL_SLICE_DEPTH, FROXEL_COLUMNS, FROXEL_COUNT, FROXEL_LIGHT_CAPACITY, FROXEL_ROWS,
        FROXEL_SLICES, FROXEL_SLICES_PER_OCTAVE,
    };
    use crate::render::instance::{ANCHOR_CELL_SIZE, anchor_and_offset, padded_columns};
    use crate::render::prepare::{InstanceWrite, MovingInstance};
    use crate::render::shadow_mask_pass::SHADOW_MASK_LIGHTS;
    use crate::render::{FramePreparer, NO_MATERIAL, NO_MESH};
    use crate::time::FixedTime;
    use crate::transform::propagate_world_transforms;

    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    const SIZE: u32 = 16;
    /// Emissive red, green and blue exposed to 1, as GT7 shows them:
    /// Polyphony's sample implementation gives 0.948 red, 0.875 green with
    /// 0.027 red, and 0.855 blue, which sRGB encodes as 249.1, 240.5 with
    /// 46.1, and 238.0. The dither moves each channel by up to a step.
    const RED: [u8; 4] = [249, 0, 0, 255];
    const GREEN: [u8; 4] = [46, 240, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 238, 255];

    /// Whether `pixel` shows `colour`, each channel within the dither's step.
    fn shows(pixel: [u8; 4], colour: [u8; 4]) -> bool {
        pixel
            .iter()
            .zip(colour)
            .all(|(&channel, expected)| channel.abs_diff(expected) <= 1)
    }

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

        assert_eq!(recorder.draw_calls, 4);
        assert!(shows(image.get_pixel(SIZE / 2, SIZE / 2).0, RED));
        // The Frame data's background, exposed (0.1, 0.2, 0.3), as GT7 shows
        // it in tests/capture.rs.
        let corner = image.get_pixel(0, 0).0;
        assert!(shows(corner, [88, 123, 149, 255]), "corner {corner:?}");
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
        assert!(shows(resized_image.get_pixel(width / 2, height / 2).0, RED));
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

        assert_eq!(recorder.draw_calls, 4);
        let quarters = [(4, 4), (12, 4), (4, 12), (12, 12)].map(|(x, y)| image.get_pixel(x, y).0);
        let expected = [RED, GREEN, BLUE, BLUE];
        assert!(
            quarters
                .iter()
                .zip(expected)
                .all(|(&pixel, colour)| shows(pixel, colour)),
            "{quarters:?}"
        );
    }

    /// Twice the Instances of a two-triangle Mesh draw twice the Instances
    /// and triangles, read back from the GPU, in the same draw calls: one in
    /// the depth pre-pass, one in phase two's depth pass, one in the lit
    /// pass and one in the tonemapping pass.
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

        assert_eq!(drawn(3), (3, 6, 4));
        assert_eq!(drawn(6), (6, 12, 4));
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
        for switches in [
            RenderSwitches {
                culling: false,
                ..RenderSwitches::default()
            },
            RenderSwitches::default(),
        ] {
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
            assert!(shows(image.get_pixel(SIZE / 2, 4).0, RED));
            assert_eq!(shows(image.get_pixel(SIZE / 2, 12).0, RED), culled);
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

    /// Quads seen from the origin: a near one over the upper left quarter of
    /// the view, one wholly behind it, one partly behind it, one turned to
    /// reach through it, and one apart. Phase two draws all five in the first
    /// frame, against an empty pyramid; from the third, it culls the hidden
    /// one, which Frame stats count out, and draws the one partly behind and
    /// the one whose nearest corner is in front; when the near one moves
    /// behind the camera, phase two draws the hidden one again in that frame.
    /// Every frame draws the image culling off draws. The Renderer is created
    /// at another size and resized first, so a pyramid, or a culling bind
    /// group, left at the old size would cull wrongly.
    #[test]
    fn an_instance_hidden_behind_a_nearer_one_is_culled_by_phase_two_until_it_reappears() {
        let Some(context) = gpu_context() else {
            return;
        };
        let camera = CameraUniform {
            view_proj: (projection_matrix(1.0) * Camera::new().view_matrix()).to_cols_array_2d(),
            exposure: 1.0,
            ..CameraUniform::default()
        };
        // Each frame's image and Instances drawn.
        let frames = |switches| {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, 24, 24, switches);
            renderer.resize(SIZE, SIZE);
            let [red, green, blue] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
                .map(|emissive| emissive_material(&mut renderer, emissive));
            let quad = load_quad(&mut renderer);
            // A quad of `size` metres turned `turn` radians about +Y, centred
            // at `centre`.
            let placed = |material: MaterialHandle, centre: [f32; 3], size: f32, turn: f32| {
                let rotation_scale = glam::Mat3::from_rotation_y(turn)
                    * glam::Mat3::from_diagonal(glam::vec3(size, size, 1.0));
                InstanceData {
                    rotation_scale: padded_columns(rotation_scale),
                    offset: centre,
                    ..instance(quad, material.index(), [0.0, 0.0])
                }
            };
            let hidden = placed(green, [-1.0, 1.0, -6.0], 0.8, 0.0);
            let partly_hidden = placed(blue, [0.0, 1.0, -6.0], 0.5, 0.0);
            // Its left edge 0.22 m in front of the near quad, its right edge
            // as far behind it.
            let through = placed(blue, [-0.93, 0.6, -3.0], 0.5, std::f32::consts::FRAC_PI_3);
            let apart = placed(red, [1.2, -0.6, -6.0], 0.5, 0.0);
            [-3.0, -3.0, -3.0, 5.0].map(|near_z| {
                let near = placed(red, [-1.0, 1.0, near_z], 2.0, 0.0);
                let frame = FrameData {
                    camera,
                    ..frame_of(vec![near, hidden, partly_hidden, through, apart])
                };
                let (image, _, gpu) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);
                (image, gpu.instances_drawn)
            })
        };
        let culled = frames(RenderSwitches::default());
        let unculled = frames(RenderSwitches {
            culling: false,
            ..RenderSwitches::default()
        });

        let drawn = |frames: &[(image::RgbaImage, u32); 4]| frames.clone().map(|(_, drawn)| drawn);
        // The near one is behind the camera in the last frame.
        assert_eq!(drawn(&culled), [5, 5, 4, 4]);
        assert_eq!(drawn(&unculled), [5, 5, 5, 5]);
        // A pixel the hidden quad covers: the near quad's red, then its green.
        assert!(shows(culled[2].0.get_pixel(4, 4).0, RED));
        assert!(shows(culled[3].0.get_pixel(4, 4).0, GREEN));
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
        let unculled = frames(RenderSwitches {
            culling: false,
            ..RenderSwitches::default()
        });

        let [
            (_, out_of_view, ..),
            (entering, both, entering_first, entering_count),
            (_, _, _, after),
        ] = &culled;
        assert_eq!((*out_of_view, *both), (3, 6));
        assert_eq!((*entering_first, *entering_count, *after), (3, 3, 0));
        assert!(shows(entering.get_pixel(SIZE / 2, 12).0, RED));
        for (frame, (culled, unculled)) in culled.iter().zip(&unculled).enumerate() {
            assert!(
                culled.0 == unculled.0,
                "frame {frame} differs with culling off"
            );
        }
    }

    /// What stands between the light and the floor in
    /// `shadows_fall_from_both_tlas_and_shadow_rays_off_draw_none`.
    #[derive(Clone, Copy)]
    enum Occluder {
        Absent,
        Static,
        Moving,
        /// Its slot freed after it moved, so it is listed no more.
        Freed,
    }

    /// A white floor ahead of the camera, under a directional light shining
    /// the way the camera looks, and a square behind the camera, out of
    /// view, that hides the light from the floor's middle: its shadow rays
    /// darken the middle and leave the corners as the frame without the
    /// square draws them, and with shadow rays off, the frame draws that
    /// unshadowed image. The square shadows alike as a Moving Instance,
    /// from the dynamic TLAS, until the frame that frees it.
    #[test]
    fn shadows_fall_from_both_tlas_and_shadow_rays_off_draw_none() {
        let Some(context) = gpu_context() else {
            return;
        };
        let camera = CameraUniform {
            view_proj: (projection_matrix(1.0) * Camera::new().view_matrix()).to_cols_array_2d(),
            exposure: 1.0,
            ..CameraUniform::default()
        };
        let light = LightUniform {
            ambient_illuminance: [0.1, 0.1, 0.1, 0.0],
            directional_light_count: 1,
            ..LightUniform::default()
        };
        // Shining the way the camera looks: a directional light's offset
        // holds the direction it shines along.
        let sun = LightData {
            offset: [0.0, 0.0, -1.0],
            intensity: [1.0; 3],
            ..LightData::default()
        };
        // One image per frame, each frame's occluder as given, rendered in
        // turn by one Renderer. After the first, which anchors the static
        // TLAS, the camera's position is given from the next Anchor cell
        // over: the same position, so the same image, as long as both TLAS
        // keep the first frame's Anchor.
        let images = |switches, occluders: &[Occluder]| {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE, switches);
            let white = renderer.material_pool.insert(
                &mut renderer.load_staging,
                MaterialData {
                    base_color: [1.0; 4],
                    metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                    emissive: [0.0, 0.0, 0.0, 1.0],
                    layers: [MaterialPool::DEFAULT_LAYER; 4],
                    extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
                },
            );
            let quad = load_quad(&mut renderer);
            // A square of `size` metres facing the camera at `distance` ahead
            // of it, which is behind it when negative.
            let square = |size: f32, distance: f32| InstanceData {
                rotation_scale: padded_columns(glam::Mat3::from_diagonal(glam::vec3(
                    size, size, 1.0,
                ))),
                offset: [0.0, 0.0, -distance],
                ..instance(quad, white.index(), [0.0, 0.0])
            };
            // The floor fills the view, 1.66 m across at 2 m; the square's
            // shadow covers its middle metre.
            let floor = square(4.0, 2.0);
            let occluder = square(1.0, -1.0);
            let freed = InstanceData {
                mesh: NO_MESH,
                ..bytemuck::Zeroable::zeroed()
            };
            occluders
                .iter()
                .enumerate()
                .map(|(index, &occluder_state)| {
                    let mut frame = match occluder_state {
                        Occluder::Absent => frame_of(vec![floor]),
                        Occluder::Static => frame_of(vec![floor, occluder]),
                        Occluder::Moving => FrameData {
                            instance_writes: [(0, false), (1, true)]
                                .map(|(slot, moving)| InstanceWrite {
                                    first_slot: slot as u32,
                                    instances: slot..slot + 1,
                                    moving,
                                })
                                .into(),
                            moving_instances: vec![MovingInstance {
                                slot: 1,
                                instance: 1,
                            }],
                            ..frame_of(vec![floor, occluder])
                        },
                        Occluder::Freed => frame_of(vec![floor, freed]),
                    };
                    (frame.camera, frame.light, frame.lights) = (camera, light, vec![sun]);
                    if index > 0 {
                        frame.camera.anchor = [1, 0, 0];
                        frame.camera.offset = [-(ANCHOR_CELL_SIZE as f32), 0.0, 0.0];
                    }
                    render_frame(&context, &mut renderer, &frame, SIZE, SIZE).0
                })
                .collect::<Vec<_>>()
        };
        let image = |switches, occluder| images(switches, &[occluder]).remove(0);
        let shadowed = image(RenderSwitches::default(), Occluder::Static);
        let unshadowed = image(RenderSwitches::default(), Occluder::Absent);
        let untraced = image(
            RenderSwitches {
                shadow_rays: false,
                ..RenderSwitches::default()
            },
            Occluder::Static,
        );

        // The shadowed middle sees the ambient term alone, about a quarter as
        // bright once tonemapped: far more than the dither's step darker.
        let middle = |image: &image::RgbaImage| image.get_pixel(SIZE / 2, SIZE / 2).0;
        assert!(
            middle(&shadowed)[0].saturating_add(32) < middle(&unshadowed)[0],
            "{:?} not darker than {:?}",
            middle(&shadowed),
            middle(&unshadowed)
        );
        assert_eq!(shadowed.get_pixel(0, 0), unshadowed.get_pixel(0, 0));
        assert!(
            untraced == unshadowed,
            "with shadow rays off, the frame drew another image than the unshadowed one"
        );
        let moved = images(
            RenderSwitches::default(),
            &[Occluder::Moving, Occluder::Moving, Occluder::Freed],
        );
        assert!(
            moved[0] == shadowed,
            "a Moving Instance shadowed otherwise than a static one"
        );
        assert!(
            moved[1] == shadowed,
            "the dynamic TLAS left the static TLAS's frame as the camera's Anchor changed"
        );
        assert!(
            moved[2] == unshadowed,
            "a freed Moving Instance still shadows"
        );
    }

    /// A white floor ahead of the camera, lit by a point light 1 m behind
    /// the camera, and a square 0.2 m across behind the camera, out of view:
    /// between the light and the floor, it darkens the floor's middle; beyond
    /// the light, which ends each shadow ray, the frame is the one without it.
    #[test]
    fn a_point_lights_shadow_ray_ends_at_the_light() {
        let Some(context) = gpu_context() else {
            return;
        };
        let camera = CameraUniform {
            view_proj: (projection_matrix(1.0) * Camera::new().view_matrix()).to_cols_array_2d(),
            exposure: 1.0,
            ..CameraUniform::default()
        };
        let light = LightUniform {
            ambient_illuminance: [0.1, 0.1, 0.1, 0.0],
            point_light_count: 1,
            ..LightUniform::default()
        };
        // 3 m from the floor's middle, which it gives about 1 lux.
        let point = LightData {
            range: 10.0,
            offset: [0.0, 0.0, 1.0],
            intensity: [9.0; 3],
            ..LightData::default()
        };
        // The frame with the square, if any, this far behind the camera.
        let image = |behind: Option<f32>| {
            let mut renderer = Renderer::new(
                &context,
                TARGET_FORMAT,
                SIZE,
                SIZE,
                RenderSwitches::default(),
            );
            let white = renderer.material_pool.insert(
                &mut renderer.load_staging,
                MaterialData {
                    base_color: [1.0; 4],
                    metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                    emissive: [0.0, 0.0, 0.0, 1.0],
                    layers: [MaterialPool::DEFAULT_LAYER; 4],
                    extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
                },
            );
            let quad = load_quad(&mut renderer);
            // A square of `size` metres facing +Z at `distance` ahead of the
            // camera, which looks down -Z, or behind it when negative.
            let square = |size: f32, distance: f32| InstanceData {
                rotation_scale: padded_columns(glam::Mat3::from_diagonal(glam::vec3(
                    size, size, 1.0,
                ))),
                offset: [0.0, 0.0, -distance],
                ..instance(quad, white.index(), [0.0, 0.0])
            };
            let mut instances = vec![square(4.0, 2.0)];
            instances.extend(behind.map(|distance| square(0.2, -distance)));
            let frame = FrameData {
                camera,
                light,
                lights: vec![point],
                ..frame_of(instances)
            };
            render_frame(&context, &mut renderer, &frame, SIZE, SIZE).0
        };
        let unoccluded = image(None);
        // 0.5 m from the light, its shadow covers the floor's middle 1.2 m; 1 m
        // beyond the light, an unbounded ray would find it from the middle
        // 0.6 m.
        let between = image(Some(0.5));
        let beyond = image(Some(2.0));

        let middle = |image: &image::RgbaImage| image.get_pixel(SIZE / 2, SIZE / 2).0;
        assert!(
            middle(&between)[0].saturating_add(32) < middle(&unoccluded)[0],
            "{:?} not darker than {:?}",
            middle(&between),
            middle(&unoccluded)
        );
        assert!(
            beyond == unoccluded,
            "a square beyond the point light shadowed the floor"
        );
    }

    /// A white floor ahead of the camera, lit by `lights`, directional lights
    /// first, and a square `size` metres across `behind` the camera, out of
    /// view, if any, between the floor's middle and the light that lights
    /// it, as `shadows_fall_from_both_tlas_and_shadow_rays_off_draw_none` and
    /// `a_point_lights_shadow_ray_ends_at_the_light` place them: the Renderer
    /// that drew it, the image, the frame's GPU stats and what it logged.
    fn shadowed_floor(
        context: &GpuContext,
        lights: &[LightData],
        square: Option<(f32, f32)>,
    ) -> (Renderer, image::RgbaImage, GpuFrameStats, String) {
        let mut renderer = Renderer::new(
            context,
            TARGET_FORMAT,
            SIZE,
            SIZE,
            RenderSwitches::default(),
        );
        let white = renderer.material_pool.insert(
            &mut renderer.load_staging,
            MaterialData {
                base_color: [1.0; 4],
                metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                emissive: [0.0, 0.0, 0.0, 1.0],
                layers: [MaterialPool::DEFAULT_LAYER; 4],
                extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
            },
        );
        let quad = load_quad(&mut renderer);
        // A square of `size` metres facing +Z at `distance` ahead of the
        // camera, which looks down -Z, or behind it when negative.
        let square_at = |size: f32, distance: f32| InstanceData {
            rotation_scale: padded_columns(glam::Mat3::from_diagonal(glam::vec3(size, size, 1.0))),
            offset: [0.0, 0.0, -distance],
            ..instance(quad, white.index(), [0.0, 0.0])
        };
        let mut instances = vec![square_at(4.0, FLOOR_DISTANCE)];
        instances.extend(square.map(|(size, behind)| square_at(size, -behind)));
        // A directional light has no range.
        let directional_light_count = lights.iter().filter(|light| light.range == 0.0).count();
        let frame = FrameData {
            camera: CameraUniform {
                view_proj: (projection_matrix(1.0) * Camera::new().view_matrix())
                    .to_cols_array_2d(),
                exposure: 1.0,
                ..CameraUniform::default()
            },
            light: LightUniform {
                ambient_illuminance: [0.1, 0.1, 0.1, 0.0],
                directional_light_count: directional_light_count as u32,
                point_light_count: (lights.len() - directional_light_count) as u32,
                ..LightUniform::default()
            },
            lights: lights.to_vec(),
            ..frame_of(instances)
        };
        let ((image, _, gpu), log) =
            logged(|| render_frame(context, &mut renderer, &frame, SIZE, SIZE));
        (renderer, image, gpu, log)
    }

    /// How far ahead of the camera `shadowed_floor`'s floor lies, along the
    /// view axis: inside a Froxel slice, not at its edge.
    const FLOOR_DISTANCE: f32 = 2.25;

    /// Shining the way the camera looks; a directional light's offset holds
    /// the direction it shines along.
    const SUN: LightData = LightData {
        anchor: [0; 3],
        range: 0.0,
        offset: [0.0, 0.0, -1.0],
        _pad_offset: 0,
        intensity: [1.0; 3],
        _pad_intensity: 0,
    };

    /// The pixel's word in the shadow mask, its two halves: a bit per light,
    /// the first the sun's, which the square hides from the floor's middle
    /// and not from its corner, then the slice of the Froxel whose lights the
    /// bits follow, the floor's, in the second half's top bits.
    #[test]
    fn the_shadow_mask_holds_each_lights_bit_and_its_froxels_slice() {
        let Some(context) = gpu_context() else {
            return;
        };
        let (renderer, ..) = shadowed_floor(&context, &[SUN], Some((1.0, 1.0)));
        let halves: Vec<u32> = read_texture(
            &context,
            "@group(0) @binding(0) var source: texture_2d<u32>;
            @group(0) @binding(1) var<storage, read_write> values: array<u32>;
            @compute @workgroup_size(1)
            fn copy(@builtin(global_invocation_id) texel: vec3<u32>) {
                let word = textureLoad(source, texel.xy, 0);
                let first = 2u * (texel.y * textureDimensions(source).x + texel.x);
                values[first] = word.r;
                values[first + 1u] = word.g;
            }",
            renderer.render_targets.shadow_mask(),
            [SIZE, SIZE],
            2,
        )
        .into_iter()
        .map(f32::to_bits)
        .collect();

        let octaves = (FLOOR_DISTANCE / FIRST_FROXEL_SLICE_DEPTH).log2();
        let slice = (octaves * FROXEL_SLICES_PER_OCTAVE).floor() as u32 + 1;
        let second_half = slice << (SHADOW_MASK_LIGHTS - 32);
        let word = |x: u32, y: u32| {
            let first = 2 * (y * SIZE + x) as usize;
            [halves[first], halves[first + 1]]
        };
        assert_eq!(word(SIZE / 2, SIZE / 2), [1, second_half]);
        assert_eq!(word(0, 0), [0, second_half]);
    }

    /// The mask holds no bit for a light past SHADOW_MASK_LIGHTS: the sun
    /// after as many unlit directional lights, or a point light after them,
    /// casts no shadow, so the frame is the one without the square, and each
    /// pixel it lights is counted and warned about.
    #[test]
    fn a_light_past_the_shadow_masks_bits_casts_no_shadow_and_is_counted() {
        let Some(context) = gpu_context() else {
            return;
        };
        let unlit = LightData {
            intensity: [0.0; 3],
            ..SUN
        };
        // 1 m behind the camera, 3 m from the floor's middle.
        let point = LightData {
            range: 10.0,
            offset: [0.0, 0.0, 1.0],
            intensity: [9.0; 3],
            ..LightData::default()
        };
        // The sun's square, 1 m across 1 m behind the camera, and the point
        // light's, 0.2 m across 0.5 m behind it, between it and the floor.
        for (light, square) in [(SUN, (1.0, 1.0)), (point, (0.2, 0.5))] {
            let mut lights = vec![unlit; SHADOW_MASK_LIGHTS as usize];
            lights.push(light);
            let (_, unshadowed, ..) = shadowed_floor(&context, &lights, None);
            let (_, past, gpu, log) = shadowed_floor(&context, &lights, Some(square));
            assert!(
                past == unshadowed,
                "a light past the mask's bits shadowed the floor"
            );
            // The floor fills the view, and the light reaches all of it.
            assert_eq!(gpu.shadow_mask_lights_overflowed, SIZE * SIZE);
            assert!(log.contains("The shadow mask is full"), "{log}");
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
        // Exposed to 1, so the emissive reaches the target as it is.
        world.insert_resource(Camera {
            ev100: ev100_for_exposure(1.0),
            ..Camera::new()
        });
        world.insert_resource(FixedTime::default());
        world.insert_resource(Environment::default());
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
                    .filter(|(x, _, pixel)| columns.contains(x) && shows(pixel.0, RED))
                    .count()
            })
        };

        let [left, before] = red_pixels(&mut world, &mut renderer);
        assert!(left > 0 && before > 0, "{left} and {before} red pixels");
        world.despawn(right);
        assert_eq!(red_pixels(&mut world, &mut renderer), [left, 0]);
    }

    /// Nine point lights over a white matte quad that fills the view, each
    /// 0.1 m in front of it in a 3 × 3 grid 0.5 m apart, reaching 0.2 m: each
    /// lights the quad under it, which stays dark between them, so lights
    /// past the four the light uniform held reach the pixels they light.
    /// Lit through Froxel lists, the image is the one brute force draws.
    #[test]
    fn nine_point_lights_each_light_the_surface_under_them() {
        let Some(context) = gpu_context() else {
            return;
        };
        const SIDE: u32 = 48;
        let view_proj = projection_matrix(1.0) * Camera::new().view_matrix();
        let grid = [-0.5, 0.0, 0.5];
        let centres: Vec<glam::Vec3> = grid
            .iter()
            .flat_map(|&y| grid.map(|x| glam::vec3(x, y, -2.0)))
            .collect();
        let image_with = |switches| {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIDE, SIDE, switches);
            let white = renderer.material_pool.insert(
                &mut renderer.load_staging,
                MaterialData {
                    base_color: [1.0; 4],
                    metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                    emissive: [0.0, 0.0, 0.0, 1.0],
                    layers: [MaterialPool::DEFAULT_LAYER; 4],
                    extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
                },
            );
            let quad = load_quad(&mut renderer);
            let wall = InstanceData {
                rotation_scale: padded_columns(glam::Mat3::from_diagonal(glam::vec3(
                    2.0, 2.0, 1.0,
                ))),
                offset: [0.0, 0.0, -2.0],
                ..instance(quad, white.index(), [0.0, 0.0])
            };
            let lights = centres
                .iter()
                .map(|centre| LightData {
                    range: 0.2,
                    offset: (*centre + glam::Vec3::Z * 0.1).to_array(),
                    intensity: [0.01; 3],
                    ..LightData::default()
                })
                .collect();
            let frame = FrameData {
                camera: CameraUniform {
                    view_proj: view_proj.to_cols_array_2d(),
                    exposure: 1.0,
                    ..CameraUniform::default()
                },
                light: LightUniform {
                    point_light_count: 9,
                    ..LightUniform::default()
                },
                lights,
                ..frame_of(vec![wall])
            };
            render_frame(&context, &mut renderer, &frame, SIDE, SIDE).0
        };
        let image = image_with(RenderSwitches::default());
        let brute_force = image_with(RenderSwitches {
            froxel_lights: false,
            ..RenderSwitches::default()
        });

        // The red of the pixel at `position`, on the quad.
        let red_at = |position: glam::Vec3| {
            let clip = view_proj * position.extend(1.0);
            let normalised = clip.truncate() / clip.w;
            let x = (normalised.x + 1.0) / 2.0 * SIDE as f32;
            let y = (1.0 - normalised.y) / 2.0 * SIDE as f32;
            image.get_pixel(x as u32, y as u32).0[0]
        };
        let lit: Vec<u8> = centres.iter().map(|&centre| red_at(centre)).collect();
        let between = red_at(glam::vec3(0.25, 0.25, -2.0));
        assert!(lit.iter().all(|&red| red > 50), "{lit:?}");
        assert!(between <= 1, "{between} between the lights");
        assert!(
            image == brute_force,
            "Froxel lights differ from brute force"
        );
    }

    /// A white quad over the left half of the view, 1.8 m away, and point
    /// lights. With the `count_point_lights` Render switch on, the light
    /// count pass counts each pixel the quad covers, with the lights its
    /// Froxel lists and those whose range reaches the quad there: three in
    /// front reaching past it, listed and reaching everywhere; one just
    /// behind it, within its Froxels' slice of depth but short of it, listed
    /// in some and reaching none. Off, nothing counts them.
    #[test]
    fn the_light_count_pass_counts_the_point_lights_each_lit_pixel_lists_and_is_reached_by() {
        let Some(context) = gpu_context() else {
            return;
        };
        const SIDE: u32 = 48;
        let view_proj = projection_matrix(1.0) * Camera::new().view_matrix();
        let counts_with = |switches, lights: &[([f32; 3], f32)]| {
            let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIDE, SIDE, switches);
            let white = renderer.material_pool.insert(
                &mut renderer.load_staging,
                MaterialData {
                    base_color: [1.0; 4],
                    metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                    emissive: [0.0, 0.0, 0.0, 1.0],
                    layers: [MaterialPool::DEFAULT_LAYER; 4],
                    extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
                },
            );
            let quad = load_quad(&mut renderer);
            // Its right edge on the view's axis, between two columns of
            // pixels.
            let left_half = InstanceData {
                rotation_scale: padded_columns(glam::Mat3::from_diagonal(glam::vec3(
                    3.0, 4.0, 1.0,
                ))),
                offset: [-1.5, 0.0, -1.8],
                ..instance(quad, white.index(), [0.0, 0.0])
            };
            let frame = FrameData {
                camera: CameraUniform {
                    view_proj: view_proj.to_cols_array_2d(),
                    exposure: 1.0,
                    ..CameraUniform::default()
                },
                light: LightUniform {
                    point_light_count: lights.len() as u32,
                    ..LightUniform::default()
                },
                lights: lights
                    .iter()
                    .map(|&(offset, range)| LightData {
                        range,
                        offset,
                        intensity: [0.01; 3],
                        ..LightData::default()
                    })
                    .collect(),
                ..frame_of(vec![left_half])
            };
            let (_, _, gpu) = render_frame(&context, &mut renderer, &frame, SIDE, SIDE);
            (
                gpu.lit_pixels,
                gpu.listed_point_lights,
                gpu.reaching_point_lights,
            )
        };
        let counting = RenderSwitches {
            count_point_lights: true,
            ..RenderSwitches::default()
        };
        let covered = SIDE / 2 * SIDE;
        let reaching = [
            ([-0.3, 0.0, -0.8], 10.0),
            ([0.3, 0.0, -0.8], 10.0),
            ([0.0, 0.3, -0.8], 10.0),
        ];
        assert_eq!(
            counts_with(counting, &reaching),
            (covered, 3 * covered, 3 * covered)
        );
        let (lit, listed, reached) = counts_with(counting, &[([-0.3, 0.0, -1.95], 0.1)]);
        assert!(
            (lit, reached) == (covered, 0) && listed > 0,
            "{lit} lit pixels, {listed} listed and {reached} reaching the light behind the quad"
        );
        assert_eq!(counts_with(RenderSwitches::default(), &reaching), (0, 0, 0));
    }

    /// The sun, then point lights about a camera 100 km out, turned, among
    /// them more than a Froxel's list holds at one spot. Against a CPU
    /// assignment in f64 of every light to every Froxel whose bounds its
    /// range reaches into, each Froxel's list holds exactly its lights, in
    /// light order, up to its capacity, and Frame stats count the lights
    /// past it, with a warning; no buffer grows.
    #[test]
    fn each_froxel_lists_the_point_lights_whose_range_reaches_into_its_bounds() {
        let Some(context) = gpu_context() else {
            return;
        };
        let camera = Camera {
            position: glam::dvec3(100_000.3, 20.0, -70_000.6),
            yaw: 0.4,
            pitch: -0.2,
            ..Camera::new()
        };
        let view_proj = projection_matrix(16.0 / 9.0) * camera.view_matrix();
        let forward = camera.forward().as_dvec3();
        let up = camera.right().cross(camera.forward()).as_dvec3();
        let right = camera.right().as_dvec3();
        // The unit normal of the plane through the camera that holds the
        // screen rectangles' sides at `side` in normalised device
        // coordinates, across the screen in x, or down it in y, pointing to
        // higher coordinates; unprojected in f64 from points on it.
        let inverse = view_proj.as_dmat4().inverse();
        let side_normal = |side: f64, in_x: bool| {
            let unproject = |at: f64, along: f64| {
                let normalised = if in_x {
                    glam::dvec4(at, along, 0.5, 1.0)
                } else {
                    glam::dvec4(along, at, 0.5, 1.0)
                };
                let point = inverse * normalised;
                point.truncate() / point.w
            };
            let normal = unproject(side, -1.0).cross(unproject(side, 1.0));
            normal.normalize() * normal.dot(unproject(side + 1.0, 0.0)).signum()
        };
        let sides = |rectangles: u32, in_x: bool| -> Vec<glam::DVec3> {
            (0..=rectangles)
                .map(|side| side_normal(2.0 * f64::from(side) / f64::from(rectangles) - 1.0, in_x))
                .collect()
        };
        let (column_sides, row_sides) = (sides(FROXEL_COLUMNS, true), sides(FROXEL_ROWS, false));
        // Where each slice starts, in view depth; the last reaches to
        // infinity.
        let slice_starts: Vec<f64> = (0..FROXEL_SLICES)
            .map(|slice| match slice {
                0 => 0.0,
                _ => {
                    let octaves = f64::from(slice - 1) / f64::from(FROXEL_SLICES_PER_OCTAVE);
                    f64::from(FIRST_FROXEL_SLICE_DEPTH) * octaves.exp2()
                }
            })
            .chain([f64::INFINITY])
            .collect();
        // Per light, whether its range reaches past each column's and each
        // row's lower side and higher side, and which slices; `None` where
        // a side or slice boundary is within 1 mm of its reach, and 1 cm
        // more a kilometre out, past what f32 rounding moves, so no verdict
        // is a tie.
        let reaches = |centre: glam::DVec3, range: f64| {
            let margin = 1e-3 + 1e-5 * centre.length();
            let past = |normals: &[glam::DVec3]| -> Option<Vec<(bool, bool)>> {
                normals
                    .iter()
                    .map(|normal| {
                        let distance = normal.dot(centre);
                        let clear =
                            (distance + range).abs() > margin && (distance - range).abs() > margin;
                        clear.then_some((distance >= -range, distance <= range))
                    })
                    .collect()
            };
            let depth = forward.dot(centre);
            let clear = slice_starts.iter().all(|start| {
                (depth + range - start).abs() > margin && (depth - range - start).abs() > margin
            });
            let slices: Vec<bool> = slice_starts
                .windows(2)
                .map(|bounds| depth + range >= bounds[0] && depth - range <= bounds[1])
                .collect();
            Some((
                past(&column_sides)?,
                past(&row_sides)?,
                clear.then_some(slices)?,
            ))
        };
        // Uniform in [0, 1), from a fixed-seed generator.
        let mut seed: u64 = 0xf70e;
        let mut random = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        // From 5 m behind the camera to 80 m ahead, and one in ten 1 to 5 km
        // ahead, in the last slices, reaching five times as far.
        let scattered = std::iter::from_fn(|| {
            let far = random() < 0.1;
            let distance = match far {
                true => 1_000.0 + 4_000.0 * random(),
                false => 85.0 * random() - 5.0,
            };
            let [across, along_up] =
                [(); 2].map(|_| (0.6 * distance.abs() + 3.0) * (2.0 * random() - 1.0));
            let range = (0.2 + 11.8 * random()) * if far { 5.0 } else { 1.0 };
            Some((forward * distance + right * across + up * along_up, range))
        });
        // More than a list holds, 7.3 m ahead, each reaching 0.4 m.
        let overflowing = std::iter::repeat_n(
            (forward * 7.3 + right * 0.31 + up * 0.17, 0.4),
            FROXEL_LIGHT_CAPACITY as usize + 5,
        );
        let placed: Vec<(glam::DVec3, f64, _)> = scattered
            .filter_map(|(centre, range)| Some((centre, range, reaches(centre, range)?)))
            .take(300)
            .chain(overflowing.map(|(centre, range)| {
                (
                    centre,
                    range,
                    reaches(centre, range).expect("the lights at one spot are no tie"),
                )
            }))
            .collect();

        let mut expected = vec![Vec::new(); FROXEL_COUNT as usize];
        // Lists in the GPU's order: by row, column, then slice.
        let (columns, rows, slices) = (
            FROXEL_COLUMNS as usize,
            FROXEL_ROWS as usize,
            FROXEL_SLICES as usize,
        );
        for (point, (_, _, (column_sides, row_sides, slices_reached))) in (1..).zip(&placed) {
            for row in 0..rows {
                for column in 0..columns {
                    if column_sides[column].0
                        && column_sides[column + 1].1
                        && row_sides[row].0
                        && row_sides[row + 1].1
                    {
                        for slice in (0..slices).filter(|&slice| slices_reached[slice]) {
                            expected[(row * columns + column) * slices + slice].push(point);
                        }
                    }
                }
            }
        }
        let overflowed: usize = expected
            .iter()
            .map(|lights| lights.len().saturating_sub(FROXEL_LIGHT_CAPACITY as usize))
            .sum();
        assert!(overflowed > 0, "the lights at one spot overflow no list");

        let mut renderer = Renderer::new(
            &context,
            TARGET_FORMAT,
            SIZE,
            SIZE,
            RenderSwitches::default(),
        );
        let sun = LightData {
            offset: [0.0, -1.0, 0.0],
            intensity: [1.0; 3],
            ..LightData::default()
        };
        let lights = std::iter::once(sun)
            .chain(placed.iter().map(|&(centre, range, _)| {
                let (anchor, offset) = anchor_and_offset(camera.position + centre);
                LightData {
                    anchor,
                    range: range as f32,
                    offset,
                    intensity: [1.0; 3],
                    ..LightData::default()
                }
            }))
            .collect();
        let (anchor, offset) = anchor_and_offset(camera.position);
        let frame = FrameData {
            camera: CameraUniform {
                view_proj: view_proj.to_cols_array_2d(),
                anchor,
                offset,
                exposure: 1.0,
                ..CameraUniform::default()
            },
            light: LightUniform {
                directional_light_count: 1,
                point_light_count: placed.len() as u32,
                ..LightUniform::default()
            },
            lights,
            ..frame_of(Vec::new())
        };
        let unstaged_bytes = |renderer: &Renderer| {
            renderer.gpu_memory_bytes() - renderer.frame_staging.gpu_memory_bytes()
        };
        let created_bytes = unstaged_bytes(&renderer);
        let ((_, _, gpu), log) =
            logged(|| render_frame(&context, &mut renderer, &frame, SIZE, SIZE));

        let stride = FROXEL_LIGHT_CAPACITY as usize + 1;
        let lists = read_storage::<u32>(
            &context,
            renderer.froxel_pass.froxel_lists(),
            FROXEL_COUNT as usize * stride,
        );
        let differing: Vec<String> = lists
            .chunks(stride)
            .zip(&expected)
            .enumerate()
            .filter_map(|(froxel, (list, lights))| {
                let held = &lights[..lights.len().min(FROXEL_LIGHT_CAPACITY as usize)];
                let listed = &list[1..][..(list[0] as usize).min(stride - 1)];
                (listed != held).then(|| format!("Froxel {froxel}: {listed:?}, not {held:?}"))
            })
            .collect();
        assert!(
            differing.is_empty(),
            "{} lists differ:\n{}",
            differing.len(),
            differing[..differing.len().min(5)].join("\n")
        );
        assert_eq!(gpu.froxel_lights_overflowed as usize, overflowed);
        assert!(log.contains("The Froxel lists are full"), "{log}");
        assert_eq!(unstaged_bytes(&renderer), created_bytes);
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
        for switches in [
            RenderSwitches::default(),
            RenderSwitches {
                culling: false,
                ..RenderSwitches::default()
            },
        ] {
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
    pub(crate) fn gpu_context() -> Option<GpuContext> {
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
                layers: [MaterialPool::DEFAULT_LAYER; 4],
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
    /// identity view-projection and exposed to 1.
    fn frame_of(instances: Vec<InstanceData>) -> FrameData {
        let slots = instances.len();
        FrameData {
            camera: CameraUniform {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                exposure: 1.0,
                ..CameraUniform::default()
            },
            light: LightUniform::default(),
            lights: Vec::new(),
            background: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            instance_slot_count: slots as u32,
            instances,
            instance_writes: vec![InstanceWrite {
                first_slot: 0,
                instances: 0..slots,
                moving: false,
            }],
            moving_instances: Vec::new(),
            bends: Vec::new(),
        }
    }

    /// Uploads and renders `frame` into a new `width` × `height` target, and
    /// reads it back, with what the frame recorded and its GPU stats.
    pub(crate) fn render_frame(
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
