//! The cooker (ADR 0018): turns every manifest source and every procedural
//! Reference scene into Blobs in the Cooked cache, cooking only the Assets
//! whose Cook key changed.

mod mesh;
mod texture;

use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub use texture::TextureSource;

use crate::ReferenceScene;
use crate::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, CookedCache, CookedMaterial,
    MATERIAL_FORMAT_VERSION, MATERIAL_MAGIC, MESH_FORMAT_VERSION, MESH_MAGIC, MaterialChannel,
    TEXTURE_FORMAT_VERSION, hex,
};

/// The cook settings a Cook key covers besides sources, formats and
/// encoders.
#[derive(Copy, Clone, Debug)]
pub struct CookSettings {
    /// The zstd level of every Texture mip.
    pub texture_zstd_level: i32,
}

impl Default for CookSettings {
    fn default() -> Self {
        Self {
            texture_zstd_level: zstd::DEFAULT_COMPRESSION_LEVEL,
        }
    }
}

/// The Asset IDs one cook wrote, and those it found up to date.
#[derive(Debug, Default)]
pub struct CookReport {
    pub cooked: Vec<AssetId>,
    pub unchanged: Vec<AssetId>,
    /// Each Mesh and the Material one of its Primitives names that is not
    /// in the Cooked cache.
    pub unresolved_materials: Vec<(AssetId, AssetId)>,
}

/// The fields of an `assets/manifest.json` entry the cooker reads.
#[derive(Deserialize)]
struct ManifestEntry {
    path: String,
    guid: String,
}

/// Every Material, Texture and Mesh the sources define.
#[derive(Default)]
struct Sources {
    /// Each Material with its Cook key.
    materials: Vec<(AssetId, String, CookedMaterial)>,
    textures: Vec<TextureSource>,
    /// Each Mesh with its Cook key and its Primitives.
    meshes: Vec<(AssetId, String, Vec<mesh::SourcePrimitive>)>,
}

/// Cooks the sources `assets/manifest.json` lists that are present, and
/// every procedural Reference scene, into `cache`.
pub fn cook(
    assets: &Path,
    cache: &CookedCache,
    settings: &CookSettings,
) -> anyhow::Result<CookReport> {
    let sources = read_sources(assets)?;
    let mut report = CookReport::default();
    for (id, cook_key, material) in &sources.materials {
        if is_fresh(cache, *id, cook_key) {
            report.unchanged.push(*id);
        } else {
            cache.write(*id, cook_key, &material.to_blob())?;
            report.cooked.push(*id);
        }
    }
    for (id, cook_key, primitives) in &sources.meshes {
        if is_fresh(cache, *id, cook_key) {
            report.unchanged.push(*id);
        } else {
            let blob = mesh::cook(primitives)
                .to_blob()
                .with_context(|| format!("cooking Mesh {id}"))?;
            cache.write(*id, cook_key, &blob)?;
            report.cooked.push(*id);
        }
    }
    let mut stale = Vec::new();
    for texture in &sources.textures {
        let cook_key = texture.cook_key(settings, TEXTURE_FORMAT_VERSION);
        if is_fresh(cache, texture.id, &cook_key) {
            report.unchanged.push(texture.id);
        } else {
            stale.push((texture, cook_key));
        }
    }
    // One Texture per core at a time: its filtering runs on its own thread,
    // and memory holds at most that many Textures' working sets. The cores
    // split among a batch's encoders.
    let threads = std::thread::available_parallelism().map_or(1, |threads| threads.get());
    for batch in stale.chunks(threads) {
        let encode_threads = threads.div_ceil(batch.len());
        let blobs: Vec<_> = std::thread::scope(|scope| {
            let cooks: Vec<_> = batch
                .iter()
                .map(|(texture, _)| scope.spawn(|| texture.cook(settings, encode_threads)))
                .collect();
            cooks
                .into_iter()
                .map(|cook| cook.join().expect("a Texture cook panicked"))
                .collect()
        });
        for ((texture, cook_key), blob) in batch.iter().zip(blobs) {
            cache.write(texture.id, cook_key, &blob?)?;
            report.cooked.push(texture.id);
        }
    }
    for (id, _, material) in &sources.materials {
        for texture in material.textures.iter().flatten() {
            if cache.entry(*texture).is_none() {
                tracing::warn!(material = %id, %texture, "Material samples a Texture the Cooked cache lacks");
            }
        }
    }
    let material_references: Vec<_> = sources
        .meshes
        .iter()
        .flat_map(|(id, _, primitives)| {
            primitives
                .iter()
                .filter_map(|(_, material)| *material)
                .map(|material| (*id, material))
        })
        .collect();
    report.unresolved_materials = unresolved_materials(cache, &material_references);
    tracing::info!(
        cooked = report.cooked.len(),
        unchanged = report.unchanged.len(),
        "cook complete"
    );
    Ok(report)
}

