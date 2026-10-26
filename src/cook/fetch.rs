//! Fetching: the files `assets/manifest.json` marks `fetched`, pinned there
//! by URL and SHA-256 rather than committed (`docs/research/test-assets.md`),
//! downloaded into `assets/`.

use std::path::{Path, PathBuf};

use anyhow::Context;
use sha2::{Digest, Sha256};

use super::{ManifestEntry, read_manifest};
use crate::cooked::hex;

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
    fetch_entries(assets, &read_manifest(assets, false)?, download)
}

/// The manifest's fetched files absent from `assets`, as paths relative to
/// it, and how many fetched files the manifest lists.
pub fn absent_fetched(assets: &Path) -> anyhow::Result<(Vec<String>, usize)> {
    let fetched: Vec<_> = read_manifest(assets, false)?
        .into_iter()
        .filter(|entry| entry.fetched)
        .collect();
    let absent = fetched
        .iter()
        .filter(|entry| !assets.join(&entry.path).exists())
        .map(|entry| entry.path.clone())
        .collect();
    Ok((absent, fetched.len()))
}

/// What a dev launch fetches before it cooks: its scene's fetched files,
/// when one is absent. A failure only warns; the launch then loads what the
/// Cooked cache holds.
pub(super) fn fetch_for_launch(assets: &Path, source_guids: &[u128]) {
    if let Err(error) = fetch_for_launch_with(assets, source_guids, download) {
        tracing::warn!(error = %format_args!("{error:#}"), "fetch failed; loading what the Cooked cache holds");
    }
}

/// `fetch_for_launch`, with `download` reading a URL's bytes. A scene's
/// fetched files are those beside a fetched source it names by GUID, as a
/// glTF sits beside the files it names. Any one absent fetches them all, so
/// the next launch finishes a fetch cut short.
fn fetch_for_launch_with(
    assets: &Path,
    source_guids: &[u128],
    download: impl Fn(&str) -> anyhow::Result<Vec<u8>>,
) -> anyhow::Result<()> {
    // Without a manifest there is nothing to fetch; the cook warns of it.
    let manifest = read_manifest(assets, true)?;
    let mut directories = Vec::new();
    for entry in manifest.iter().filter(|entry| entry.fetched) {
        if source_guids.contains(&entry.guid()?) {
            directories.push(directory_of(&entry.path));
        }
    }
    let scene_files: Vec<_> = manifest
        .iter()
        .filter(|entry| entry.fetched && directories.contains(&directory_of(&entry.path)))
        .collect();
    if scene_files
        .iter()
        .all(|entry| assets.join(&entry.path).exists())
    {
        return Ok(());
    }
    tracing::info!("fetching the scene's absent sources");
    fetch_entries(assets, scene_files, download)?;
    Ok(())
}

/// The `/`-separated directory of a manifest path, empty at the root.
fn directory_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(directory, _)| directory)
}

/// `fetch` of `entries`, with `download` reading a URL's bytes. A file is
/// written beside its place and renamed into it, so an interrupted fetch
/// leaves no partial file where the cook reads.
fn fetch_entries<'manifest>(
    assets: &Path,
    entries: impl IntoIterator<Item = &'manifest ManifestEntry>,
    download: impl Fn(&str) -> anyhow::Result<Vec<u8>>,
) -> anyhow::Result<Vec<(String, Fetch)>> {
    let mut fetches = Vec::new();
    for entry in entries.into_iter().filter(|entry| entry.fetched) {
        let path = assets.join(&entry.path);
        if std::fs::read(&path).is_ok_and(|contents| sha256_hex(&contents) == entry.sha256) {
            fetches.push((entry.path.clone(), Fetch::Kept));
            continue;
        }
        tracing::info!(path = %entry.path, source = %entry.source, "downloading");
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
        let mut partial = path.clone().into_os_string();
        partial.push(".fetching");
        let partial = PathBuf::from(partial);
        std::fs::write(&partial, &contents)
            .with_context(|| format!("writing {}", partial.display()))?;
        std::fs::rename(&partial, &path)
            .with_context(|| format!("renaming {} to {}", partial.display(), path.display()))?;
        fetches.push((entry.path.clone(), Fetch::Fetched));
    }
    Ok(fetches)
}

