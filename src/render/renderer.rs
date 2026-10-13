use std::time::Instant;

use crate::assets::{MaterialPool, MeshPool};
use crate::gpu::GpuContext;
use crate::render::{
    CameraUniform, DEPTH_FORMAT, FrameData, FrameStaging, INSTANCE_BUFFER_INITIAL_CAPACITY,
    InstanceData, LightUniform, Vertex, create_depth_texture,
};
use crate::stats::{FrameRecorder, texture_bytes};

/// The label of the Renderer's one render pass, and its name in frame stats.
pub(crate) const MAIN_PASS: &str = "main_pass";

/// Draws Frame data into a target view, the window's surface or an offscreen
/// texture, from GPU resources and Pools it owns; it reads no ECS World.
pub struct Renderer {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    aspect_ratio: f32,
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    pub(crate) mesh_pool: MeshPool,
    pub(crate) material_pool: MaterialPool,
    instance_buffer: wgpu::Buffer,
    instance_bind_group: wgpu::BindGroup,
    instance_capacity: u64,
    instance_bind_group_layout: wgpu::BindGroupLayout,
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
        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let instance_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Instance Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Material-as-ID: one bind group (SSBO + per-channel texture arrays +
        // sampler) serves every material. Created before the pipeline layout
        // because the layout references its bind group layout.
        let material_pool = MaterialPool::new(&device, &queue);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&material_pool.bind_group_layout),
                    Some(&light_bind_group_layout),
                    Some(&instance_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::desc())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light Buffer"),
            size: std::mem::size_of::<LightUniform>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: INSTANCE_BUFFER_INITIAL_CAPACITY * std::mem::size_of::<InstanceData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Instance Bind Group"),
            layout: &instance_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: instance_buffer.as_entire_binding(),
            }],
        });

        let instance_capacity = INSTANCE_BUFFER_INITIAL_CAPACITY;

        let depth_texture_view = create_depth_texture(&device, width, height);

        Self {
            mesh_pool: MeshPool::new(&device, 1 << 20, 256 << 10),
            material_pool,
            device,
            queue,
            aspect_ratio: width as f32 / height as f32,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            depth_texture_view,
            light_buffer,
            light_bind_group,
            instance_buffer,
            instance_bind_group,
            instance_capacity,
            instance_bind_group_layout,
            transient_saves_memory: context.adapter.get_info().transient_saves_memory == Some(true),
            frame_staging: FrameStaging::default(),
            frame_encoder: None,
        }
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.depth_texture_view = create_depth_texture(&self.device, width, height);
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// The width of the target over its height.
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    /// Records the copies of `frame`'s Instances and uniforms into the
    /// frame's encoder, which `render` submits.
    pub fn upload(&mut self, frame: &FrameData, recorder: &mut FrameRecorder) {
        let started = Instant::now();
        self.ensure_instance_capacity(frame.instances.len() as u64);
        let uploads: [(&wgpu::Buffer, &[u8]); 3] = [
            (
                &self.instance_buffer,
                bytemuck::cast_slice(&frame.instances),
            ),
            (&self.camera_buffer, bytemuck::bytes_of(&frame.camera)),
            (&self.light_buffer, bytemuck::bytes_of(&frame.light)),
        ];
        // The window skips `render` while occluded or without a surface
        // texture; submitting the unrendered frame's copies returns its
        // staging slot.
        if let Some(unrendered) = self.frame_encoder.take() {
            self.queue.submit(std::iter::once(unrendered.finish()));
        }
        let mut encoder = create_frame_encoder(&self.device);
        recorder.uploaded_bytes += self
            .frame_staging
            .write(&self.device, &mut encoder, &uploads);
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

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(MAIN_PASS),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: target,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.material_pool.bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);
            render_pass.set_bind_group(3, &self.instance_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.mesh_pool.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.mesh_pool.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            for draw in &frame.draws {
                render_pass.draw_indexed(
                    draw.indices.clone(),
                    draw.base_vertex,
                    draw.instance_index..draw.instance_index + 1,
                );
            }
            recorder.draw_calls += frame.draws.len() as u32;
        }

        let command_buffer = encoder.finish();
        recorder.stage("encode", started);

        let started = Instant::now();
        // submit will accept anything that implements IntoIter
        let submission = self.queue.submit(std::iter::once(command_buffer));
        recorder.stage("submit", started);
        submission
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
            + texture_bytes(
                self.depth_texture_view.texture(),
                self.transient_saves_memory,
            )
            + self.mesh_pool.gpu_memory_bytes()
            + self
                .material_pool
                .gpu_memory_bytes(self.transient_saves_memory)
            + self.frame_staging.gpu_memory_bytes()
    }

    fn ensure_instance_capacity(&mut self, needed: u64) {
        if needed <= self.instance_capacity {
            return;
        }

        let old_capacity = self.instance_capacity;
        let mut new_capacity = old_capacity * 2;
        while new_capacity < needed {
            new_capacity *= 2;
        }

        self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: new_capacity * std::mem::size_of::<InstanceData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        self.instance_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Instance Bind Group"),
            layout: &self.instance_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.instance_buffer.as_entire_binding(),
            }],
        });

        self.instance_capacity = new_capacity;
        tracing::info!(
            old = old_capacity,
            new = new_capacity,
            "instance buffer resized"
        );
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
    use crate::assets::{MaterialData, Mesh, Primitive};
    use crate::capture::read_back;
    use crate::render::prepare::Draw;

    const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    const SIZE: u32 = 16;

    #[test]
    fn renders_hand_built_frame_data_without_a_world() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
        };
        let mut renderer = Renderer::new(&context, TARGET_FORMAT, SIZE, SIZE);

        // A triangle around the target's centre, counter-clockwise towards
        // the camera, drawn with an identity view-projection.
        let vertex = |x: f32, y: f32| Vertex {
            position: [x, y, 0.5],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
        };
        let (vertex_offset, index_offset, index_count) = renderer.mesh_pool.push_primitive(
            &renderer.device,
            &renderer.queue,
            &[vertex(-0.5, -0.5), vertex(0.5, -0.5), vertex(0.0, 0.5)],
            &[0, 1, 2],
        );
        renderer.mesh_pool.insert(Mesh {
            primitives: vec![Primitive {
                vertex_offset,
                index_offset,
                index_count,
            }],
        });
        // Black and unlit, so only the red emissive reaches the target.
        let material = renderer.material_pool.insert(
            &renderer.queue,
            MaterialData {
                base_color: [0.0, 0.0, 0.0, 1.0],
                metallic_roughness: [0.0, 1.0, 0.0, 0.0],
                emissive: [1.0, 0.0, 0.0, 1.0],
                texture_indices: [MaterialPool::DEFAULT_LAYER; 4],
                extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
            },
        );
        let identity = glam::Mat4::IDENTITY.to_cols_array_2d();
        let frame = FrameData {
            camera: CameraUniform {
                view_proj: identity,
            },
            light: LightUniform {
                direction: [0.0, 0.0, -1.0, 0.0],
                ..LightUniform::default()
            },
            instances: vec![InstanceData {
                model: identity,
                normal_matrix: identity,
                material_id: material.index(),
                _pad: [0; 3],
            }],
            draws: vec![Draw {
                indices: index_offset..index_offset + index_count,
                base_vertex: vertex_offset as i32,
                instance_index: 0,
            }],
        };

        let target = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Frame Data Test Target"),
            size: wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
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
        renderer.upload(&frame, &mut recorder);
        renderer.render(
            &frame,
            &target.create_view(&wgpu::TextureViewDescriptor::default()),
            None,
            &mut recorder,
        );
        let image = read_back(&context, &target).expect("read back failed");

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
}
