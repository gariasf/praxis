//! What a dev launch cooks (ADR 0018): each Asset it loads whose Blob is
//! missing or stale, as it loads it, with every Asset that one names.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::{
    CookSettings, Referrer, Sources, dangling_references, fetch, is_fresh, mesh_blob, read_sources,
};
use crate::ReferenceScene;
use crate::assets::{AssetLoader, MaterialChannel, MaterialHandle, MeshHandle};
use crate::cooked::{AssetId, CookedCache, CookedChunk, TEXTURE_FORMAT_VERSION};

/// Loads through another loader, first cooking each Asset a load asks for,
/// and each Asset it names, whose Blob is missing or stale.
pub(crate) struct CookingLoader<'a> {
    loader: &'a mut dyn AssetLoader,
    cache: &'a CookedCache,
    cook: LaunchCook,
}

/// What a launch cooks from: its scene's sources, and each Asset it has
/// checked.
pub(crate) struct LaunchCook {
    settings: CookSettings,
    scene: ReferenceScene,
    sources: Sources,
    /// Where each Asset this cook may write lies in `sources`.
    entry_by_id: HashMap<AssetId, SourceEntry>,
    /// Each Asset checked so far, once per launch.
    checked: HashSet<AssetId>,
}

/// An Asset by its kind and its index in `Sources`.
#[derive(Copy, Clone)]
enum SourceEntry {
    Material(usize),
    Mesh(usize),
    Chunk(usize),
    Texture(usize),
}

impl<'a> CookingLoader<'a> {
    /// A loader of `scene`'s Assets through `loader` that cooks them from
    /// `assets` into `cache` as they load, fetching an absent fetched source
    /// first. It reads the scene's sources and cooks nothing yet.
    pub(crate) fn new(
        scene: ReferenceScene,
        assets: &Path,
        cache: &'a CookedCache,
        settings: CookSettings,
        loader: &'a mut dyn AssetLoader,
    ) -> anyhow::Result<Self> {
        fetch::fetch_for_launch(assets, scene.source_files());
        let sources = read_sources(assets, &[scene], Some(scene.source_files()))?;
        Ok(Self::from_sources(scene, sources, cache, settings, loader))
    }

    /// A loader that cooks `scene`'s Assets from `sources`; a Chunk they
    /// keep is never cooked.
    fn from_sources(
        scene: ReferenceScene,
        sources: Sources,
        cache: &'a CookedCache,
        settings: CookSettings,
        loader: &'a mut dyn AssetLoader,
    ) -> Self {
        let materials = sources.materials.iter().enumerate();
        let materials = materials.map(|(index, (id, ..))| (*id, SourceEntry::Material(index)));
        let meshes = sources.meshes.iter().enumerate();
        let meshes = meshes.map(|(index, (id, ..))| (*id, SourceEntry::Mesh(index)));
        let chunks = (sources.chunks.iter().enumerate())
            .filter(|(_, (id, ..))| !sources.kept_chunks.contains(id))
            .map(|(index, (id, ..))| (*id, SourceEntry::Chunk(index)));
        let textures = sources.textures.iter().enumerate();
        let textures = textures.map(|(index, texture)| (texture.id, SourceEntry::Texture(index)));
        let entry_by_id = materials
            .chain(meshes)
            .chain(chunks)
            .chain(textures)
            .collect();
        let cook = LaunchCook {
            settings,
            scene,
            sources,
            entry_by_id,
            checked: HashSet::new(),
        };
        Self {
            loader,
            cache,
            cook,
        }
    }

    /// Each source the launch could not read, whose Blobs stay as they are.
    pub(crate) fn absent_sources(&self) -> &[String] {
        &self.cook.sources.absent
    }

    /// What it cooks from, for the launch's later loads.
    pub(crate) fn into_cook(self) -> LaunchCook {
        self.cook
    }

    fn cook_for_spawn(&mut self, id: AssetId) -> anyhow::Result<()> {
        self.cook.cook_for_scene(self.cache, id)
    }
}

impl LaunchCook {
    /// Cooks `id` into `cache`, and each Asset it names, where missing or
    /// stale, as the scene names it.
    pub(crate) fn cook_for_scene(
        &mut self,
        cache: &CookedCache,
        id: AssetId,
    ) -> anyhow::Result<()> {
        self.cook_on_miss(cache, Referrer::Scene(self.scene), id)
    }

