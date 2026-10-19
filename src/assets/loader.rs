use anyhow::Context;

use crate::assets::{MaterialChannel, MaterialHandle, MaterialPool, MeshHandle, MeshPool};
use crate::cooked::{AssetId, CookedCache};
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

    /// Loads the Mesh `id` into the Arenas with the Material its Primitives
    /// name. Until M3's draw path draws each Primitive with its own, a Mesh
    /// draws with its first Primitive's, after a warning when another
    /// Primitive names a different one.
    pub fn mesh(&mut self, id: AssetId) -> anyhow::Result<MeshHandle> {
        let mesh = self.cache.mesh(id)?;
        let material = mesh
            .primitives
            .first()
            .and_then(|primitive| primitive.material);
        if mesh
            .primitives
            .iter()
            .any(|primitive| primitive.material != material)
        {
            tracing::warn!(mesh = %id, "the Mesh's Primitives name different Materials; it draws with the first's");
        }
        let material = material
            .map(|material| self.material(material))
            .transpose()
            .with_context(|| format!("loading the Material of Mesh {id}"))?;
        self.mesh_pool
            .load(self.staging, &mesh, material)
            .with_context(|| format!("the Arenas refused Mesh {id}"))
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
    fn a_mesh_whose_primitives_name_different_materials_warns_and_draws_with_the_first() {
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
            textures: [None; 5],
        };
        for id in [1, 2] {
            cache
                .write(AssetId(id), "key", &material.to_blob())
                .unwrap();
        }
        // Mesh 3's Primitives both name Material 1; Mesh 4's name 2, then 9,
        // which the cache lacks.
        for (id, materials) in [(3, [1, 1]), (4, [2, 9])] {
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
        let (uniform, uniform_log) = logged(|| loader.mesh(AssetId(3)).unwrap());
        let (mixed, mixed_log) = logged(|| loader.mesh(AssetId(4)).unwrap());

        assert!(
            !uniform_log.contains("different Materials"),
            "{uniform_log}"
        );
        assert!(mixed_log.contains("different Materials"), "{mixed_log}");
        // Each loaded only the first Material its Primitives name: 1, then 2.
        let materials = [uniform, mixed].map(|mesh| {
            let material = mesh_pool.get(mesh).unwrap().material;
            material.map(MaterialHandle::index)
        });
        assert_eq!(materials, [Some(0), Some(1)]);
        std::fs::remove_dir_all(&directory).ok();
    }
}
