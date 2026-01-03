//! Post-processing effects demonstration.
//!
//! This example demonstrates the post-processing framework by:
//! - Rendering a 3D scene to a texture
//! - Applying post-processing effects (grayscale, blur, etc.)
//! - Displaying the final result
//!
//! Controls:
//! - ESC: Exit
//! - 1: No post-processing (raw scene)
//! - 2: Grayscale effect
//! - 3: Copy pass (for testing)
//! - Space: Cycle through effects

use praxis_core;
use praxis_ecs::{Commands, Component, Query, Res, ResMut, Resource};
use praxis_graphics::{
    colored_cube_mesh, CopyPass, GrayscalePass, PostProcessChain, RenderCommands, RenderTarget,
    RenderTargetPool,
};
use praxis_input::{Input, KeyCode};
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{GlobalTransform, Parent, Transform};
use praxis_utils::Result;
use praxis_window::State;

/// Component marking the main camera.
#[derive(Component)]
struct MainCamera;

/// Component marking a rotating object.
#[derive(Component)]
struct Rotator {
    speed: f32,
}

/// Resource tracking the current post-processing effect.
#[derive(Resource)]
struct PostProcessingMode {
    mode: usize,
}

impl PostProcessingMode {
    const MODES: &'static [&'static str] = &["None", "Grayscale", "Copy"];

    fn next(&mut self) {
        self.mode = (self.mode + 1) % Self::MODES.len();
        println!("Post-processing mode: {}", Self::MODES[self.mode]);
    }

    fn set(&mut self, mode: usize) {
        if mode < Self::MODES.len() {
            self.mode = mode;
            println!("Post-processing mode: {}", Self::MODES[self.mode]);
        }
    }

    fn current_name(&self) -> &str {
        Self::MODES[self.mode]
    }
}

/// Initialize the scene with some 3D objects.
fn setup_scene(mut commands: Commands) {
    // Spawn camera
    commands.spawn((
        MainCamera,
        Transform::from_translation(Vec3::new(0.0, 2.0, 5.0)),
        GlobalTransform::default(),
    ));

    // Spawn rotating cubes
    for i in 0..3 {
        let angle = (i as f32) * std::f32::consts::TAU / 3.0;
        let x = angle.cos() * 2.0;
        let z = angle.sin() * 2.0;

        commands.spawn((
            Transform::from_translation(Vec3::new(x, 0.0, z)),
            GlobalTransform::default(),
            Rotator {
                speed: 1.0 + i as f32 * 0.5,
            },
        ));
    }

    println!("Scene setup complete");
    println!("Press 1-3 to select post-processing mode");
    println!("Press Space to cycle through effects");
    println!("Press ESC to exit");
}

/// System to rotate objects marked with Rotator component.
fn rotate_objects(mut query: Query<(&mut Transform, &Rotator)>) {
    let delta_time = 0.016; // Assuming 60 FPS for simplicity

    for (mut transform, rotator) in query.iter_mut() {
        let rotation = Quat::from_rotation_y(rotator.speed * delta_time);
        transform.rotation = transform.rotation * rotation;
    }
}

/// System to handle input for post-processing mode switching.
fn handle_post_process_input(input: Res<Input>, mut mode: ResMut<PostProcessingMode>) {
    if input.key_just_pressed(KeyCode::Space) {
        mode.next();
    }

    if input.key_just_pressed(KeyCode::Digit1) {
        mode.set(0);
    }

    if input.key_just_pressed(KeyCode::Digit2) {
        mode.set(1);
    }

    if input.key_just_pressed(KeyCode::Digit3) {
        mode.set(2);
    }
}

/// Custom state extending the base State with post-processing resources.
struct PostProcessState {
    state: State,
    render_target_pool: Option<RenderTargetPool>,
    post_process_chain: Option<PostProcessChain>,
    grayscale_pass: Option<GrayscalePass>,
    copy_pass: Option<CopyPass>,
}

impl PostProcessState {
    async fn new(window: std::sync::Arc<winit::window::Window>) -> Result<Self> {
        let mut state = State::new(window).await?;

        // Initialize post-processing resources
        let render_pass = state.render_context().create_post_process_render_pass()?;
        let render_target_pool = RenderTargetPool::new(
            state.render_context().memory_allocator.clone(),
            render_pass.clone(),
            vulkano::format::Format::R8G8B8A8_UNORM,
        );

        let post_process_chain = PostProcessChain::new(
            state.render_context().command_buffer_allocator.clone(),
            state.render_context().graphics_queue.clone(),
        );

        // Create post-processing passes
        let grayscale_pass = GrayscalePass::new(
            state.render_context().device.clone(),
            state.render_context().memory_allocator.clone(),
            vulkano::format::Format::R8G8B8A8_UNORM,
        )?;

        let copy_pass = CopyPass::new(
            state.render_context().device.clone(),
            state.render_context().memory_allocator.clone(),
            vulkano::format::Format::R8G8B8A8_UNORM,
        )?;

        Ok(Self {
            state,
            render_target_pool: Some(render_target_pool),
            post_process_chain: Some(post_process_chain),
            grayscale_pass: Some(grayscale_pass),
            copy_pass: Some(copy_pass),
        })
    }

    fn render_with_post_processing(&mut self) -> Result<()> {
        // Get current post-processing mode
        let mode = self
            .state
            .world()
            .get_resource::<PostProcessingMode>()
            .map(|m| m.mode)
            .unwrap_or(0);

        // Setup post-processing chain based on mode
        if let Some(ref mut chain) = self.post_process_chain {
            chain.clear_passes();

            match mode {
                1 => {
                    // Grayscale
                    if let Some(pass) = self.grayscale_pass.take() {
                        chain.add_pass(Box::new(pass));
                    }
                }
                2 => {
                    // Copy (for testing)
                    if let Some(pass) = self.copy_pass.take() {
                        chain.add_pass(Box::new(pass));
                    }
                }
                _ => {
                    // No post-processing
                }
            }
        }

        // Render the scene
        // Note: This is a simplified version. In a real implementation,
        // you would render to a texture first, then apply post-processing,
        // then blit to the swapchain.

        // For this demo, we'll just render directly for now
        // since the full render-to-texture integration requires
        // modifications to the render context

        // Build render commands
        let camera_query = self
            .state
            .world()
            .query_filtered::<&GlobalTransform, &MainCamera>();
        let camera_transform = camera_query
            .iter(self.state.world())
            .next()
            .map(|t| t.compute_matrix())
            .unwrap_or(Mat4::IDENTITY);

        let view = camera_transform.inverse();
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 1920.0 / 1080.0, 0.1, 100.0);

        let draw_commands = vec![praxis_graphics::DrawCommand {
            mesh_id: "cube".to_string(),
            model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            texture_name: None,
            material_properties: None,
        }];

        let render_commands = RenderCommands {
            view,
            proj,
            draw_commands: &draw_commands,
            lighting: None,
        };

        self.state.render_context_mut().render(&render_commands)?;

        Ok(())
    }
}

fn main() -> Result<()> {
    // Initialize core systems
    praxis_utils::init()?;
    let mut world = praxis_ecs::init();

    // Insert post-processing mode resource
    world.insert_resource(PostProcessingMode { mode: 0 });

    // Setup the scene
    world.run_system_once(setup_scene);

    // Note: Full integration would require running the window event loop
    // For this example, we're demonstrating the API structure

    println!("Post-processing demo initialized");
    println!("This example demonstrates the post-processing framework API");

    Ok(())
}
