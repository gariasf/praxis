use anyhow::Context;

use crate::assets::{
    MaterialChannel, MaterialHandle, MaterialPool, MaterialUse, MeshHandle, MeshPool, MeshUse,
};
use crate::cooked::{AssetId, CookedCache, CookedChunk, CookedMaterial, CookedMesh, CookedTexture};
use crate::render::LoadStaging;

/// What a scene's spawn loads its Assets through, by Asset ID (ADR 0018),
/// and the Handles its Renderables hold.
pub trait AssetLoader {
    /// Loads the Material `id` with the Textures it samples, or counts one
    /// more user of it if it is loaded. A Material pool without room frees
    /// unused Meshes, least recently used first, until a Material they named
    /// is unused.
    fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle>;

    /// Loads the Texture `id` into `channel`, as a Material sampling it
    /// would, and returns its Layer.
    fn texture(&mut self, channel: MaterialChannel, id: AssetId) -> anyhow::Result<u32>;

    /// Loads the Mesh `id` with the Materials its Primitives name, each
    /// loaded once, which each Primitive draws with; or counts one more user
    /// of it if it is loaded.
    fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Loads the Mesh `id` as `mesh` does, for the GPU to bend each frame
    /// (`Bend`): drawn, and traced by no ray, as its BLAS would hold its rest
    /// shape. Its cooked bounds must hold every shape it bends to.
    fn bent_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Loads the Mesh `id` as `bent_mesh` does, a copy of its own that no
    /// ray traces: one that casts no shadow where it is placed.
    fn untraced_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.bent_mesh(id)
    }

    /// Loads the Mesh `id` as `bent_mesh` does, and traced through a BLAS
    /// updated each frame from the shape the GPU bends it to.
    fn refitted_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Loads the Mesh `id` as a shadow proxy, which rays trace through its
    /// BLAS and no pass draws, so it loads no Material.
    fn shadow_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle>;

    /// Reads the Chunk `id`, loading none of the Assets it names.
    #[cfg_attr(not(feature = "cook"), allow(dead_code))]
    fn chunk(&mut self, id: AssetId) -> anyhow::Result<CookedChunk>;

    /// Counts one user fewer of each of `meshes`, loaded by `mesh`, and of
    /// `materials`: what a load that failed after loading them gives back.
    fn remove_users(&mut self, meshes: &[AssetId], materials: &[AssetId]);
}

/// Where the Pools read decoded Assets from, by Asset ID: the Cooked cache,
/// or what the IO pool decoded.
pub trait AssetReader {
    fn material(&self, id: AssetId) -> anyhow::Result<CookedMaterial>;
    /// The Texture `id` with every mip, or its Resident tail alone where the
    /// IO pool decoded it for a Chunk.
    fn texture(&self, id: AssetId) -> anyhow::Result<CookedTexture>;
    fn mesh(&self, id: AssetId) -> anyhow::Result<CookedMesh>;
    #[cfg_attr(not(feature = "cook"), allow(dead_code))]
    fn chunk(&self, id: AssetId) -> anyhow::Result<CookedChunk>;

    /// Hears that the pools freed `assets`, so a later read of one loads it
    /// afresh.
    fn forget(&self, _assets: &[AssetId]) {}
}

impl AssetReader for CookedCache {
    fn material(&self, id: AssetId) -> anyhow::Result<CookedMaterial> {
        CookedCache::material(self, id)
    }

    fn texture(&self, id: AssetId) -> anyhow::Result<CookedTexture> {
        CookedCache::texture(self, id)
    }

    fn mesh(&self, id: AssetId) -> anyhow::Result<CookedMesh> {
        CookedCache::mesh(self, id)
    }

    fn chunk(&self, id: AssetId) -> anyhow::Result<CookedChunk> {
        CookedCache::chunk(self, id)
    }
}

/// Loads Assets from `reader` into the Pools, through the load's staging.
pub struct PoolLoader<'a> {
    pub device: &'a wgpu::Device,
    pub staging: &'a mut LoadStaging,
    pub reader: &'a dyn AssetReader,
    pub mesh_pool: &'a mut MeshPool,
    pub material_pool: &'a mut MaterialPool,
}

