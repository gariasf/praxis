use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use praxis_scene::{
    EditorCamera, EditorData, EntityDefinition, GizmoMode, SceneDefinition, SceneLoader,
    TransformDef, ViewportSettings,
};

fn create_simple_entity(name: &str, x: f32, y: f32, z: f32) -> EntityDefinition {
    EntityDefinition::new()
        .with_name(name)
        .with_transform(TransformDef::from_translation(x, y, z))
        .with_mesh("mesh")
}

fn create_camera_entity(name: &str, x: f32, y: f32, z: f32) -> EntityDefinition {
    EntityDefinition::perspective_camera(name, (x, y, z), 1.0472, 1.77)
}

fn create_light_entity(name: &str, _x: f32, _y: f32, _z: f32) -> EntityDefinition {
    EntityDefinition::directional_light(name, (0.0, -1.0, 0.0), (1.0, 1.0, 1.0), 1.0)
}

fn create_hierarchy_entity(name: &str, depth: usize, children_per_node: usize) -> EntityDefinition {
    let mut entity = EntityDefinition::new()
        .with_name(name)
        .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0));

    if depth > 0 {
        for i in 0..children_per_node {
            let child_name = format!("{name}_{i}");
            let child = create_hierarchy_entity(&child_name, depth - 1, children_per_node);
            entity = entity.with_child(child);
        }
    }

    entity
}

fn create_scene_with_entities(entity_count: usize) -> SceneDefinition {
    let mut scene = SceneDefinition::new(format!("Benchmark Scene {entity_count}"));

    scene.metadata.description = Some("Benchmark scene for performance testing".to_string());
    scene.metadata.author = Some("Benchmark Suite".to_string());
    scene.metadata.version = Some("1.0.0".to_string());
    scene.metadata.tags = vec!["benchmark".to_string(), "test".to_string()];

    scene.add_entity(create_camera_entity("MainCamera", 0.0, 5.0, 10.0));
    scene.add_entity(create_light_entity("Sun", 0.0, 10.0, 0.0));

    for i in 0..entity_count {
        let x = (i % 10) as f32 * 2.0;
        let y = 0.0;
        let z = (i / 10) as f32 * 2.0;
        let name = format!("Entity_{i}");
        scene.add_entity(create_simple_entity(&name, x, y, z));
    }

    scene
}

fn create_scene_with_hierarchy(depth: usize, children_per_node: usize) -> SceneDefinition {
    let mut scene = SceneDefinition::new("Hierarchy Benchmark Scene");

    scene.add_entity(create_camera_entity("MainCamera", 0.0, 5.0, 10.0));
    scene.add_entity(create_light_entity("Sun", 0.0, 10.0, 0.0));

    scene.add_entity(create_hierarchy_entity("Root", depth, children_per_node));

    scene
}

fn create_scene_with_editor_data(entity_count: usize) -> SceneDefinition {
    let mut scene = create_scene_with_entities(entity_count);

    let mut editor_camera = EditorCamera::new();
    editor_camera.position = (10.0, 8.0, 15.0);
    editor_camera.target = (0.0, 1.0, 0.0);
    editor_camera.distance = 20.0;
    editor_camera.pitch = -0.4;
    editor_camera.yaw = 0.8;
    editor_camera.fov = 60.0;

    let mut viewport = ViewportSettings::new();
    viewport.show_grid = true;
    viewport.show_gizmos = true;
    viewport.gizmo_mode = GizmoMode::Translate;
    viewport.grid_size = 20;
    viewport.grid_spacing = 1.0;

    let selected: Vec<String> = (0..entity_count.min(10))
        .map(|i| format!("Entity_{i}"))
        .collect();

    let editor_data = EditorData::new()
        .with_camera(editor_camera)
        .with_selected_entities(selected)
        .with_viewport(viewport);

    scene.set_editor_data(editor_data);

    scene
}

