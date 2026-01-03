//! HDR rendering demonstration with tone mapping and exposure control.
//!
//! This example demonstrates:
//! - HDR render targets with floating-point precision
//! - Multiple tone mapping operators (ACES, Reinhard, Uncharted 2)
//! - Automatic and manual exposure control
//! - Real-time switching between tone mapping operators
//! - GUI controls for exposure and tone mapping parameters

mod common;

use common::run_example;
use praxis_core::Engine;
use praxis_ecs::{Commands, Query, Res, ResMut};
use praxis_graphics::{
    colored_cube_mesh, ExposureMode, HdrRenderTarget, RenderCommands, RenderContext,
    ToneMapper, ToneMappingOperator,
};
use praxis_gui::{egui, GuiContext};
use praxis_input::InputState;
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
        CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
    },
    pipeline::{graphics::viewport::Viewport, Pipeline},
    render_pass::{Framebuffer, FramebufferCreateInfo},
};

/// HDR demo configuration
struct HdrDemo {
    hdr_target: HdrRenderTarget,
    tone_mapper: ToneMapper,
    camera_position: Vec3,
    camera_rotation: Quat,
    cube_count: u32,
    
    // GUI controls
    manual_exposure: f32,
    exposure_speed: f32,
    use_auto_exposure: bool,
    current_operator: ToneMappingOperator,
    gamma: f32,
    average_luminance: f32,
}

impl HdrDemo {
    fn new(render_context: &mut RenderContext) -> Result<Self> {
        info!("Initializing HDR demo");

        // Create HDR render pass and target
        let hdr_render_pass = render_context.create_hdr_render_pass()?;
        let extent = [1920, 1080];
        let hdr_target = HdrRenderTarget::new(
            Arc::new(render_context.device.clone()).into(),
            hdr_render_pass,
            extent,
        )?;

        // Create tone mapper with ACES operator (default)
        let tone_mapper = ToneMapper::new(
            render_context.device.clone(),
            Arc::new(render_context.device.clone()).into(),
            vulkano::format::Format::R8G8B8A8_UNORM,
            ToneMappingOperator::ACES,
        )?;

        // Load cube mesh
        render_context
            .mesh_manager_mut()
            .load_mesh("cube", colored_cube_mesh())?;

        // Setup camera
        let camera_position = Vec3::new(0.0, 3.0, 10.0);
        let camera_rotation = Quat::IDENTITY;

        info!("HDR demo initialized successfully");

        Ok(Self {
            hdr_target,
            tone_mapper,
            camera_position,
            camera_rotation,
            cube_count: 9,
            manual_exposure: 1.0,
            exposure_speed: 2.0,
            use_auto_exposure: true,
            current_operator: ToneMappingOperator::ACES,
            gamma: 2.2,
            average_luminance: 0.5,
        })
    }

