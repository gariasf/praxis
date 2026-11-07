use std::collections::HashMap;
use std::collections::hash_map::Entry;

use bevy_ecs::prelude::*;

use crate::assets::{AssetLoader, MaterialHandle, MeshHandle};
use crate::components::{PointLight, Transform};
use crate::cooked::{AssetId, chunk_origin};

/// The loaded Chunks, in load order. Prepare gives each its own range of
/// Instance slots and writes its Placements there; no entity holds them.
#[derive(Resource, Default)]
pub struct LoadedChunks(pub Vec<LoadedChunk>);

/// A loaded Chunk: its Placements and point lights at their World
/// positions, with the Handles its Placements draw with.
pub struct LoadedChunk {
    pub id: AssetId,
    pub placements: Vec<ChunkPlacement>,
    pub lights: Vec<(glam::DVec3, PointLight)>,
}

/// A Placement as prepare draws it: its World transform, its Mesh, and the
/// Material that overrides its Mesh's, if any.
pub struct ChunkPlacement {
    pub transform: Transform,
    pub mesh: MeshHandle,
    pub material: Option<MaterialHandle>,
}

/// Loads the Chunk `id`, at `coordinates`, through `loader`: each Mesh and
/// Material its Placements name, by Asset ID, once for the Chunk, so each
/// counts the Chunk as one user; and its Placements and lights, in the
/// order it holds them, at the Chunk's origin plus their translations.
pub(crate) fn load_chunk(
    loader: &mut dyn AssetLoader,
    id: AssetId,
    coordinates: [i32; 2],
) -> anyhow::Result<LoadedChunk> {
    let chunk = loader.chunk(id)?;
    let origin = chunk_origin(coordinates);
    let mut meshes = HashMap::new();
    let mut materials = HashMap::new();
    let mut placements = Vec::with_capacity(chunk.placements.len());
    for placement in &chunk.placements {
        let mesh = match meshes.entry(placement.mesh) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => *entry.insert(loader.mesh(placement.mesh)?),
        };
        let material = match placement.material {
            None => None,
            Some(material) => Some(match materials.entry(material) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => *entry.insert(loader.material(material)?),
            }),
        };
        placements.push(ChunkPlacement {
            transform: Transform {
                translation: origin + placement.translation,
                rotation: placement.rotation,
                scale: placement.scale,
            },
            mesh,
            material,
        });
    }
    let lights = chunk
        .lights
        .iter()
        .map(|light| {
            let point_light = PointLight {
                color: light.color,
                intensity: light.intensity,
                range: light.range,
            };
            (origin + light.translation, point_light)
        })
        .collect();
    Ok(LoadedChunk {
        id,
        placements,
        lights,
    })
}

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;
    use crate::assets::pools::tests::{gpu_context, logged, mesh};
    use crate::assets::{MaterialPool, MeshPool, MeshUse, PoolCapacities, PoolLoader};
    use crate::camera::Camera;
    use crate::components::{MaterialRef, MeshRef};
    use crate::cooked::{CookedCache, CookedChunk, CookedLight, CookedMaterial, CookedPlacement};
    use crate::environment::Environment;
    use crate::render::instance::ANCHOR_CELL_SIZE;
    use crate::render::{FrameData, FramePreparer, LoadStaging, RenderCapacities};
    use crate::stats::gpu_memory::tests::gpu_memory;
    use crate::time::FixedTime;
    use crate::transform::propagate_world_transforms;

    /// Placements of Mesh 1 drawn with Material 2, at each x from the
    /// Chunk's origin.
    fn chunk(placements_x: &[f64], lights: Vec<CookedLight>) -> CookedChunk {
        CookedChunk {
            placements: placements_x
                .iter()
                .map(|&x| CookedPlacement {
                    mesh: AssetId(1),
                    material: Some(AssetId(2)),
                    translation: glam::dvec3(x, 0.0, 0.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                })
                .collect(),
            lights,
            bounds: [glam::DVec3::ZERO, glam::DVec3::ONE],
        }
    }

    /// The slots `frame` wrote, in order, each with its Instance's World x.
    fn written_x(frame: &FrameData) -> Vec<(u32, f64)> {
        frame
            .instance_writes
            .iter()
            .flat_map(|write| {
                (write.first_slot..)
                    .zip(&frame.instances[write.instances.clone()])
                    .map(|(slot, instance)| {
                        let x = f64::from(instance.anchor[0]) * ANCHOR_CELL_SIZE
                            + f64::from(instance.offset[0]);
                        (slot, x)
                    })
            })
            .collect()
    }

    /// Two Chunks that name one Mesh and one Material load each once, each
    /// counting both Chunks as users; each Chunk's Placements take their own
    /// range of slots, the second's past a Renderable spawned between the
    /// loads, are written once, and its lights join the frame's.
    #[test]
    fn two_chunks_share_one_pool_entry_and_each_owns_a_slot_range() {
        let Some(context) = gpu_context() else {
            return;
        };
        let directory =
            std::env::temp_dir().join(format!("praxis-chunk-slots-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let material = CookedMaterial {
            base_color: [1.0; 4],
            metallic: 0.0,
            roughness: 1.0,
            emissive: [0.0; 3],
            normal_scale: 1.0,
            textures: [None; 5],
        };
        let lamp = CookedLight {
            translation: glam::dvec3(4.0, 5.0, 6.0),
            color: glam::Vec3::ONE,
            intensity: 100.0,
            range: 10.0,
        };
        let blobs = [
            (1, mesh(3, vec![0, 1, 2]).to_blob().unwrap()),
            (2, material.to_blob()),
            (10, chunk(&[1.0, 2.0], vec![lamp]).to_blob().unwrap()),
            (11, chunk(&[3.0], Vec::new()).to_blob().unwrap()),
        ];
        for (id, blob) in blobs {
            cache.write(AssetId(id), "key", &blob).unwrap();
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
            cache: &cache,
            mesh_pool: &mut mesh_pool,
            material_pool: &mut material_pool,
        };
        let first = load_chunk(&mut loader, AssetId(10), [0, 0]).unwrap();
        let second = load_chunk(&mut loader, AssetId(11), [1, 0]).unwrap();

        let handles: Vec<_> = first
            .placements
            .iter()
            .chain(&second.placements)
            .map(|placement| (placement.mesh, placement.material.unwrap()))
            .collect();
        assert!(handles.iter().all(|&handle| handle == handles[0]));
        assert_eq!(mesh_pool.users(AssetId(1), MeshUse::DrawnAndTraced), 2);
        assert_eq!(material_pool.users(AssetId(2)), 2);

        let mut world = World::new();
        world.insert_resource(Camera::new());
        world.insert_resource(FixedTime::default());
        world.insert_resource(Environment::default());
        world.insert_resource(LoadedChunks(vec![first]));
        let mut preparer = FramePreparer::new(&mut world, RenderCapacities::checked_in());
        let mut propagation = Schedule::default();
        propagation.add_systems(propagate_world_transforms);
        let mut prepare = |world: &mut World| {
            propagation.run(world);
            let frame =
                preparer.prepare(world, mesh_pool.meshes(), material_pool.generations(), 1.0);
            (
                written_x(frame),
                frame.instance_slot_count,
                frame.light.point_light_count,
            )
        };

        let (first_frame, log) = logged(|| prepare(&mut world));
        assert!(!log.contains("WARN"), "{log}");
        assert_eq!(first_frame, (vec![(0, 1.0), (1, 2.0)], 2, 1));
        let (mesh, material) = handles[0];
        world.spawn((
            Transform::from_translation(glam::dvec3(7.0, 0.0, 0.0)),
            MeshRef(mesh),
            MaterialRef(material),
        ));
        assert_eq!(prepare(&mut world), (vec![(2, 7.0)], 3, 1));
        world.resource_mut::<LoadedChunks>().0.push(second);
        assert_eq!(prepare(&mut world), (vec![(3, 259.0)], 4, 1));
        assert_eq!(prepare(&mut world), (vec![], 4, 1));
        std::fs::remove_dir_all(&directory).ok();
    }
}
