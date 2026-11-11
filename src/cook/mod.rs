//! The cooker (ADR 0018): turns every manifest source and every procedural
//! Reference scene into Blobs in the Cooked cache, cooking only the Assets
//! whose Cook key changed.

mod chunk;
mod export;
mod fetch;
mod launch;
pub(crate) mod mesh;
mod texture;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use export::AssetRecorder;
pub use export::{SCENE_FORMAT_VERSION, export_scene};
pub use fetch::{Fetch, absent_fetched, fetch};
pub(crate) use launch::CookingLoader;
pub use texture::{ImageGenerator, LoadedImage, TextureSource};

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
    /// Each Chunk by its scene and coordinates, generated as it cooks.
    chunks: Vec<(ReferenceScene, [i32; 2])>,
    /// Each read Mesh's bounds, which a Chunk's bounds hold.
    mesh_bounds: HashMap<AssetId, [glam::Vec3; 2]>,
    /// Each scene's Chunks cut from its whole static content, once, for a
    /// scene whose generator makes no Chunk alone.
    cut_chunks: Vec<(ReferenceScene, BTreeMap<[i32; 2], CookedChunk>)>,
    /// Each Chunk generated so far a Placement of which names a Mesh this
    /// cook did not read, so its Blob, if any, stays as it is.
    kept_chunks: Vec<AssetId>,
    absent: Vec<String>,
    /// The GUID of each source some or all of whose Blobs stay as they are,
    /// as the source, or an image it names, is absent.
    kept_sources: Vec<u128>,
    /// Each Texture a read source's Materials sample whose image this cook
    /// did not read, so its Blob, if any, stays as it is.
    kept_textures: Vec<AssetId>,
}

/// A Chunk as a cook generates it.
struct GeneratedChunk {
    id: AssetId,
    referrer: Referrer,
    cook_key: String,
    chunk: CookedChunk,
    /// Whether a Placement names a Mesh this cook did not read, so the
    /// Chunk's Blob, if any, stays as it is.
    kept: bool,
}

impl GeneratedChunk {
    /// Each Mesh and Material the Chunk's Placements name, once.
    fn references(&self) -> impl Iterator<Item = (Referrer, AssetId)> + '_ {
        let placements = self.chunk.placements.iter();
        let named = placements.flat_map(|placement| [Some(placement.mesh), placement.material]);
        let mut named: Vec<_> = named.flatten().collect();
        named.sort_unstable();
        named.dedup();
        named.into_iter().map(|id| (self.referrer, id))
    }
}

impl Sources {
    /// `scene`'s Chunk at `coordinates`, its bounds holding its Placements'
    /// read Meshes: generated alone where the scene's generator can, else cut
    /// from the scene's whole static content, which is generated once.
    fn chunk(
        &mut self,
        scene: ReferenceScene,
        coordinates: [i32; 2],
    ) -> anyhow::Result<GeneratedChunk> {
        let referrer = Referrer::Chunk(scene, coordinates);
        let chunk = match scene.chunk_content(coordinates) {
            Some((placements, lights)) => {
                chunk::chunks(placements, lights, &self.mesh_bounds).remove(&coordinates)
            }
            None => {
                let cut_scene =
                    (self.cut_chunks.iter()).position(|(cut_scene, _)| *cut_scene == scene);
                let index = cut_scene.unwrap_or_else(|| {
                    let (placements, lights) = scene.static_content();
                    let chunks = chunk::chunks(placements, lights, &self.mesh_bounds);
                    self.cut_chunks.push((scene, chunks));
                    self.cut_chunks.len() - 1
                });
                let (_, chunks) = &self.cut_chunks[index];
                chunks.get(&coordinates).cloned()
            }
        };
        let chunk = chunk
            .with_context(|| format!("{referrer} holds none of the scene's static content"))?;
        let unread = (chunk.placements.iter())
            .find(|placement| !self.mesh_bounds.contains_key(&placement.mesh));
        if let Some(placement) = unread {
            tracing::warn!(chunk = %referrer, mesh = %placement.mesh, "a Placement's Mesh is not among the sources this cook reads; the Chunk's Blob, if any, stays as it is");
        }
        let kept = unread.is_some();
        Ok(GeneratedChunk {
            id: scene.chunk_id(coordinates),
            referrer,
            cook_key: chunk::cook_key(&chunk),
            chunk,
            kept,
        })
    }