    /// Cooks `id`, which `referrer` names, and each Asset it names, into
    /// `cache` where the Blob is missing or stale. An ID this cook does not
    /// make warns, as the cook's dangling references do, unless its Blob may
    /// be one kept.
    fn cook_on_miss(
        &mut self,
        cache: &CookedCache,
        referrer: Referrer,
        id: AssetId,
    ) -> anyhow::Result<()> {
        if !self.checked.insert(id) {
            return Ok(());
        }
        let Some(&entry) = self.entry_by_id.get(&id) else {
            let may_be_kept = |reference: &(Referrer, AssetId)| self.sources.may_be_kept(reference);
            dangling_references(cache, &HashSet::new(), &[(referrer, id)], may_be_kept);
            return Ok(());
        };
        let sources = &self.sources;
        let (cook_key, named): (String, Vec<_>) = match entry {
            SourceEntry::Material(index) => {
                let (_, cook_key, material) = &sources.materials[index];
                let textures = material.textures.iter().flatten();
                let named = textures.map(|texture| (Referrer::Material(id), *texture));
                (cook_key.clone(), named.collect())
            }
            SourceEntry::Mesh(index) => {
                let (_, _, cook_key, primitives) = &sources.meshes[index];
                let materials = primitives.iter().filter_map(|(_, material)| *material);
                let named = materials.map(|material| (Referrer::Mesh(id), material));
                (cook_key.clone(), named.collect())
            }
            SourceEntry::Chunk(index) => {
                let (_, referrer, cook_key, chunk) = &sources.chunks[index];
                let placements = chunk.placements.iter();
                let named =
                    placements.flat_map(|placement| [Some(placement.mesh), placement.material]);
                let named = named.flatten().map(|named| (*referrer, named));
                (cook_key.clone(), named.collect())
            }
            SourceEntry::Texture(index) => {
                let texture = &sources.textures[index];
                let cook_key = texture.cook_key(&self.settings, TEXTURE_FORMAT_VERSION);
                (cook_key, Vec::new())
            }
        };
        for (referrer, named) in named {
            self.cook_on_miss(cache, referrer, named)?;
        }
        if is_fresh(cache, id, &cook_key) {
            return Ok(());
        }
        let sources = &self.sources;
        let blob = match entry {
            SourceEntry::Material(index) => {
                let (_, _, material) = &sources.materials[index];
                material.to_blob()
            }
            SourceEntry::Mesh(index) => {
                let (_, name, _, primitives) = &sources.meshes[index];
                mesh_blob(id, name, primitives)?
            }
            SourceEntry::Chunk(index) => {
                let (_, _, _, chunk) = &sources.chunks[index];
                chunk.to_blob()?
            }
            SourceEntry::Texture(index) => {
                let threads =
                    std::thread::available_parallelism().map_or(1, |threads| threads.get());
                sources.textures[index].cook(&self.settings, threads)?
            }
        };
        cache.write(id, &cook_key, &blob)?;
        tracing::debug!(asset = %id, "cooked on a miss");
        Ok(())
    }
}