/// Every Texture the cook reads: each image a glTF Material samples, per
/// Channel, and each procedural Reference scene's generated image.
pub fn texture_sources(assets: &Path) -> anyhow::Result<Vec<TextureSource>> {
    Ok(read_sources(assets)?.textures)
}

fn read_sources(assets: &Path) -> anyhow::Result<Sources> {
    let manifest_path = assets.join("manifest.json");
    let manifest: Vec<ManifestEntry> = serde_json::from_slice(
        &std::fs::read(&manifest_path)
            .with_context(|| format!("reading {}", manifest_path.display()))?,
    )
    .with_context(|| format!("parsing {}", manifest_path.display()))?;

    let mut sources = Sources::default();
    for entry in &manifest {
        if !(entry.path.ends_with(".gltf") || entry.path.ends_with(".glb")) {
            continue;
        }
        let path = assets.join(&entry.path);
        let Ok(source) = std::fs::read(&path) else {
            tracing::warn!(path = %path.display(), "source absent; its cooked Blobs stay as they are");
            continue;
        };
        let guid = u128::from_str_radix(&entry.guid, 16)
            .with_context(|| format!("{}: guid {:?} is not hex", entry.path, entry.guid))?;
        read_gltf(guid, &path, &source, &mut sources)
            .with_context(|| format!("reading {}", path.display()))?;
    }
    for scene in ReferenceScene::ALL {
        let source = AssetSource::Scene(scene.name());
        for (name, material) in scene.materials() {
            let id = AssetId::new(source, AssetKind::Material, AssetName::Name(&name));
            // A procedural Asset's source bytes are its generated bytes.
            let cook_key = material_cook_key(&material.to_blob());
            sources.materials.push((id, cook_key, material));
        }
        for (name, channel, image) in scene.textures() {
            let id = AssetId::new(source, AssetKind::Texture(channel), AssetName::Name(name));
            let name = format!("{} {name}", scene.name());
            sources
                .textures
                .push(TextureSource::generated(id, channel, name, image));
        }
        for (name, geometry, material) in scene.meshes() {
            let id = AssetId::new(source, AssetKind::Mesh, AssetName::Name(name));
            let material = material.map(|material| {
                AssetId::new(source, AssetKind::Material, AssetName::Name(material))
            });
            let primitives = vec![(geometry, material)];
            sources
                .meshes
                .push((id, mesh_cook_key(&primitives), primitives));
        }
    }
    Ok(sources)
}

/// A Material has no cook settings and no encoder, so its Cook key covers
/// its Blob, which is all it cooks from, and its format.
fn material_cook_key(blob: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MATERIAL_MAGIC);
    hasher.update(MATERIAL_FORMAT_VERSION.to_le_bytes());
    hasher.update(blob);
    hex(&hasher.finalize())
}

/// A Mesh's Cook key: SHA-256 over its format, its encoders' versions and
/// settings, and each Primitive's geometry and Material, which are all it
/// cooks from, each part length-prefixed so no two Meshes share an encoding.
fn mesh_cook_key(primitives: &[mesh::SourcePrimitive]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MESH_MAGIC);
    hasher.update(MESH_FORMAT_VERSION.to_le_bytes());
    let mut hash_part = |part: &[u8]| {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    };
    hash_part(mesh::encoder_settings().as_bytes());
    for (geometry, material) in primitives {
        hash_part(bytemuck::cast_slice(&geometry.positions));
        hash_part(bytemuck::cast_slice(&geometry.normals));
        hash_part(bytemuck::cast_slice(&geometry.uvs));
        hash_part(bytemuck::cast_slice(&geometry.indices));
        hash_part(&material.map_or(0, |material| material.0).to_be_bytes());
    }
    hex(&hasher.finalize())
}

/// Whether the index holds `id` under `cook_key`, its Blob present.
fn is_fresh(cache: &CookedCache, id: AssetId, cook_key: &str) -> bool {
    cache
        .entry(id)
        .is_some_and(|entry| entry.cook_key == cook_key)
}

/// The `(Mesh, Material)` references whose Material is not in `cache`, each
/// warned about.
fn unresolved_materials(
    cache: &CookedCache,
    references: &[(AssetId, AssetId)],
) -> Vec<(AssetId, AssetId)> {
    let unresolved: Vec<_> = references
        .iter()
        .copied()
        .filter(|(_, material)| cache.entry(*material).is_none())
        .collect();
    for (mesh, material) in &unresolved {
        tracing::warn!(%mesh, %material, "a Primitive names a Material the Cooked cache lacks");
    }
    unresolved
}

