use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use praxis_ecs::{Children, GlobalTransform, Parent, Schedule, Transform, World};
use praxis_math::{Quat, Vec3};

fn create_hierarchy(world: &mut World, depth: usize, breadth: usize) -> Vec<praxis_ecs::Entity> {
    let mut entities = Vec::new();

    let root = world.spawn((
        Transform::from_xyz(10.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));
    entities.push(root);

    let mut current_level = vec![root];

    for level in 0..depth {
        let mut next_level = Vec::new();

        for parent_entity in &current_level {
            let mut children_list = Vec::new();

            for child_idx in 0..breadth {
                let offset = child_idx as f32 * 2.0;
                let child = world.spawn((
                    Transform::from_xyz(offset, level as f32, 0.0),
                    GlobalTransform::default(),
                    Parent(*parent_entity),
                ));

                children_list.push(child);
                next_level.push(child);
                entities.push(child);
            }

            world
                .insert_component(*parent_entity, Children::with_children(children_list))
                .unwrap();
        }

        current_level = next_level;
    }

    entities
}

fn bench_transform_propagation_flat(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_propagation_flat");

    for entity_count in [10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            &entity_count,
            |b, &entity_count| {
                let mut world = World::new();

                for i in 0..entity_count {
                    world.spawn((
                        Transform::from_xyz(i as f32, 0.0, 0.0),
                        GlobalTransform::default(),
                    ));
                }

                let mut schedule = Schedule::default();
                schedule.add_systems((
                    praxis_ecs::systems::sync_parent_child_relationships,
                    praxis_ecs::systems::propagate_transforms,
                ));

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);
                });
            },
        );
    }

    group.finish();
}

fn bench_transform_propagation_hierarchical(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_propagation_hierarchical");

    let test_cases: Vec<(usize, usize)> = vec![
        (3, 2), // depth=3, breadth=2: 15 entities
        (4, 2), // depth=4, breadth=2: 31 entities
        (3, 4), // depth=3, breadth=4: 85 entities
        (4, 4), // depth=4, breadth=4: 341 entities
        (5, 3), // depth=5, breadth=3: 364 entities
    ];

    for (depth, breadth) in test_cases {
        let entity_count: usize = (1..=depth).fold(1, |acc, d| acc + breadth.pow(d as u32));

        group.bench_with_input(
            BenchmarkId::new(format!("depth{}_breadth{}", depth, breadth), entity_count),
            &(depth, breadth),
            |b, &(depth, breadth)| {
                let mut world = World::new();
                create_hierarchy(&mut world, depth, breadth);

                let mut schedule = Schedule::default();
                schedule.add_systems((
                    praxis_ecs::systems::sync_parent_child_relationships,
                    praxis_ecs::systems::propagate_transforms,
                    praxis_ecs::systems::propagate_transforms_for_reparented,
                    praxis_ecs::systems::propagate_transforms_for_changed_children,
                ));

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);
                });
            },
        );
    }

    group.finish();
}

fn bench_transform_propagation_with_rotation(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_propagation_rotation");

    for entity_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            &entity_count,
            |b, &entity_count| {
                let mut world = World::new();

                let root = world.spawn((
                    Transform {
                        translation: Vec3::new(10.0, 0.0, 0.0),
                        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
                        scale: Vec3::ONE,
                    },
                    GlobalTransform::default(),
                ));

                for i in 0..entity_count - 1 {
                    let child = world.spawn((
                        Transform {
                            translation: Vec3::new(i as f32 * 0.5, 0.0, 0.0),
                            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_6),
                            scale: Vec3::ONE,
                        },
                        GlobalTransform::default(),
                        Parent(root),
                    ));

                    if i == 0 {
                        world
                            .insert_component(root, Children::with_children(vec![child]))
                            .unwrap();
                    } else {
                        let mut children = world.inner().get::<Children>(root).unwrap().clone();
                        children.push(child);
                        world.insert_component(root, children).unwrap();
                    }
                }

                let mut schedule = Schedule::default();
                schedule.add_systems((
                    praxis_ecs::systems::sync_parent_child_relationships,
                    praxis_ecs::systems::propagate_transforms,
                ));

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);
                });
            },
        );
    }

    group.finish();
}

fn bench_transform_propagation_deep_hierarchy(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_propagation_deep");

    for depth in [5, 10, 20, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            let mut world = World::new();

            let mut previous_entity = world.spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
            ));

            for _i in 1..depth {
                let child = world.spawn((
                    Transform::from_xyz(1.0, 0.0, 0.0),
                    GlobalTransform::default(),
                    Parent(previous_entity),
                ));

                world
                    .insert_component(previous_entity, Children::with_children(vec![child]))
                    .unwrap();

                previous_entity = child;
            }

            let mut schedule = Schedule::default();
            schedule.add_systems((
                praxis_ecs::systems::sync_parent_child_relationships,
                praxis_ecs::systems::propagate_transforms,
            ));

            b.iter(|| {
                schedule.run(world.inner_mut());
                black_box(&world);
            });
        });
    }

    group.finish();
}

fn bench_parent_child_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("parent_child_sync");

    for entity_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            &entity_count,
            |b, &entity_count| {
                let mut world = World::new();

                let parent = world.spawn((Transform::default(), GlobalTransform::default()));

                for i in 0..entity_count {
                    world.spawn((
                        Transform::from_xyz(i as f32, 0.0, 0.0),
                        GlobalTransform::default(),
                        Parent(parent),
                    ));
                }

                let mut schedule = Schedule::default();
                schedule.add_systems(praxis_ecs::systems::sync_parent_child_relationships);

                b.iter(|| {
                    schedule.run(world.inner_mut());
                    black_box(&world);
                });
            },
        );
    }

    group.finish();
}

fn bench_transform_modification_propagation(c: &mut Criterion) {
    let mut world = World::new();
    create_hierarchy(&mut world, 4, 3);

    let mut schedule = Schedule::default();
    schedule.add_systems((
        praxis_ecs::systems::sync_parent_child_relationships,
        praxis_ecs::systems::propagate_transforms,
        praxis_ecs::systems::propagate_transforms_for_changed_children,
    ));

    c.bench_function("transform_modification_propagation", |b| {
        b.iter(|| {
            let entities: Vec<_> = world
                .inner_mut()
                .query::<praxis_ecs::Entity>()
                .iter(world.inner_mut())
                .collect();

            if let Some(&entity) = entities.get(5) {
                if let Some(mut transform) = world.inner_mut().get_mut::<Transform>(entity) {
                    transform.translation.x += 0.1;
                }
            }

            schedule.run(world.inner_mut());
            black_box(&world);
        });
    });
}

criterion_group!(
    benches,
    bench_transform_propagation_flat,
    bench_transform_propagation_hierarchical,
    bench_transform_propagation_with_rotation,
    bench_transform_propagation_deep_hierarchy,
    bench_parent_child_sync,
    bench_transform_modification_propagation
);
criterion_main!(benches);
