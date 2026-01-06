use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use praxis_assets::{AssetLoader, GltfLoader, MeshLoader};
use std::fs;

fn generate_obj_mesh(vertex_count: usize) -> String {
    let mut obj_content = String::from("# Generated test mesh\n");

    // Generate vertices
    for i in 0..vertex_count {
        let t = i as f32 / vertex_count as f32;
        let x = t * 10.0;
        let y = (t * std::f32::consts::TAU).sin();
        let z = (t * std::f32::consts::TAU).cos();
        obj_content.push_str(&format!("v {x} {y} {z}\n"));
    }

    // Generate normals
    for _ in 0..vertex_count {
        obj_content.push_str("vn 0.0 1.0 0.0\n");
    }

    // Generate texture coordinates
    for i in 0..vertex_count {
        let t = i as f32 / vertex_count as f32;
        obj_content.push_str(&format!("vt {t} {t}\n"));
    }

    // Generate faces (triangles)
    for i in (0..vertex_count - 2).step_by(3) {
        let idx1 = i + 1;
        let idx2 = i + 2;
        let idx3 = i + 3;
        obj_content.push_str(&format!(
            "f {idx1}//{idx1} {idx2}//{idx2} {idx3}//{idx3}\n"
        ));
    }

    obj_content
}

fn generate_gltf_mesh(vertex_count: usize) -> String {
    // Generate a minimal GLTF with embedded data
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    for i in 0..vertex_count {
        let t = i as f32 / vertex_count as f32;
        positions.push(t * 10.0);
        positions.push((t * std::f32::consts::TAU).sin());
        positions.push((t * std::f32::consts::TAU).cos());
    }

    for i in (0..vertex_count - 2).step_by(3) {
        indices.push(i as u16);
        indices.push((i + 1) as u16);
        indices.push((i + 2) as u16);
    }

    // Convert to base64 for embedded buffers
    let positions_bytes: Vec<u8> = positions.iter().flat_map(|&f| f.to_le_bytes()).collect();
    let indices_bytes: Vec<u8> = indices.iter().flat_map(|&i| i.to_le_bytes()).collect();

    let positions_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &positions_bytes);
    let indices_base64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &indices_bytes);

    let positions_len = positions_bytes.len();
    let indices_len = indices_bytes.len();

    format!(
        r#"{{
  "asset": {{
    "version": "2.0"
  }},
  "scene": 0,
  "scenes": [
    {{
      "nodes": [0]
    }}
  ],
  "nodes": [
    {{
      "mesh": 0
    }}
  ],
  "meshes": [
    {{
      "primitives": [
        {{
          "attributes": {{
            "POSITION": 0
          }},
          "indices": 1
        }}
      ]
    }}
  ],
  "buffers": [
    {{
      "uri": "data:application/octet-stream;base64,{}",
      "byteLength": {}
    }},
    {{
      "uri": "data:application/octet-stream;base64,{}",
      "byteLength": {}
    }}
  ],
  "bufferViews": [
    {{
      "buffer": 0,
      "byteOffset": 0,
      "byteLength": {},
      "target": 34962
    }},
    {{
      "buffer": 1,
      "byteOffset": 0,
      "byteLength": {},
      "target": 34963
    }}
  ],
  "accessors": [
    {{
      "bufferView": 0,
      "byteOffset": 0,
      "componentType": 5126,
      "count": {},
      "type": "VEC3",
      "min": [0.0, -1.0, -1.0],
      "max": [10.0, 1.0, 1.0]
    }},
    {{
      "bufferView": 1,
      "byteOffset": 0,
      "componentType": 5123,
      "count": {},
      "type": "SCALAR"
    }}
  ]
}}"#,
        positions_base64,
        positions_len,
        indices_base64,
        indices_len,
        positions_len,
        indices_len,
        vertex_count,
        indices.len()
    )
}