/// The Textures `material` samples, indexed by `MaterialChannel`.
fn material_textures<'a>(material: &gltf::Material<'a>) -> [Option<gltf::Texture<'a>>; 5] {
    let pbr = material.pbr_metallic_roughness();
    [
        pbr.base_color_texture().map(|info| info.texture()),
        material.normal_texture().map(|normal| normal.texture()),
        pbr.metallic_roughness_texture().map(|info| info.texture()),
        material
            .occlusion_texture()
            .map(|occlusion| occlusion.texture()),
        material.emissive_texture().map(|info| info.texture()),
    ]
}

/// Every Material and Mesh of the glTF file `source` at `path`, and every
/// image its Materials sample, once per Channel. A Texture's Asset ID is the
/// glTF file's GUID, the image's name (or index) and the Channel, whether the
/// image is embedded or a file of its own. Each Material and Mesh is keyed
/// by what it cooks from, so that a change elsewhere in the file, to an
/// embedded image say, re-cooks neither.
fn read_gltf(guid: u128, path: &Path, source: &[u8], sources: &mut Sources) -> anyhow::Result<()> {
    let gltf::Gltf { document, blob } = gltf::Gltf::from_slice(source)?;
    let buffers =
        gltf::import_buffers(&document, path.parent(), blob).context("reading its buffers")?;
    let asset_source = AssetSource::File(guid);
    let image_names = asset_names(document.images().map(|image| image.name()));
    let texture_id = |channel, texture: &gltf::Texture| {
        let name = image_names[texture.source().index()];
        AssetId::new(asset_source, AssetKind::Texture(channel), name)
    };
    let material_names = asset_names(document.materials().map(|material| material.name()));
    let mut cooked_images = HashSet::new();
    for (material, name) in document.materials().zip(material_names) {
        let textures = material_textures(&material);
        let mut texture_ids = [None; 5];
        for (channel, texture) in MaterialChannel::ALL.into_iter().zip(textures) {
            let Some(texture) = texture else { continue };
            let id = texture_id(channel, &texture);
            texture_ids[channel as usize] = Some(id);
            if !cooked_images.insert(id) {
                continue;
            }
            let image = texture.source();
            let name = format!("{} image {:?}", path.display(), image_names[image.index()]);
            let bytes = match image.source() {
                gltf::image::Source::View { view, .. } => {
                    let buffer = &buffers[view.buffer().index()];
                    buffer
                        .get(view.offset()..view.offset() + view.length())
                        .with_context(|| format!("{name}: buffer view out of bounds"))?
                        .to_vec()
                }
                gltf::image::Source::Uri { uri, .. } if uri.starts_with("data:") => {
                    tracing::warn!(image = name, "data URI images are not cooked");
                    continue;
                }
                gltf::image::Source::Uri { uri, .. } => {
                    let image_path = path.parent().unwrap_or(Path::new("")).join(uri);
                    let Ok(bytes) = std::fs::read(&image_path) else {
                        tracing::warn!(path = %image_path.display(), "image absent; its cooked Texture stays as it is");
                        continue;
                    };
                    bytes
                }
            };
            sources
                .textures
                .push(TextureSource::encoded(id, channel, name, bytes));
        }
        let pbr = material.pbr_metallic_roughness();
        let id = AssetId::new(asset_source, AssetKind::Material, name);
        let material = CookedMaterial {
            base_color: pbr.base_color_factor(),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            emissive: material.emissive_factor(),
            textures: texture_ids,
        };
        let cook_key = material_cook_key(&material.to_blob());
        sources.materials.push((id, cook_key, material));
    }
    for (id, primitives) in mesh::gltf_meshes(guid, &document, &buffers)? {
        sources
            .meshes
            .push((id, mesh_cook_key(&primitives), primitives));
    }
    Ok(())
}

/// Each item's Asset name: its glTF name, or its index when the name is
/// missing or shared with another item.
fn asset_names<'a>(names: impl Iterator<Item = Option<&'a str>>) -> Vec<AssetName<'a>> {
    let names: Vec<_> = names.collect();
    let unique = |name: &str| names.iter().filter(|other| **other == Some(name)).count() == 1;
    names
        .iter()
        .enumerate()
        .map(|(index, name)| match name {
            Some(name) if unique(name) => AssetName::Name(name),
            _ => AssetName::Index(index),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_material_missing_from_the_cache_is_unresolved() {
        let cache = CookedCache::new(
            std::env::temp_dir().join(format!("praxis-empty-cache-{}", std::process::id())),
        );
        let reference = (AssetId(1), AssetId(2));
        assert_eq!(unresolved_materials(&cache, &[reference]), [reference]);
    }
}
