//! The Chunk cook (ADR 0018): a Reference scene's static content cut into
//! Chunks, each Placement and light in the Chunk that holds its pivot.

use std::collections::{BTreeMap, HashMap};

use glam::{BVec3, DVec3, Vec3};
use sha2::{Digest, Sha256};

use super::mesh::SourcePrimitive;
use crate::cooked::{
    AssetId, CHUNK_EDGE_METRES, CHUNK_FORMAT_VERSION, CHUNK_MAGIC, CookedChunk, CookedLight,
    CookedPlacement, chunk_origin, hex,
};

/// Each Chunk that holds the pivot of one of `placements` or `lights`,
/// translated from the World's origin, by its coordinates. Its bounds hold
/// each of its Placements' Mesh boxes in `mesh_bounds`, placed; a Mesh
/// missing there adds only its pivot.
pub(super) fn chunks(
    placements: Vec<CookedPlacement>,
    lights: Vec<CookedLight>,
    mesh_bounds: &HashMap<AssetId, [Vec3; 2]>,
) -> BTreeMap<[i32; 2], CookedChunk> {
    let mut chunks = split(placements, lights);
    for chunk in chunks.values_mut() {
        let CookedChunk {
            placements, bounds, ..
        } = chunk;
        for placement in placements.iter() {
            let [lowest, highest] =
                (mesh_bounds.get(&placement.mesh).copied()).unwrap_or([Vec3::ZERO; 2]);
            for corner in 0..8 {
                let is_highest = BVec3::new(corner & 1 != 0, corner & 2 != 0, corner & 4 != 0);
                let local = Vec3::select(is_highest, highest, lowest) * placement.scale;
                let placed =
                    placement.translation + placement.rotation.as_dquat() * local.as_dvec3();
                *bounds = [bounds[0].min(placed), bounds[1].max(placed)];
            }
        }
    }
    chunks
}

/// Each Chunk that holds the pivot of one of `placements` or `lights`,
/// translated from the World's origin, by its coordinates, each Placement
/// and light in the order given and translated from its Chunk's origin; its
/// bounds hold nothing.
pub(super) fn split(
    placements: Vec<CookedPlacement>,
    lights: Vec<CookedLight>,
) -> BTreeMap<[i32; 2], CookedChunk> {
    let mut chunks = BTreeMap::new();
    for mut placement in placements {
        let (origin, chunk) = chunk_holding(&mut chunks, placement.translation);
        placement.translation -= origin;
        chunk.placements.push(placement);
    }
    for mut light in lights {
        let (origin, chunk) = chunk_holding(&mut chunks, light.translation);
        light.translation -= origin;
        chunk.lights.push(light);
    }
    chunks
}

/// The origin of the Chunk that holds `pivot` (ADR 0014), and its entry.
fn chunk_holding(
    chunks: &mut BTreeMap<[i32; 2], CookedChunk>,
    pivot: DVec3,
) -> (DVec3, &mut CookedChunk) {
    let coordinates = [pivot.x, pivot.z].map(|metres| (metres / CHUNK_EDGE_METRES).floor() as i32);
    let chunk = chunks.entry(coordinates).or_insert_with(|| CookedChunk {
        placements: Vec::new(),
        lights: Vec::new(),
        bounds: [DVec3::INFINITY, DVec3::NEG_INFINITY],
    });
    (chunk_origin(coordinates), chunk)
}

/// The box that holds every source vertex of `primitives` and every box
/// their geometry adds, which holds the cooked Mesh's bounds.
pub(super) fn source_bounds(primitives: &[SourcePrimitive]) -> [Vec3; 2] {
    let positions = primitives
        .iter()
        .flat_map(|(geometry, _)| &geometry.positions)
        .map(|position| Vec3::from(*position));
    let added = primitives
        .iter()
        .filter_map(|(geometry, _)| geometry.bounds)
        .flat_map(|[lowest, highest]| [Vec3::from(lowest), Vec3::from(highest)]);
    let corners = positions.chain(added);
    [
        corners.clone().fold(Vec3::INFINITY, Vec3::min),
        corners.fold(Vec3::NEG_INFINITY, Vec3::max),
    ]
}