fn bench_obj_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("obj_parsing");

    for vertex_count in [100, 500, 1000, 5000, 10000] {
        let obj_content = generate_obj_mesh(vertex_count);
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("bench_obj_{vertex_count}.obj"));
        fs::write(&test_file, &obj_content).expect("Failed to write test file");

        group.throughput(Throughput::Elements(vertex_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(vertex_count),
            &test_file,
            |b, path| {
                let loader = MeshLoader::new();
                b.iter(|| {
                    let mesh = loader.load(path).expect("Failed to load OBJ");
                    black_box(mesh);
                });
            },
        );

        fs::remove_file(&test_file).ok();
    }

    group.finish();
}

fn bench_obj_file_io(c: &mut Criterion) {
    let obj_content = generate_obj_mesh(5000);
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bench_obj_io.obj");
    fs::write(&test_file, &obj_content).expect("Failed to write test file");

    c.bench_function("obj_file_read", |b| {
        b.iter(|| {
            let content = fs::read_to_string(&test_file).expect("Failed to read file");
            black_box(content);
        });
    });

    fs::remove_file(&test_file).ok();
}

fn bench_gltf_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("gltf_parsing");

    for vertex_count in [100, 500, 1000, 5000] {
        let gltf_content = generate_gltf_mesh(vertex_count);
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join(format!("bench_gltf_{vertex_count}.gltf"));
        fs::write(&test_file, &gltf_content).expect("Failed to write test file");

        group.throughput(Throughput::Elements(vertex_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(vertex_count),
            &test_file,
            |b, path| {
                let loader = GltfLoader::new();
                b.iter(|| {
                    let asset = loader.load_gltf(path).expect("Failed to load GLTF");
                    black_box(asset);
                });
            },
        );

        fs::remove_file(&test_file).ok();
    }

    group.finish();
}

fn bench_obj_with_normals_and_uvs(c: &mut Criterion) {
    let obj_content = generate_obj_mesh(5000);
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bench_obj_full.obj");
    fs::write(&test_file, &obj_content).expect("Failed to write test file");

    let loader = MeshLoader::new();

    c.bench_function("obj_parse_with_normals_and_uvs", |b| {
        b.iter(|| {
            let mesh = loader.load(&test_file).expect("Failed to load OBJ");
            black_box(mesh);
        });
    });

    fs::remove_file(&test_file).ok();
}

fn bench_obj_positions_only(c: &mut Criterion) {
    let mut obj_content = String::from("# Simple mesh\n");

    // Only positions, no normals or UVs
    for i in 0..5000 {
        let t = i as f32 / 5000.0;
        obj_content.push_str(&format!(
            "v {} {} {}\n",
            t * 10.0,
            (t * std::f32::consts::TAU).sin(),
            (t * std::f32::consts::TAU).cos()
        ));
    }

    for i in (0..5000 - 2).step_by(3) {
        obj_content.push_str(&format!("f {} {} {}\n", i + 1, i + 2, i + 3));
    }

    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("bench_obj_simple.obj");
    fs::write(&test_file, &obj_content).expect("Failed to write test file");

    let loader = MeshLoader::new();

    c.bench_function("obj_parse_positions_only", |b| {
        b.iter(|| {
            let mesh = loader.load(&test_file).expect("Failed to load OBJ");
            black_box(mesh);
        });
    });

    fs::remove_file(&test_file).ok();
}

fn bench_real_asset_loading(c: &mut Criterion) {
    // Benchmark loading the actual cube.obj asset if it exists
    let cube_path = "assets/models/cube.obj";

    if std::path::Path::new(cube_path).exists() {
        let loader = MeshLoader::new();

        c.bench_function("obj_load_real_cube_asset", |b| {
            b.iter(|| {
                let mesh = loader.load(cube_path).expect("Failed to load cube");
                black_box(mesh);
            });
        });
    }
}

criterion_group!(
    benches,
    bench_obj_parsing,
    bench_obj_file_io,
    bench_gltf_parsing,
    bench_obj_with_normals_and_uvs,
    bench_obj_positions_only,
    bench_real_asset_loading
);
criterion_main!(benches);