fn bench_scene_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_serialization");

    for entity_count in [10, 50, 100, 500, 1000] {
        let scene = create_scene_with_entities(entity_count);
        let loader = SceneLoader::new();

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            &scene,
            |b, scene| {
                b.iter(|| {
                    let ron = loader.save_to_string(scene).expect("Failed to serialize");
                    black_box(ron);
                });
            },
        );
    }

    group.finish();
}

fn bench_scene_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_deserialization");

    for entity_count in [10, 50, 100, 500, 1000] {
        let scene = create_scene_with_entities(entity_count);
        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).expect("Failed to serialize");

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            &ron_string,
            |b, ron| {
                b.iter(|| {
                    let scene = loader.load_from_string(ron).expect("Failed to deserialize");
                    black_box(scene);
                });
            },
        );
    }

    group.finish();
}

fn bench_scene_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_roundtrip");

    for entity_count in [10, 50, 100, 500] {
        let scene = create_scene_with_entities(entity_count);
        let loader = SceneLoader::new();

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            &scene,
            |b, scene| {
                b.iter(|| {
                    let ron = loader.save_to_string(scene).expect("Failed to serialize");
                    let loaded = loader
                        .load_from_string(&ron)
                        .expect("Failed to deserialize");
                    black_box(loaded);
                });
            },
        );
    }

    group.finish();
}

fn bench_scene_with_hierarchy_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_hierarchy_serialization");

    for (depth, children) in [(2, 3), (3, 3), (4, 2), (5, 2)] {
        let scene = create_scene_with_hierarchy(depth, children);
        let loader = SceneLoader::new();
        let total_entities = scene.total_entity_count();

        group.throughput(Throughput::Elements(total_entities as u64));
        group.bench_with_input(
            BenchmarkId::new("depth_children", format!("{depth}_{children}")),
            &scene,
            |b, scene| {
                b.iter(|| {
                    let ron = loader.save_to_string(scene).expect("Failed to serialize");
                    black_box(ron);
                });
            },
        );
    }

    group.finish();
}

fn bench_scene_with_hierarchy_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_hierarchy_deserialization");

    for (depth, children) in [(2, 3), (3, 3), (4, 2), (5, 2)] {
        let scene = create_scene_with_hierarchy(depth, children);
        let loader = SceneLoader::new();
        let ron_string = loader.save_to_string(&scene).expect("Failed to serialize");
        let total_entities = scene.total_entity_count();

        group.throughput(Throughput::Elements(total_entities as u64));
        group.bench_with_input(
            BenchmarkId::new("depth_children", format!("{depth}_{children}")),
            &ron_string,
            |b, ron| {
                b.iter(|| {
                    let scene = loader.load_from_string(ron).expect("Failed to deserialize");
                    black_box(scene);
                });
            },
        );
    }

    group.finish();
}

fn bench_scene_with_editor_data_serialization(c: &mut Criterion) {
    let scene = create_scene_with_editor_data(100);
    let loader = SceneLoader::new();

    c.bench_function("scene_with_editor_data_serialize", |b| {
        b.iter(|| {
            let ron = loader.save_to_string(&scene).expect("Failed to serialize");
            black_box(ron);
        });
    });
}

fn bench_scene_with_editor_data_deserialization(c: &mut Criterion) {
    let scene = create_scene_with_editor_data(100);
    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).expect("Failed to serialize");

    c.bench_function("scene_with_editor_data_deserialize", |b| {
        b.iter(|| {
            let scene = loader
                .load_from_string(&ron_string)
                .expect("Failed to deserialize");
            black_box(scene);
        });
    });
}

fn bench_scene_to_runtime(c: &mut Criterion) {
    let scene = create_scene_with_editor_data(500);

    c.bench_function("scene_to_runtime_conversion", |b| {
        b.iter(|| {
            let runtime = scene.to_runtime_scene();
            black_box(runtime);
        });
    });
}

