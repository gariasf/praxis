use praxis_core::run;
use praxis_ecs::{Commands, Query, Res, ResMut, Resource, World};
use praxis_graphics::{
    colored_cube_mesh, sphere_mesh, BloomConfig, DrawCommand, MaterialProperties,
    RenderCommands, RenderContext,
};
use praxis_input::{InputState, Key};
use praxis_math::{Mat4, Vec3};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::{info, Result};
use std::f32::consts::PI;

#[derive(Resource)]
struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 10.0),
            yaw: -90.0,
            pitch: 0.0,
        }
    }
}

#[derive(Resource)]
struct Time {
    elapsed: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self { elapsed: 0.0 }
    }
}

#[derive(Resource)]
struct BloomConfigResource {
    config: BloomConfig,
}

impl Default for BloomConfigResource {
    fn default() -> Self {
        Self {
            config: BloomConfig::new()
                .with_brightness_threshold(1.0)
                .with_blur_iterations(5)
                .with_exposure(1.0)
                .with_bloom_intensity(0.4),
        }
    }
}

fn spawn_objects_system(mut commands: Commands) {
    info!("Spawning objects for bloom demo");

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GlobalTransform::default(),
    ));

    for i in 0..8 {
        let angle = (i as f32 / 8.0) * 2.0 * PI;
        let radius = 5.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        commands.spawn((
            Transform::from_translation(Vec3::new(x, 0.0, z)),
            GlobalTransform::default(),
        ));
    }

    for y in 0..3 {
        for x in 0..3 {
            commands.spawn((
                Transform::from_translation(Vec3::new(
                    (x as f32 - 1.0) * 3.0,
                    2.0 + y as f32 * 3.0,
                    -10.0,
                )),
                GlobalTransform::default(),
            ));
        }
    }
}

fn update_time_system(mut time: ResMut<Time>, input: Res<InputState>) {
    time.elapsed += input.delta_time();
}

fn animate_objects_system(time: Res<Time>, mut query: Query<&mut Transform>) {
    for (i, mut transform) in query.iter_mut().enumerate() {
        let speed = 0.5 + (i as f32 * 0.1);
        let offset = i as f32 * 0.5;
        let y = (time.elapsed * speed + offset).sin() * 2.0;

        transform.translation.y = y;
        transform.rotation = praxis_math::Quat::from_rotation_y(time.elapsed * speed + offset);
    }
}

fn input_system(input: Res<InputState>, mut bloom_config: ResMut<BloomConfigResource>) {
    if input.key_just_pressed(Key::Digit1) {
        bloom_config.config.brightness_threshold =
            (bloom_config.config.brightness_threshold - 0.1).max(0.1);
        info!(
            "Brightness threshold: {}",
            bloom_config.config.brightness_threshold
        );
    }
    if input.key_just_pressed(Key::Digit2) {
        bloom_config.config.brightness_threshold =
            (bloom_config.config.brightness_threshold + 0.1).min(5.0);
        info!(
            "Brightness threshold: {}",
            bloom_config.config.brightness_threshold
        );
    }

    if input.key_just_pressed(Key::Digit3) {
        bloom_config.config.blur_iterations =
            bloom_config.config.blur_iterations.saturating_sub(1).max(1);
        info!("Blur iterations: {}", bloom_config.config.blur_iterations);
    }
    if input.key_just_pressed(Key::Digit4) {
        bloom_config.config.blur_iterations = (bloom_config.config.blur_iterations + 1).min(10);
        info!("Blur iterations: {}", bloom_config.config.blur_iterations);
    }

    if input.key_just_pressed(Key::Digit5) {
        bloom_config.config.exposure = (bloom_config.config.exposure - 0.1).max(0.1);
        info!("Exposure: {}", bloom_config.config.exposure);
    }
    if input.key_just_pressed(Key::Digit6) {
        bloom_config.config.exposure = (bloom_config.config.exposure + 0.1).min(5.0);
        info!("Exposure: {}", bloom_config.config.exposure);
    }

    if input.key_just_pressed(Key::Digit7) {
        bloom_config.config.bloom_intensity =
            (bloom_config.config.bloom_intensity - 0.05).max(0.0);
        info!("Bloom intensity: {}", bloom_config.config.bloom_intensity);
    }
    if input.key_just_pressed(Key::Digit8) {
        bloom_config.config.bloom_intensity =
            (bloom_config.config.bloom_intensity + 0.05).min(2.0);
        info!("Bloom intensity: {}", bloom_config.config.bloom_intensity);
    }
}

fn render_system(
    render_context: &mut RenderContext,
    query: Query<&GlobalTransform>,
    camera: Res<Camera>,
    _bloom_config: Res<BloomConfigResource>,
) -> Result<()> {
    let aspect_ratio = 1920.0 / 1080.0;
    let proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 100.0);

    let view = Mat4::look_at_rh(
        camera.position,
        camera.position + Vec3::new(0.0, 0.0, -1.0),
        Vec3::Y,
    );

    let mut draw_commands = Vec::new();

    for (i, global_transform) in query.iter().enumerate() {
        let mesh_id = if i == 0 { "cube" } else { "sphere" };

        let brightness = 1.0 + (i as f32 * 0.5);
        let color_variant = (i as f32 * 0.3).sin();

        let material = MaterialProperties::new()
            .with_albedo([
                1.0,
                0.5 + color_variant * 0.5,
                0.2 + color_variant * 0.3,
                1.0,
            ])
            .with_emissive([brightness, brightness * 0.5, brightness * 0.2, 1.0])
            .with_metallic(0.8)
            .with_roughness(0.3);

        draw_commands.push(DrawCommand {
            mesh_id: mesh_id.to_string(),
            model: global_transform.compute_matrix(),
            texture_name: None,
            material_properties: Some(material),
        });
    }

    let cmds = RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: None,
    };

    render_context.render(&cmds)?;

    Ok(())
}

fn main() -> Result<()> {
    info!("Starting bloom demo");
    info!("Controls:");
    info!("  1/2   - Decrease/Increase brightness threshold");
    info!("  3/4   - Decrease/Increase blur iterations");
    info!("  5/6   - Decrease/Increase exposure");
    info!("  7/8   - Decrease/Increase bloom intensity");
    info!("  ESC   - Exit");
    info!("");
    info!("Note: This demo shows objects with emissive materials.");
    info!("To see the bloom effect in action, integrate BloomEffect into");
    info!("the render pipeline using render-to-texture workflow.");

    run(|world: &mut World| {
        world.insert_resource(Camera::default());
        world.insert_resource(Time::default());
        world.insert_resource(BloomConfigResource::default());

        let render_context = world.get_resource_mut::<RenderContext>().unwrap();

        render_context
            .mesh_manager_mut()
            .load_mesh("cube", colored_cube_mesh())?;

        render_context
            .mesh_manager_mut()
            .load_mesh("sphere", sphere_mesh(1.0, 32, 16))?;

        Ok(())
    })?;

    Ok(())
}
