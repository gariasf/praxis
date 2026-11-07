//! The cooker (ADR 0018): turns every manifest source and every procedural
//! Reference scene into Blobs in the Cooked cache, cooking only the Assets
//! whose Cook key changed.

mod chunk;
mod export;
mod fetch;
pub(crate) mod mesh;
mod texture;

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use export::AssetRecorder;
pub use export::{SCENE_FORMAT_VERSION, export_scene};
pub use fetch::{Fetch, absent_fetched, fetch};
pub use texture::{LoadedImage, TextureSource};

use crate::ReferenceScene;
use crate::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, CookedCache, CookedChunk, CookedMaterial,
    MATERIAL_FORMAT_VERSION, MATERIAL_MAGIC, MaterialChannel, TEXTURE_FORMAT_VERSION, hex,
};
use crate::render::CrowdForm;
use crate::state::Simulation;
use crate::stats::FrameRecorder;

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
    /// Each reference to an Asset ID this cook neither wrote nor found up to
    /// date, with what names it.
    pub dangling_references: Vec<(Referrer, AssetId)>,
    /// Each source the cook could not read, whose Blobs stay as they are.
    pub absent_sources: Vec<String>,
}

/// What names an Asset by its ID.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Referrer {
    /// A Material, of the Textures it samples.
    Material(AssetId),
    /// A Mesh, of the Materials its Primitives name.
    Mesh(AssetId),
    /// A Reference scene, of the Assets its spawn loads.
    Scene(ReferenceScene),
    /// A Reference scene's Chunk at its coordinates, x then z, of the Meshes
    /// and Materials its Placements name.
    Chunk(ReferenceScene, [i32; 2]),
}

impl std::fmt::Display for Referrer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Material(id) => write!(formatter, "Material {id}"),
            Self::Mesh(id) => write!(formatter, "Mesh {id}"),
            Self::Scene(scene) => write!(formatter, "Reference scene {}", scene.name()),
            Self::Chunk(scene, [x, z]) => {
                write!(
                    formatter,
                    "Chunk ({x}, {z}) of Reference scene {}",
                    scene.name()
                )
            }
        }
    }
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

/// Every Material, Texture, Mesh and Chunk the sources define.
#[derive(Default)]
struct Sources {
    /// Each Material with its Cook key.
    materials: Vec<(AssetId, String, CookedMaterial)>,
    textures: Vec<TextureSource>,
    /// Each Mesh with its name, its Cook key and its Primitives.
    meshes: Vec<(AssetId, String, String, Vec<mesh::SourcePrimitive>)>,
    /// Each Chunk with what names it in a warning and its Cook key.
    chunks: Vec<(AssetId, Referrer, String, CookedChunk)>,
    absent: Vec<String>,
    /// The GUID of each source some or all of whose Blobs stay as they are,
    /// as the source, or an image it names, is absent.
    kept_sources: Vec<u128>,
    /// Each Texture a read source's Materials sample whose image this cook
    /// did not read, so its Blob, if any, stays as it is.
    kept_textures: Vec<AssetId>,
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
    let sources = read_sources(assets, scenes, None)?;
    let references = scene_references(scenes, &sources.chunks)?;
    cook_sources(&sources, &references, cache, settings)
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
    let references = scene_references(&[scene], &sources.chunks)?;
    cook_sources(&sources, &references, cache, settings)
}

/// Each Asset ID `scenes` load as they spawn from `chunks`, once per scene.
fn scene_references(
    scenes: &[ReferenceScene],
    chunks: &[(AssetId, Referrer, String, CookedChunk)],
) -> anyhow::Result<Vec<(Referrer, AssetId)>> {
    let mut references = Vec::new();
    for &scene in scenes {
        let mut recorder = AssetRecorder::new(chunks.iter().map(|(id, _, _, chunk)| (*id, chunk)));
        // The default crowd form loads the shadow proxies besides every
        // Asset the traced forms load.
        Simulation::new(
            scene,
            glam::DVec3::ZERO,
            CrowdForm::default(),
            &mut recorder,
            &FrameRecorder::new(),
        )?;
        let mut ids: Vec<_> = recorder.ids().collect();
        ids.sort();
        ids.dedup();
        references.extend(ids.into_iter().map(|id| (Referrer::Scene(scene), id)));
    }
    Ok(references)
}

