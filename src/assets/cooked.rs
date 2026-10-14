//! The Cooked cache as the runtime reads it (ADR 0018): Asset IDs, Blobs
//! named by the SHA-256 of their bytes, and a cook index from Asset ID to
//! Blob and Cook key, one file per Asset so concurrent cooks never lose an
//! entry.

use std::fmt;
use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use sha2::{Digest, Sha256};

pub use super::MaterialChannel;

/// Starts every Material Blob, followed by `MATERIAL_FORMAT_VERSION`.
pub(crate) const MATERIAL_MAGIC: [u8; 4] = *b"PXMA";
/// Bumped whenever the Material Blob layout, or how the cooker fills it,
/// changes: it stands in for the cooker version in the Cook key, and old
/// Blobs are refused.
pub(crate) const MATERIAL_FORMAT_VERSION: u32 = 1;

/// What an Asset is. Its number, and a Texture's Channel, are part of every
/// Asset ID: never renumber.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AssetKind {
    Mesh,
    Material,
    Texture(MaterialChannel),
}

/// Where an Asset comes from.
#[derive(Copy, Clone, Debug)]
pub enum AssetSource<'a> {
    /// A manifest source file, by its GUID.
    File(u128),
    /// A procedural Reference scene, by its name.
    Scene(&'a str),
}

/// An Asset's name inside its source.
#[derive(Copy, Clone, Debug)]
pub enum AssetName<'a> {
    Name(&'a str),
    /// The glTF index, when the name is missing or not unique.
    Index(usize),
}

/// The stable name of an Asset.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetId(pub u128);

impl AssetId {
    /// The first 128 bits of SHA-256 over the source, the kind, the name
    /// and a Texture's Channel, in ADR 0018's order. Each part is tagged and each string length-prefixed, so no two
    /// inputs share an encoding.
    pub fn new(source: AssetSource, kind: AssetKind, name: AssetName) -> Self {
        let mut hasher = Sha256::new();
        match source {
            AssetSource::File(guid) => {
                hasher.update(b"G");
                hasher.update(guid.to_be_bytes());
            }
            AssetSource::Scene(scene) => {
                hasher.update(b"S");
                hash_str(&mut hasher, scene);
            }
        }
        match kind {
            AssetKind::Mesh => hasher.update([0]),
            AssetKind::Material => hasher.update([1]),
            AssetKind::Texture(_) => hasher.update([2]),
        }
        match name {
            AssetName::Name(name) => {
                hasher.update(b"N");
                hash_str(&mut hasher, name);
            }
            AssetName::Index(index) => {
                hasher.update(b"I");
                hasher.update((index as u64).to_le_bytes());
            }
        }
        if let AssetKind::Texture(channel) = kind {
            hasher.update([channel as u8]);
        }
        let digest = hasher.finalize();
        Self(u128::from_be_bytes(
            digest[..16].try_into().expect("SHA-256 is 32 bytes"),
        ))
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:032x}", self.0)
    }
}

fn hash_str(hasher: &mut Sha256, text: &str) {
    hasher.update((text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
}

/// A Material Asset: PBR factors plus one Texture Asset ID per Channel,
/// indexed by `MaterialChannel`; a missing Channel resolves to Layer 0 at
/// load.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedMaterial {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: [f32; 3],
    pub textures: [Option<AssetId>; 5],
}

impl CookedMaterial {
    /// Nine little-endian f32 factors, then 16 bytes per Channel, all zero
    /// for none.
    const PAYLOAD_BYTES: usize = 9 * 4 + 5 * 16;

    #[cfg(feature = "cook")]
    pub(crate) fn to_blob(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(8 + Self::PAYLOAD_BYTES);
        blob.extend(MATERIAL_MAGIC);
        blob.extend(MATERIAL_FORMAT_VERSION.to_le_bytes());
        let [red, green, blue, alpha] = self.base_color;
        let [emissive_red, emissive_green, emissive_blue] = self.emissive;
        for factor in [
            red,
            green,
            blue,
            alpha,
            self.metallic,
            self.roughness,
            emissive_red,
            emissive_green,
            emissive_blue,
        ] {
            blob.extend(factor.to_le_bytes());
        }
        for texture in self.textures {
            blob.extend(texture.map_or(0, |id| id.0).to_be_bytes());
        }
        blob
    }

    pub fn from_blob(blob: &[u8]) -> anyhow::Result<Self> {
        let payload = payload(blob, MATERIAL_MAGIC, MATERIAL_FORMAT_VERSION)?;
        anyhow::ensure!(
            payload.len() == Self::PAYLOAD_BYTES,
            "Material Blob is {} bytes, expected {}: re-cook with `praxis cook`",
            payload.len(),
            Self::PAYLOAD_BYTES
        );
        let (factor_bytes, texture_bytes) = payload.split_at(9 * 4);
        let factors: Vec<f32> = factor_bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        let mut textures = [None; 5];
        for (texture, bytes) in textures.iter_mut().zip(texture_bytes.as_chunks::<16>().0) {
            let id = u128::from_be_bytes(*bytes);
            *texture = (id != 0).then_some(AssetId(id));
        }
        Ok(Self {
            base_color: factors[0..4].try_into().expect("4 factors"),
            metallic: factors[4],
            roughness: factors[5],
            emissive: factors[6..9].try_into().expect("3 factors"),
            textures,
        })
    }
}

/// `blob` after its magic and format version, or a refusal asking for a
/// re-cook when either does not match: there is no migration code.
fn payload(blob: &[u8], magic: [u8; 4], version: u32) -> anyhow::Result<&[u8]> {
    let found_magic = blob.get(..4);
    let found_version = blob
        .get(4..8)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("4 bytes")));
    anyhow::ensure!(
        found_magic == Some(&magic[..]) && found_version == Some(version),
        "Blob is not a {} version {version} Blob: re-cook with `praxis cook`",
        String::from_utf8_lossy(&magic)
    );
    Ok(&blob[8..])
}

