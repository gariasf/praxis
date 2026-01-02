//! Camera system demonstration.
//!
//! This example demonstrates the camera system in praxis_ecs, showing:
//! - Creating perspective and orthographic cameras
//! - Camera activation/deactivation
//! - Camera priorities
//! - Automatic view and projection matrix computation
//! - Camera query helpers

use praxis_ecs::{
    Camera, CameraMatrices, OrthographicCameraBundle, OrthographicProjection,
    PerspectiveCameraBundle, PerspectiveProjection, Query, Schedule, Transform, World, camera,
};
use praxis_math::Vec3;

fn main() {
    println!("=== Camera System Demo ===\n");

    let mut world = World::new();

    println!("Creating cameras...");

    let perspective_cam1 = world.spawn(PerspectiveCameraBundle::new(
        Vec3::new(0.0, 5.0, 10.0),
        70.0_f32.to_radians(),
        16.0 / 9.0,
    ));
    println!("Created perspective camera 1 at position (0, 5, 10) with priority 0");

    let mut high_priority_bundle =
        PerspectiveCameraBundle::new(Vec3::new(5.0, 5.0, 5.0), 60.0_f32.to_radians(), 16.0 / 9.0);
    high_priority_bundle.camera.priority = 10;
    let perspective_cam2 = world.spawn(high_priority_bundle);
    println!("Created perspective camera 2 at position (5, 5, 5) with priority 10");

    let ortho_cam = world.spawn(OrthographicCameraBundle::new(
        Vec3::new(0.0, 10.0, 0.0),
        20.0,
        10.0,
    ));
    println!("Created orthographic camera at position (0, 10, 0) with priority 0\n");

    let mut schedule = Schedule::default();
    schedule.add_systems((
        praxis_ecs::systems::update_perspective_cameras,
        praxis_ecs::systems::update_orthographic_cameras,
    ));

    println!("Running camera update systems...");
    world.inner_mut().run_schedule(&mut schedule);
    println!("Camera matrices computed successfully\n");

    println!("=== Testing Camera Queries ===\n");

    let perspective_cameras = world.query::<camera::ActivePerspectiveCameras>();
    let orthographic_cameras = world.query::<camera::ActiveOrthographicCameras>();

    println!("Active perspective cameras:");
    for (entity, camera, transform, _projection, _matrices) in perspective_cameras.iter() {
        println!(
            "  Entity: {:?}, Position: {:?}, Priority: {}",
            entity, transform.translation, camera.priority
        );
    }

    if let Some((entity, camera, matrices)) =
        camera::primary_perspective_camera(&perspective_cameras)
    {
        println!(
            "\nPrimary perspective camera: Entity {:?}, Priority: {}",
            entity, camera.priority
        );
        println!("  View matrix: {:?}", matrices.view);
        println!("  Projection matrix: {:?}", matrices.projection);
    }

    println!("\nActive orthographic cameras:");
    for (entity, camera, transform, _projection, _matrices) in orthographic_cameras.iter() {
        println!(
            "  Entity: {:?}, Position: {:?}, Priority: {}",
            entity, transform.translation, camera.priority
        );
    }

    println!("\n=== Testing Camera Activation/Deactivation ===\n");

    if let Some(mut camera) = world.inner_mut().get_mut::<Camera>(perspective_cam1) {
        camera.deactivate();
        println!(
            "Deactivated perspective camera 1 (Entity: {:?})",
            perspective_cam1
        );
    }

    world.inner_mut().run_schedule(&mut schedule);

    let perspective_cameras = world.query::<camera::ActivePerspectiveCameras>();
    let active_count = perspective_cameras
        .iter()
        .filter(|(_, cam, _, _, _)| cam.is_active)
        .count();
    println!("Active perspective cameras: {}", active_count);

    if let Some((entity, camera, _matrices)) =
        camera::primary_perspective_camera(&perspective_cameras)
    {
        println!(
            "New primary perspective camera: Entity {:?}, Priority: {}",
            entity, camera.priority
        );
    }

    println!("\n=== Testing Sorted Camera Access ===\n");

    if let Some(mut camera) = world.inner_mut().get_mut::<Camera>(perspective_cam1) {
        camera.activate();
    }

    world.inner_mut().run_schedule(&mut schedule);

    let perspective_cameras = world.query::<camera::ActivePerspectiveCameras>();
    let sorted = camera::sorted_perspective_cameras(&perspective_cameras);

    println!("Cameras sorted by priority (low to high):");
    for (entity, camera, _matrices) in sorted {
        println!("  Entity: {:?}, Priority: {}", entity, camera.priority);
    }

    println!("\n=== Testing Camera Transforms ===\n");

    if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(perspective_cam1) {
        transform.translation = Vec3::new(10.0, 10.0, 10.0);
        println!("Moved perspective camera 1 to (10, 10, 10)");
    }

    world.inner_mut().run_schedule(&mut schedule);

    let perspective_cameras = world.query::<camera::ActivePerspectiveCameras>();
    for (entity, _camera, transform, _projection, matrices) in perspective_cameras.iter() {
        if entity == perspective_cam1 {
            println!("Camera 1 updated:");
            println!("  Position: {:?}", transform.translation);
            println!("  View matrix updated: {:?}", matrices.view);
        }
    }

    println!("\n=== Demo Complete ===");
}
