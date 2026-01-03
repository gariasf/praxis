//! Asset loader trait and implementation tests.
//!
//! These tests verify the AssetLoader trait implementation,
//! generic loading patterns, and loader extensibility.

use praxis_assets::{AssetLoader, MeshLoader};
use praxis_graphics::MeshData;
use std::fs;

/// Test that MeshLoader implements AssetLoader trait.
#[test]
fn test_mesh_loader_implements_trait() {
    fn assert_implements_asset_loader<T: AssetLoader<MeshData>>(_loader: T) {}

    let loader = MeshLoader::new();
    assert_implements_asset_loader(loader);
}

/// Test AssetLoader trait with generic functions.
#[test]
fn test_generic_asset_loading() {
    fn load_with_trait<L: AssetLoader<MeshData>>(
        loader: &L,
        path: &str,
    ) -> praxis_utils::Result<MeshData> {
        loader.load(path)
    }

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("generic_trait_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let loader = MeshLoader::new();
    let result = load_with_trait(&loader, test_file.to_str().unwrap());

    assert!(result.is_ok());
    fs::remove_file(&test_file).ok();
}

/// Test supported extensions method.
#[test]
fn test_supported_extensions() {
    let loader = MeshLoader::new();
    let extensions = loader.supported_extensions();

    assert!(!extensions.is_empty());
    assert!(extensions.contains(&"obj"));
    assert_eq!(extensions.len(), 1);
}

/// Test that loader can be used through Box.
#[test]
fn test_boxed_loader() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("boxed_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let loader = Box::new(MeshLoader::new());
    let result = loader.load(&test_file);

    assert!(result.is_ok());
    fs::remove_file(&test_file).ok();
}

/// Test loader creation methods.
#[test]
fn test_loader_creation() {
    let loader1 = MeshLoader::new();
    let loader2 = MeshLoader::default();

    assert_eq!(
        loader1.supported_extensions(),
        loader2.supported_extensions()
    );
}

/// Test that multiple loaders work independently.
#[test]
fn test_multiple_independent_loaders() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("multi_loader_test.obj");
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

/// Test loader with reference counting.
#[test]
fn test_arc_loader() {
    use std::sync::Arc;

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("arc_loader_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let loader = Arc::new(MeshLoader::new());
    let loader_clone = Arc::clone(&loader);

    let result1 = loader.load(&test_file);
    let result2 = loader_clone.load(&test_file);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    fs::remove_file(&test_file).ok();
}

/// Test generic function accepting any loader.
#[test]
fn test_generic_loader_function() {
    fn process_with_loader<L>(loader: L, path: &str) -> praxis_utils::Result<usize>
    where
        L: AssetLoader<MeshData>,
    {
        let mesh = loader.load(path)?;
        Ok(mesh.positions.len())
    }

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("generic_func_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let loader = MeshLoader::new();
    let result = process_with_loader(loader, test_file.to_str().unwrap());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 3);

    fs::remove_file(&test_file).ok();
}

/// Test that loader is Send.
#[test]
fn test_loader_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<MeshLoader>();
}

/// Test that loader is Sync.
#[test]
fn test_loader_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<MeshLoader>();
}

/// Test loader with different error scenarios.
#[test]
fn test_loader_error_handling() {
    let loader = MeshLoader::new();

    let result1 = loader.load("nonexistent.obj");
    assert!(result1.is_err());

    let temp_dir = std::env::temp_dir();
    let invalid_file = temp_dir.join("invalid.obj");
    fs::write(&invalid_file, "invalid content").expect("Failed to write file");

    let result2 = loader.load(&invalid_file);
    assert!(result2.is_err());

    fs::remove_file(&invalid_file).ok();
}

/// Test that extension check is case-insensitive concept.
#[test]
fn test_extension_verification() {
    let loader = MeshLoader::new();
    let extensions = loader.supported_extensions();

    assert!(extensions.iter().any(|&ext| ext == "obj"));
}

