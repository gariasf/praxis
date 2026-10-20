use std::time::Instant;

use crate::assets::{ARENA_CAPACITY_BYTES, MaterialPool, MeshPool};
use crate::gpu::GpuContext;
use crate::render::bind_groups::{FRAME, MATERIALS, SCENE_GEOMETRY};
use crate::render::main_pass::MainPass;
use crate::render::targets::RenderTargets;
use crate::render::{
    CameraUniform, FrameData, FrameStaging, INSTANCE_CAPACITY, InstanceData, LightUniform,
    LoadStaging,
};
use crate::stats::FrameRecorder;

/// Draws Frame data into a target view, the window's surface or an offscreen
/// texture, from GPU resources and Pools it owns; it reads no ECS World. It
/// holds the bind groups passes share and the render targets, and calls the
/// passes in ADR 0019's order.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    aspect_ratio: f32,
    render_targets: RenderTargets,
    main_pass: MainPass,
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
    /// Set when a frame's uploads failed after it queued Instance writes,
    /// which then never landed: the next prepare must write every Instance.
    lost_instance_writes: bool,
    transient_saves_memory: bool,
    frame_staging: FrameStaging,
    /// The frame's encoder from `update`, holding its upload copies, until
    /// `render` records the pass into it and submits it.
    frame_encoder: Option<wgpu::CommandEncoder>,
}

impl Renderer {
    /// A Renderer with empty Pools that draws into `width` × `height` views of
    /// `target_format` textures, both non-zero.
    pub fn new(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
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

        let main_pass = MainPass::new(
            &device,
            target_format,
            &[
                (&FRAME, &frame_bind_group_layout),
                (&MATERIALS, &material_pool.bind_group_layout),
                (&SCENE_GEOMETRY, &scene_geometry_bind_group_layout),
            ],
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
            [&camera_buffer, &light_buffer, &instance_buffer]
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
            ],
        );

        let render_targets = RenderTargets::new(&device, width, height);

        Self {
            mesh_pool,
            material_pool,
            load_staging,
            device,
            queue,
            aspect_ratio: width as f32 / height as f32,
            render_targets,
            main_pass,
            camera_buffer,
            light_buffer,
            frame_bind_group,
            instance_buffer,
            scene_geometry_bind_group,
            lost_instance_writes: false,
            transient_saves_memory: context.adapter.get_info().transient_saves_memory == Some(true),
            frame_staging,
            frame_encoder: None,
        }
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        // A pass whose bind groups read a target rebuilds them here, after
        // the render targets.
        self.render_targets = RenderTargets::new(&self.device, width, height);
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// The width of the target over its height.
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Records the copies of `frame`'s uniforms and the Instances it writes
    /// into the frame's encoder, which `render` submits.
    pub fn upload(&mut self, frame: &FrameData, recorder: &mut FrameRecorder) {
        let started = Instant::now();
        // The window skips `render` while occluded or without a surface
        // texture; submitting the unrendered frame's copies returns its
        // staging slot.
        if let Some(unrendered) = self.frame_encoder.take() {
            self.queue.submit(std::iter::once(unrendered.finish()));
        }
        let mut encoder = create_frame_encoder(&self.device);
        let instance_size = std::mem::size_of::<InstanceData>() as wgpu::BufferAddress;
        let instance_uploads = frame.instance_writes.iter().map(|write| {
            (
                &self.instance_buffer,
                u64::from(write.first_slot) * instance_size,
                bytemuck::cast_slice(&frame.instances[write.instances.clone()]),
            )
        });
        let uploads = [
            (&self.camera_buffer, 0, bytemuck::bytes_of(&frame.camera)),
            (&self.light_buffer, 0, bytemuck::bytes_of(&frame.light)),
        ]
        .into_iter()
        .chain(instance_uploads);
        let uploaded_bytes = self
            .frame_staging
            .write(&self.device, &mut encoder, uploads);
        recorder.uploaded_bytes += uploaded_bytes;
        // 0 bytes: the staging slot failed to map, as logged, and the
        // frame's Instance writes never land.
        if uploaded_bytes == 0 {
            self.lost_instance_writes |= !frame.instances.is_empty();
        } else {
            recorder.instances_written += frame.instances.len() as u32;
        }
        self.frame_encoder = Some(encoder);
        recorder.stage("upload", started);
    }

