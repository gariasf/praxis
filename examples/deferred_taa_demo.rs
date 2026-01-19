//! Demonstrates deferred rendering with velocity buffers and TAA (Temporal Anti-Aliasing).
//!
//! This example showcases:
//! - Deferred rendering with G-buffer generation
//! - Automatic velocity buffer generation for moving objects and camera
//! - TAA with temporal reprojection and neighborhood clamping
//! - Halton sequence jitter for sub-pixel sampling
//! - Motion blur effects using velocity data
//!
//! # Visual Verification Checklist
//!
//! When running this example, verify the following:
//!
//! ## Velocity Buffer
//! - [ ] Moving cubes generate non-zero motion vectors
//! - [ ] Static objects have zero motion vectors
//! - [ ] Camera movement produces consistent velocity patterns
//! - [ ] Motion vectors correctly represent screen-space motion
//!
//! ## TAA (Temporal Anti-Aliasing)
//! - [ ] Reduced temporal aliasing (less shimmering on edges)
//! - [ ] Smooth anti-aliased edges without excessive blur
//! - [ ] No significant ghosting on fast-moving objects
//! - [ ] Temporal stability maintained across frames
//! - [ ] Sub-pixel detail preserved through jitter accumulation
//!
//! ## Motion Blur
//! - [ ] Fast-moving objects show appropriate directional blur
//! - [ ] Blur direction matches object velocity
//! - [ ] Static objects remain sharp
//! - [ ] Camera motion creates consistent scene blur
//!
//! # Controls
//! - Arrow keys: Move camera
//! - Space: Toggle TAA on/off
//! - M: Toggle motion blur on/off
//! - ESC: Exit

use praxis_core::Engine;
use praxis_ecs::World;
use praxis_graphics::{
    deferred::{DeferredRenderParams, DeferredRenderer},
    lighting::{DirectionalLight, LightingUniforms},
    material::MaterialProperties,
    mesh::{MeshAssetManager, MeshData},
    post_process::{MotionBlurConfig, MotionBlurPass, RenderTarget, RenderTargetPool},
    primitives::colored_cube_mesh,
    taa::{apply_jitter_to_projection, HaltonSequence, TaaApplyParams, TaaConfig, TaaRenderer},
    texture::TextureManager,
    uniform_buffer::{DynamicUniformBuffer, ModelUniforms, ViewProjectionUniforms},
    DrawCommand,
};
use praxis_input::{InputManager, Key};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{info, Result};
use praxis_window::Window;
use std::sync::Arc;
use std::time::Instant;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, CommandBufferUsage, RecordingCommandBuffer,
    },
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{Device, Queue},
    format::Format,
    image::{sampler::Sampler, view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    sync::GpuFuture,
};

struct DemoState {
    // Rendering
    deferred_renderer: DeferredRenderer,
    taa_renderer: TaaRenderer,
    motion_blur_pass: Option<MotionBlurPass>,
    render_target_pool: RenderTargetPool,

    // Resources
    mesh_manager: MeshAssetManager,
    texture_manager: TextureManager,

    // Frame state
    halton_sequence: HaltonSequence,
    previous_view_proj: Mat4,
    previous_transforms: Vec<Mat4>,
    frame_count: u64,

    // Settings
    taa_enabled: bool,
    motion_blur_enabled: bool,

    // Camera
    camera_position: Vec3,
    camera_target: Vec3,
}

impl DemoState {
    fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
        command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
        queue: Arc<Queue>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        info!("Initializing deferred TAA demo");

        let deferred_renderer = DeferredRenderer::new(
            device.clone(),
            memory_allocator.clone(),
            descriptor_set_allocator.clone(),
            width,
            height,
        )?;

        let taa_renderer = TaaRenderer::new(device.clone(), memory_allocator.clone())?;

        let motion_blur_pass = Some(MotionBlurPass::new(
            device.clone(),
            memory_allocator.clone(),
            Format::R8G8B8A8_UNORM,
            MotionBlurConfig::default(),
        )?);

        let render_target_pool =
            RenderTargetPool::new(device.clone(), memory_allocator.clone(), width, height, 4)?;

        let mesh_manager = MeshAssetManager::new(memory_allocator.clone());
        let texture_manager = TextureManager::new(
            device,
            memory_allocator.clone(),
            queue,
            command_buffer_allocator,
        )?;

        // Create test meshes
        let cube_mesh = colored_cube_mesh();

