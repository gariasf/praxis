//! Fetching: the files `assets/manifest.json` marks `fetched`, pinned there
//! by URL and SHA-256 rather than committed (`docs/research/test-assets.md`),
//! downloaded into `assets/`.

use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::cooked::hex;

/// The fields of an `assets/manifest.json` entry fetching reads.
#[derive(Deserialize)]
struct ManifestEntry {
    path: String,
    guid: String,
    source: String,
    sha256: String,
    fetched: bool,
}

/// What fetching did with one fetched file.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Fetch {
    /// Present with the manifest's SHA-256, so kept.
    Kept,
    /// Downloaded: absent, or different, before.
    Fetched,
}

/// Fetches each fetched file of the manifest in `assets` that is absent or
/// differs from its SHA-256, from its pinned URL, and returns each fetched
/// file's path, relative to `assets`, with what was done with it. A download
/// whose SHA-256 differs from the manifest's fails, and is not written.
pub fn fetch(assets: &Path) -> anyhow::Result<Vec<(String, Fetch)>> {
    fetch_entries(assets, &read_manifest(assets)?, download)
}

/// What a dev launch fetches before it cooks: when one of its sources, by
/// GUID, is a fetched file that is absent, every fetched file not yet right,
/// so the files a glTF names arrive with it. A failure only warns, and the
/// launch loads what the Cooked cache holds, as it would have.
pub(super) fn fetch_for_launch(assets: &Path, source_guids: &[u128]) {
    // Without a manifest there is nothing to fetch; the cook warns of it.
    let Ok(manifest) = read_manifest(assets) else {
        return;
    };
    if !lacks_fetched_source(&manifest, source_guids, |path| assets.join(path).exists()) {
        return;
    }
    tracing::info!("fetching the scene's absent sources");
    if let Err(error) = fetch_entries(assets, &manifest, download) {
        tracing::warn!(error = %format_args!("{error:#}"), "fetch failed; loading what the Cooked cache holds");
    }
}

/// Whether one of `source_guids` names a fetched file that `exists` does not
/// find.
fn lacks_fetched_source(
    manifest: &[ManifestEntry],
    source_guids: &[u128],
    exists: impl Fn(&str) -> bool,
) -> bool {
    manifest.iter().any(|entry| {
        entry.fetched
            && u128::from_str_radix(&entry.guid, 16).is_ok_and(|guid| source_guids.contains(&guid))
            && !exists(&entry.path)
    })
}

/// `fetch`, with `download` reading a URL's bytes.
fn fetch_entries(
    assets: &Path,
    manifest: &[ManifestEntry],
    download: impl Fn(&str) -> anyhow::Result<Vec<u8>>,
) -> anyhow::Result<Vec<(String, Fetch)>> {
    let mut fetches = Vec::new();
    for entry in manifest.iter().filter(|entry| entry.fetched) {
        let path = assets.join(&entry.path);
        if std::fs::read(&path).is_ok_and(|contents| sha256_hex(&contents) == entry.sha256) {
            fetches.push((entry.path.clone(), Fetch::Kept));
            continue;
        }
        let contents =
            download(&entry.source).with_context(|| format!("downloading {}", entry.path))?;
        let actual = sha256_hex(&contents);
        anyhow::ensure!(
            actual == entry.sha256,
            "{}: sha256 is {actual}, manifest says {}",
            entry.path,
            entry.sha256
        );
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)
                .with_context(|| format!("creating {}", directory.display()))?;
        }
        std::fs::write(&path, &contents).with_context(|| format!("writing {}", path.display()))?;
        tracing::info!(path = %entry.path, bytes = contents.len(), "fetched");
        fetches.push((entry.path.clone(), Fetch::Fetched));
    }
    Ok(fetches)
}

fn read_manifest(assets: &Path) -> anyhow::Result<Vec<ManifestEntry>> {
    let path = assets.join("manifest.json");
    let manifest = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&manifest).with_context(|| format!("parsing {}", path.display()))
}

