//! The cooker (ADR 0018): turns every manifest source and every procedural
//! Reference scene into Blobs in the Cooked cache, cooking only the Assets
//! whose Cook key changed.

mod fetch;
pub(crate) mod mesh;
mod texture;

use std::collections::HashSet;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub use fetch::{Fetch, absent_fetched, fetch};
pub use texture::{LoadedImage, TextureSource};

use crate::ReferenceScene;
use crate::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, CookedCache, CookedMaterial,
    MATERIAL_FORMAT_VERSION, MATERIAL_MAGIC, MaterialChannel, TEXTURE_FORMAT_VERSION, hex,
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
    /// Each source the cook could not read, whose Blobs stay as they are.
    pub absent_sources: Vec<String>,
}

/// The fields of an `assets/manifest.json` entry the cooker and fetching
/// read; a committed file's needs only its path and GUID.
#[derive(Deserialize)]
struct ManifestEntry {
    path: String,
    guid: String,
    /// Whether the file is downloaded from `source` rather than committed.
    #[serde(default)]
    fetched: bool,
    #[serde(default)]
    source: String,
    #[serde(default)]
    sha256: String,
}

impl ManifestEntry {
    fn guid(&self) -> anyhow::Result<u128> {
        u128::from_str_radix(&self.guid, 16)
            .with_context(|| format!("{}: guid {:?} is not hex", self.path, self.guid))
    }
}

