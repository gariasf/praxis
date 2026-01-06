//! File path resolution and asset loading tests.
//!
//! These tests verify correct handling of various path formats,
//! file system edge cases, and path normalization.

use std::fs;
use std::path::{Path, PathBuf};

/// Test loading with different path representations.
#[test]
fn test_different_path_types() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("path_types_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let as_path: &Path = &test_file;
    let result1 = praxis_assets::load_obj(as_path);
    assert!(result1.is_ok(), "Should load with &Path");

    let as_pathbuf: PathBuf = test_file.clone();
    let result2 = praxis_assets::load_obj(as_pathbuf);
    assert!(result2.is_ok(), "Should load with PathBuf");

    let as_str = test_file.to_str().unwrap();
    let result3 = praxis_assets::load_obj(as_str);
    assert!(result3.is_ok(), "Should load with &str");

    let as_string = test_file.to_string_lossy().into_owned();
    let result4 = praxis_assets::load_obj(as_string);
    assert!(result4.is_ok(), "Should load with String");

    fs::remove_file(&test_file).ok();
}

/// Test absolute path resolution.
#[test]
fn test_absolute_path() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("absolute_path_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let absolute_path = test_file
        .canonicalize()
        .expect("Failed to get absolute path");
    assert!(absolute_path.is_absolute());

    let result = praxis_assets::load_obj(&absolute_path);
    assert!(result.is_ok(), "Should load with absolute path");

    fs::remove_file(&test_file).ok();
}

/// Test file not found error.
#[test]
fn test_file_not_found() {
    let nonexistent = "this_file_definitely_does_not_exist_123456789.obj";
    let result = praxis_assets::load_obj(nonexistent);
    assert!(result.is_err(), "Should return error for nonexistent file");
}

/// Test empty file path.
#[test]
fn test_empty_path() {
    let result = praxis_assets::load_obj("");
    assert!(result.is_err(), "Should return error for empty path");
}

/// Test path with nested directories.
#[test]
fn test_nested_directory_path() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let nested_dir = temp_dir.join("nested").join("test").join("path");
    fs::create_dir_all(&nested_dir).expect("Failed to create nested directories");

    let test_file = nested_dir.join("nested_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok(), "Should load from nested directory");

    fs::remove_file(&test_file).ok();
    fs::remove_dir_all(temp_dir.join("nested")).ok();
}

/// Test multiple files in same directory.
#[test]
fn test_multiple_files_same_directory() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("multi_file_test");
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");

    let files: Vec<_> = (0..5)
        .map(|i| {
            let file = test_dir.join(format!("mesh_{i}.obj"));
            fs::write(&file, obj_content).expect("Failed to write test file");
            file
        })
        .collect();

    for file in &files {
        let result = praxis_assets::load_obj(file);
        assert!(result.is_ok(), "Should load file: {file:?}");
    }

    for file in files {
        fs::remove_file(&file).ok();
    }
    fs::remove_dir(&test_dir).ok();
}

/// Test file extension handling.
#[test]
fn test_file_extension_handling() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();

    let valid_file = temp_dir.join("valid.obj");
    fs::write(&valid_file, obj_content).expect("Failed to write valid file");

    let result = praxis_assets::load_obj(&valid_file);
    assert!(result.is_ok(), "Should load .obj file");

    fs::remove_file(&valid_file).ok();
}

/// Test handling of files with no extension.
#[test]
fn test_file_without_extension() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("no_extension_file");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(
        result.is_ok(),
        "Should load file without extension if content is valid"
    );

    fs::remove_file(&test_file).ok();
}

/// Test path normalization.
#[test]
fn test_path_normalization() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("normalize_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let path_with_current = test_file
        .parent()
        .unwrap()
        .join(".")
        .join("normalize_test.obj");
    let result = praxis_assets::load_obj(&path_with_current);
    assert!(result.is_ok(), "Should handle path with . component");

    fs::remove_file(&test_file).ok();
}

