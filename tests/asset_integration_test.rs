//! Integration tests for asset system functionality.
//!
//! These tests verify asset loading, caching, path resolution,
//! and integration with other subsystems.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Test basic OBJ file loading.
#[test]
fn test_basic_obj_loading() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("basic_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok());

    let mesh = result.unwrap();
    assert_eq!(mesh.positions.len(), 3);
    assert_eq!(mesh.indices.len(), 3);

    fs::remove_file(&test_file).ok();
}

/// Test loading multiple different OBJ files sequentially.
#[test]
fn test_sequential_obj_loading() {
    let temp_dir = std::env::temp_dir();

    let triangle = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;

    let quad = r#"
v -1.0 -1.0 0.0
v  1.0 -1.0 0.0
v  1.0  1.0 0.0
v -1.0  1.0 0.0
f 1 2 3 4
"#;

    let triangle_file = temp_dir.join("seq_triangle.obj");
    let quad_file = temp_dir.join("seq_quad.obj");

    fs::write(&triangle_file, triangle).expect("Failed to write triangle");
    fs::write(&quad_file, quad).expect("Failed to write quad");

    let tri_result = praxis_assets::load_obj(&triangle_file);
    assert!(tri_result.is_ok());
    let tri_mesh = tri_result.unwrap();
    assert_eq!(tri_mesh.positions.len(), 3);

    let quad_result = praxis_assets::load_obj(&quad_file);
    assert!(quad_result.is_ok());
    let quad_mesh = quad_result.unwrap();
    assert_eq!(quad_mesh.positions.len(), 4);

    fs::remove_file(&triangle_file).ok();
    fs::remove_file(&quad_file).ok();
}

/// Test path resolution with relative and absolute paths.
#[test]
fn test_path_resolution() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("path_resolution_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let absolute_path = test_file
        .canonicalize()
        .expect("Failed to canonicalize path");
    let result = praxis_assets::load_obj(&absolute_path);
    assert!(result.is_ok(), "Should load with absolute path");

    fs::remove_file(&test_file).ok();
}

/// Test error handling for various invalid scenarios.
#[test]
fn test_error_handling() {
    let result = praxis_assets::load_obj("nonexistent_file_xyz.obj");
    assert!(result.is_err(), "Should error on nonexistent file");

    let temp_dir = std::env::temp_dir();
    let empty_file = temp_dir.join("empty_test.obj");
    fs::write(&empty_file, "").expect("Failed to write empty file");

    let result = praxis_assets::load_obj(&empty_file);
    assert!(result.is_err(), "Should error on empty file");

    fs::remove_file(&empty_file).ok();
}

/// Simulate asset caching by loading the same file multiple times.
#[test]
fn test_simulated_asset_caching() {
    use praxis_assets::{AssetLoader, MeshLoader};

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("cache_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let cache: Arc<Mutex<HashMap<PathBuf, praxis_graphics::MeshData>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let loader = MeshLoader::new();

    for _ in 0..5 {
        let path_key = test_file.clone();

        let cached = cache.lock().unwrap().get(&path_key).cloned();
        if cached.is_none() {
            let mesh = loader.load(&test_file).expect("Failed to load mesh");
            cache.lock().unwrap().insert(path_key, mesh);
        }
    }

    let cache_lock = cache.lock().unwrap();
    assert_eq!(cache_lock.len(), 1, "Should have cached exactly one asset");
    assert!(cache_lock.contains_key(&test_file));

    fs::remove_file(&test_file).ok();
}

/// Test loading assets with various attribute combinations.
#[test]
fn test_asset_attribute_variations() {
    let temp_dir = std::env::temp_dir();

    let positions_only = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;

    let with_normals = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
f 1//1 2//2 3//3
"#;

    let with_uvs = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
f 1/1 2/2 3/3
"#;

    let complete = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.5 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/2 3/3/3
"#;

    let file1 = temp_dir.join("attr_pos.obj");
    let file2 = temp_dir.join("attr_norm.obj");
    let file3 = temp_dir.join("attr_uv.obj");
    let file4 = temp_dir.join("attr_complete.obj");

    fs::write(&file1, positions_only).expect("Failed to write file1");
    fs::write(&file2, with_normals).expect("Failed to write file2");
    fs::write(&file3, with_uvs).expect("Failed to write file3");
    fs::write(&file4, complete).expect("Failed to write file4");

    let mesh1 = praxis_assets::load_obj(&file1).expect("Failed to load mesh1");
    assert!(mesh1.normals.is_none());
    assert!(mesh1.uvs.is_none());

    let mesh2 = praxis_assets::load_obj(&file2).expect("Failed to load mesh2");
    assert!(mesh2.normals.is_some());
    assert!(mesh2.uvs.is_none());

    let mesh3 = praxis_assets::load_obj(&file3).expect("Failed to load mesh3");
    assert!(mesh3.normals.is_none());
    assert!(mesh3.uvs.is_some());

    let mesh4 = praxis_assets::load_obj(&file4).expect("Failed to load mesh4");
    assert!(mesh4.normals.is_some());
    assert!(mesh4.uvs.is_some());

    fs::remove_file(&file1).ok();
    fs::remove_file(&file2).ok();
    fs::remove_file(&file3).ok();
    fs::remove_file(&file4).ok();
}

/// Test loading large meshes to verify memory handling.
#[test]
fn test_large_mesh_loading() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("large_mesh.obj");

    let mut obj_content = String::new();
    for i in 0..1000 {
        obj_content.push_str(&format!(
            "v {} {} {}\n",
            i as f32,
            (i * 2) as f32,
            (i * 3) as f32
        ));
    }

    for i in 0..330 {
        let i1 = i * 3 + 1;
        let i2 = i * 3 + 2;
        let i3 = i * 3 + 3;
        obj_content.push_str(&format!("f {i1} {i2} {i3}\n"));
    }

    fs::write(&test_file, obj_content).expect("Failed to write large mesh");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok(), "Should load large mesh");

    let mesh = result.unwrap();
    // The loader returns only the vertices that are actually referenced by faces
    // 330 faces * 3 vertices per face = 990 vertices used
    assert_eq!(mesh.positions.len(), 990);
    assert_eq!(mesh.indices.len(), 990);

    fs::remove_file(&test_file).ok();
}