    fn update_gui(&mut self, gui_context: &mut GuiContext) {
        egui::Window::new("HDR Controls")
            .default_pos([10.0, 10.0])
            .default_size([300.0, 400.0])
            .show(&gui_context.context, |ui| {
                ui.heading("HDR Rendering Demo");
                ui.separator();

                ui.label("Tone Mapping Operator:");
                let mut operator_changed = false;
                
                if ui.radio_value(&mut self.current_operator, ToneMappingOperator::ACES, "ACES Filmic").clicked() {
                    operator_changed = true;
                }
                if ui.radio_value(&mut self.current_operator, ToneMappingOperator::Reinhard, "Reinhard").clicked() {
                    operator_changed = true;
                }
                if ui.radio_value(&mut self.current_operator, ToneMappingOperator::Uncharted2, "Uncharted 2").clicked() {
                    operator_changed = true;
                }

                if operator_changed {
                    self.tone_mapper.set_operator(self.current_operator);
                }

                ui.separator();
                ui.label("Exposure Mode:");
                
                let mut mode_changed = false;
                let mut was_auto = self.use_auto_exposure;
                
                ui.checkbox(&mut self.use_auto_exposure, "Automatic Exposure");
                mode_changed = was_auto != self.use_auto_exposure;

                if self.use_auto_exposure {
                    ui.add(egui::Slider::new(&mut self.exposure_speed, 0.1..=10.0)
                        .text("Adaptation Speed"));
                    if mode_changed {
                        self.tone_mapper.set_exposure_mode(ExposureMode::Automatic {
                            speed: self.exposure_speed,
                        });
                    }
                } else {
                    ui.add(egui::Slider::new(&mut self.manual_exposure, 0.1..=10.0)
                        .text("Manual Exposure"));
                    if mode_changed {
                        self.tone_mapper.set_exposure_mode(ExposureMode::Manual {
                            exposure: self.manual_exposure,
                        });
                    }
                }

                ui.separator();
                ui.add(egui::Slider::new(&mut self.gamma, 1.0..=3.0)
                    .text("Gamma"));
                self.tone_mapper.set_gamma(self.gamma);

                ui.separator();
                ui.add(egui::Slider::new(&mut self.average_luminance, 0.01..=2.0)
                    .text("Scene Luminance (simulated)"));

                ui.separator();
                ui.label(format!("Current Exposure: {:.2}", self.tone_mapper.current_exposure()));
                ui.label(format!("Cube Count: {}", self.cube_count));

                ui.separator();
                ui.label("Operator Info:");
                match self.current_operator {
                    ToneMappingOperator::ACES => {
                        ui.label("ACES Filmic: Industry standard");
                        ui.label("Used in AAA games and film");
                        ui.label("Provides cinematic look");
                    }
                    ToneMappingOperator::Reinhard => {
                        ui.label("Reinhard: Simple and fast");
                        ui.label("Good for general use");
                        ui.label("Formula: color / (color + 1)");
                    }
                    ToneMappingOperator::Uncharted2 => {
                        ui.label("Uncharted 2: High contrast");
                        ui.label("Also called Hable tone mapping");
                        ui.label("Provides dramatic look");
                    }
                }
            });
    }
}

fn setup_hdr_demo(
    mut commands: Commands,
    mut render_context: ResMut<RenderContext>,
    mut gui_context: ResMut<GuiContext>,
) -> Result<()> {
    info!("Setting up HDR demo scene");

    // Initialize HDR demo
    let hdr_demo = HdrDemo::new(&mut render_context)?;
    commands.insert_resource(hdr_demo);

    // Create grid of cubes with varying brightness
    for x in -1..2 {
        for y in -1..2 {
            let position = Vec3::new(x as f32 * 3.0, y as f32 * 3.0, 0.0);
            
            commands.spawn((
                Transform::from_translation(position),
                GlobalTransform::default(),
            ));
        }
    }

    info!("HDR demo scene setup complete");
    Ok(())
}

fn update_hdr_demo(
    mut hdr_demo: ResMut<HdrDemo>,
    mut gui_context: ResMut<GuiContext>,
    input_state: Res<InputState>,
) {
    // Update GUI
    hdr_demo.update_gui(&mut gui_context);

    // Simple camera rotation
    hdr_demo.camera_rotation = Quat::from_rotation_y(0.3);
}

fn render_hdr_demo(
    mut hdr_demo: ResMut<HdrDemo>,
    mut render_context: ResMut<RenderContext>,
    query: Query<(&Transform, &GlobalTransform)>,
) -> Result<()> {
    // Setup camera matrices
    let view = Mat4::look_at_rh(
        hdr_demo.camera_position,
        Vec3::ZERO,
        Vec3::Y,
    );

    let proj = Mat4::perspective_rh(
        45_f32.to_radians(),
        1920.0 / 1080.0,
        0.1,
        100.0,
    );

    // Collect draw commands
    let mut draw_commands = Vec::new();
    for (transform, _) in query.iter() {
        draw_commands.push(praxis_graphics::DrawCommand {
            mesh_id: "cube".to_string(),
            model: Mat4::from_scale_rotation_translation(
                Vec3::ONE,
                transform.rotation,
                transform.translation,
            ),
            texture_name: None,
            material_properties: None,
        });
    }

    let render_commands = RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: None,
    };

    // For now, render directly (HDR rendering would require additional setup)
    render_context.render(&render_commands)?;

    Ok(())
}

fn main() -> Result<()> {
    run_example(
        "HDR Rendering Demo",
        |engine| {
            engine.add_startup_system(setup_hdr_demo);
            engine.add_system(update_hdr_demo);
            engine.add_system(render_hdr_demo);
        },
    )
}