/// The bytes at `url`, read by `curl`, which Windows 10 and later, macOS and
/// CI's Ubuntu ship: redirects followed, an HTTP error a failure.
fn download(url: &str) -> anyhow::Result<Vec<u8>> {
    let output = std::process::Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--location", url])
        .output()
        .context("running curl")?;
    anyhow::ensure!(
        output.status.success(),
        "curl {}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(output.stdout)
}

fn sha256_hex(contents: &[u8]) -> String {
    hex(&Sha256::digest(contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty assets directory of the test's own.
    fn assets_directory(test: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("praxis-fetch-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        directory
    }

    /// A manifest entry for `path`, pinned to `contents`, at a URL naming the
    /// path.
    fn entry(path: &str, guid: u128, contents: &[u8], fetched: bool) -> ManifestEntry {
        ManifestEntry {
            path: path.to_owned(),
            guid: format!("{guid:032x}"),
            source: format!("https://example.invalid/{path}"),
            sha256: sha256_hex(contents),
            fetched,
        }
    }

    #[test]
    fn a_file_with_its_pinned_sha256_is_kept_and_any_other_fetched() {
        let assets = assets_directory("kept-and-fetched");
        std::fs::create_dir_all(assets.join("model")).unwrap();
        std::fs::write(assets.join("model/kept.bin"), b"kept").unwrap();
        std::fs::write(assets.join("model/changed.bin"), b"old").unwrap();
        let manifest = [
            entry("model/kept.bin", 1, b"kept", true),
            entry("model/changed.bin", 2, b"new", true),
            entry("model/absent.bin", 3, b"absent", true),
            entry("committed.bin", 4, b"committed", false),
        ];
        let served = |url: &str| match url.trim_start_matches("https://example.invalid/") {
            "model/changed.bin" => Ok(b"new".to_vec()),
            "model/absent.bin" => Ok(b"absent".to_vec()),
            path => panic!("{path} downloaded"),
        };

        let fetches = fetch_entries(&assets, &manifest, served).unwrap();

        assert_eq!(
            fetches,
            [
                ("model/kept.bin".to_owned(), Fetch::Kept),
                ("model/changed.bin".to_owned(), Fetch::Fetched),
                ("model/absent.bin".to_owned(), Fetch::Fetched),
            ]
        );
        assert_eq!(
            std::fs::read(assets.join("model/changed.bin")).unwrap(),
            b"new"
        );
        assert_eq!(
            std::fs::read(assets.join("model/absent.bin")).unwrap(),
            b"absent"
        );
        assert!(!assets.join("committed.bin").exists());
    }

    #[test]
    fn a_download_unlike_its_pinned_sha256_is_refused_and_not_written() {
        let assets = assets_directory("refused");
        let manifest = [entry("model/pinned.bin", 1, b"pinned", true)];

        let error = fetch_entries(&assets, &manifest, |_| Ok(b"other".to_vec())).unwrap_err();

        assert_eq!(
            format!("{error:#}"),
            format!(
                "model/pinned.bin: sha256 is {}, manifest says {}",
                sha256_hex(b"other"),
                sha256_hex(b"pinned")
            )
        );
        assert!(!assets.join("model").exists());
    }

    #[test]
    fn a_launch_fetches_only_for_an_absent_fetched_source_of_its_own() {
        let manifest = [
            entry("helmet/helmet.gltf", 1, b"helmet", true),
            entry("helmet/helmet.bin", 2, b"buffer", true),
            entry("committed.glb", 3, b"committed", false),
        ];
        let present = |_: &str| true;
        let absent = |_: &str| false;

        assert!(lacks_fetched_source(&manifest, &[1], absent));
        assert!(!lacks_fetched_source(&manifest, &[1], present));
        // A procedural scene names no source, and a committed one is no
        // fetched file, so neither fetches even when absent.
        assert!(!lacks_fetched_source(&manifest, &[], absent));
        assert!(!lacks_fetched_source(&manifest, &[3], absent));
    }
}