fn bench_scene_metadata_serialization(c: &mut Criterion) {
    let mut scene = SceneDefinition::new("Metadata Test");

    scene.metadata.description = Some("A".repeat(1000));
    scene.metadata.author = Some("Test Author".to_string());
    scene.metadata.version = Some("1.0.0".to_string());
    scene.metadata.tags = (0..100).map(|i| format!("tag_{i}")).collect();

    for i in 0..50 {
        scene.add_entity(create_simple_entity(
            &format!("Entity_{i}"),
            0.0,
            0.0,
            0.0,
        ));
    }

    let loader = SceneLoader::new();

    c.bench_function("scene_heavy_metadata_serialize", |b| {
        b.iter(|| {
            let ron = loader.save_to_string(&scene).expect("Failed to serialize");
            black_box(ron);
        });
    });
}

fn bench_minimal_scene_serialization(c: &mut Criterion) {
    let scene = SceneDefinition::new("Minimal Scene");
    let loader = SceneLoader::new();

    c.bench_function("minimal_scene_serialize", |b| {
        b.iter(|| {
            let ron = loader.save_to_string(&scene).expect("Failed to serialize");
            black_box(ron);
        });
    });
}

fn bench_minimal_scene_deserialization(c: &mut Criterion) {
    let scene = SceneDefinition::new("Minimal Scene");
    let loader = SceneLoader::new();
    let ron_string = loader.save_to_string(&scene).expect("Failed to serialize");

    c.bench_function("minimal_scene_deserialize", |b| {
        b.iter(|| {
            let scene = loader
                .load_from_string(&ron_string)
                .expect("Failed to deserialize");
            black_box(scene);
        });
    });
}

fn bench_complex_scene_with_all_features(c: &mut Criterion) {
    let mut scene = SceneDefinition::new("Complex Scene");

    scene.metadata.description = Some("Complex benchmark scene".to_string());
    scene.metadata.author = Some("Test".to_string());

    scene.add_entity(create_camera_entity("Camera1", 0.0, 5.0, 10.0));
    scene.add_entity(create_camera_entity("Camera2", 10.0, 5.0, 0.0));
    scene.add_entity(create_light_entity("Sun", 0.0, 10.0, 0.0));
    scene.add_entity(create_light_entity("Moon", 0.0, -10.0, 0.0));

    for i in 0..50 {
        let mut entity = create_simple_entity(&format!("Entity_{i}"), i as f32, 0.0, 0.0);
        for j in 0..3 {
            let child = create_simple_entity(&format!("Child_{i}_{j}"), j as f32, 0.0, 0.0);
            entity = entity.with_child(child);
        }
        scene.add_entity(entity);
    }

    scene.set_editor_data(
        EditorData::new()
            .with_camera(EditorCamera::new())
            .with_viewport(ViewportSettings::new()),
    );

    let loader = SceneLoader::new();

    c.bench_function("complex_scene_serialize", |b| {
        b.iter(|| {
            let ron = loader.save_to_string(&scene).expect("Failed to serialize");
            black_box(ron);
        });
    });

    let ron_string = loader.save_to_string(&scene).expect("Failed to serialize");

    c.bench_function("complex_scene_deserialize", |b| {
        b.iter(|| {
            let scene = loader
                .load_from_string(&ron_string)
                .expect("Failed to deserialize");
            black_box(scene);
        });
    });
}

criterion_group!(
    benches,
    bench_scene_serialization,
    bench_scene_deserialization,
    bench_scene_roundtrip,
    bench_scene_with_hierarchy_serialization,
    bench_scene_with_hierarchy_deserialization,
    bench_scene_with_editor_data_serialization,
    bench_scene_with_editor_data_deserialization,
    bench_scene_to_runtime,
    bench_scene_metadata_serialization,
    bench_minimal_scene_serialization,
    bench_minimal_scene_deserialization,
    bench_complex_scene_with_all_features
);
criterion_main!(benches);