impl AssetLoader for PoolLoader<'_> {
    fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle> {
        self.material_for(id, MaterialUse::Ordinary)
    }

    fn texture(&mut self, channel: MaterialChannel, id: AssetId) -> anyhow::Result<u32> {
        let loaded = self.material_pool.load_texture(
            self.device,
            self.staging,
            self.reader,
            channel,
            id,
            &mut free_unused_mesh(self.mesh_pool, None),
        );
        self.forget_freed_meshes();
        loaded
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

    fn chunk(&mut self, id: AssetId) -> anyhow::Result<CookedChunk> {
        self.reader.chunk(id)
    }

    fn remove_users(&mut self, meshes: &[AssetId], materials: &[AssetId]) {
        for &mesh in meshes {
            self.mesh_pool.remove_user(mesh, MeshUse::DrawnAndTraced);
        }
        for &material in materials {
            self.material_pool.remove_user(material);
        }
    }
}

impl PoolLoader<'_> {
    /// Loads the Mesh `id` as a Far proxy's: drawn and traced, and its first
    /// Primitive's Material, the one baked for it, in the Material
    /// table's count for Far proxies; or counts one more user of it if it is
    /// loaded.
    pub(crate) fn far_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::FarProxy)
    }

    /// Loads the Material `id` as `AssetLoader::material` does, into an entry
    /// of `material_use`'s count. A Far proxy's Material without room frees
    /// unused Far proxies' Meshes alone, as only they hold its count's
    /// entries.
    fn material_for(
        &mut self,
        id: AssetId,
        material_use: MaterialUse,
    ) -> anyhow::Result<MaterialHandle> {
        let freed_for_room = (material_use == MaterialUse::FarProxy).then_some(MeshUse::FarProxy);
        let loaded = self.material_pool.load_making_room(
            self.device,
            self.staging,
            self.reader,
            id,
            material_use,
            &mut free_unused_mesh(self.mesh_pool, freed_for_room),
        );
        self.forget_freed_meshes();
        loaded
    }

    /// Loads the Mesh `id` for `mesh_use`, once per Asset ID unless it is
    /// bent, with the Materials its Primitives name unless no pass draws it.
    fn mesh_for(&mut self, id: AssetId, mesh_use: MeshUse) -> anyhow::Result<MeshHandle> {
        if let Some(handle) = self.mesh_pool.add_user(id, mesh_use) {
            return Ok(handle);
        }
        let mesh = self.reader.mesh(id)?;
        let materials = if mesh_use.drawn() {
            self.materials(id, &mesh, mesh_use)?
        } else {
            vec![None; mesh.primitives.len()]
        };
        let mut released_materials = Vec::new();
        let handle = self.mesh_pool.load_asset(
            self.staging,
            id,
            &mesh,
            &materials,
            mesh_use,
            &mut released_materials,
        );
        for material in released_materials {
            self.material_pool.remove_user(material);
        }
        self.forget_freed_meshes();
        handle.with_context(|| format!("the Mesh pool refused Mesh {id}"))
    }

    /// Tells the reader which Meshes the Mesh pool freed, before anything
    /// reads one again.
    fn forget_freed_meshes(&mut self) {
        let freed = self.mesh_pool.take_freed();
        if !freed.is_empty() {
            self.reader.forget(&freed);
        }
    }

    /// The Materials `mesh`'s Primitives name, Mesh `id`'s, loaded for
    /// `mesh_use`: the first Primitive's in the Material table's count for
    /// Far proxies when it is a Far proxy's. One that fails gives back the
    /// users of those loaded before it.
    fn materials(
        &mut self,
        id: AssetId,
        mesh: &CookedMesh,
        mesh_use: MeshUse,
    ) -> anyhow::Result<Vec<Option<MaterialHandle>>> {
        let mut handles = Vec::with_capacity(mesh.primitives.len());
        for (index, primitive) in mesh.primitives.iter().enumerate() {
            let Some(material) = primitive.material else {
                handles.push(None);
                continue;
            };
            let material_use = if mesh_use == MeshUse::FarProxy && index == 0 {
                MaterialUse::FarProxy
            } else {
                MaterialUse::Ordinary
            };
            match self.material_for(material, material_use) {
                Ok(handle) => handles.push(Some(handle)),
                Err(error) => {
                    let loaded = mesh.primitives.iter().zip(&handles);
                    for material in
                        loaded.filter_map(|(primitive, handle)| handle.and(primitive.material))
                    {
                        self.material_pool.remove_user(material);
                    }
                    return Err(error.context(format!("loading a Material of Mesh {id}")));
                }
            }
        }
        Ok(handles)
    }
}

