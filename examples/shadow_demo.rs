//! Shadow mapping demonstration.
//!
//! This example demonstrates the shadow mapping system with:
//! - Cascaded shadow maps (CSM)
//! - PCF filtering for soft shadows
//! - Dynamic lighting with directional lights
//! - Real-time shadow updates

use praxis_core::PraxisEngine;
use praxis_ecs::{Commands, Query, Res, ResMut};
use praxis_graphics::{
    colored_cube_mesh, quad_mesh, DirectionalLightData, LightingUniforms, MaterialProperties,
    ShadowConfig, ShadowMapManager, MAX_DIRECTIONAL_LIGHTS,
};
use praxis_input::InputState;
use praxis_math::{Mat4, Quat, Vec3};
use praxis_scene::{GlobalTransform, Transform};
use praxis_utils::Result;

/// Main entry point for the shadow demo.
fn main() -> Result<()> {
    // Create and run the engine
    let mut engine = PraxisEngine::new("Shadow Mapping Demo")?;

    // Register startup system to initialize the scene
    engine.add_startup_system(setup_scene);

    // Register update systems
    engine.add_system(update_camera);
    engine.add_system(update_lighting);
    engine.add_system(animate_objects);

    // Run the engine
    engine.run()
}

/// Camera component for tracking camera state.
#[derive(praxis_ecs::Component)]
struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 10.0, 20.0),
            yaw: 0.0,
            pitch: -30.0,
            distance: 20.0,
        }
    }
}

/// Light animation component.
#[derive(praxis_ecs::Component)]
struct AnimatedLight {
    time: f32,
    speed: f32,
}

impl Default for AnimatedLight {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 0.5,
        }
    }
}

/// Rotating object component.
#[derive(praxis_ecs::Component)]
struct Rotating {
    axis: Vec3,
    speed: f32,
}

/// Setup the initial scene with objects, lighting, and shadow configuration.
fn setup_scene(mut commands: Commands, mut ctx: ResMut<praxis_graphics::RenderContext>) {
    // Load meshes
    ctx.mesh_manager_mut()
        .load_mesh("cube", colored_cube_mesh())
        .expect("Failed to load cube mesh");

    ctx.mesh_manager_mut()
        .load_mesh("ground", quad_mesh())
        .expect("Failed to load ground mesh");

    // Create ground plane
    commands.spawn((
        Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(50.0),
        },
        GlobalTransform::default(),
    ));

    // Create cubes at various positions
    let cube_positions = [
        Vec3::new(-5.0, 1.0, 0.0),
        Vec3::new(5.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, -5.0),
        Vec3::new(0.0, 1.0, 5.0),
        Vec3::new(-3.0, 2.5, -3.0),
        Vec3::new(3.0, 2.5, 3.0),
    ];

    for (i, pos) in cube_positions.iter().enumerate() {
        let mut entity = commands.spawn((
            Transform {
                position: *pos,
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(1.0),
            },
            GlobalTransform::default(),
        ));

        // Make some cubes rotate
        if i % 2 == 0 {
            entity.insert(Rotating {
                axis: Vec3::new(1.0, 1.0, 0.0).normalize(),
                speed: 45.0_f32.to_radians() * (i as f32 * 0.5 + 1.0),
            });
        }
    }

    // Create tall pillar to show shadow
    commands.spawn((
        Transform {
            position: Vec3::new(0.0, 5.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, 10.0, 1.0),
        },
        GlobalTransform::default(),
    ));

    // Setup camera
    commands.spawn(Camera::default());

    // Setup animated light
    commands.spawn(AnimatedLight::default());

    // Initialize shadow mapping with default configuration
    // This will be created in the render system when needed
    println!("Scene setup complete!");
    println!("Controls:");
    println!("  Arrow keys: Rotate camera");
    println!("  +/-: Adjust camera distance");
    println!("  ESC: Exit");
}

/// Update camera based on input.
fn update_camera(mut query: Query<&mut Camera>, input: Res<InputState>) {
    for mut camera in query.iter_mut() {
        // Camera rotation with arrow keys
        if input.is_key_held(praxis_input::Key::ArrowLeft) {
            camera.yaw -= 1.5;
        }
        if input.is_key_held(praxis_input::Key::ArrowRight) {
            camera.yaw += 1.5;
        }
        if input.is_key_held(praxis_input::Key::ArrowUp) {
            camera.pitch = (camera.pitch - 1.5).clamp(-89.0, 89.0);
        }
        if input.is_key_held(praxis_input::Key::ArrowDown) {
            camera.pitch = (camera.pitch + 1.5).clamp(-89.0, 89.0);
        }

        // Camera distance with +/- keys
        if input.is_key_held(praxis_input::Key::Equal) {
            camera.distance = (camera.distance - 0.5).max(5.0);
        }
        if input.is_key_held(praxis_input::Key::Minus) {
            camera.distance = (camera.distance + 0.5).min(50.0);
        }

        // Calculate camera position
        let yaw_rad = camera.yaw.to_radians();
        let pitch_rad = camera.pitch.to_radians();

        camera.position = Vec3::new(
            camera.distance * pitch_rad.cos() * yaw_rad.sin(),
            camera.distance * pitch_rad.sin(),
            camera.distance * pitch_rad.cos() * yaw_rad.cos(),
        );
    }
}

/// Update lighting based on animated light.
fn update_lighting(
    mut query: Query<&mut AnimatedLight>,
    mut ctx: ResMut<praxis_graphics::RenderContext>,
) {
    for mut light in query.iter_mut() {
        light.time += 0.016 * light.speed; // Assuming ~60 FPS

        // Animate light direction in a circle
        let angle = light.time;
        let light_direction = Vec3::new(angle.cos(), -0.7, angle.sin()).normalize();

        // Setup lighting
        let mut lighting = LightingUniforms::default();

        // Main directional light (sun)
        lighting.directional_lights[0] = DirectionalLightData {
            direction: [light_direction.x, light_direction.y, light_direction.z, 0.0],
            color: [1.0, 0.95, 0.8, 0.0],
            intensity: 1.2,
            _padding: [0.0; 3],
        };
        lighting.directional_light_count = 1;

        // Ambient lighting
        lighting.ambient_color = [0.15, 0.15, 0.2, 0.0];

        // Update lighting
        ctx.lighting_buffer_mut()
            .update(&lighting)
            .expect("Failed to update lighting");
    }
}

/// Animate rotating objects.
fn animate_objects(mut query: Query<(&mut Transform, &Rotating)>) {
    for (mut transform, rotating) in query.iter_mut() {
        let delta_rotation = Quat::from_axis_angle(rotating.axis, rotating.speed * 0.016);
        transform.rotation = delta_rotation * transform.rotation;
    }
}