    /// Draws `frame`, uploaded by `upload`, into `target`, a view in the
    /// format and size this Renderer renders to, and times its one pass into
    /// `timestamp_writes`. Returns the frame's submission.
    pub fn render(
        &mut self,
        frame: &FrameData,
        target: &wgpu::TextureView,
        timestamp_writes: Option<wgpu::RenderPassTimestampWrites<'_>>,
        recorder: &mut FrameRecorder,
    ) -> wgpu::SubmissionIndex {
        let started = Instant::now();
        let mut encoder = self
            .frame_encoder
            .take()
            .unwrap_or_else(|| create_frame_encoder(&self.device));
        self.material_pool.refresh_bind_group(&self.device);

        self.main_pass.record(
            &mut encoder,
            target,
            &self.render_targets,
            &[
                (&FRAME, &self.frame_bind_group),
                (&MATERIALS, &self.material_pool.bind_group),
                (&SCENE_GEOMETRY, &self.scene_geometry_bind_group),
            ],
            self.mesh_pool.index_buffer(),
            &frame.draws,
            timestamp_writes,
        );
        recorder.draw_calls += frame.draws.len() as u32;

        let command_buffer = encoder.finish();
        recorder.stage("encode", started);

        let started = Instant::now();
        // Loads staged since the last frame land before its draws.
        self.load_staging.submit(&self.queue);
        // submit will accept anything that implements IntoIter
        let submission = self.queue.submit(std::iter::once(command_buffer));
        recorder.stage("submit", started);
        submission
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
    /// included; the render target is the caller's.
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
            + self.mesh_pool.gpu_memory_bytes()
            + self
                .material_pool
                .gpu_memory_bytes(self.transient_saves_memory)
            + self.frame_staging.gpu_memory_bytes()
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

    use super::*;
    use crate::assets::Primitive;
    use crate::assets::material::MaterialData;
    use crate::capture::read_back;
    use crate::cooked::{CookedMesh, CookedMeshlet, CookedPrimitive, CookedVertex};
    use crate::render::instance::padded_columns;
    use crate::render::prepare::{Draw, InstanceWrite};

    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    const SIZE: u32 = 16;

    #[test]
    fn renders_hand_built_frame_data_without_a_world() {
        let Some(context) = gpu_context() else {
            return;
        };
        let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE);
        let frame = triangle_frame(&mut renderer);
        let (image, recorder) = render_frame(&context, &mut renderer, &frame, SIZE, SIZE);

        assert_eq!(recorder.draw_calls, 1);
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
        let mut resized = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE);
        let frame = triangle_frame(&mut resized);
        // A frame at the first size before the resize, as a window draws.
        render_frame(&context, &mut resized, &frame, SIZE, SIZE);
        resized.resize(width, height);
        let (resized_image, _) = render_frame(&context, &mut resized, &frame, width, height);

        let mut created = Renderer::new(&context, TARGET_FORMAT, width, height);
        let frame = triangle_frame(&mut created);
        let (created_image, _) = render_frame(&context, &mut created, &frame, width, height);

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
    /// unlit and red emissive, loaded into `renderer`'s Pools.
    fn triangle_frame(renderer: &mut Renderer) -> FrameData {
        // Counter-clockwise towards the camera, drawn with an identity
        // view-projection; its QTangent is the identity, so its normal is +Z.
        let vertex = |x: f32, y: f32| CookedVertex {
            position: [x, y, 0.5],
            uv: [0.0, 0.0],
            qtangent: [0, 0, 0, i16::MAX],
        };
        let triangle = CookedMesh {
            vertices: vec![vertex(-0.5, -0.5), vertex(0.5, -0.5), vertex(0.0, 0.5)],
            indices: vec![0, 1, 2],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: 3,
                material: None,
                meshlets: vec![CookedMeshlet {
                    vertices: vec![0, 1, 2],
                    triangles: vec![[0, 1, 2]],
                }],
            }],
            bounds: [[-0.5, -0.5, 0.5], [0.5, 0.5, 0.5]],
        };
        // Loaded first, outside the target, so the triangle's draw finds its
        // vertices only past its base vertex.
        let outside = CookedMesh {
            vertices: vec![vertex(2.0, 2.0), vertex(3.0, 2.0), vertex(2.0, 3.0)],
            ..triangle.clone()
        };
        renderer
            .mesh_pool
            .load(&mut renderer.load_staging, &outside, &[None])
            .expect("the outside triangle fits");
        let mesh = renderer
            .mesh_pool
            .load(&mut renderer.load_staging, &triangle, &[None])
            .expect("the triangle fits");
        let &Primitive {
            vertex_offset,
            index_offset,
            index_count,
        } = &renderer.mesh_pool.get(mesh).expect("loaded").primitives[0];
        assert!(vertex_offset > 0 && index_offset > 0);
        // Black and unlit, so only the red emissive reaches the target.
        let material = renderer.material_pool.insert(
            &mut renderer.load_staging,
            MaterialData {
                base_color: [0.0, 0.0, 0.0, 1.0],
                metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                emissive: [1.0, 0.0, 0.0, 1.0],
                texture_indices: [MaterialPool::DEFAULT_LAYER; 4],
                extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
            },
        );
        let identity = padded_columns(glam::Mat3::IDENTITY);
        FrameData {
            camera: CameraUniform {
                view_proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
                ..CameraUniform::default()
            },
            light: LightUniform::default(),
            instances: vec![InstanceData {
                rotation_scale: identity,
                normal_matrix: identity,
                anchor: [0; 3],
                material_id: material.index(),
                offset: [0.0; 3],
                _pad: 0,
            }],
            instance_writes: vec![InstanceWrite {
                first_slot: 0,
                instances: 0..1,
            }],
            draws: vec![Draw {
                indices: index_offset..index_offset + index_count,
                base_vertex: vertex_offset as i32,
                instance_index: 0,
            }],
        }
    }

    /// Uploads and renders `frame` into a new `width` × `height` target, and
    /// reads it back, with what the frame recorded.
    fn render_frame(
        context: &GpuContext,
        renderer: &mut Renderer,
        frame: &FrameData,
        width: u32,
        height: u32,
    ) -> (image::RgbaImage, FrameRecorder) {
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
            None,
            &mut recorder,
        );
        let pixels = read_back(context, &target).expect("read back failed");
        let image =
            image::RgbaImage::from_raw(width, height, pixels).expect("width × height pixels");
        (image, recorder)
    }
}
