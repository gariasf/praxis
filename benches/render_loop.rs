use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use praxis_ecs::{
    camera, Camera, CameraMatrices, PerspectiveProjection, Schedule, Transform, World,
};

fn setup_world_with_cameras(camera_count: usize) -> World {
    let mut world = World::new();

    for i in 0..camera_count {
        let priority = i as i32;
        world.spawn((
            Camera::with_priority(priority),
            Transform::from_xyz(0.0, (i as f32) * 5.0, 10.0),
            PerspectiveProjection::default(),
            CameraMatrices::default(),
        ));
    }

    world
}

fn bench_camera_matrix_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("camera_matrix_updates");

    for camera_count in [1, 5, 10, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(camera_count),
            &camera_count,
            |b, &camera_count| {
                let mut world = setup_world_with_cameras(camera_count);
                let mut schedule = Schedule::default();
                schedule.add_systems(praxis_ecs::systems::update_perspective_cameras);

                b.iter(|| {
                    world.inner_mut().run_schedule(&mut schedule);
                    black_box(&world);
                });
            },
        );
    }

    group.finish();
}

fn bench_camera_query_primary(c: &mut Criterion) {
    let mut group = c.benchmark_group("camera_query_primary");

    for camera_count in [1, 5, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(camera_count),
            &camera_count,
            |b, &camera_count| {
                let mut world = setup_world_with_cameras(camera_count);
                let mut schedule = Schedule::default();
                schedule.add_systems(praxis_ecs::systems::update_perspective_cameras);
                world.inner_mut().run_schedule(&mut schedule);

                b.iter(|| {
                    let query = world
                        .inner_mut()
                        .query::<camera::ActivePerspectiveCameras>();
                    let primary = camera::primary_perspective_camera(&query);
                    black_box(primary);
                });
            },
        );
    }

    group.finish();
}

fn bench_camera_query_sorted(c: &mut Criterion) {
    let mut group = c.benchmark_group("camera_query_sorted");

    for camera_count in [1, 5, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(camera_count),
            &camera_count,
            |b, &camera_count| {
                let mut world = setup_world_with_cameras(camera_count);
                let mut schedule = Schedule::default();
                schedule.add_systems(praxis_ecs::systems::update_perspective_cameras);
                world.inner_mut().run_schedule(&mut schedule);

                b.iter(|| {
                    let query = world
                        .inner_mut()
                        .query::<camera::ActivePerspectiveCameras>();
                    let sorted = camera::sorted_perspective_cameras(&query);
                    black_box(sorted);
                });
            },
        );
    }

    group.finish();
}

fn bench_frame_timing_simulation(c: &mut Criterion) {
    use praxis_utils::timing::FrameTimer;

    c.bench_function("frame_timer_update", |b| {
        let mut timer = FrameTimer::new();

        b.iter(|| {
            timer.tick();
            black_box(timer.delta());
            black_box(timer.fps());
        });
    });
}

criterion_group!(
    benches,
    bench_camera_matrix_updates,
    bench_camera_query_primary,
    bench_camera_query_sorted,
    bench_frame_timing_simulation
);
criterion_main!(benches);
