use std::sync::Arc;

use wgpu::{Buffer, util::DeviceExt};
use winit::{keyboard::KeyCode, window::Window};

use crate::camera::{Camera, CameraUniform};
use crate::light::LightUniform;
use crate::model::ModelUniform;
use crate::texture::create_depth_texture;
use crate::vertex::Vertex;

pub struct GpuPrimitive {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    texture_bind_group: wgpu::BindGroup,
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    camera_buffer: Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture_view: wgpu::TextureView,
    pub keys_pressed: std::collections::HashSet<KeyCode>,
    last_frame: std::time::Instant,
    pub camera: Camera,
    primitives: Vec<GpuPrimitive>,
    light_buffer: Buffer,
    light_bind_group: wgpu::BindGroup,
    model_buffers: Vec<(Buffer, wgpu::BindGroup)>,
    transforms: Vec<glam::Mat4>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);

        // Assuming an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let is_surface_configured = size.width > 0 && size.height > 0;
        if is_surface_configured {
            surface.configure(&device, &config);
        }
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
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

        let model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    Some(&camera_bind_group_layout),
                    Some(&texture_bind_group_layout),
                    Some(&light_bind_group_layout),
                    Some(&model_bind_group_layout),
                ],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
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

        let model = crate::model::load_model("assets/DamagedHelmet.glb")?;

        let transforms = vec![
            glam::Mat4::from_translation(glam::vec3(0.0, 0.0, 0.0)),
            glam::Mat4::from_translation(glam::vec3(2.0, 0.0, 0.0)),
            glam::Mat4::from_translation(glam::vec3(-2.0, 0.0, 0.0)),
        ];

        let mut primitives = Vec::new();
        for prim in &model.primitives {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&prim.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&prim.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            // Fallback: 1x1 white texture if no base color image
            let (tex_data, tex_width, tex_height) = match &prim.base_color_image {
                Some((data, w, h)) => (data.as_slice(), *w, *h),
                None => (&[255u8, 255, 255, 255] as &[u8], 1, 1),
            };

            let texture = device.create_texture_with_data(
                &queue,
                &wgpu::TextureDescriptor {
                    label: Some("Diffuse Texture"),
                    size: wgpu::Extent3d {
                        width: tex_width,
                        height: tex_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                tex_data,
            );

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                ..Default::default()
            });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Texture Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            primitives.push(GpuPrimitive {
                vertex_buffer,
                index_buffer,
                num_indices: prim.indices.len() as u32,
                texture_bind_group,
            });
        }

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

        let model_buffers: Vec<(Buffer, wgpu::BindGroup)> = transforms
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("Model Buffer {i}")),
                    size: std::mem::size_of::<ModelUniform>() as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("Model Bind Group {i}")),
                    layout: &model_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });
                (buffer, bind_group)
            })
            .collect();

        let depth_width = config.width.max(1);
        let depth_height = config.height.max(1);
        let depth_texture_view = create_depth_texture(&device, depth_width, depth_height);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            is_surface_configured,
            render_pipeline,
            window,
            camera_buffer,
            camera_bind_group,
            depth_texture_view,
            camera: Camera::new(),
            keys_pressed: std::collections::HashSet::new(),
            last_frame: std::time::Instant::now(),
            primitives,
            light_buffer,
            light_bind_group,
            model_buffers,
            transforms,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture_view = create_depth_texture(&self.device, width, height);
            self.is_surface_configured = true;
        }
    }

    pub fn update(&mut self) {
        if !self.is_surface_configured {
            return;
        }

        let now = std::time::Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        let forward = self.camera.forward();
        let right = self.camera.right();
        let speed = self.camera.speed * dt;

        if self.keys_pressed.contains(&KeyCode::KeyW) {
            self.camera.position += forward * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            self.camera.position -= forward * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            self.camera.position += right * speed;
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            self.camera.position -= right * speed;
        }
        if self.keys_pressed.contains(&KeyCode::Space) {
            self.camera.position.y += speed;
        }
        if self.keys_pressed.contains(&KeyCode::ShiftLeft) {
            self.camera.position.y -= speed;
        }

        let aspect = self.config.width as f32 / self.config.height as f32;
        let view = self.camera.view_matrix();
        let proj = glam::Mat4::perspective_rh(45_f32.to_radians(), aspect, 0.1, 100.0);

        let view_proj = proj * view;
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
                0.0,
            ],
        };

        let light_uniform = LightUniform {
            direction: [0.5, -1.0, -0.3, 0.0],
            color: [1.0, 1.0, 1.0, 1.0],
            ambient: [1.0, 1.0, 1.0, 0.1],
        };

        for (i, &transform) in self.transforms.iter().enumerate() {
            let model_uniform = ModelUniform {
                model: transform.to_cols_array_2d(),
                normal_matrix: transform.inverse().transpose().to_cols_array_2d(),
            };
            self.queue.write_buffer(
                &self.model_buffers[i].0,
                0,
                bytemuck::cast_slice(&[model_uniform]),
            );
        }

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

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.surface.configure(&self.device, &self.config);
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                // Skip this frame
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                // The surface texture was lost; bail with an accurate error.
                anyhow::bail!("Surface texture lost");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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
                        view: &view,
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_bind_group, &[]);

            for (i, _) in self.transforms.iter().enumerate() {
                render_pass.set_bind_group(3, &self.model_buffers[i].1, &[]);

                for prim in &self.primitives {
                    render_pass.set_bind_group(1, &prim.texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, prim.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(prim.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..prim.num_indices, 0, 0..1);
                }
            }
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