/// What a Material pool load calls once it has no unused Material left to
/// free: frees `mesh_pool`'s least recently used unused Mesh, loaded for
/// `mesh_use` when one is given, counting a user fewer of each Material it
/// named, and says whether it freed one.
fn free_unused_mesh(
    mesh_pool: &mut MeshPool,
    mesh_use: Option<MeshUse>,
) -> impl FnMut(&mut MaterialPool) -> bool + '_ {
    let mut released_materials = Vec::new();
    move |material_pool| {
        let freed = mesh_pool.free_least_recently_used(mesh_use, &mut released_materials);
        for material in released_materials.drain(..) {
            material_pool.remove_user(material);
        }
        freed
    }
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::assets::PoolCapacities;
    use crate::assets::pools::tests::{gpu_context, logged};
    use crate::cooked::{
        CookedMaterial, CookedMesh, CookedMeshlet, CookedPrimitive, CookedTexture, CookedVertex,
    };
    use crate::stats::gpu_memory::tests::gpu_memory;
    use crate::streaming::chunks::load_far_proxy;

    /// A Far proxy's Mesh loads with a BLAS, as one Placement at its
    /// Chunk's centre on the ground, open land's with an empty
    /// first Primitive; the Material baked for it takes an entry of the Far
    /// proxies' count and its ground's one of the others'. With that count
    /// full, a further Far proxy is refused while the first is in use, and
    /// frees it once it is not, leaving a Chunk's unused Mesh loaded.
    #[test]
    fn a_far_proxy_loads_traced_and_its_baked_material_makes_room_among_far_proxies_alone() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-far-proxy-loader-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        let (ground, baked, other_baked) = (AssetId(1), AssetId(2), AssetId(3));
        for id in [ground, baked, other_baked] {
            cache.write(id, "key", &material.to_blob()).unwrap();
        }
        let triangle = crate::assets::pools::tests::mesh(3, vec![0, 1, 2]);
        let primitive = |material| CookedPrimitive {
            material: Some(material),
            ..triangle.primitives[0].clone()
        };
        let empty = CookedPrimitive {
            index_count: 0,
            material: None,
            meshlets: Vec::new(),
            ..primitive(ground)
        };
        let (far_proxy, other_far_proxy, open_land, chunk_mesh) =
            (AssetId(11), AssetId(12), AssetId(13), AssetId(14));
        let shell_and_ground = |baked| vec![primitive(baked), primitive(ground)];
        for (id, primitives) in [
            (far_proxy, shell_and_ground(baked)),
            (other_far_proxy, shell_and_ground(other_baked)),
            (open_land, vec![empty, primitive(ground)]),
            (chunk_mesh, vec![primitive(ground)]),
        ] {
            let mesh = CookedMesh {
                primitives,
                ..triangle.clone()
            };
            cache.write(id, "key", &mesh.to_blob().unwrap()).unwrap();
        }
        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let capacities = PoolCapacities {
            far_proxy_materials: 1,
            ..PoolCapacities::checked_in()
        };
        let mut mesh_pool = MeshPool::new(&context.device, &memory, &capacities);
        let mut material_pool =
            MaterialPool::new(&context.device, &memory, &mut staging, &capacities, false);
        let mut loader = PoolLoader {
            device: &context.device,
            staging: &mut staging,
            reader: &cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        };

        let chunk = load_far_proxy(&mut loader, far_proxy, [2, -1]).unwrap();
        let [placement] = &chunk.placements[..] else {
            panic!("a Far proxy is one Placement");
        };
        assert_eq!(
            placement.transform.translation,
            glam::dvec3(640.0, 0.0, -128.0)
        );
        assert!(loader.mesh_pool.blas(placement.mesh).is_some());
        let open_land_proxy = loader.far_proxy(open_land).unwrap();
        let open_land_mesh = loader.mesh_pool.get(open_land_proxy).unwrap();
        assert!(
            !open_land_mesh.needs_material_handle,
            "a Primitive without triangles needs no Material"
        );
        let unused = loader.mesh(chunk_mesh).unwrap();
        loader
            .mesh_pool
            .remove_user(chunk_mesh, MeshUse::DrawnAndTraced);
        assert_eq!(loader.material_pool.users(ground), 3);

        let (refused, _) = logged(|| loader.far_proxy(other_far_proxy));
        assert!(refused.is_err(), "a second baked Material loaded");
        assert!(loader.mesh_pool.get(unused).is_some());

        loader.mesh_pool.remove_user(far_proxy, MeshUse::FarProxy);
        loader.far_proxy(other_far_proxy).unwrap();
        assert!(loader.mesh_pool.get(placement.mesh).is_none());
        assert!(!loader.material_pool.is_loaded(baked));
        assert!(loader.mesh_pool.get(unused).is_some());
        std::fs::remove_dir_all(&directory).ok();
    }

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
                        texel_density: 0.0,
                        thin_radius_metres: None,
                        meshlets: vec![CookedMeshlet {
                            vertices: vec![0, 1, 2],
                            triangles: vec![[0, 1, 2]],
                        }],
                    })
                    .into(),
                coarser_lods: Vec::new(),
                bounds: [[0.0; 3]; 2],
            };
            cache
                .write(AssetId(id), "key", &mesh.to_blob().unwrap())
                .unwrap();
        }

        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let capacities = PoolCapacities::checked_in();
        let mut mesh_pool = MeshPool::new(&context.device, &memory, &capacities);
        let mut material_pool =
            MaterialPool::new(&context.device, &memory, &mut staging, &capacities, false);
        let mut loader = PoolLoader {
            device: &context.device,
            staging: &mut staging,
            reader: &cache,
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
                .map(|handle| handle.index)
                .collect::<Vec<_>>()
        });
        let users_of_material_2 = loader.material_pool.users(AssetId(2));
        let Err(missing) = loader.mesh(AssetId(5)) else {
            panic!("Mesh 5 loaded without its second Primitive's Material");
        };
        assert_eq!(loader.material_pool.users(AssetId(2)), users_of_material_2);

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

    /// With room for one Material, Material 1 is kept while Mesh 3, whose
    /// Primitives name it, is loaded, though the Chunk that loaded it left;
    /// once Mesh 3's last user leaves too, loading Material 2 frees Mesh 3,
    /// then Material 1, and takes its entry.
    #[test]
    fn a_material_a_loaded_mesh_names_is_freed_only_after_the_mesh() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-loader-frees-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        for id in [1, 2] {
            cache
                .write(AssetId(id), "key", &material.to_blob())
                .unwrap();
        }
        let primitive = CookedPrimitive {
            first_index: 0,
            index_count: 3,
            material: Some(AssetId(1)),
            texel_density: 0.0,
            thin_radius_metres: None,
            meshlets: vec![CookedMeshlet {
                vertices: vec![0, 1, 2],
                triangles: vec![[0, 1, 2]],
            }],
        };
        let mesh = CookedMesh {
            vertices: vec![CookedVertex::default(); 3],
            indices: vec![0, 1, 2],
            primitives: vec![primitive.clone(), primitive],
            coarser_lods: Vec::new(),
            bounds: [[0.0; 3]; 2],
        };
        cache
            .write(AssetId(3), "key", &mesh.to_blob().unwrap())
            .unwrap();
        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let capacities = PoolCapacities {
            materials: 1,
            ..PoolCapacities::checked_in()
        };
        let mut mesh_pool = MeshPool::new(&context.device, &memory, &capacities);
        let mut material_pool =
            MaterialPool::new(&context.device, &memory, &mut staging, &capacities, false);
        let mut loader = PoolLoader {
            device: &context.device,
            staging: &mut staging,
            reader: &cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        };
        let named = loader.material(AssetId(1)).unwrap();
        let mesh = loader.mesh(AssetId(3)).unwrap();
        loader.material_pool.remove_user(AssetId(1));

        let (refused, log) = logged(|| loader.material(AssetId(2)));
        assert!(refused.is_err(), "a Material a loaded Mesh names was freed");
        assert!(log.contains("Material refused"), "not logged: {log:?}");
        assert!(loader.material_pool.get(named).is_some());

        loader
            .mesh_pool
            .remove_user(AssetId(3), MeshUse::DrawnAndTraced);
        let loaded = loader.material(AssetId(2)).unwrap();
        assert!(loader.mesh_pool.get(mesh).is_none());
        assert!(loader.material_pool.get(named).is_none());
        assert_eq!(loaded.index(), named.index());
        assert_eq!(loader.mesh_pool.take_frame_frees(), 1);
        assert_eq!(loader.material_pool.take_frame_frees(), 1);
        std::fs::remove_dir_all(&directory).ok();
    }

    /// With room for one Texture class beside Layer 0's, a Texture of a
    /// shape whose one slice Material 1's Texture holds frees Mesh 3, which
    /// names Material 1, then Material 1 and its Texture, and takes the
    /// slice.
    #[test]
    fn a_texture_without_room_frees_the_unused_mesh_holding_its_slice() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-loader-slices-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let texture = CookedTexture {
            format: wgpu::TextureFormat::Bc7RgbaUnormSrgb,
            width: 4,
            height: 4,
            first_level: 0,
            levels: vec![vec![0; 16]; 3],
        };
        for (material_id, texture_id) in [(1, 10), (2, 11)] {
            cache
                .write(AssetId(texture_id), "key", &texture.to_blob(1).unwrap())
                .unwrap();
            let material = CookedMaterial {
                base_color: [1.0; 4],
                metallic: 0.0,
                roughness: 1.0,
                emissive: [0.0; 3],
                normal_scale: 1.0,
                textures: [Some(AssetId(texture_id)), None, None, None, None],
            };
            cache
                .write(AssetId(material_id), "key", &material.to_blob())
                .unwrap();
        }
        let mesh = CookedMesh {
            vertices: vec![CookedVertex::default(); 3],
            indices: vec![0, 1, 2],
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: 3,
                material: Some(AssetId(1)),
                texel_density: 0.0,
                thin_radius_metres: None,
                meshlets: vec![CookedMeshlet {
                    vertices: vec![0, 1, 2],
                    triangles: vec![[0, 1, 2]],
                }],
            }],
            coarser_lods: Vec::new(),
            bounds: [[0.0; 3]; 2],
        };
        cache
            .write(AssetId(3), "key", &mesh.to_blob().unwrap())
            .unwrap();
        let mut staging = LoadStaging::new(context.device.clone());
        let memory = gpu_memory(&context);
        let capacities = PoolCapacities {
            texture_classes_per_channel: 2,
            ..PoolCapacities::checked_in()
        };
        let mut mesh_pool = MeshPool::new(&context.device, &memory, &capacities);
        let mut material_pool =
            MaterialPool::new(&context.device, &memory, &mut staging, &capacities, false);
        let mut loader = PoolLoader {
            device: &context.device,
            staging: &mut staging,
            reader: &cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        };
        let mesh = loader.mesh(AssetId(3)).unwrap();
        let named = loader.material(AssetId(1)).unwrap();
        let albedo_layer =
            |pool: &MaterialPool, handle| pool.get(handle).map(|data| data.layers[0]);
        let slice = albedo_layer(loader.material_pool, named);
        loader.material_pool.remove_user(AssetId(1));
        loader
            .mesh_pool
            .remove_user(AssetId(3), MeshUse::DrawnAndTraced);

        let loaded = loader.material(AssetId(2)).unwrap();
        assert!(loader.mesh_pool.get(mesh).is_none());
        assert!(loader.material_pool.get(named).is_none());
        assert_eq!(albedo_layer(loader.material_pool, loaded), slice);
        assert_ne!(slice, Some(MaterialPool::DEFAULT_LAYER));
        assert_eq!(loader.material_pool.take_frame_frees(), 2);
        std::fs::remove_dir_all(&directory).ok();
    }
}