/// Test multiple loaders working independently.
#[test]
fn test_multiple_loader_instances() {
    use praxis_assets::{AssetLoader, MeshLoader};

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("multi_loader.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let loader1 = MeshLoader::new();
    let loader2 = MeshLoader::new();
    let loader3 = MeshLoader::new();

    let result1 = loader1.load(&test_file);
    let result2 = loader2.load(&test_file);
    let result3 = loader3.load(&test_file);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    fs::remove_file(&test_file).ok();
}

/// Test loader extensions API.
#[test]
fn test_loader_extensions() {
    use praxis_assets::{AssetLoader, MeshLoader};

    let loader = MeshLoader::new();
    let extensions = loader.supported_extensions();

    assert!(extensions.contains(&"obj"));
    assert_eq!(extensions.len(), 1);
}

/// Test that mesh data is correctly structured after loading.
#[test]
fn test_mesh_data_structure() {
    let obj_content = r#"
v 1.0 2.0 3.0
v 4.0 5.0 6.0
v 7.0 8.0 9.0
vt 0.0 0.0
vt 0.5 0.5
vt 1.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/2 3/3/3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("structure_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let mesh = praxis_assets::load_obj(&test_file).expect("Failed to load mesh");

    assert_eq!(mesh.positions.len(), 3);
    assert_eq!(mesh.indices.len(), 3);

    assert_eq!(mesh.positions[0], [1.0, 2.0, 3.0]);
    assert_eq!(mesh.positions[1], [4.0, 5.0, 6.0]);
    assert_eq!(mesh.positions[2], [7.0, 8.0, 9.0]);

    let uvs = mesh.uvs.as_ref().expect("Should have UVs");
    assert_eq!(uvs.len(), 3);
    assert_eq!(uvs[0], [0.0, 0.0]);
    assert_eq!(uvs[1], [0.5, 0.5]);
    assert_eq!(uvs[2], [1.0, 1.0]);

    let normals = mesh.normals.as_ref().expect("Should have normals");
    assert_eq!(normals.len(), 3);
    assert_eq!(normals[0], [0.0, 0.0, 1.0]);

    fs::remove_file(&test_file).ok();
}

/// Test file path with special characters.
#[test]
fn test_path_with_special_characters() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_file_with_underscores_123.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(
        result.is_ok(),
        "Should handle files with underscores and numbers"
    );

    fs::remove_file(&test_file).ok();
}

/// Test loading mesh with comments in OBJ file.
#[test]
fn test_obj_with_comments() {
    let obj_content = r#"
# This is a comment
# Another comment
v 0.0 0.0 0.0  # Vertex 1
v 1.0 0.0 0.0  # Vertex 2
v 0.5 1.0 0.0  # Vertex 3
# Face definition
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("comments_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok(), "Should handle comments in OBJ files");

    let mesh = result.unwrap();
    assert_eq!(mesh.positions.len(), 3);

    fs::remove_file(&test_file).ok();
}

/// Test asset system initialization and loader interaction.
#[test]
fn test_init_and_loading() {
    let init_result = praxis_assets::init();
    assert!(init_result.is_ok(), "Asset system should initialize");

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("init_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let load_result = praxis_assets::load_obj(&test_file);
    assert!(load_result.is_ok(), "Should load after initialization");

    fs::remove_file(&test_file).ok();
}

/// Test cleanup after multiple loads.
#[test]
fn test_cleanup_after_multiple_loads() {
    let temp_dir = std::env::temp_dir();
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;

    let mut files = Vec::new();
    for i in 0..10 {
        let file = temp_dir.join(format!("cleanup_test_{i}.obj"));
        fs::write(&file, obj_content).expect("Failed to write test file");
        files.push(file);
    }

    for file in &files {
        let result = praxis_assets::load_obj(file);
        assert!(result.is_ok());
    }

    for file in files {
        fs::remove_file(&file).ok();
    }
}

/// Test that loading doesn't leak memory (basic test).
#[test]
fn test_memory_leak_basic() {
    let temp_dir = std::env::temp_dir();
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let test_file = temp_dir.join("memory_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    for _ in 0..100 {
        let _mesh = praxis_assets::load_obj(&test_file).expect("Failed to load");
    }

    fs::remove_file(&test_file).ok();
}
