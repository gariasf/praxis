use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const MANIFEST: &str = "manifest.json";
const ATTRIBUTION: &str = "ATTRIBUTION.md";

/// One asset file. Policy: docs/research/test-assets.md.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    /// Relative to the assets directory, `/`-separated.
    path: String,
    /// 128 bits in lowercase hex, written once: Asset IDs derive from it,
    /// so it survives moving and renaming the file (ADR 0018).
    guid: Option<String>,
    /// Pinned download URL.
    source: String,
    /// SPDX licence expression.
    license: String,
    /// Repeated verbatim in the attribution file.
    credit: String,
    /// Upstream commit, release or date.
    version: String,
    sha256: String,
    /// Kept out of shipped builds: non-commercial or otherwise restricted.
    dev_only: bool,
    /// Kept out of git and downloaded by pinned URL, so it may be absent.
    fetched: bool,
}

#[test]
fn every_asset_is_listed_licensed_and_credited() {
    let problems = manifest_problems(&Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"));
    assert!(
        problems.is_empty(),
        "assets/ does not match {MANIFEST} and {ATTRIBUTION}:\n{}",
        problems.join("\n")
    );
}

#[test]
fn manifest_check_reports_every_kind_of_problem() {
    // One directory per process: concurrent runs can share a target directory.
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("asset_manifest-{}", std::process::id()));
    if directory.exists() {
        fs::remove_dir_all(&directory).expect("clear the scratch directory");
    }
    let files: [(&str, &[u8]); 3] = [
        ("good.bin", b"abc"),
        ("nested/changed.bin", b""),
        ("unlisted.bin", b""),
    ];
    for (path, contents) in files {
        let path = directory.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    let manifest = json!([
        entry("good.bin", json!({})),
        entry("nested/changed.bin", json!({})),
        entry("absent.bin", json!({})),
        entry("fetched.bin", json!({"fetched": true})),
        entry(
            "non-commercial.bin",
            json!({"fetched": true, "license": "CC-BY-4.0 AND cc-by-nc-4.0"}),
        ),
        entry(
            "uncredited.bin",
            json!({"fetched": true, "credit": "Credited"})
        ),
        entry("blank.bin", json!({"fetched": true, "version": " "})),
        entry("unpinned.bin", json!({"fetched": true, "sha256": ""})),
        entry("good.bin", json!({})),
        entry("no-guid.bin", json!({"fetched": true, "guid": null})),
        entry(
            "short-guid.bin",
            json!({"fetched": true, "guid": "0123abcd"})
        ),
        entry(
            "same-guid.bin",
            json!({"fetched": true, "guid": guid_for("good.bin")})
        ),
    ]);
    fs::write(directory.join(MANIFEST), manifest.to_string()).unwrap();
    fs::write(directory.join(ATTRIBUTION), "Credited author\n").unwrap();

    assert_eq!(
        manifest_problems(&directory),
        [
            format!("nested/changed.bin: sha256 is {EMPTY_SHA256}, manifest says {ABC_SHA256}"),
            "absent.bin: listed but not in the directory".to_string(),
            "non-commercial.bin: non-commercial licence without dev_only".to_string(),
            "uncredited.bin: credit missing from ATTRIBUTION.md".to_string(),
            "blank.bin: empty version".to_string(),
            "unpinned.bin: sha256 is not 64 lowercase hex digits".to_string(),
            "good.bin: listed twice".to_string(),
            "no-guid.bin: no guid".to_string(),
            "short-guid.bin: guid is not 32 lowercase hex digits".to_string(),
            "same-guid.bin: guid also names good.bin".to_string(),
            "unlisted.bin: no manifest entry".to_string(),
        ]
    );
    fs::remove_dir_all(&directory).expect("remove the scratch directory");
}

/// SHA-256 of "abc" (FIPS 180-2, appendix B.1) and of the empty message
/// (NIST CAVP SHA256ShortMsg, Len = 0).
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

/// A committed, credited, CC0 entry for a file holding "abc", with
/// `changes` applied.
fn entry(path: &str, changes: Value) -> Value {
    let mut entry = json!({
        "path": path,
        "guid": guid_for(path),
        "source": "https://example.com/asset.bin",
        "license": "CC0-1.0",
        "credit": "Credited author",
        "version": "1",
        "sha256": ABC_SHA256,
        "dev_only": false,
        "fetched": false,
    });
    for (field, value) in changes.as_object().expect("changes are an object") {
        entry[field] = value.clone();
    }
    entry
}

/// A GUID unique to `path`.
fn guid_for(path: &str) -> String {
    sha256_hex_of(path.as_bytes())[..32].to_string()
}

/// Each way `assets` disagrees with its manifest and attribution file, as
/// `path: problem`; empty when they agree.
fn manifest_problems(assets: &Path) -> Vec<String> {
    let mut on_disk = BTreeSet::new();
    files_under(assets, assets, &mut on_disk);
    on_disk.remove(MANIFEST);
    on_disk.remove(ATTRIBUTION);

    let manifest = fs::read_to_string(assets.join(MANIFEST)).expect("read the manifest");
    let entries: Vec<Entry> =
        serde_json::from_str(&manifest).unwrap_or_else(|error| panic!("{MANIFEST}: {error}"));
    let attribution =
        fs::read_to_string(assets.join(ATTRIBUTION)).expect("read the attribution file");

    let mut problems = Vec::new();
    let mut listed = BTreeSet::new();
    let mut guids = BTreeMap::new();
    for entry in &entries {
        let path = &entry.path;
        if !listed.insert(path.as_str()) {
            problems.push(format!("{path}: listed twice"));
            continue;
        }
        for (field, value) in [
            ("path", &entry.path),
            ("source", &entry.source),
            ("license", &entry.license),
            ("credit", &entry.credit),
            ("version", &entry.version),
        ] {
            if value.trim().is_empty() {
                problems.push(format!("{path}: empty {field}"));
            }
        }
        match &entry.guid {
            None => problems.push(format!("{path}: no guid")),
            Some(guid) if !is_lowercase_hex(guid, 32) => {
                problems.push(format!("{path}: guid is not 32 lowercase hex digits"))
            }
            Some(guid) => {
                if let Some(first) = guids.insert(guid.as_str(), path.as_str()) {
                    problems.push(format!("{path}: guid also names {first}"));
                }
            }
        }
        if !is_lowercase_hex(&entry.sha256, 64) {
            problems.push(format!("{path}: sha256 is not 64 lowercase hex digits"));
        }
        // SPDX licence identifiers match case-insensitively.
        if entry.license.to_ascii_uppercase().contains("-NC-") && !entry.dev_only {
            problems.push(format!("{path}: non-commercial licence without dev_only"));
        }
        if !attribution.lines().any(|line| line == entry.credit) {
            problems.push(format!("{path}: credit missing from {ATTRIBUTION}"));
        }
        if on_disk.contains(path) {
            let actual = sha256_hex(&assets.join(path));
            if actual != entry.sha256 {
                problems.push(format!(
                    "{path}: sha256 is {actual}, manifest says {}",
                    entry.sha256
                ));
            }
        } else if !entry.fetched {
            problems.push(format!("{path}: listed but not in the directory"));
        }
    }
    problems.extend(
        on_disk
            .iter()
            .filter(|path| !listed.contains(path.as_str()))
            .map(|path| format!("{path}: no manifest entry")),
    );
    problems
}

fn is_lowercase_hex(text: &str, digits: usize) -> bool {
    text.len() == digits
        && text
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn sha256_hex(path: &Path) -> String {
    let contents =
        fs::read(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    sha256_hex_of(&contents)
}

fn sha256_hex_of(contents: &[u8]) -> String {
    Sha256::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Every file under `directory`, as `/`-separated paths relative to `root`.
fn files_under(root: &Path, directory: &Path, files: &mut BTreeSet<String>) {
    for directory_entry in fs::read_dir(directory).expect("read the assets directory") {
        let directory_entry = directory_entry.expect("read an assets directory entry");
        let path = directory_entry.path();
        if directory_entry
            .file_type()
            .expect("read a file type")
            .is_dir()
        {
            files_under(root, &path, files);
        } else {
            let components: Vec<_> = path
                .strip_prefix(root)
                .expect("a path under the assets root")
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect();
            files.insert(components.join("/"));
        }
    }
}