fn cook_sources(
    sources: &Sources,
    scene_references: &[(Referrer, AssetId)],
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
    for (id, name, cook_key, primitives) in &sources.meshes {
        if is_fresh(cache, *id, cook_key) {
            report.unchanged.push(*id);
        } else {
            let blob = mesh::cook(primitives)
                .and_then(|mesh| mesh.to_blob())
                .with_context(|| format!("cooking Mesh {name} ({id})"))?;
            cache.write(*id, cook_key, &blob)?;
            report.cooked.push(*id);
        }
    }
    for (id, _, cook_key, chunk) in &sources.chunks {
        if is_fresh(cache, *id, cook_key) {
            report.unchanged.push(*id);
        } else {
            cache.write(*id, cook_key, &chunk.to_blob()?)?;
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
    let texture_references = sources.materials.iter().flat_map(|(id, _, material)| {
        let textures = material.textures.iter().flatten();
        textures.map(|texture| (Referrer::Material(*id), *texture))
    });
    let material_references = sources.meshes.iter().flat_map(|(id, _, _, primitives)| {
        let materials = primitives.iter().filter_map(|(_, material)| *material);
        materials.map(|material| (Referrer::Mesh(*id), material))
    });
    let placement_references = sources.chunks.iter().flat_map(|(_, referrer, _, chunk)| {
        let placements = chunk.placements.iter();
        let named = placements.flat_map(|placement| [Some(placement.mesh), placement.material]);
        named.flatten().map(|id| (*referrer, id))
    });
    let references: Vec<_> = texture_references
        .chain(material_references)
        .chain(placement_references)
        .chain(scene_references.iter().copied())
        .collect();
    let made: HashSet<_> = report
        .cooked
        .iter()
        .chain(&report.unchanged)
        .copied()
        .collect();
    // A glTF file's Materials sample its own images, and a Primitive names a
    // Material of its own file or scene, so only a Texture whose image this
    // cook did not read may be kept; a scene's references, only while one of
    // its own sources is.
    let may_be_kept = |(referrer, id): &(Referrer, AssetId)| match referrer {
        Referrer::Scene(scene) | Referrer::Chunk(scene, _) => scene
            .source_files()
            .iter()
            .any(|guid| sources.kept_sources.contains(guid)),
        Referrer::Material(_) | Referrer::Mesh(_) => sources.kept_textures.contains(id),
    };
    report.dangling_references = dangling_references(cache, &made, &references, may_be_kept);
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
            sources.kept_sources.push(guid);
            continue;
        };
        let absent_before = sources.absent.len();
        read_gltf(guid, &path, &source, &mut sources)
            .with_context(|| format!("reading {}", path.display()))?;
        if sources.absent.len() > absent_before {
            sources.kept_sources.push(guid);
        }
    }
    for guid in unlisted {
        tracing::warn!(manifest = %manifest_path.display(), guid = %format!("{guid:032x}"), "no source listed with this GUID; its cooked Blobs stay as they are");
        let absent = format!("GUID {guid:032x}, unlisted in {}", manifest_path.display());
        sources.absent.push(absent);
        sources.kept_sources.push(guid);
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
            let name = format!("{} {name}", scene.name());
            let cook_key = mesh::cook_key(&primitives);
            sources.meshes.push((id, name, cook_key, primitives));
        }
    }
    let mesh_bounds: HashMap<_, _> = sources
        .meshes
        .iter()
        .map(|(id, _, _, primitives)| (*id, chunk::source_bounds(primitives)))
        .collect();
    for &scene in scenes {
        let (placements, lights) = scene.static_content();
        for (coordinates, chunk) in chunk::chunks(placements, lights, &mesh_bounds) {
            let id = scene.chunk_id(coordinates);
            let cook_key = chunk::cook_key(&chunk);
            let referrer = Referrer::Chunk(scene, coordinates);
            sources.chunks.push((id, referrer, cook_key, chunk));
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

/// The references to an Asset this cook did not make, each warned of. The
/// Cooked cache is never pruned, so an ID it holds may be one no source
/// defines any more; it resolves a reference only where `may_be_kept` says
/// the Blob is one the cook kept, its source absent.
fn dangling_references(
    cache: &CookedCache,
    made: &HashSet<AssetId>,
    references: &[(Referrer, AssetId)],
    may_be_kept: impl Fn(&(Referrer, AssetId)) -> bool,
) -> Vec<(Referrer, AssetId)> {
    // ponytail: an absent source keeps its Blobs, and an Asset ID does not
    // say which source it came from, so a scene with an absent source passes
    // every cached ID, a stale one included.
    let dangling: Vec<_> = references
        .iter()
        .copied()
        .filter(|reference| {
            let (_, id) = reference;
            !(made.contains(id) || may_be_kept(reference) && cache.entry(*id).is_some())
        })
        .collect();
    for (referrer, id) in &dangling {
        tracing::warn!(%referrer, asset = %id, "references an Asset ID this cook does not make");
    }
    dangling
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
                sources.kept_textures.push(id);
                continue;
            };
            // Toksvig: metallic-roughness cooks from its Material's normal
            // map too (the first Material's, when several share the image),
            // and stays as it is while that is absent.
            if channel == MaterialChannel::MetallicRoughness
                && let Some(normal) = &textures[MaterialChannel::Normal as usize]
            {
                let Some(normal_texture) = texture_source(MaterialChannel::Normal, normal)? else {
                    sources.kept_textures.push(id);
                    continue;
                };
                source = source.with_normal_texture(normal_texture);
            }
            sources.textures.push(source);
        }
        let id = AssetId::new(asset_source, AssetKind::Material, name);
        let material = gltf_material(&material, texture_ids);
        let cook_key = material_cook_key(&material.to_blob());
        sources.materials.push((id, cook_key, material));
    }
    for (id, name, primitives) in mesh::gltf_meshes(guid, &document, &buffers)? {
        let name = match name {
            AssetName::Name(name) => format!("{} {name:?}", path.display()),
            AssetName::Index(index) => format!("{} mesh {index}", path.display()),
        };
        let cook_key = mesh::cook_key(&primitives);
        sources.meshes.push((id, name, cook_key, primitives));
    }
    // A normal Texture a metallic-roughness Texture depends on is read twice.
    let mut absent_images = absent_images.into_inner();
    absent_images.sort();
    absent_images.dedup();
    sources.absent.extend(absent_images);
    Ok(())
}