    /// Whether the Blob `reference` names may be one a cook kept as it is:
    /// a Chunk naming a Mesh this cook did not read, or an Asset whose source
    /// is absent. A glTF file's Materials sample its own images, and a
    /// Primitive names a Material of its own file or scene, so only a
    /// Texture whose image this cook did not read may be kept; a scene's
    /// references, only while one of its own sources is.
    fn may_be_kept(&self, (referrer, id): &(Referrer, AssetId)) -> bool {
        if self.kept_chunks.contains(id) {
            return true;
        }
        match referrer {
            Referrer::Scene(scene) | Referrer::Chunk(scene, _) => scene
                .source_files()
                .iter()
                .any(|guid| self.kept_sources.contains(guid)),
            Referrer::Material(_) | Referrer::Mesh(_) => self.kept_textures.contains(id),
        }
    }
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
    let mut sources = read_sources(assets, scenes, None)?;
    let references = scene_references(scenes)?;
    cook_sources(&mut sources, &references, cache, settings)
}

/// Cooks `scene` whole, as `praxis cook <scene>`: the manifest sources the
/// scene references, those present, and its procedural Assets. A source the
/// manifest marks fetched is fetched first when absent. The Blobs of a
/// source still absent, or of every source when the manifest is absent, stay
/// as they are.
pub fn cook_scene(
    scene: ReferenceScene,
    assets: &Path,
    cache: &CookedCache,
    settings: &CookSettings,
) -> anyhow::Result<CookReport> {
    fetch::fetch_for_launch(assets, scene.source_files());
    let mut sources = read_sources(assets, &[scene], Some(scene.source_files()))?;
    let references = scene_references(&[scene])?;
    cook_sources(&mut sources, &references, cache, settings)
}

