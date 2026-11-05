use anyhow::Context;

use crate::assets::{MaterialChannel, MaterialHandle, MaterialPool, MeshHandle, MeshPool, MeshUse};
use crate::cooked::{AssetId, CookedCache, CookedMesh};
use crate::render::LoadStaging;

/// What a scene's spawn loads its Assets through, by Asset ID (ADR 0018),
/// and the Handles its Renderables hold.
pub trait AssetLoader {
    /// Loads the Material `id` with the Textures it samples.
    fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle>;

    /// Loads the Texture `id` into `channel`, as a Material sampling it
    /// would, and returns its Layer.
    fn texture(&mut self, channel: MaterialChannel, id: AssetId) -> anyhow::Result<u32>;

    /// Loads the Mesh `id` with the Materials its Primitives name, each
    /// loaded once, which each Primitive draws with.
    fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Loads the Mesh `id` as `mesh` does, for the GPU to bend each frame
    /// (`Bend`): drawn, and traced by no ray, as its BLAS would hold its rest
    /// shape. Its cooked bounds must hold every shape it bends to.
    fn bent_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Loads the Mesh `id` as `bent_mesh` does, and traced through a BLAS
    /// updated each frame from the shape the GPU bends it to.
    fn refitted_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Loads the Mesh `id` as a shadow proxy, which rays trace through its
    /// BLAS and no pass draws, so it loads no Material.
    fn shadow_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;
}

/// Loads Assets from a Cooked cache into the Pools, through the load's
/// staging.
pub struct PoolLoader<'a> {
    pub device: &'a wgpu::Device,
    pub staging: &'a mut LoadStaging,
    pub cache: &'a CookedCache,
    pub mesh_pool: &'a mut MeshPool,
    pub material_pool: &'a mut MaterialPool,
}

impl AssetLoader for PoolLoader<'_> {
    fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle> {
        self.material_pool
            .load(self.device, self.staging, self.cache, id)
    }

    fn texture(&mut self, channel: MaterialChannel, id: AssetId) -> anyhow::Result<u32> {
        self.material_pool
            .load_texture(self.device, self.staging, self.cache, channel, id)
    }

    fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::DrawnAndTraced)
    }

    fn bent_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::Drawn)
    }

    fn refitted_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::Refitted)
    }

    fn shadow_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::Traced)
    }
}

impl PoolLoader<'_> {
    /// Loads the Mesh `id` for `mesh_use`, once per Asset ID unless it is
    /// bent, with the Materials its Primitives name unless no pass draws it.
    fn mesh_for(&mut self, id: AssetId, mesh_use: MeshUse) -> anyhow::Result<MeshHandle> {
        if let Some(handle) = self.mesh_pool.handle(id, mesh_use) {
            return Ok(handle);
        }
        let mesh = self.cache.mesh(id)?;
        let materials = if mesh_use.drawn() {
            self.materials(id, &mesh)?
        } else {
            vec![None; mesh.primitives.len()]
        };
        self.mesh_pool
            .load_asset(self.staging, id, &mesh, &materials, mesh_use)
            .with_context(|| format!("the Mesh pool refused Mesh {id}"))
    }

    /// The Materials `mesh`'s Primitives name, Mesh `id`'s.
    fn materials(
        &mut self,
        id: AssetId,
        mesh: &CookedMesh,
    ) -> anyhow::Result<Vec<Option<MaterialHandle>>> {
        mesh.primitives
            .iter()
            .map(|primitive| {
                primitive
                    .material
                    .map(|material| {
                        self.material(material)
                            .with_context(|| format!("loading a Material of Mesh {id}"))
                    })
                    .transpose()
            })
            .collect()
    }
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::assets::ARENA_CAPACITY_BYTES;
    use crate::assets::pools::tests::{gpu_context, logged};
    use crate::cooked::{CookedMaterial, CookedMesh, CookedMeshlet, CookedPrimitive, CookedVertex};

    #[test]
    fn an_asset_loaded_twice_keeps_one_entry_and_a_bent_mesh_its_own() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-asset-loader-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        for id in [1, 2, 6] {
            cache
                .write(AssetId(id), "key", &material.to_blob())
                .unwrap();
        }
        // Mesh 3's Primitives both name Material 1; Mesh 4's name 2, then 1;
        // Mesh 5's 2, then 9, which the cache lacks.
        for (id, materials) in [(3, [1, 1]), (4, [2, 1]), (5, [2, 9])] {
            let mesh = CookedMesh {
                vertices: vec![CookedVertex::default(); 3],
                indices: vec![0, 1, 2],
                primitives: materials
                    .map(|material| CookedPrimitive {
                        first_index: 0,
                        index_count: 3,
                        material: Some(AssetId(material)),
                        meshlets: vec![CookedMeshlet {
                            vertices: vec![0, 1, 2],
                            triangles: vec![[0, 1, 2]],
                        }],
                    })
                    .into(),
                bounds: [[0.0; 3]; 2],
            };
            cache
                .write(AssetId(id), "key", &mesh.to_blob().unwrap())
                .unwrap();
        }

        let mut staging = LoadStaging::new(context.device.clone());
        let mut mesh_pool = MeshPool::new(&context.device, ARENA_CAPACITY_BYTES);
        let mut material_pool = MaterialPool::new(&context.device, &mut staging);
        let mut loader = PoolLoader {
            device: &context.device,
            staging: &mut staging,
            cache: &cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        };
        let (handles, log) = logged(|| {
            let mesh = loader.mesh(AssetId(3)).unwrap();
            let arena_bytes = loader.mesh_pool.arena_bytes_in_use();
            assert_eq!(loader.mesh(AssetId(3)).unwrap(), mesh);
            assert_eq!(loader.mesh_pool.arena_bytes_in_use(), arena_bytes);
            let proxy = loader.shadow_proxy(AssetId(3)).unwrap();
            assert_eq!(loader.shadow_proxy(AssetId(3)).unwrap(), proxy);
            let bent = [
                loader.bent_mesh(AssetId(3)).unwrap(),
                loader.bent_mesh(AssetId(3)).unwrap(),
                loader.refitted_mesh(AssetId(3)).unwrap(),
                loader.refitted_mesh(AssetId(3)).unwrap(),
            ];
            let next = loader.mesh(AssetId(4)).unwrap();
            [mesh, proxy]
                .into_iter()
                .chain(bent)
                .chain([next])
                .map(|handle| handle.0)
                .collect::<Vec<_>>()
        });
        let Err(missing) = loader.mesh(AssetId(5)) else {
            panic!("Mesh 5 loaded without its second Primitive's Material");
        };

        assert!(!log.contains("WARN"), "{log}");
        // Mesh 3 and its shadow proxy hold one entry each however often
        // they load, and each bent Mesh its own, so Mesh 4 takes the next.
        assert_eq!(handles, [0, 1, 2, 3, 4, 5, 6]);
        assert!(format!("{missing:#}").contains("loading a Material of Mesh"));
        // Materials 1 and 2 load once each, however many Primitives name
        // them, so Material 6 takes the third entry.
        let materials = [1, 2, 1, 6].map(|id| loader.material(AssetId(id)).unwrap().index());
        assert_eq!(materials, [0, 1, 0, 2]);
        std::fs::remove_dir_all(&directory).ok();
    }
}
