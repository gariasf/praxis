use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use praxis_ecs::{Schedule, Transform, World};
use praxis_math::Vec3;
use praxis_physics::{
    cleanup_physics_entities, clear_collision_event_receivers, physics_step_system,
    populate_collision_events, sync_physics_transforms_system, Collider, CollisionEventReceiver,
    ContactEvents, PhysicsConfig, PhysicsTime, PhysicsVelocity, PhysicsWorld, RigidBody,
};

fn setup_physics_world(object_count: usize, with_collisions: bool) -> World {
    let mut world = World::new();

    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    world.insert_resource(PhysicsTime::new());
    world.insert_resource(ContactEvents::new());

    world.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(50.0, 0.5, 50.0),
    ));

    for i in 0..object_count {
        let x = (i % 10) as f32 * 2.0 - 10.0;
        let z = (i / 10) as f32 * 2.0 - 10.0;
        let y = 10.0 + (i as f32 * 2.0);

        if with_collisions {
            world.spawn((
                Transform::from_xyz(x, y, z),
                RigidBody::Dynamic,
                Collider::sphere(0.5),
                PhysicsVelocity::default(),
                CollisionEventReceiver::new(),
            ));
        } else {
            world.spawn((
                Transform::from_xyz(x, y, z),
                RigidBody::Dynamic,
                Collider::sphere(0.5),
                PhysicsVelocity::default(),
            ));
        }
    }

    world
}

fn bench_physics_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_step");

    for object_count in [10, 50, 100, 250, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                let mut world = setup_physics_world(object_count, false);
                let mut schedule = Schedule::default();
                schedule.add_systems((
                    cleanup_physics_entities,
                    sync_physics_transforms_system,
                    physics_step_system,
                    sync_physics_transforms_system,
                ));

                {
                    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
                    physics_time.accumulator += 1.0 / 60.0;
                }

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);

                    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
                    physics_time.accumulator += 1.0 / 60.0;
                });
            },
        );
    }

    group.finish();
}

fn bench_physics_step_with_collision_events(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_step_with_collisions");

    for object_count in [10, 50, 100, 250] {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                let mut world = setup_physics_world(object_count, true);
                let mut schedule = Schedule::default();
                schedule.add_systems((
                    cleanup_physics_entities,
                    clear_collision_event_receivers,
                    sync_physics_transforms_system,
                    physics_step_system,
                    sync_physics_transforms_system,
                    populate_collision_events,
                ));

                {
                    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
                    physics_time.accumulator += 1.0 / 60.0;
                }

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);

                    let mut physics_time = world.inner_mut().resource_mut::<PhysicsTime>();
                    physics_time.accumulator += 1.0 / 60.0;
                });
            },
        );
    }

    group.finish();
}

fn bench_physics_raycast(c: &mut Criterion) {
    let mut world = setup_physics_world(100, false);
    let mut schedule = Schedule::default();
    schedule.add_systems((
        cleanup_physics_entities,
        sync_physics_transforms_system,
        physics_step_system,
        sync_physics_transforms_system,
    ));
    schedule.run(world.inner_mut());

    c.bench_function("physics_raycast", |b| {
        b.iter(|| {
            let physics_world = world.inner().resource::<PhysicsWorld>();
            let origin = Vec3::new(0.0, 15.0, 0.0);
            let direction = Vec3::new(0.0, -1.0, 0.0);
            let result = physics_world.raycast(origin, direction, 100.0, true);
            black_box(result);
        });
    });
}

fn bench_physics_point_inside(c: &mut Criterion) {
    let mut world = setup_physics_world(100, false);
    let mut schedule = Schedule::default();
    schedule.add_systems((
        cleanup_physics_entities,
        sync_physics_transforms_system,
        physics_step_system,
        sync_physics_transforms_system,
    ));
    schedule.run(world.inner_mut());

    c.bench_function("physics_point_inside", |b| {
        b.iter(|| {
            let physics_world = world.inner().resource::<PhysicsWorld>();
            let point = Vec3::new(0.0, 0.0, 0.0);
            let result = physics_world.point_inside(point);
            black_box(result);
        });
    });
}

fn bench_transform_sync_to_physics(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_sync_to_physics");

    for object_count in [10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            &object_count,
            |b, &object_count| {
                let mut world = setup_physics_world(object_count, false);
                let mut schedule = Schedule::default();
                schedule.add_systems(sync_physics_transforms_system);

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_physics_step,
    bench_physics_step_with_collision_events,
    bench_physics_raycast,
    bench_physics_point_inside,
    bench_transform_sync_to_physics
);
criterion_main!(benches);