/// The bytes at `url`, read by `curl`, which Windows 10 and later, macOS and
/// CI's Ubuntu ship: redirects followed, an HTTP error a failure, and a
/// connection that takes over 30 s to open, or moves under 1 kB/s for 60 s,
/// given up.
fn download(url: &str) -> anyhow::Result<Vec<u8>> {
    let output = std::process::Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--connect-timeout",
            "30",
            "--speed-limit",
            "1000",
            "--speed-time",
            "60",
            url,
        ])
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
    use std::cell::RefCell;

    use super::*;

    /// An empty assets directory of the test's own, removed when dropped.
    struct Assets(PathBuf);

    impl Assets {
        fn new(test: &str) -> Self {
            let directory =
                std::env::temp_dir().join(format!("praxis-fetch-{test}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&directory);
            std::fs::create_dir_all(&directory).unwrap();
            Self(directory)
        }
    }

    impl std::ops::Deref for Assets {
        type Target = Path;

        fn deref(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Assets {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const URL: &str = "https://example.invalid/";

    /// A manifest entry for `path`, pinned to contents that are the path's
    /// own bytes, at a URL naming the path.
    fn entry(path: &str, guid: u128, fetched: bool) -> ManifestEntry {
        ManifestEntry {
            path: path.to_owned(),
            guid: format!("{guid:032x}"),
            fetched,
            source: format!("{URL}{path}"),
            sha256: sha256_hex(path.as_bytes()),
        }
    }

    /// The pinned contents of the file at `url`.
    fn served(url: &str) -> anyhow::Result<Vec<u8>> {
        Ok(url.trim_start_matches(URL).as_bytes().to_vec())
    }

    fn refused(url: &str) -> anyhow::Result<Vec<u8>> {
        panic!("{url} downloaded")
    }

    #[test]
    fn a_file_with_its_pinned_sha256_is_kept_and_any_other_fetched() {
        let assets = Assets::new("kept-and-fetched");
        std::fs::create_dir_all(assets.join("model")).unwrap();
        std::fs::write(assets.join("model/kept.bin"), "model/kept.bin").unwrap();
        std::fs::write(assets.join("model/changed.bin"), "old").unwrap();
        let manifest = [
            entry("model/kept.bin", 1, true),
            entry("model/changed.bin", 2, true),
            entry("model/absent.bin", 3, true),
            entry("committed.bin", 4, false),
        ];
        let downloaded = RefCell::new(Vec::new());

        let fetches = fetch_entries(&assets, &manifest, |url| {
            downloaded.borrow_mut().push(url.to_owned());
            served(url)
        })
        .unwrap();

        assert_eq!(
            fetches,
            [
                ("model/kept.bin".to_owned(), Fetch::Kept),
                ("model/changed.bin".to_owned(), Fetch::Fetched),
                ("model/absent.bin".to_owned(), Fetch::Fetched),
            ]
        );
        assert_eq!(
            downloaded.into_inner(),
            [
                format!("{URL}model/changed.bin"),
                format!("{URL}model/absent.bin")
            ]
        );
        for path in ["model/changed.bin", "model/absent.bin"] {
            assert_eq!(std::fs::read(assets.join(path)).unwrap(), path.as_bytes());
        }
        assert!(!assets.join("committed.bin").exists());
    }

    #[test]
    fn a_download_unlike_its_pinned_sha256_is_refused_and_not_written() {
        let assets = Assets::new("refused");
        let manifest = [entry("model/pinned.bin", 1, true)];

        let error = fetch_entries(&assets, &manifest, |_| Ok(b"other".to_vec())).unwrap_err();

        assert_eq!(
            format!("{error:#}"),
            format!(
                "model/pinned.bin: sha256 is {}, manifest says {}",
                sha256_hex(b"other"),
                sha256_hex(b"model/pinned.bin")
            )
        );
        assert!(!assets.join("model").exists());
    }

    #[test]
    fn a_launch_fetches_its_scenes_files_while_one_is_absent_and_no_others() {
        let assets = Assets::new("launch");
        let manifest: Vec<_> = [
            entry("helmet/helmet.gltf", 1, true),
            entry("helmet/helmet.bin", 2, true),
            entry("other/other.gltf", 3, true),
            entry("committed.glb", 4, false),
        ]
        .iter()
        .map(|entry| {
            serde_json::json!({
                "path": entry.path,
                "guid": entry.guid,
                "fetched": entry.fetched,
                "source": entry.source,
                "sha256": entry.sha256,
            })
        })
        .collect();
        std::fs::write(
            assets.join("manifest.json"),
            serde_json::Value::Array(manifest).to_string(),
        )
        .unwrap();

        // A procedural scene names no source, and a committed one is no
        // fetched file.
        fetch_for_launch_with(&assets, &[], refused).unwrap();
        fetch_for_launch_with(&assets, &[4], refused).unwrap();

        // A fetch cut short after the glTF...
        let cut_short = |url: &str| -> anyhow::Result<Vec<u8>> {
            if url.ends_with(".bin") {
                anyhow::bail!("connection reset");
            }
            served(url)
        };
        assert!(fetch_for_launch_with(&assets, &[1], cut_short).is_err());
        assert!(assets.join("helmet/helmet.gltf").exists());

        // ...is finished by the next launch, which downloads only what the
        // helmet still lacks.
        let downloaded = RefCell::new(Vec::new());
        fetch_for_launch_with(&assets, &[1], |url| {
            downloaded.borrow_mut().push(url.to_owned());
            served(url)
        })
        .unwrap();
        assert_eq!(downloaded.into_inner(), [format!("{URL}helmet/helmet.bin")]);

        // Once they are all present, a launch downloads nothing.
        fetch_for_launch_with(&assets, &[1], refused).unwrap();
        assert!(!assets.join("other").exists());
    }
}
