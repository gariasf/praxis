use anyhow::Context;

use crate::assets::{MaterialChannel, MaterialHandle, MaterialPool, MeshHandle, MeshPool, MeshUse};
use crate::cooked::{AssetId, CookedCache, CookedMesh};
use crate::render::LoadStaging;

/// What a scene load reads Assets with: a Cooked cache, and the Pools they
/// load into through the load's staging (ADR 0018).
pub struct AssetLoader<'a> {
    pub device: &'a wgpu::Device,
    pub staging: &'a mut LoadStaging,
    pub cache: &'a CookedCache,
    pub mesh_pool: &'a mut MeshPool,
    pub material_pool: &'a mut MaterialPool,
}

impl AssetLoader<'_> {
    /// Loads the Material `id` with the Textures it samples.
    pub fn material(&mut self, id: AssetId) -> anyhow::Result<MaterialHandle> {
        self.material_pool
            .load(self.device, self.staging, self.cache, id)
    }

    /// Loads the Texture `id` into `channel`, as a Material sampling it
    /// would, and returns its Layer.
    pub fn texture(&mut self, channel: MaterialChannel, id: AssetId) -> anyhow::Result<u32> {
        self.material_pool
            .load_texture(self.device, self.staging, self.cache, channel, id)
    }

    /// Loads the Mesh `id` into the Mesh pool with the Materials its
    /// Primitives name, each loaded once, which each Primitive draws with.
    pub fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        let mesh = self.cache.mesh(id)?;
        let materials = self.materials(id, &mesh)?;
        self.mesh_pool
            .load(self.staging, &mesh, &materials)
            .with_context(|| format!("the Mesh pool refused Mesh {id}"))
    }

    /// Loads the Mesh `id` as `mesh` does, for the GPU to bend each frame
    /// (`Bend`): drawn, and traced by no ray, as its BLAS would hold its rest
    /// shape. Its cooked bounds must hold every shape it bends to.
    pub fn bent_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::Drawn)
    }

    /// Loads the Mesh `id` as `bent_mesh` does, and traced through a BLAS
    /// updated each frame from the shape the GPU bends it to.
    pub fn refitted_mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        self.mesh_for(id, MeshUse::Refitted)
    }

    /// Loads the Mesh `id` as `mesh` does, for `mesh_use`.
    fn mesh_for(&mut self, id: AssetId, mesh_use: MeshUse) -> anyhow::Result<MeshHandle> {
        let mesh = self.cache.mesh(id)?;
        let materials = self.materials(id, &mesh)?;
        self.mesh_pool
            .load_for(self.staging, &mesh, &materials, mesh_use)
            .with_context(|| format!("the Mesh pool refused Mesh {id}"))
    }

    /// Loads the Mesh `id` as a shadow proxy, which rays trace through its
    /// BLAS and no pass draws, so it loads no Material.
    pub fn shadow_proxy(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        let mesh = self.cache.mesh(id)?;
        let materials = vec![None; mesh.primitives.len()];
        self.mesh_pool
            .load_for(self.staging, &mesh, &materials, MeshUse::Traced)
            .with_context(|| format!("the Mesh pool refused Mesh {id}"))
    }

    /// The Materials `mesh`'s Primitives name, Mesh `id`'s, each loaded once.
    fn materials(
        &mut self,
        id: AssetId,
        mesh: &CookedMesh,
    ) -> anyhow::Result<Vec<Option<MaterialHandle>>> {
        let mut materials = Vec::with_capacity(mesh.primitives.len());
        for (index, primitive) in mesh.primitives.iter().enumerate() {
            let earlier = mesh.primitives[..index]
                .iter()
                .position(|earlier| earlier.material == primitive.material);
            let material = match (earlier, primitive.material) {
                (Some(earlier), _) => materials[earlier],
                (None, Some(material)) => Some(
                    self.material(material)
                        .with_context(|| format!("loading a Material of Mesh {id}"))?,
                ),
                (None, None) => None,
            };
            materials.push(material);
        }
        Ok(materials)
    }
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::assets::ARENA_CAPACITY_BYTES;
    use crate::assets::pools::tests::logged;
    use crate::cooked::{CookedMaterial, CookedMesh, CookedMeshlet, CookedPrimitive, CookedVertex};
    use crate::gpu::GpuContext;

    #[test]
    fn a_mesh_loads_each_material_its_primitives_name_once_without_a_warning() {
        let context = match pollster::block_on(GpuContext::headless()) {
            Ok(context) => context,
            Err(error) if error.is::<wgpu::RequestAdapterError>() => {
                // Not `eprintln!`, which libtest captures from passing tests.
                writeln!(std::io::stderr(), "skipped: no GPU adapter: {error}").ok();
                return;
            }
            Err(error) => panic!("headless GPU context failed: {error:#}"),
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
        for id in [1, 2] {
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
        let mut loader = AssetLoader {
            device: &context.device,
            staging: &mut staging,
            cache: &cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        };
        let ((), log) = logged(|| {
            loader.mesh(AssetId(3)).unwrap();
            loader.mesh(AssetId(4)).unwrap();
        });
        let Err(missing) = loader.mesh(AssetId(5)) else {
            panic!("Mesh 5 loaded without its second Primitive's Material");
        };

        assert!(!log.contains("WARN"), "{log}");
        // Each Primitive's Material loads, a Mesh's repeated one once:
        // Material 1 for Mesh 3, then 2 and 1 for Mesh 4, and 2 for Mesh 5,
        // before its second Primitive's fails.
        assert!(format!("{missing:#}").contains("loading a Material of Mesh"));
        let next = loader.material(AssetId(1)).unwrap();
        assert_eq!(next.index(), 4);
        std::fs::remove_dir_all(&directory).ok();
    }
}