/// The entries of the manifest in `assets`; none when it is absent and
/// `may_be_absent`, as a dev launch allows.
fn read_manifest(assets: &Path, may_be_absent: bool) -> anyhow::Result<Vec<ManifestEntry>> {
    let path = assets.join("manifest.json");
    match std::fs::read(&path) {
        Ok(manifest) => {
            serde_json::from_slice(&manifest).with_context(|| format!("parsing {}", path.display()))
        }
        Err(error) if may_be_absent && error.kind() == std::io::ErrorKind::NotFound => {
            Ok(Vec::new())
        }
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
}

/// Every Material, Texture and Mesh the sources define.
#[derive(Default)]
struct Sources {
    /// Each Material with its Cook key.
    materials: Vec<(AssetId, String, CookedMaterial)>,
    textures: Vec<TextureSource>,
    /// Each Mesh with its Cook key and its Primitives.
    meshes: Vec<(AssetId, String, Vec<mesh::SourcePrimitive>)>,
    absent: Vec<String>,
}

/// The source directory a launch cooks from, relative to the working
/// directory.
pub const ASSETS_DIRECTORY: &str = "assets";

/// Cooks the sources `assets/manifest.json` lists that are present, and the
/// procedural Assets of `scenes`, into `cache`; `praxis cook` cooks every
/// Reference scene's.
pub fn cook(
    assets: &Path,
    scenes: &[ReferenceScene],
    cache: &CookedCache,
    settings: &CookSettings,
) -> anyhow::Result<CookReport> {
    cook_sources(&read_sources(assets, scenes, None)?, cache, settings)
}

/// What a dev launch of `scene` cooks before it loads: the manifest
/// sources the scene references, those present, and its procedural Assets.
/// A source the manifest marks fetched is fetched first when absent. The
/// Blobs of a source still absent, or of every source when the manifest is
/// absent, stay as they are.
pub fn cook_scene(
    scene: ReferenceScene,
    assets: &Path,
    cache: &CookedCache,
    settings: &CookSettings,
) -> anyhow::Result<CookReport> {
    fetch::fetch_for_launch(assets, scene.source_files());
    let sources = read_sources(assets, &[scene], Some(scene.source_files()))?;
    cook_sources(&sources, cache, settings)
}

fn cook_sources(
    sources: &Sources,
    cache: &CookedCache,
    settings: &CookSettings,
) -> anyhow::Result<CookReport> {
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
    report.absent_sources = sources.absent.clone();
    tracing::info!(
        cooked = report.cooked.len(),
        unchanged = report.unchanged.len(),
        "cook complete"
    );
    Ok(report)
}

/// Every Texture a cook of `scenes` reads: each image a glTF Material
/// samples, per Channel, and each of the scenes' generated images.
pub fn texture_sources(
    assets: &Path,
    scenes: &[ReferenceScene],
) -> anyhow::Result<Vec<TextureSource>> {
    Ok(read_sources(assets, scenes, None)?.textures)
}

/// The sources of `scenes` and of every manifest file, or with
/// `source_guids`, a dev launch's, only of the files with those GUIDs; a dev
/// launch also runs without a manifest.
fn read_sources(
    assets: &Path,
    scenes: &[ReferenceScene],
    source_guids: Option<&[u128]>,
) -> anyhow::Result<Sources> {
    let manifest_path = assets.join("manifest.json");
    // Without one, a dev launch's GUIDs all warn as unlisted, below.
    let manifest = read_manifest(assets, source_guids.is_some())?;

    let mut sources = Sources::default();
    let mut unlisted: Vec<u128> = source_guids.unwrap_or_default().to_vec();
    for entry in &manifest {
        if !(entry.path.ends_with(".gltf") || entry.path.ends_with(".glb")) {
            continue;
        }
        let guid = entry.guid()?;
        unlisted.retain(|listed| *listed != guid);
        if source_guids.is_some_and(|guids| !guids.contains(&guid)) {
            continue;
        }
        let path = assets.join(&entry.path);
        let Ok(source) = std::fs::read(&path) else {
            tracing::warn!(path = %path.display(), "source absent; its cooked Blobs stay as they are");
            sources.absent.push(path.display().to_string());
            continue;
        };
        read_gltf(guid, &path, &source, &mut sources)
            .with_context(|| format!("reading {}", path.display()))?;
    }
    for guid in unlisted {
        tracing::warn!(manifest = %manifest_path.display(), guid = %format!("{guid:032x}"), "no source listed with this GUID; its cooked Blobs stay as they are");
        let absent = format!("GUID {guid:032x}, unlisted in {}", manifest_path.display());
        sources.absent.push(absent);
    }
    for &scene in scenes {
        let materials = scene.materials();
        for (name, material) in &materials {
            let id = scene.material_id(name);
            // A procedural Asset's source bytes are its generated bytes.
            let cook_key = material_cook_key(&material.to_blob());
            sources.materials.push((id, cook_key, material.clone()));
        }
        let textures: Vec<_> = scene
            .textures()
            .into_iter()
            .map(|(name, channel, image)| {
                let id = scene.texture_id(channel, &name);
                let name = format!("{} {name}", scene.name());
                TextureSource::generated(id, channel, name, image)
            })
            .collect();
        // Toksvig, as in `read_gltf`: a metallic-roughness Texture cooks from
        // its Material's normal Texture too (the first Material's, when
        // several sample it).
        let normal_textures: Vec<_> = textures
            .iter()
            .map(|texture| {
                let [_, normal, ..] = materials
                    .iter()
                    .map(|(_, material)| material.textures)
                    .find(|[_, _, metallic_roughness, ..]| {
                        *metallic_roughness == Some(texture.id)
                    })?;
                textures
                    .iter()
                    .find(|other| Some(other.id) == normal)
                    .cloned()
            })
            .collect();
        for (texture, normal_texture) in textures.into_iter().zip(normal_textures) {
            sources.textures.push(match normal_texture {
                Some(normal_texture) => texture.with_normal_texture(normal_texture),
                None => texture,
            });
        }
        for (name, primitives) in scene.meshes() {
            let id = scene.mesh_id(name);
            let primitives: Vec<_> = primitives
                .into_iter()
                .map(|(geometry, material)| {
                    let material = material.map(|material| scene.material_id(material));
                    (geometry, material)
                })
                .collect();
            sources
                .meshes
                .push((id, mesh::cook_key(&primitives), primitives));
        }
    }
    Ok(sources)
}

/// A Material's cook only serialises it, with no settings and no encoder,
/// so its Cook key covers its Blob and its format.
fn material_cook_key(blob: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MATERIAL_MAGIC);
    hasher.update(MATERIAL_FORMAT_VERSION.to_le_bytes());
    hasher.update(blob);
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
    let absent_images = std::cell::RefCell::new(Vec::new());
    // The image `texture` samples, as a Texture of `channel`; none when the
    // image is absent or a data URI.
    let texture_source = |channel, texture: &gltf::Texture| -> anyhow::Result<_> {
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
                return Ok(None);
            }
            gltf::image::Source::Uri { uri, .. } => {
                let image_path = path.parent().unwrap_or(Path::new("")).join(uri);
                let Ok(bytes) = std::fs::read(&image_path) else {
                    tracing::warn!(path = %image_path.display(), "image absent; its cooked Texture stays as it is");
                    absent_images
                        .borrow_mut()
                        .push(image_path.display().to_string());
                    return Ok(None);
                };
                bytes
            }
        };
        let id = texture_id(channel, texture);
        Ok(Some(TextureSource::encoded(id, channel, name, bytes)))
    };
    let mut cooked_images = HashSet::new();
    for (material, name) in document.materials().zip(material_names) {
        let textures = material_textures(&material);
        let mut texture_ids = [None; 5];
        for (channel, texture) in MaterialChannel::ALL.into_iter().zip(&textures) {
            let Some(texture) = texture else { continue };
            let id = texture_id(channel, texture);
            texture_ids[channel as usize] = Some(id);
            if !cooked_images.insert(id) {
                continue;
            }
            let Some(mut source) = texture_source(channel, texture)? else {
                continue;
            };
            // Toksvig: metallic-roughness cooks from its Material's normal
            // map too (the first Material's, when several share the image),
            // and stays as it is while that is absent.
            if channel == MaterialChannel::MetallicRoughness
                && let Some(normal) = &textures[MaterialChannel::Normal as usize]
            {
                let Some(normal_texture) = texture_source(MaterialChannel::Normal, normal)? else {
                    continue;
                };
                source = source.with_normal_texture(normal_texture);
            }
            sources.textures.push(source);
        }
        let pbr = material.pbr_metallic_roughness();
        let id = AssetId::new(asset_source, AssetKind::Material, name);
        let material = CookedMaterial {
            base_color: pbr.base_color_factor(),
            metallic: pbr.metallic_factor(),
            roughness: pbr.roughness_factor(),
            emissive: material.emissive_factor(),
            normal_scale: material
                .normal_texture()
                .map_or(1.0, |normal| normal.scale()),
            textures: texture_ids,
        };
        let cook_key = material_cook_key(&material.to_blob());
        sources.materials.push((id, cook_key, material));
    }
    for (id, primitives) in mesh::gltf_meshes(guid, &document, &buffers)? {
        sources
            .meshes
            .push((id, mesh::cook_key(&primitives), primitives));
    }
    // A normal Texture a metallic-roughness Texture depends on is read twice.
    let mut absent_images = absent_images.into_inner();
    absent_images.sort();
    absent_images.dedup();
    sources.absent.extend(absent_images);
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

    /// Each scene's dev launch finds the sources it references.
    #[test]
    fn every_source_a_reference_scene_references_is_in_the_manifest() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/manifest.json");
        let manifest: Vec<ManifestEntry> =
            serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
        for scene in ReferenceScene::ALL {
            for guid in scene.source_files() {
                assert!(
                    manifest
                        .iter()
                        .any(|entry| entry.guid == format!("{guid:032x}")),
                    "{}: no manifest source has GUID {guid:032x}",
                    scene.name()
                );
            }
        }
    }
}