/// Each Asset ID `scenes` load as they spawn, before any Chunk, once per
/// scene; a Chunk names what its Placements load.
fn scene_references(scenes: &[ReferenceScene]) -> anyhow::Result<Vec<(Referrer, AssetId)>> {
    let mut references = Vec::new();
    for &scene in scenes {
        let mut recorder = AssetRecorder::default();
        // The default crowd form loads the shadow proxies besides every
        // Asset the traced forms load.
        Simulation::spawned(
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
    sources: &mut Sources,
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
            cache.write(*id, cook_key, &mesh_blob(*id, name, primitives)?)?;
            report.cooked.push(*id);
        }
    }
    let (mut placement_references, mut kept_chunks) = (Vec::new(), Vec::new());
    for (scene, coordinates) in sources.chunks.clone() {
        let generated = sources.chunk(scene, coordinates)?;
        placement_references.extend(generated.references());
        let (id, cook_key) = (generated.id, &generated.cook_key);
        if generated.kept {
            kept_chunks.push(id);
        } else if is_fresh(cache, id, cook_key) {
            report.unchanged.push(id);
        } else {
            cache.write(id, cook_key, &generated.chunk.to_blob()?)?;
            report.cooked.push(id);
        }
    }
    sources.kept_chunks.extend(kept_chunks);
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
    let may_be_kept = |reference: &(Referrer, AssetId)| sources.may_be_kept(reference);
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
            .map(|(name, channel, generate)| {
                let id = scene.texture_id(channel, &name);
                let name = format!("{} {name}", scene.name());
                TextureSource::generated(id, channel, name, generate)
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
    sources.mesh_bounds = (sources.meshes.iter())
        .map(|(id, _, _, primitives)| (*id, chunk::source_bounds(primitives)))
        .collect();
    for &scene in scenes {
        let coordinates = scene.chunk_coordinates().into_iter();
        sources
            .chunks
            .extend(coordinates.map(|coordinates| (scene, coordinates)));
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

/// The Blob of Mesh `id`, named `name` in messages, cooked from
/// `primitives`.
fn mesh_blob(
    id: AssetId,
    name: &str,
    primitives: &[mesh::SourcePrimitive],
) -> anyhow::Result<Vec<u8>> {
    mesh::cook(primitives)
        .and_then(|mesh| mesh.to_blob())
        .with_context(|| format!("cooking Mesh {name} ({id})"))
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
    let extensions: Vec<&str> = document.extensions_used().collect();
    if !extensions.is_empty() {
        tracing::warn!(path = %path.display(), ?extensions, "the cook reads no glTF extension; what these add is dropped");
    }
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
        let scene = ReferenceScene::PbrSweep;
        let (placements, _) = scene.static_content();
        let placed = (placements.iter())
            .find(|placement| crate::cooked::chunk_holding(placement.translation) == [0, 0])
            .unwrap();
        let (removed_mesh, removed_material, removed_texture) =
            (placed.mesh, placed.material.unwrap(), AssetId(3));
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
        let chunk_referrer = Referrer::Chunk(scene, [0, 0]);
        let mut sources = Sources {
            materials: vec![(material_id, "material cook key".into(), material)],
            chunks: vec![(scene, [0, 0])],
            mesh_bounds: HashMap::from([(removed_mesh, [glam::Vec3::ZERO; 2])]),
            ..Sources::default()
        };
        let report = cook_sources(&mut sources, &[], &cache, &CookSettings::default()).unwrap();
        let dangling = &report.dangling_references;
        assert_eq!(
            dangling[0],
            (Referrer::Material(material_id), removed_texture)
        );
        for removed in [removed_mesh, removed_material] {
            assert!(
                dangling.contains(&(chunk_referrer, removed)),
                "{dangling:?}"
            );
        }
        assert_eq!(
            chunk_referrer.to_string(),
            "Chunk (0, 0) of Reference scene pbr-sweep"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    /// texture-wall loads its Textures by Asset ID, with no Material naming
    /// them, so its references come from the Textures its spawn loads.
    #[test]
    fn a_scene_references_the_textures_it_loads() {
        let scene = ReferenceScene::TextureWall;
        let references = scene_references(&[scene]).unwrap();
        let texture = scene.texture_id(MaterialChannel::Albedo, "texture-0");
        assert!(references.contains(&(Referrer::Scene(scene), texture)));
    }

    /// `praxis cook world` cooks 1,600 Chunks, and `world`'s Textures, each
    /// of one size and cooked to the full mip chain one of them cooks to,
    /// take more than 8,000,000,000 bytes of GPU memory together: more than
    /// the texture pool's Budget, so its residency must free and evict.
    #[test]
    fn the_world_cooks_1600_chunks_and_over_8_gb_of_unique_textures() {
        let scene = ReferenceScene::World;
        let sources = read_sources(Path::new(ASSETS_DIRECTORY), &[scene], Some(&[])).unwrap();
        assert_eq!(sources.chunks.len(), 1600);

        let textures = &sources.textures;
        let ids: HashSet<_> = textures.iter().map(|texture| texture.id).collect();
        assert_eq!(ids.len(), textures.len());
        let samples = [0, 1, 24, textures.len() - 1].map(|index| textures[index].decode().unwrap());
        for (index, sample) in samples.iter().enumerate() {
            for other in &samples[index + 1..] {
                assert_ne!(
                    sample.pixels, other.pixels,
                    "two of `world`'s Textures match"
                );
            }
        }
        let side = samples[0].width;
        assert!(
            samples
                .iter()
                .all(|sample| (sample.width, sample.height) == (side, side))
        );
        let threads = std::thread::available_parallelism().map_or(1, |threads| threads.get());
        let blob = textures[0].cook(&CookSettings::default(), threads).unwrap();
        let cooked = crate::cooked::CookedTexture::from_blob(&blob).unwrap();
        assert_eq!(
            (cooked.width, cooked.height, cooked.first_level),
            (side, side, 0)
        );
        let chain_bytes: usize = cooked.levels.iter().map(Vec::len).sum();
        let library_bytes = chain_bytes * textures.len();
        assert!(library_bytes > 8_000_000_000, "{library_bytes} bytes");
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

    /// A glTF source that uses extensions logs one warning naming it and
    /// each of them; a source that uses none logs nothing.
    #[test]
    fn a_source_using_gltf_extensions_warns_naming_each() {
        let read = |source: &str| {
            crate::assets::pools::tests::logged(|| {
                let mut sources = Sources::default();
                read_gltf(
                    1,
                    Path::new("lacquered.gltf"),
                    source.as_bytes(),
                    &mut sources,
                )
            })
        };

        let (result, log) = read(
            r#"{"asset":{"version":"2.0"},"extensionsUsed":["KHR_materials_clearcoat","KHR_texture_transform"]}"#,
        );
        result.unwrap();
        let warnings: Vec<&str> = log.lines().filter(|line| line.contains("WARN")).collect();
        assert_eq!(warnings.len(), 1, "{log:?}");
        for named in [
            "lacquered.gltf",
            "KHR_materials_clearcoat",
            "KHR_texture_transform",
        ] {
            assert!(warnings[0].contains(named), "{named} not named: {log:?}");
        }

        let (result, log) = read(r#"{"asset":{"version":"2.0"}}"#);
        result.unwrap();
        assert_eq!(log, "");
    }
}