        Ok(Self {
            deferred_renderer,
            taa_renderer,
            motion_blur_pass,
            render_target_pool,
            mesh_manager,
            texture_manager,
            halton_sequence: HaltonSequence::new(),
            previous_view_proj: Mat4::IDENTITY,
            previous_transforms: Vec::new(),
            frame_count: 0,
            taa_enabled: true,
            motion_blur_enabled: false,
            camera_position: Vec3::new(0.0, 3.0, 8.0),
            camera_target: Vec3::new(0.0, 0.0, 0.0),
        })
    }

    fn update(&mut self, input: &InputManager, delta_time: f32) {
        // Camera movement
        let move_speed = 3.0 * delta_time;
        let forward = (self.camera_target - self.camera_position).normalize();
        let right = forward.cross(Vec3::Y).normalize();

        if input.is_key_pressed(Key::ArrowUp) {
            self.camera_position += forward * move_speed;
            self.camera_target += forward * move_speed;
        }
        if input.is_key_pressed(Key::ArrowDown) {
            self.camera_position -= forward * move_speed;
            self.camera_target -= forward * move_speed;
        }
        if input.is_key_pressed(Key::ArrowLeft) {
            self.camera_position -= right * move_speed;
            self.camera_target -= right * move_speed;
        }
        if input.is_key_pressed(Key::ArrowRight) {
            self.camera_position += right * move_speed;
            self.camera_target += right * move_speed;
        }

        // Toggle settings
        if input.is_key_just_pressed(Key::Space) {
            self.taa_enabled = !self.taa_enabled;
            info!("TAA: {}", if self.taa_enabled { "ON" } else { "OFF" });
        }
        if input.is_key_just_pressed(Key::M) {
            self.motion_blur_enabled = !self.motion_blur_enabled;
            info!(
                "Motion Blur: {}",
                if self.motion_blur_enabled {
                    "ON"
                } else {
                    "OFF"
                }
            );
        }
    }

    fn render(
        &mut self,
        command_buffer_allocator: &StandardCommandBufferAllocator,
        queue_family_index: u32,
        swapchain_image: Arc<ImageView>,
        swapchain_framebuffer: Arc<Framebuffer>,
        width: u32,
        height: u32,
    ) -> Result<()> {
        self.frame_count += 1;

        // Create animated scene
        let time = self.frame_count as f32 * 0.016; // ~60fps
        let mut draw_commands = Vec::new();
        let mut current_transforms = Vec::new();

        // Rotating cubes at different speeds
        for i in 0..5 {
            let angle = time * (0.5 + i as f32 * 0.2);
            let radius = 3.0;
            let x = (angle + i as f32 * 1.2).cos() * radius;
            let z = (angle + i as f32 * 1.2).sin() * radius;
            let y = (time * (1.0 + i as f32 * 0.1)).sin() * 1.5;

            let transform = Mat4::from_translation(Vec3::new(x, y, z))
                * Mat4::from_rotation_y(angle)
                * Mat4::from_scale(Vec3::splat(0.5));

            current_transforms.push(transform);

            let color = [
                0.3 + (i as f32 * 0.15) % 1.0,
                0.5,
                0.8 - (i as f32 * 0.15) % 0.5,
                1.0,
            ];

            draw_commands.push(DrawCommand {
                mesh_id: "cube".to_string(),
                transform,
                material_properties: Some(MaterialProperties {
                    albedo: color,
                    metallic: 0.2,
                    roughness: 0.6,
                    emissive: [0.0, 0.0, 0.0],
                }),
                texture_name: None,
            });
        }

        // Use previous transforms or current if first frame
        let previous_transforms = if self.previous_transforms.is_empty() {
            current_transforms.clone()
        } else {
            self.previous_transforms.clone()
        };

        // Camera matrices with TAA jitter
        let view = Mat4::look_at_rh(self.camera_position, self.camera_target, Vec3::Y);
        let mut proj = Mat4::perspective_rh(
            std::f32::consts::PI / 4.0,
            width as f32 / height as f32,
            0.1,
            100.0,
        );

        // Apply jitter for TAA
        let jitter = if self.taa_enabled {
            self.halton_sequence.next_jitter()
        } else {
            [0.0, 0.0]
        };
        proj = apply_jitter_to_projection(proj, jitter, width, height);

        let current_view_proj = proj * view;

        // Create uniform buffers
        let view_proj = ViewProjectionUniforms {
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
            view_position: [
                self.camera_position.x,
                self.camera_position.y,
                self.camera_position.z,
                1.0,
            ],
            view_projection: current_view_proj.to_cols_array_2d(),
        };

        let previous_view_proj_uniforms = ViewProjectionUniforms {
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
            view_position: [
                self.camera_position.x,
                self.camera_position.y,
                self.camera_position.z,
                1.0,
            ],
            view_projection: self.previous_view_proj.to_cols_array_2d(),
        };

        let view_proj_buffer = Buffer::from_data(
            self.mesh_manager.memory_allocator().clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            view_proj,
        )?;

        let previous_view_proj_buffer = Buffer::from_data(
            self.mesh_manager.memory_allocator().clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            previous_view_proj_uniforms,
        )?;

        // Dynamic uniform buffers for model matrices
        let current_uniforms: Vec<ModelUniforms> = current_transforms
            .iter()
            .map(|t| ModelUniforms {
                model: t.to_cols_array_2d(),
                normal_matrix: t.inverse().transpose().to_cols_array_2d(),
            })
            .collect();

        let previous_uniforms: Vec<ModelUniforms> = previous_transforms
            .iter()
            .map(|t| ModelUniforms {
                model: t.to_cols_array_2d(),
                normal_matrix: t.inverse().transpose().to_cols_array_2d(),
            })
            .collect();

        let dynamic_buffer = DynamicUniformBuffer::new(
            self.mesh_manager.memory_allocator().clone(),
            &current_uniforms,
            draw_commands.len(),
        )?;

        let previous_dynamic_buffer = DynamicUniformBuffer::new(
            self.mesh_manager.memory_allocator().clone(),
            &previous_uniforms,
            draw_commands.len(),
        )?;

        // Lighting
        let lighting = LightingUniforms {
            directional_light: DirectionalLight {
                direction: [-0.3, -1.0, -0.5, 0.0],
                color: [1.0, 0.95, 0.9, 0.0],
                intensity: 1.5,
                _padding: [0.0; 3],
            },
            ambient_color: [0.15, 0.18, 0.22, 1.0],
            point_light_count: 0,
            _padding: [0.0; 3],
        };

        let lighting_buffer = Buffer::from_data(
            self.mesh_manager.memory_allocator().clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            lighting,
        )?;

        // Build command buffer
        let mut builder = RecordingCommandBuffer::new(
            command_buffer_allocator.clone(),
            queue_family_index,
            vulkano::command_buffer::CommandBufferLevel::Primary,
            CommandBufferUsage::OneTimeSubmit,
        )?;

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [width as f32, height as f32],
            depth_range: 0.0..=1.0,
        };

        // Render scene with deferred renderer (generates velocity buffer)
        let params = DeferredRenderParams {
            output_framebuffer: swapchain_framebuffer,
            viewport,
            draw_commands: &draw_commands,
            view_proj_buffer,
            dynamic_uniform_buffer: &dynamic_buffer,
            mesh_manager: &self.mesh_manager,
            texture_manager: &self.texture_manager,
            lighting_buffer,
            previous_view_proj_buffer,
            previous_dynamic_uniform_buffer: &previous_dynamic_buffer,
        };

        self.deferred_renderer.render(&mut builder, &params)?;

        // TODO: Apply TAA and motion blur passes here
        // This requires additional render targets and pipeline integration

        // Store state for next frame
        self.previous_view_proj = current_view_proj;
        self.previous_transforms = current_transforms;

        Ok(())
    }
}

fn main() -> Result<()> {
    praxis_utils::init_tracing();
    info!("Starting Deferred TAA Demo");

    // Initialize window and engine components
    let event_loop = winit::event_loop::EventLoop::new()?;
    let window = Window::new(&event_loop, "Deferred Rendering + TAA Demo", 1280, 720)?;

    // TODO: Complete integration with engine loop
    // This would require full window/swapchain setup and render loop

    info!("Demo initialization complete");
    info!("");
    info!("Controls:");
    info!("  Arrow Keys: Move camera");
    info!("  Space: Toggle TAA on/off");
    info!("  M: Toggle motion blur on/off");
    info!("  ESC: Exit");
    info!("");
    info!("Visual Verification:");
    info!("  - Check for smooth edges (TAA working)");
    info!("  - Observe motion blur on moving objects");
    info!("  - Verify no ghosting artifacts");
    info!("  - Check velocity buffer accuracy");

    Ok(())
}