impl AssetLoader for CookingLoader<'_> {
    fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle> {
        self.cook_for_spawn(id)?;
        self.loader.material(id)
    }

    fn texture(&mut self, channel: MaterialChannel, id: AssetId) -> anyhow::Result<u32> {
        self.cook_for_spawn(id)?;
        self.loader.texture(channel, id)
    }

    fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.cook_for_spawn(id)?;
        self.loader.mesh(id)
    }

    fn bent_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.cook_for_spawn(id)?;
        self.loader.bent_mesh(id)
    }

    fn refitted_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.cook_for_spawn(id)?;
        self.loader.refitted_mesh(id)
    }

    fn shadow_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.cook_for_spawn(id)?;
        self.loader.shadow_proxy(id)
    }

    fn chunk(&mut self, id: AssetId) -> anyhow::Result<CookedChunk> {
        self.cook_for_spawn(id)?;
        self.loader.chunk(id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::*;
    use crate::assets::pools::tests::{gpu_context, logged};
    use crate::assets::{MaterialPool, MeshPool, PoolCapacities, PoolLoader};
    use crate::cook::{chunk, mesh};
    use crate::cooked::CookedPlacement;
    use crate::render::LoadStaging;
    use crate::scenes::ground_plane;
    use crate::stats::gpu_memory::tests::gpu_memory;
    use crate::streaming::load_chunk;

    /// A scratch directory for one test in one process.
    fn scratch(test: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("praxis-launch-{test}-{}", std::process::id()));
        std::fs::remove_dir_all(&directory).ok();
        directory
    }

    /// Runs `load` with a loader into fresh Pools from `cache`; without a
    /// GPU adapter, returns false and runs nothing.
    fn with_pool_loader(cache: &CookedCache, load: impl FnOnce(&mut PoolLoader)) -> bool {
        let Some(context) = gpu_context() else {
            return false;
        };
        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let capacities = PoolCapacities::checked_in();
        let mut mesh_pool = MeshPool::new(&context.device, &memory, &capacities);
        let mut material_pool =
            MaterialPool::new(&context.device, &memory, &mut staging, &capacities, false);
        load(&mut PoolLoader {
            device: &context.device,
            staging: &mut staging,
            reader: cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        });
        true
    }

    /// The names of the files in `directory`, none when it is absent.
    fn file_names(directory: &Path) -> BTreeSet<String> {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return BTreeSet::new();
        };
        let names = entries.map(|entry| entry.unwrap().file_name().into_string().unwrap());
        names.collect()
    }

    /// A launch cooks nothing before it loads. Loading one Chunk into an
    /// empty Cooked cache then cooks that Chunk and each Mesh, Material and
    /// Texture it names, and no other Blob: none of town's other 15 Chunks
    /// or the Meshes only they place, none of uv-grid's other 3, and none of
    /// `world`'s other 1,599 Chunks or the Textures only they name.
    #[test]
    fn loading_one_chunk_cooks_that_chunk_and_the_assets_it_names() {
        let mut textures_cooked = 0;
        for scene in [
            ReferenceScene::Town,
            ReferenceScene::UvGrid,
            ReferenceScene::World,
        ] {
            let directory = scratch(&format!("one-chunk-{}", scene.name()));
            let cache_path = directory.join("cooked");
            let cache = CookedCache::new(&cache_path);
            let (coordinates, id) = ([0, 0], scene.chunk_id([0, 0]));
            let loaded = with_pool_loader(&cache, |pool_loader| {
                let assets = directory.join("assets");
                let settings = CookSettings::default();
                let mut loader =
                    CookingLoader::new(scene, &assets, &cache, settings, pool_loader).unwrap();
                assert_eq!(file_names(&cache_path.join("index")), BTreeSet::new());
                load_chunk(&mut loader, id, coordinates).unwrap();
            });
            if !loaded {
                return;
            }

            let chunk = cache.chunk(id).unwrap();
            let meshes = chunk.placements.iter().map(|placement| placement.mesh);
            let meshes: BTreeSet<_> = meshes.collect();
            let mut materials: BTreeSet<_> = chunk
                .placements
                .iter()
                .filter_map(|placement| placement.material)
                .collect();
            for mesh in &meshes {
                let primitives = cache.mesh(*mesh).unwrap().primitives;
                materials.extend(primitives.iter().filter_map(|primitive| primitive.material));
            }
            let mut textures = BTreeSet::new();
            for material in &materials {
                let named = cache.material(*material).unwrap().textures;
                textures.extend(named.into_iter().flatten());
            }
            textures_cooked += textures.len();
            let expected: BTreeSet<_> = [id]
                .into_iter()
                .chain(meshes)
                .chain(materials)
                .chain(textures)
                .collect();
            let expected_names: BTreeSet<_> = expected.iter().map(AssetId::to_string).collect();
            assert_eq!(file_names(&cache_path.join("index")), expected_names);
            let expected_blobs: BTreeSet<_> = expected
                .iter()
                .map(|id| cache.entry(*id).unwrap().blob)
                .collect();
            assert_eq!(file_names(&cache_path.join("blobs")), expected_blobs);
            std::fs::remove_dir_all(&directory).unwrap();
        }
        assert!(textures_cooked > 0, "no loaded Chunk named a Texture");
    }

    /// The sources of Mesh 1, a plane `half_size` metres to each edge, and of
    /// Chunk 2, uv-grid's Chunk (0, 0), which places it at the World's origin.
    fn plane_sources(half_size: f32) -> Sources {
        let mesh = AssetId(1);
        let primitives = vec![(ground_plane(half_size, 1.0), None)];
        let mesh_bounds = HashMap::from([(mesh, chunk::source_bounds(&primitives))]);
        let placement = CookedPlacement {
            mesh,
            material: None,
            translation: glam::DVec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };
        let referrer = Referrer::Chunk(ReferenceScene::UvGrid, [0, 0]);
        let chunks = chunk::chunks(vec![placement], Vec::new(), &mesh_bounds).into_values();
        let chunks = chunks.map(|chunk| (AssetId(2), referrer, chunk::cook_key(&chunk), chunk));
        Sources {
            meshes: vec![(
                mesh,
                "plane".into(),
                mesh::cook_key(&primitives),
                primitives,
            )],
            chunks: chunks.collect(),
            ..Sources::default()
        }
    }

    /// A Mesh whose source changed after its Blob was cooked keeps that Blob
    /// through the launch's start, and re-cooks when a Chunk placing it
    /// loads, as does the Chunk, whose bounds grew with it.
    #[test]
    fn a_changed_mesh_re_cooks_when_a_chunk_placing_it_loads() {
        let directory = scratch("changed-mesh");
        let cache = CookedCache::new(directory.join("cooked"));
        let (scene, mesh, chunk) = (ReferenceScene::UvGrid, AssetId(1), AssetId(2));
        let cook_keys = || [mesh, chunk].map(|id| cache.entry(id).unwrap().cook_key);
        let settings = CookSettings::default();
        let loaded = with_pool_loader(&cache, |pool_loader| {
            let sources = plane_sources(1.0);
            let mut loader =
                CookingLoader::from_sources(scene, sources, &cache, settings, pool_loader);
            load_chunk(&mut loader, chunk, [0, 0]).unwrap();
            let cooked = cook_keys();

            let changed = plane_sources(2.0);
            let (_, _, mesh_key, _) = &changed.meshes[0];
            let (_, _, chunk_key, _) = &changed.chunks[0];
            let changed_keys = [mesh_key.clone(), chunk_key.clone()];
            assert_ne!(changed_keys, cooked);
            let mut loader =
                CookingLoader::from_sources(scene, changed, &cache, settings, pool_loader);
            assert_eq!(cook_keys(), cooked, "a launch cooked before it loaded");
            load_chunk(&mut loader, chunk, [0, 0]).unwrap();
            assert_eq!(cook_keys(), changed_keys);
        });
        if loaded {
            assert_eq!(cache.mesh(mesh).unwrap().bounds[1], [2.0, 0.0, 2.0]);
            assert_eq!(cache.chunk(chunk).unwrap().bounds[1].x, 2.0);
        }
        std::fs::remove_dir_all(&directory).ok();
    }

    /// A Chunk placing a Mesh no source defines any more warns, naming the
    /// Chunk and the Mesh, and loads the Blob the Cooked cache still holds.
    #[test]
    fn a_chunk_placing_a_removed_mesh_warns_and_loads_its_stale_blob() {
        let directory = scratch("removed-mesh");
        let cache = CookedCache::new(directory.join("cooked"));
        let (scene, mesh, chunk) = (ReferenceScene::UvGrid, AssetId(1), AssetId(2));
        let settings = CookSettings::default();
        with_pool_loader(&cache, |pool_loader| {
            let sources = plane_sources(1.0);
            let mut loader =
                CookingLoader::from_sources(scene, sources, &cache, settings, pool_loader);
            load_chunk(&mut loader, chunk, [0, 0]).unwrap();

            let chunks = plane_sources(1.0).chunks;
            let removed = Sources {
                chunks,
                ..Sources::default()
            };
            let mut loader =
                CookingLoader::from_sources(scene, removed, &cache, settings, pool_loader);
            let (loaded, log) = logged(|| load_chunk(&mut loader, chunk, [0, 0]));
            loaded.unwrap();
            let warning = "references an Asset ID this cook does not make";
            for expected in [
                warning,
                "Chunk (0, 0) of Reference scene uv-grid",
                &mesh.to_string(),
            ] {
                assert!(log.contains(expected), "{expected:?} missing from {log}");
            }
        });
        std::fs::remove_dir_all(&directory).ok();
    }
}