/// `material`'s factors, with the Textures `textures` names per Channel.
fn gltf_material(material: &gltf::Material, textures: [Option<AssetId>; 5]) -> CookedMaterial {
    let pbr = material.pbr_metallic_roughness();
    CookedMaterial {
        base_color: pbr.base_color_factor(),
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        emissive: material.emissive_factor(),
        normal_scale: material
            .normal_texture()
            .map_or(1.0, |normal| normal.scale()),
        textures,
    }
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

    /// A Texture or Material that only the Cooked cache holds, which this
    /// cook neither wrote nor found up to date, is stale and dangles; while a
    /// source is absent, the cache's Blobs are kept and it passes. One the
    /// cache lacks always dangles.
    #[test]
    fn a_reference_only_the_stale_cache_holds_dangles() {
        let directory =
            std::env::temp_dir().join(format!("praxis-stale-cache-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let (stale, made, missing) = (AssetId(1), AssetId(2), AssetId(3));
        cache.write(stale, "cook key", b"stale Blob").unwrap();
        let made_ids = HashSet::from([made]);
        let references = [
            (Referrer::Material(AssetId(10)), stale),
            (Referrer::Mesh(AssetId(11)), stale),
            (Referrer::Mesh(AssetId(11)), made),
            (Referrer::Material(AssetId(10)), missing),
        ];
        assert_eq!(
            dangling_references(&cache, &made_ids, &references, |_| false),
            [references[0], references[1], references[3]]
        );
        assert_eq!(
            dangling_references(&cache, &made_ids, &references, |_| true),
            [references[3]]
        );
        let only_the_mesh_kept =
            |(referrer, _): &(Referrer, AssetId)| matches!(referrer, Referrer::Mesh(_));
        assert_eq!(
            dangling_references(&cache, &made_ids, &references, only_the_mesh_kept),
            [references[0], references[3]]
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    /// A Placement and a Material naming Asset IDs that only the stale
    /// Cooked cache still holds warn, each naming its referrer and the ID.
    #[test]
    fn a_placement_or_material_naming_a_removed_asset_dangles() {
        let directory =
            std::env::temp_dir().join(format!("praxis-removed-assets-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let (removed_mesh, removed_material, removed_texture) =
            (AssetId(1), AssetId(2), AssetId(3));
        for removed in [removed_mesh, removed_material, removed_texture] {
            cache.write(removed, "cook key", b"stale Blob").unwrap();
        }
        let material_id = AssetId(10);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [Some(removed_texture), None, None, None, None],
        };
        let placement = crate::cooked::CookedPlacement {
            mesh: removed_mesh,
            material: Some(removed_material),
            translation: glam::DVec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };
        let chunk = CookedChunk {
            placements: vec![placement],
            lights: Vec::new(),
            bounds: [glam::DVec3::ZERO; 2],
        };
        let chunk_referrer = Referrer::Chunk(ReferenceScene::Town, [-2, 1]);
        let sources = Sources {
            materials: vec![(material_id, "material cook key".into(), material)],
            chunks: vec![(AssetId(20), chunk_referrer, "chunk cook key".into(), chunk)],
            ..Sources::default()
        };
        let report = cook_sources(&sources, &[], &cache, &CookSettings::default()).unwrap();
        assert_eq!(
            report.dangling_references,
            [
                (Referrer::Material(material_id), removed_texture),
                (chunk_referrer, removed_mesh),
                (chunk_referrer, removed_material),
            ]
        );
        assert_eq!(
            chunk_referrer.to_string(),
            "Chunk (-2, 1) of Reference scene town"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    /// texture-wall loads its Textures by Asset ID, with no Material naming
    /// them, so its references come from the Textures its spawn loads.
    #[test]
    fn a_scene_references_the_textures_it_loads() {
        let scene = ReferenceScene::TextureWall;
        let (placements, lights) = scene.static_content();
        let chunks: Vec<_> = chunk::split(placements, lights)
            .into_iter()
            .map(|(coordinates, chunk)| {
                let referrer = Referrer::Chunk(scene, coordinates);
                (scene.chunk_id(coordinates), referrer, String::new(), chunk)
            })
            .collect();
        let references = scene_references(&[scene], &chunks).unwrap();
        let texture = scene.texture_id(MaterialChannel::Albedo, "texture-0");
        assert!(references.contains(&(Referrer::Scene(scene), texture)));
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
