use bevy_ecs::prelude::*;

use crate::assets::{
    LoadedImage, MaterialChannel, MaterialData, MaterialPool, Mesh, MeshPool, Primitive,
};
use crate::camera::{Camera, fly_camera};
use crate::components::{MaterialRef, MeshRef, Transform};
use crate::gpu::GpuContext;
use crate::helmet::{HelmetAssets, RuntimeHelmets, despawn_helmet, spawn_helmet};
use crate::input::{Input, clear_just_pressed};
use crate::render::{
    CameraUniform, INSTANCE_BUFFER_INITIAL_CAPACITY, InstanceData, LightUniform, Vertex,
    create_depth_texture, prepare_renderables,
};
use crate::time::Time;

/// The Scene in its ECS World and the renderer that draws it into a target
/// view: the window's surface or an offscreen texture.
pub struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    aspect_ratio: f32,
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    // World is an internal detail, it should not be public. But this will do for now.
    pub world: World,
    instance_buffer: wgpu::Buffer,
    instance_bind_group: wgpu::BindGroup,
    instance_capacity: u64,
    schedule: Schedule,
    instance_bind_group_layout: wgpu::BindGroupLayout,
}

impl State {
    /// A State that renders into `width` × `height` views of `target_format`
    /// textures, both non-zero, seen through `camera`.
    pub fn new(
        context: &GpuContext,
        target_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        camera: Camera,
    ) -> anyhow::Result<State> {
        let _span = tracing::info_span!("state_init").entered();

        let device = context.device.clone();
        let queue = context.queue.clone();
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

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
        // because the layout references its bind group layout; moved into the
        // `World` after materials are uploaded.
        let mut material_pool = MaterialPool::new(&device, &queue);

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
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
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

        let mut world = World::new();
        world.insert_resource(MeshPool::new(&device, 1 << 20, 256 << 10));
        world.insert_resource(Input::default());
        world.insert_resource(Time::default());
        world.insert_resource(camera);
        world.insert_resource(RuntimeHelmets::default());

        let mut schedule = Schedule::default();
        schedule
            .add_systems((fly_camera, spawn_helmet, despawn_helmet, clear_just_pressed).chain());

        let model = crate::assets::load_model("assets/DamagedHelmet.glb")?;

        let mut mesh_pool = world.resource_mut::<MeshPool>();
        let mut primitives = Vec::new();
        for (primitive_index, prim) in model.primitives.iter().enumerate() {
            let _prim_span = tracing::debug_span!("upload_primitive", primitive_index).entered();

            let (vertex_offset, index_offset, index_count) =
                mesh_pool.push_primitive(&device, &queue, &prim.vertices, &prim.indices);

            tracing::debug!(
                vertices = prim.vertices.len(),
                indices = prim.indices.len(),
                "primitive uploaded"
            );

            primitives.push(Primitive {
                vertex_offset,
                index_offset,
                index_count,
            });
        }
        let mesh = Mesh { primitives };
        let mesh_handle = mesh_pool.insert(mesh);

        // Translate the model's material into the SSBO and upload its albedo
        // into the channel array. DamagedHelmet is single-material; multi-
        // material meshes (one MaterialRef per primitive) are deferred.
        if model.primitives.len() > 1 {
            tracing::warn!(
                primitives = model.primitives.len(),
                "multi-primitive model; only primitive 0's material is used (multi-material deferred)"
            );
        }
        let first_primitive = model
            .primitives
            .first()
            .ok_or_else(|| anyhow::anyhow!("model has no primitives"))?;
        let mut upload_channel = |channel: MaterialChannel, image: Option<&LoadedImage>| -> u32 {
            match image {
                Some(image) => {
                    material_pool.upload(&queue, channel, &image.pixels, image.width, image.height)
                }
                None => {
                    tracing::debug!(
                        ?channel,
                        "model has no texture for channel; using the neutral layer"
                    );
                    MaterialPool::DEFAULT_LAYER
                }
            }
        };
        let albedo_layer = upload_channel(
            MaterialChannel::Albedo,
            first_primitive.base_color_image.as_ref(),
        );
        let normal_layer = upload_channel(
            MaterialChannel::Normal,
            first_primitive.normal_image.as_ref(),
        );
        let metallic_roughness_layer = upload_channel(
            MaterialChannel::MetallicRoughness,
            first_primitive.metallic_roughness_image.as_ref(),
        );
        let occlusion_layer = upload_channel(
            MaterialChannel::Occlusion,
            first_primitive.occlusion_image.as_ref(),
        );
        let emissive_layer = upload_channel(
            MaterialChannel::Emissive,
            first_primitive.emissive_image.as_ref(),
        );
        let material = MaterialData {
            base_color: first_primitive.base_color_factor,
            metallic_roughness: [
                first_primitive.metallic_factor,
                first_primitive.roughness_factor,
                0.0,
                0.0,
            ],
            emissive: [
                first_primitive.emissive_factor[0],
                first_primitive.emissive_factor[1],
                first_primitive.emissive_factor[2],
                1.0,
            ],
            texture_indices: [
                albedo_layer,
                normal_layer,
                metallic_roughness_layer,
                occlusion_layer,
            ],
            extra: [emissive_layer, 0, 0, 0],
        };
        let material_handle = material_pool.insert(&queue, material);

        // Debug material sweep: a pure factor sweep over the neutral default
        // layers with a white base color. Top row dielectric, bottom row metal;
        // roughness grows left to right. The textured glTF helmet sits between
        // the rows. Placed 8 units ahead of the default camera.
        for (row_index, metallic) in [0.0_f32, 1.0].into_iter().enumerate() {
            for (column_index, roughness) in [0.1_f32, 0.3, 0.5, 0.7, 0.9].into_iter().enumerate() {
                let variant_handle = material_pool.insert(
                    &queue,
                    MaterialData {
                        base_color: [1.0, 1.0, 1.0, 1.0],
                        metallic_roughness: [metallic, roughness, 0.0, 0.0],
                        emissive: [0.0, 0.0, 0.0, 1.0],
                        texture_indices: [MaterialPool::DEFAULT_LAYER; 4],
                        extra: [MaterialPool::DEFAULT_LAYER, 0, 0, 0],
                    },
                );
                let column_offset = (column_index as f32 - 2.0) * 2.0;
                let row_offset = 2.6 - row_index as f32 * 5.2;
                world.spawn((
                    Transform(glam::Affine3A::from_translation(glam::vec3(
                        column_offset,
                        row_offset,
                        -8.0,
                    ))),
                    MeshRef(mesh_handle),
                    MaterialRef(variant_handle),
                ));
            }
        }
        world.spawn((
            Transform(glam::Affine3A::from_translation(glam::vec3(0.0, 0.0, -8.0))),
            MeshRef(mesh_handle),
            MaterialRef(material_handle),
        ));

        world.insert_resource(material_pool);
        world.insert_resource(HelmetAssets {
            mesh: mesh_handle,
            material: material_handle,
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

        Ok(Self {
            device,
            queue,
            aspect_ratio: width as f32 / height as f32,
            render_pipeline,
            camera_buffer,
            camera_bind_group,
            depth_texture_view,
            light_buffer,
            light_bind_group,
            world,
            instance_buffer,
            instance_bind_group,
            instance_capacity,
            schedule,
            instance_bind_group_layout,
        })
    }

    /// Matches a new target size, both non-zero.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.depth_texture_view = create_depth_texture(&self.device, width, height);
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// Advances the Scene by `delta_time` seconds and uploads the frame's data.
    pub fn update(&mut self, delta_time: f32) {
        self.world.resource_mut::<Time>().delta_time = delta_time;
        self.schedule.run(&mut self.world);

        let camera = self.world.resource::<Camera>();

        let view = camera.view_matrix();
        let proj = glam::Mat4::perspective_rh(45_f32.to_radians(), self.aspect_ratio, 0.1, 100.0);

        let view_proj = proj * view;
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 0.0],
        };

        let light_uniform = LightUniform {
            direction: [0.5, -1.0, -0.3, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            ambient: [1.0, 1.0, 1.0, 0.1],
            point_positions: [
                [2.0, 2.0, 2.0, 0.0],
                [-2.0, 2.0, 2.0, 0.0],
                [2.0, 2.0, -2.0, 0.0],
                [-2.0, 2.0, -2.0, 0.0],
            ],
            point_colors: [
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0, 0.0, 1.0],
            ],
            num_point_lights: [4.0, 0.0, 0.0, 0.0],
        };

        let instance_data = prepare_renderables(&mut self.world);
        self.ensure_instance_capacity(instance_data.len() as u64);
        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );

        self.queue.write_buffer(
            &self.light_buffer,
            0,
            bytemuck::cast_slice(&[light_uniform]),
        );
    }

    /// Draws the frame into `target`, a view in the format and size this
    /// State renders to.
    pub fn render(&mut self, target: &wgpu::TextureView) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
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
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            let material_pool = self.world.resource::<MaterialPool>();

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &material_pool.bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);
            render_pass.set_bind_group(3, &self.instance_bind_group, &[]);

            // Same query signature as `prepare_renderables` so `entity_index`
            // lines up with the instance buffer it built.
            let mut renderable_query = self.world.query::<(&Transform, &MeshRef, &MaterialRef)>();
            let mesh_pool = self.world.resource::<MeshPool>();

            render_pass.set_vertex_buffer(0, mesh_pool.vertex_buffer.slice(..));
            render_pass
                .set_index_buffer(mesh_pool.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            for (entity_index, (_transform, mesh_ref, _material_ref)) in
                renderable_query.iter(&self.world).enumerate()
            {
                let handle = mesh_ref.0;
                let Some(mesh) = mesh_pool.get(handle) else {
                    tracing::warn!(handle = handle.0, "MeshHandle not in pool, skipping entity");
                    continue;
                };

                for primitive in &mesh.primitives {
                    let instance_idx = entity_index as u32;
                    render_pass.draw_indexed(
                        primitive.index_offset..primitive.index_offset + primitive.index_count,
                        primitive.vertex_offset as i32, // base_vertex
                        instance_idx..instance_idx + 1,
                    );
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
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