/// A Chunk's Cook key: SHA-256 over its format, its compression and its
/// payload, which holds everything it cooks from, its Meshes' bounds
/// included.
pub(super) fn cook_key(chunk: &CookedChunk) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CHUNK_MAGIC);
    hasher.update(CHUNK_FORMAT_VERSION.to_le_bytes());
    hasher.update(zstd::zstd_safe::version_number().to_le_bytes());
    hasher.update(CookedChunk::ZSTD_LEVEL.to_le_bytes());
    hasher.update(chunk.payload());
    hex(&hasher.finalize())
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;

    use super::*;
    use crate::ReferenceScene;
    use crate::components::{Bend, MeshRef, PointLight};
    use crate::cook::export::AssetRecorder;
    use crate::cook::{CookSettings, cook_scene};
    use crate::cooked::CookedCache;
    use crate::render::CrowdForm;
    use crate::state::Simulation;
    use crate::stats::FrameRecorder;
    use crate::streaming::LoadedChunks;
    use crate::transform::PreviousWorldTransform;

    /// The 256 m Chunk over `translation`, x then z, and `translation` from
    /// its origin.
    fn in_chunk(translation: DVec3) -> ([i32; 2], DVec3) {
        let coordinates = [translation.x, translation.z].map(|metres| (metres / 256.0).floor());
        let origin = DVec3::new(coordinates[0], 0.0, coordinates[1]) * 256.0;
        (
            coordinates.map(|coordinate| coordinate as i32),
            translation - origin,
        )
    }

    /// A Chunk's coordinates and a Placement's fields as bits, so that two
    /// lists compare exactly in any order.
    type PlacementBits = ([i32; 2], u128, u128, [u64; 3], [u32; 4], [u32; 3]);

    fn placement_bits(coordinates: [i32; 2], placement: &CookedPlacement) -> PlacementBits {
        (
            coordinates,
            placement.mesh.0,
            placement.material.map_or(0, |id| id.0),
            placement.translation.to_array().map(f64::to_bits),
            placement.rotation.to_array().map(f32::to_bits),
            placement.scale.to_array().map(f32::to_bits),
        )
    }

    fn light_bits(coordinates: [i32; 2], light: &CookedLight) -> ([i32; 2], [u64; 3], [u32; 5]) {
        let [red, green, blue] = light.color.to_array();
        let values = [red, green, blue, light.intensity, light.range];
        (
            coordinates,
            light.translation.to_array().map(f64::to_bits),
            values.map(f32::to_bits),
        )
    }

    /// town's Chunks, read back by Asset ID, hold exactly what its generator
    /// places that does not move, and its lights: each in the 256 m Chunk
    /// over its pivot, translated from that Chunk's origin, its Mesh's cooked
    /// box inside the Chunk's bounds. Its spawn loads them in Chunk order,
    /// with no entity but its birds'. A second cook rewrites nothing.
    #[test]
    fn towns_chunks_hold_what_its_generator_places_and_load_in_chunk_order() {
        let scene = ReferenceScene::Town;
        let directory =
            std::env::temp_dir().join(format!("praxis-town-chunks-{}", std::process::id()));
        let cache = CookedCache::new(directory.join("cooked"));
        let assets = directory.join("assets");
        let settings = CookSettings::default();
        let report = cook_scene(scene, &assets, &cache, &settings).unwrap();
        assert_eq!(report.dangling_references, []);

        let cooked_chunks: Vec<_> = (-8..8)
            .flat_map(|x| (-8..8).map(move |z| scene.chunk_id([x, z])))
            .filter(|id| report.cooked.contains(id))
            .map(|id| (id, cache.chunk(id).unwrap()))
            .collect();
        let mut recorder = AssetRecorder::new(cooked_chunks.iter().map(|(id, chunk)| (*id, chunk)));
        let mut simulation = Simulation::new(
            scene,
            DVec3::ZERO,
            CrowdForm::default(),
            &mut recorder,
            &FrameRecorder::new(),
        )
        .unwrap();
        let world = &mut simulation.world;
        let renderables = world.query::<&MeshRef>().iter(world).count();
        assert_eq!(renderables, 10_000, "town spawns static content");
        assert_eq!(world.query::<&PointLight>().iter(world).count(), 0);
        let chunks = &world.resource::<LoadedChunks>().0;
        let loaded: Vec<_> = chunks
            .iter()
            .flat_map(|chunk| &chunk.placements)
            .map(|placement| {
                let transform = placement.transform;
                let (coordinates, translation) = in_chunk(transform.translation);
                placement_bits(
                    coordinates,
                    &CookedPlacement {
                        mesh: recorder.meshes[placement.mesh.index as usize],
                        material: placement
                            .material
                            .map(|material| recorder.materials[material.index as usize]),
                        translation,
                        rotation: transform.rotation,
                        scale: transform.scale,
                    },
                )
            })
            .collect();
        let loaded_lights: Vec<_> = chunks
            .iter()
            .flat_map(|chunk| &chunk.lights)
            .map(|(position, light)| {
                let (coordinates, translation) = in_chunk(*position);
                light_bits(
                    coordinates,
                    &CookedLight {
                        translation,
                        color: light.color,
                        intensity: light.intensity,
                        range: light.range,
                    },
                )
            })
            .collect();

        let mut mesh_boxes = HashMap::new();
        let (mut cooked, mut cooked_lights) = (Vec::new(), Vec::new());
        let mut coordinates_cooked = Vec::new();
        for x in -8..8 {
            for z in -8..8 {
                let id = scene.chunk_id([x, z]);
                if !report.cooked.contains(&id) {
                    continue;
                }
                coordinates_cooked.push([x, z]);
                let chunk = cache.chunk(id).unwrap();
                for placement in &chunk.placements {
                    let pivot = placement.translation;
                    assert!(
                        (0.0..256.0).contains(&pivot.x) && (0.0..256.0).contains(&pivot.z),
                        "a pivot {pivot} m from Chunk ({x}, {z})'s origin lies outside it"
                    );
                    let [lowest, highest] = *mesh_boxes
                        .entry(placement.mesh)
                        .or_insert_with(|| cache.mesh(placement.mesh).unwrap().bounds);
                    let corners = (0..8).map(|corner: usize| {
                        Vec3::from_array(std::array::from_fn(|axis| {
                            [lowest, highest][corner >> axis & 1][axis]
                        }))
                    });
                    for corner in corners {
                        let placed = pivot
                            + placement.rotation.as_dquat() * (corner * placement.scale).as_dvec3();
                        let [chunk_lowest, chunk_highest] = chunk.bounds;
                        assert!(
                            placed.cmpge(chunk_lowest - 1e-9).all()
                                && placed.cmple(chunk_highest + 1e-9).all(),
                            "Chunk ({x}, {z})'s bounds do not hold {placed}"
                        );
                    }
                    cooked.push(placement_bits([x, z], placement));
                }
                for light in &chunk.lights {
                    cooked_lights.push(light_bits([x, z], light));
                }
            }
        }
        assert_eq!(coordinates_cooked, scene.chunk_coordinates());

        assert!(
            loaded == cooked,
            "town does not load its Chunks' Placements in Chunk order"
        );
        assert_eq!(loaded_lights, cooked_lights);

        let (placements, lights) = scene.static_content();
        let mut generated: Vec<_> = placements
            .iter()
            .map(|placement| {
                let (coordinates, translation) = in_chunk(placement.translation);
                placement_bits(
                    coordinates,
                    &CookedPlacement {
                        translation,
                        ..*placement
                    },
                )
            })
            .collect();
        let mut generated_lights: Vec<_> = lights
            .iter()
            .map(|light| {
                let (coordinates, translation) = in_chunk(light.translation);
                light_bits(
                    coordinates,
                    &CookedLight {
                        translation,
                        ..*light
                    },
                )
            })
            .collect();
        for list in [&mut generated, &mut cooked] {
            list.sort_unstable();
        }
        for list in [&mut generated_lights, &mut cooked_lights] {
            list.sort_unstable();
        }
        assert_eq!(cooked.len(), generated.len());
        assert!(
            cooked == generated,
            "town's Chunks do not hold what its generator places"
        );
        assert_eq!(cooked_lights, generated_lights);

        let again = cook_scene(scene, &assets, &cache, &settings).unwrap();
        assert_eq!(again.cooked, []);
        std::fs::remove_dir_all(directory).unwrap();
    }

    /// Each scene's Chunks lie at the coordinates its spawn loads, and its
    /// spawn loads all they hold, spawning an entity only for what moves: no
    /// point light, and no Renderable that neither moves in the fixed
    /// schedule nor bends.
    #[test]
    fn every_scene_loads_its_static_content_from_its_chunks() {
        for scene in ReferenceScene::ALL {
            let name = scene.name();
            let (placements, lights) = scene.static_content();
            let counts = (placements.len(), lights.len());
            let chunks = split(placements, lights);
            let coordinates: Vec<_> = chunks.keys().copied().collect();
            assert_eq!(coordinates, scene.chunk_coordinates(), "{name}");
            let mut recorder = AssetRecorder::new(
                chunks
                    .iter()
                    .map(|(&coordinates, chunk)| (scene.chunk_id(coordinates), chunk)),
            );
            let mut simulation = Simulation::new(
                scene,
                DVec3::ZERO,
                CrowdForm::default(),
                &mut recorder,
                &FrameRecorder::new(),
            )
            .unwrap();
            let world = &mut simulation.world;
            let loaded = &world.resource::<LoadedChunks>().0;
            let loaded_counts = (
                loaded.iter().map(|chunk| chunk.placements.len()).sum(),
                loaded.iter().map(|chunk| chunk.lights.len()).sum(),
            );
            assert_eq!(loaded_counts, counts, "{name}");
            let point_lights = world.query::<&PointLight>().iter(world).count();
            assert_eq!(point_lights, 0, "{name} spawns a point light");
            let still = world
                .query_filtered::<(), (
                    With<MeshRef>,
                    Without<PreviousWorldTransform>,
                    Without<Bend>,
                )>()
                .iter(world)
                .count();
            assert_eq!(still, 0, "{name} spawns a Renderable that does not move");
        }
    }
}