/// Test loader reusability.
#[test]
fn test_loader_reusability() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();

    let loader = MeshLoader::new();

    for i in 0..10 {
        let test_file = temp_dir.join(format!("reuse_{}.obj", i));
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = loader.load(&test_file);
        assert!(result.is_ok(), "Iteration {} should succeed", i);

        fs::remove_file(&test_file).ok();
    }
}

/// Test loader state independence.
#[test]
fn test_loader_state_independence() {
    let obj1 = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;

    let obj2 = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
f 1 2 3 4
"#;

    let temp_dir = std::env::temp_dir();
    let file1 = temp_dir.join("state1.obj");
    let file2 = temp_dir.join("state2.obj");

    fs::write(&file1, obj1).expect("Failed to write file1");
    fs::write(&file2, obj2).expect("Failed to write file2");

    let loader = MeshLoader::new();

    let mesh1 = loader.load(&file1).expect("Failed to load mesh1");
    let mesh2 = loader.load(&file2).expect("Failed to load mesh2");

    assert_eq!(mesh1.positions.len(), 3);
    assert_eq!(mesh2.positions.len(), 4);

    fs::remove_file(&file1).ok();
    fs::remove_file(&file2).ok();
}

/// Test loader with collection of paths.
#[test]
fn test_loader_with_path_collection() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();

    let paths: Vec<_> = (0..5)
        .map(|i| {
            let path = temp_dir.join(format!("collection_{}.obj", i));
            fs::write(&path, obj_content).expect("Failed to write file");
            path
        })
        .collect();

    let loader = MeshLoader::new();
    let mut meshes = Vec::new();

    for path in &paths {
        let mesh = loader.load(path).expect("Failed to load mesh");
        meshes.push(mesh);
    }

    assert_eq!(meshes.len(), 5);
    for mesh in &meshes {
        assert_eq!(mesh.positions.len(), 3);
    }

    for path in paths {
        fs::remove_file(&path).ok();
    }
}

/// Test that loader correctly validates OBJ content.
#[test]
fn test_loader_validates_content() {
    let temp_dir = std::env::temp_dir();
    let loader = MeshLoader::new();

    let valid_obj = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let valid_file = temp_dir.join("valid_content.obj");
    fs::write(&valid_file, valid_obj).expect("Failed to write valid file");

    let result = loader.load(&valid_file);
    assert!(result.is_ok(), "Valid OBJ should load");

    fs::remove_file(&valid_file).ok();

    let empty_file = temp_dir.join("empty_content.obj");
    fs::write(&empty_file, "").expect("Failed to write empty file");

    let result = loader.load(&empty_file);
    assert!(result.is_err(), "Empty file should fail");

    fs::remove_file(&empty_file).ok();
}

/// Test loader with AsRef<Path> trait bound.
#[test]
fn test_loader_asref_path() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("asref_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let loader = MeshLoader::new();

    let path_buf = test_file.clone();
    let result1 = loader.load(path_buf);
    assert!(result1.is_ok());

    let path_ref: &std::path::Path = &test_file;
    let result2 = loader.load(path_ref);
    assert!(result2.is_ok());

    let string_path = test_file.to_string_lossy().to_string();
    let result3 = loader.load(string_path);
    assert!(result3.is_ok());

    fs::remove_file(&test_file).ok();
}

/// Test that loader methods don't panic.
#[test]
fn test_loader_no_panic() {
    let loader = MeshLoader::new();

    let result = std::panic::catch_unwind(|| {
        let _ = loader.load("nonexistent.obj");
    });
    assert!(result.is_ok(), "Loader should not panic on error");

    let result = std::panic::catch_unwind(|| {
        let _ = loader.supported_extensions();
    });
    assert!(result.is_ok(), "Extensions method should not panic");
}

/// Test loader clone semantics (via creating new instances).
#[test]
fn test_loader_cloning_pattern() {
    let loader1 = MeshLoader::new();
    let loader2 = MeshLoader::new();

    assert_eq!(
        loader1.supported_extensions(),
        loader2.supported_extensions()
    );
}