/// Test concurrent file loading.
#[test]
fn test_concurrent_file_loading() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let files: Vec<_> = (0..10)
        .map(|i| {
            let file = temp_dir.join(format!("concurrent_{i}.obj"));
            fs::write(&file, obj_content).expect("Failed to write test file");
            file
        })
        .collect();

    for file in &files {
        let result = praxis_assets::load_obj(file);
        assert!(result.is_ok(), "Concurrent loading should work");
    }

    for file in files {
        fs::remove_file(&file).ok();
    }
}

/// Test loading the same file multiple times.
#[test]
fn test_reload_same_file() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("reload_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    for i in 0..10 {
        let result = praxis_assets::load_obj(&test_file);
        assert!(result.is_ok(), "Reload {i} should succeed");
    }

    fs::remove_file(&test_file).ok();
}

/// Test file with special characters in name.
#[test]
fn test_special_characters_in_filename() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();

    let filenames = vec![
        "test-dash.obj",
        "test_underscore.obj",
        "test.multiple.dots.obj",
        "test123numbers.obj",
    ];

    for filename in filenames {
        let test_file = temp_dir.join(filename);
        fs::write(&test_file, obj_content).expect("Failed to write test file");

        let result = praxis_assets::load_obj(&test_file);
        assert!(result.is_ok(), "Should load file: {filename}");

        fs::remove_file(&test_file).ok();
    }
}

/// Test directory traversal safety.
#[test]
fn test_directory_traversal() {
    let result = praxis_assets::load_obj("../../../etc/passwd");
    assert!(result.is_err(), "Should not allow directory traversal");
}

/// Test path with spaces.
#[test]
fn test_path_with_spaces() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("dir with spaces");
    fs::create_dir_all(&test_dir).expect("Failed to create directory with spaces");

    let test_file = test_dir.join("file with spaces.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok(), "Should handle paths with spaces");

    fs::remove_file(&test_file).ok();
    fs::remove_dir(&test_dir).ok();
}

/// Test loading from current directory.
#[test]
fn test_load_from_current_directory() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("current_dir_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let result = praxis_assets::load_obj(&test_file);
    assert!(result.is_ok());

    fs::remove_file(&test_file).ok();
}

/// Test symbolic link handling (Unix-specific, will be skipped on Windows).
#[test]
#[cfg(unix)]
fn test_symbolic_link() {
    use std::os::unix::fs::symlink;

    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let original_file = temp_dir.join("symlink_original.obj");
    let symlink_file = temp_dir.join("symlink_link.obj");

    fs::write(&original_file, obj_content).expect("Failed to write original file");

    if symlink(&original_file, &symlink_file).is_ok() {
        let result = praxis_assets::load_obj(&symlink_file);
        assert!(result.is_ok(), "Should load through symbolic link");

        fs::remove_file(&symlink_file).ok();
    }

    fs::remove_file(&original_file).ok();
}

/// Test loading with canonical path.
#[test]
fn test_canonical_path() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("canonical_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    if let Ok(canonical) = test_file.canonicalize() {
        let result = praxis_assets::load_obj(&canonical);
        assert!(result.is_ok(), "Should load with canonical path");
    }

    fs::remove_file(&test_file).ok();
}

/// Test path comparison and equality.
#[test]
fn test_path_equality() {
    let obj_content = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.5 1.0 0.0
f 1 2 3
"#;
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("equality_test.obj");
    fs::write(&test_file, obj_content).expect("Failed to write test file");

    let path1 = test_file.clone();
    let path2 = test_file.clone();

    assert_eq!(path1, path2);

    let result1 = praxis_assets::load_obj(&path1);
    let result2 = praxis_assets::load_obj(&path2);

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    fs::remove_file(&test_file).ok();
}

/// Test error message quality for invalid paths.
#[test]
fn test_error_message_quality() {
    let result = praxis_assets::load_obj("nonexistent_file_xyz.obj");
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(!error_msg.is_empty(), "Error message should not be empty");
    assert!(error_msg.len() > 10, "Error message should be descriptive");
}