/// The Cooked cache in a directory: `blobs/<SHA-256 of the Blob>` and
/// `index/<Asset ID>`, which holds the Blob's hash and the Cook key.
pub struct CookedCache {
    root: PathBuf,
}

/// One cook index entry, as lowercase hex.
pub(crate) struct IndexEntry {
    pub blob: String,
    // Read by the cooker only.
    #[cfg_attr(not(feature = "cook"), allow(dead_code))]
    pub cook_key: String,
}

impl CookedCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn blob_path(&self, blob: &str) -> PathBuf {
        self.root.join("blobs").join(blob)
    }

    fn index_path(&self, id: AssetId) -> PathBuf {
        self.root.join("index").join(id.to_string())
    }

    /// The index entry for `id` whose Blob is present, if any.
    pub(crate) fn entry(&self, id: AssetId) -> Option<IndexEntry> {
        let text = fs::read_to_string(self.index_path(id)).ok()?;
        let (blob, cook_key) = text.trim_end().split_once(' ')?;
        self.blob_path(blob).exists().then(|| IndexEntry {
            blob: blob.to_string(),
            cook_key: cook_key.to_string(),
        })
    }

    /// The Blob the index names for `id`.
    pub fn read(&self, id: AssetId) -> anyhow::Result<Vec<u8>> {
        let entry = self.entry(id).with_context(|| {
            format!(
                "Asset {id} is not in the Cooked cache at {}: run `praxis cook`",
                self.root.display()
            )
        })?;
        let path = self.blob_path(&entry.blob);
        fs::read(&path).with_context(|| format!("reading Blob {}", path.display()))
    }

    pub fn material(&self, id: AssetId) -> anyhow::Result<CookedMaterial> {
        CookedMaterial::from_blob(&self.read(id)?).with_context(|| format!("reading Material {id}"))
    }

    /// Stores `blob` under its hash and points `id` at it with `cook_key`.
    /// Each file is written whole and renamed into place, so a reader or a
    /// concurrent cook sees the old file or the new one, never a torn one.
    #[cfg(feature = "cook")]
    pub(crate) fn write(&self, id: AssetId, cook_key: &str, blob: &[u8]) -> anyhow::Result<()> {
        let hash = hex(&Sha256::digest(blob));
        let blob_path = self.blob_path(&hash);
        if !blob_path.exists() {
            write_atomically(&blob_path, blob)?;
        }
        write_atomically(
            &self.index_path(id),
            format!("{hash} {cook_key}\n").as_bytes(),
        )
    }
}

#[cfg(feature = "cook")]
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(feature = "cook")]
fn write_atomically(path: &std::path::Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    // Unique per process and per write, so concurrent writers never share
    // a temporary file.
    static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(0);
    let directory = path.parent().expect("cache paths have a parent");
    fs::create_dir_all(directory).with_context(|| format!("creating {}", directory.display()))?;
    let temporary = directory.join(format!(
        ".{}-{}.tmp",
        std::process::id(),
        NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    // Synced before the rename, so a crash cannot leave an empty file under
    // a name the next cook trusts.
    let mut file = fs::File::create(&temporary)
        .with_context(|| format!("creating {}", temporary.display()))?;
    std::io::Write::write_all(&mut file, bytes)
        .and_then(|()| file.sync_all())
        .with_context(|| format!("writing {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| format!("renaming into {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An oracle independent of the code above: SHA-256 of the encoding
    /// `G`, 16 GUID bytes, kind 1, `N`, length 8 as u64 LE, `material`,
    /// computed with `shasum -a 256` and truncated to 128 bits.
    #[test]
    fn asset_id_is_truncated_sha256_of_its_encoding() {
        let id = AssetId::new(
            AssetSource::File(0x000102030405060708090a0b0c0d0e0f),
            AssetKind::Material,
            AssetName::Name("material"),
        );
        assert_eq!(id.to_string(), "397412b2d75d55daad25a42ab0cbbb05");
    }

    #[cfg(feature = "cook")]
    #[test]
    fn material_blob_round_trips_and_refuses_another_version() {
        let material = CookedMaterial {
            base_color: [0.1, 0.2, 0.3, 1.0],
            metallic: 0.5,
            roughness: 0.25,
            emissive: [1.0, 0.0, 2.0],
            textures: [Some(AssetId(7)), None, None, Some(AssetId(u128::MAX)), None],
        };
        let blob = material.to_blob();
        assert_eq!(CookedMaterial::from_blob(&blob).unwrap(), material);

        for byte in [0, 4] {
            let mut mismatched = blob.clone();
            mismatched[byte] += 1;
            let error = CookedMaterial::from_blob(&mismatched)
                .unwrap_err()
                .to_string();
            assert!(error.contains("re-cook with `praxis cook`"), "{error}");
        }
    }
}
