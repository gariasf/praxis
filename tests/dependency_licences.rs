use std::process::Command;

use serde_json::{Value, json};

/// Every licence expression in the dependency graph, verbatim. A new
/// expression fails the test until a human reads it and adds it here; only
/// permissive licences, or expressions that offer one, belong on this list.
const ALLOWED: &[&str] = &[
    "(MIT OR Apache-2.0) AND Unicode-3.0",
    "0BSD OR MIT OR Apache-2.0",
    "Apache-2.0",
    "Apache-2.0 AND MIT",
    "Apache-2.0 OR MIT",
    "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
    "Apache-2.0/MIT",
    "BSD-2-Clause",
    "BSD-2-Clause OR Apache-2.0 OR MIT",
    "BSD-3-Clause",
    "BSD-3-Clause OR Apache-2.0",
    "BSD-3-Clause OR MIT OR Apache-2.0",
    "ISC",
    "MIT",
    "MIT OR Apache-2.0",
    "MIT OR Apache-2.0 OR LGPL-2.1-or-later",
    "MIT OR Apache-2.0 OR Zlib",
    "MIT OR Zlib OR Apache-2.0",
    "MIT/Apache-2.0",
    "Unlicense OR MIT",
    "Unlicense/MIT",
    "Zlib",
    "Zlib AND (MIT OR Apache-2.0)",
    "Zlib OR Apache-2.0 OR MIT",
];

#[test]
fn every_dependency_has_an_allowed_licence() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("parse cargo metadata");
    let refused = refused_licences(&metadata);
    assert!(
        refused.is_empty(),
        "licences outside the allowlist in tests/dependency_licences.rs:\n{}",
        refused.join("\n")
    );
}

#[test]
fn licence_check_refuses_unlisted_and_missing_licences() {
    let metadata = json!({"packages": [
        {"name": "allowed", "version": "1.0.0", "license": "MIT OR Apache-2.0"},
        {"name": "copyleft", "version": "2.0.0", "license": "GPL-3.0-only"},
        {"name": "unlicensed", "version": "3.0.0", "license": null},
    ]});
    assert_eq!(
        refused_licences(&metadata),
        [
            "copyleft 2.0.0: GPL-3.0-only",
            "unlicensed 3.0.0: no licence field"
        ]
    );
}

/// One line per package whose licence expression is not on the allowlist.
fn refused_licences(metadata: &Value) -> Vec<String> {
    metadata["packages"]
        .as_array()
        .expect("packages array")
        .iter()
        .filter_map(|package| match package["license"].as_str() {
            Some(licence) if ALLOWED.contains(&licence) => None,
            licence => Some(format!(
                "{} {}: {}",
                package["name"].as_str().unwrap_or("?"),
                package["version"].as_str().unwrap_or("?"),
                licence.unwrap_or("no licence field"),
            )),
        })
        .collect()
}
