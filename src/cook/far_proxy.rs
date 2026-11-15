//! The Far proxy cook (ADR 0026): a Chunk's static Placements merged into one
//! Mesh about the Chunk's centre.

mod bake;

use std::collections::HashMap;

use anyhow::Context;
use glam::{DVec3, Vec3};

use super::mesh::{self, SourcePrimitive};
use crate::camera::FIELD_OF_VIEW;
use crate::cooked::{
    AssetId, CHUNK_EDGE_METRES, CookedChunk, CookedMesh, CookedPlacement, MESH_MAX_LODS,
};
use crate::scenes::Geometry;

pub(super) const TRIANGLE_CAP: usize = 5832;
const TARGET_ROWS: f64 = 1440.0;
const GRID_CELL_METRES: f32 = 16.0;
const NORMAL_STEPS: f32 = 512.0;

pub(super) fn pixel_metres(load_radius_metres: f64) -> f32 {
    (load_radius_metres * 2.0 * (f64::from(FIELD_OF_VIEW) / 2.0).tan() / TARGET_ROWS) as f32
}

pub(super) fn settings(pixel_metres: f32) -> String {
    format!(
        "far proxy: Placements within {pixel_metres} m of another's surface dropped, positions welded, simplified to {pixel_metres} m absolute under {TRIANGLE_CAP} triangles, each coarser LOD at twice the error of the one before and kept up to {} of it, faces split on normals in steps of 1/{NORMAL_STEPS}, {}",
        mesh::LOD_KEPT_INDICES_CEILING,
        bake::settings()
    )
}

struct Placed {
    triangles: Vec<[Vec3; 3]>,
    lowest: Vec3,
    highest: Vec3,
}

fn from_pivot(placement: &CookedPlacement, origin_from_pivot: DVec3, position: [f32; 3]) -> Vec3 {
    let turned = placement.rotation * (Vec3::from(position) * placement.scale);
    (origin_from_pivot + placement.translation + turned.as_dvec3()).as_vec3()
}

fn placed(
    placement: &CookedPlacement,
    origin_from_pivot: DVec3,
    primitives: &[SourcePrimitive],
) -> Placed {
    let mut triangles = Vec::new();
    for (geometry, _) in primitives {
        let corner = |index: u32| {
            from_pivot(
                placement,
                origin_from_pivot,
                geometry.positions[index as usize],
            )
        };
        let corners = geometry.indices.as_chunks::<3>().0.iter();
        triangles.extend(corners.map(|triangle| triangle.map(corner)));
    }
    let corners = triangles.as_flattened().iter().copied();
    Placed {
        lowest: corners.clone().fold(Vec3::INFINITY, Vec3::min),
        highest: corners.fold(Vec3::NEG_INFINITY, Vec3::max),
        triangles,
    }
}

fn distance_to_triangle(point: Vec3, [first, second, third]: [Vec3; 3]) -> f32 {
    let edges = [(first, second), (second, third), (third, first)];
    let normal = (second - first).cross(third - first);
    let over_its_face = edges
        .iter()
        .all(|(from, to)| (*to - *from).cross(point - *from).dot(normal) >= 0.0);
    if over_its_face && normal != Vec3::ZERO {
        return (point - first).dot(normal).abs() / normal.length();
    }
    let to_edge = edges.into_iter().map(|(from, to)| {
        let edge = to - from;
        let along = (point - from).dot(edge) / edge.length_squared().max(f32::MIN_POSITIVE);
        point.distance(from + edge * along.clamp(0.0, 1.0))
    });
    to_edge.fold(f32::INFINITY, f32::min)
}

fn within_another(surfaces: &[Placed], candidate_count: usize, reach_metres: f32) -> Vec<bool> {
    let cell =
        |point: Vec3| [point.x, point.z].map(|metres| (metres / GRID_CELL_METRES).floor() as i32);
    let mut grid: HashMap<[i32; 2], Vec<usize>> = HashMap::new();
    for (index, surface) in surfaces.iter().enumerate() {
        if surface.triangles.is_empty() {
            continue;
        }
        let [lowest_x, lowest_z] = cell(surface.lowest - reach_metres);
        let [highest_x, highest_z] = cell(surface.highest + reach_metres);
        for x in lowest_x..=highest_x {
            for z in lowest_z..=highest_z {
                grid.entry([x, z]).or_default().push(index);
            }
        }
    }
    let within = |candidate: &Placed, surface: &Placed| {
        candidate.lowest.cmpge(surface.lowest - reach_metres).all()
            && candidate
                .highest
                .cmple(surface.highest + reach_metres)
                .all()
            && candidate.triangles.as_flattened().iter().all(|&corner| {
                let mut distances = (surface.triangles.iter())
                    .map(|&triangle| distance_to_triangle(corner, triangle));
                distances.any(|distance| distance <= reach_metres)
            })
    };
    (0..candidate_count)
        .map(|index| {
            let candidate = &surfaces[index];
            grid.get(&cell(candidate.lowest)).is_some_and(|nearby| {
                (nearby.iter()).any(|&other| other != index && within(candidate, &surfaces[other]))
            })
        })
        .collect()
}

fn simplified(positions: &[[f32; 3]], indices: &[u32], error_metres: f32) -> (Vec<u32>, f32) {
    if indices.is_empty() {
        return (Vec::new(), 0.0);
    }
    let adapter =
        meshopt::VertexDataAdapter::new(bytemuck::cast_slice(positions), size_of::<[f32; 3]>(), 0)
            .expect("a whole number of positions");
    let mut reached_metres = 0.0;
    let indices = meshopt::simplify(
        indices,
        &adapter,
        0,
        error_metres,
        meshopt::SimplifyOptions::ErrorAbsolute,
        Some(&mut reached_metres),
    );
    (indices, reached_metres)
}

fn faceted(positions: &[[f32; 3]], indices: &[u32]) -> Geometry {
    let mut geometry = Geometry::default();
    let mut vertex_of: HashMap<([u32; 3], [i16; 3]), u32> = HashMap::new();
    for triangle in indices.as_chunks::<3>().0 {
        let [first, second, third] = triangle.map(|index| Vec3::from(positions[index as usize]));
        let normal = (second - first).cross(third - first).normalize_or_zero();
        if normal == Vec3::ZERO {
            continue;
        }
        // Two triangles of one face may differ in their normals' last bits.
        let facing = (normal * NORMAL_STEPS)
            .round()
            .to_array()
            .map(|step| step as i16);
        for corner in [first, second, third] {
            let key = (corner.to_array().map(f32::to_bits), facing);
            let vertex = *vertex_of.entry(key).or_insert_with(|| {
                geometry.positions.push(corner.into());
                geometry.normals.push(normal.into());
                geometry.uvs.push([0.0; 2]);
                geometry.positions.len() as u32 - 1
            });
            geometry.indices.push(vertex);
        }
    }
    geometry
}

/// `neighbours` gives each neighbouring Chunk with its origin from `chunk`'s.
/// The proxy's first Primitive is every Placement but the ground's, with UVs
/// in its atlas, its second the ground tiles; its pivot is the Chunk's centre
/// on the ground.
pub(super) fn cook<'sources>(
    chunk: &CookedChunk,
    neighbours: &[(DVec3, &CookedChunk)],
    primitives_of: &dyn Fn(AssetId) -> Option<&'sources [SourcePrimitive]>,
    ground: AssetId,
    pixel_metres: f32,
) -> anyhow::Result<CookedMesh> {
    let pivot = DVec3::new(CHUNK_EDGE_METRES / 2.0, 0.0, CHUNK_EDGE_METRES / 2.0);
    let primitives = |placement: &CookedPlacement| {
        primitives_of(placement.mesh)
            .with_context(|| format!("Mesh {} is not among the cook's sources", placement.mesh))
    };
    let (ground_tiles, parts): (Vec<CookedPlacement>, Vec<CookedPlacement>) =
        (chunk.placements.iter()).partition(|placement| placement.mesh == ground);

    let mut surfaces = Vec::new();
    for placement in &parts {
        surfaces.push(placed(placement, -pivot, primitives(placement)?));
    }
    let lowest = surfaces
        .iter()
        .fold(Vec3::INFINITY, |lowest, surface| lowest.min(surface.lowest));
    let highest = (surfaces.iter()).fold(Vec3::NEG_INFINITY, |highest, surface| {
        highest.max(surface.highest)
    });
    let mut reach_of_mesh = HashMap::new();
    for (origin, neighbour) in neighbours {
        let neighbouring = neighbour.placements.iter();
        for placement in neighbouring.filter(|placement| placement.mesh != ground) {
            let source = primitives(placement)?;
            let mesh_reach_metres = *reach_of_mesh.entry(placement.mesh).or_insert_with(|| {
                let positions = source.iter().flat_map(|(geometry, _)| &geometry.positions);
                let reaches = positions.map(|position| Vec3::from(*position).length());
                reaches.fold(0.0, f32::max)
            });
            let reach_metres =
                mesh_reach_metres * placement.scale.abs().max_element() + pixel_metres;
            let centre = (*origin - pivot + placement.translation).as_vec3();
            let reaches_the_chunk = centre.x + reach_metres >= lowest.x
                && centre.x - reach_metres <= highest.x
                && centre.z + reach_metres >= lowest.z
                && centre.z - reach_metres <= highest.z;
            if reaches_the_chunk {
                surfaces.push(placed(placement, *origin - pivot, source));
            }
        }
    }

    let dropped = within_another(&surfaces, parts.len(), pixel_metres);
    let kept = surfaces
        .iter()
        .zip(&dropped)
        .filter(|(_, dropped)| !**dropped);
    let corners: Vec<[f32; 3]> = kept
        .flat_map(|(surface, _)| surface.triangles.as_flattened())
        .map(|corner| corner.to_array())
        .collect();
    let (vertex_count, remap) = meshopt::generate_vertex_remap(&corners, None);
    let positions = meshopt::remap_vertex_buffer(&corners, vertex_count, &remap);
    let indices = meshopt::remap_index_buffer(None, corners.len(), &remap);
    let (finest, _) = simplified(&positions, &indices, pixel_metres);
    anyhow::ensure!(
        finest.len() / 3 <= TRIANGLE_CAP,
        "the Far proxy keeps {} triangles at {pixel_metres} m of error, past its cap of {TRIANGLE_CAP}",
        finest.len() / 3
    );

    let mut ground_geometry = Geometry::default();
    let mut ground_material = None;
    for placement in &ground_tiles {
        let [(geometry, material)] = primitives(placement)? else {
            anyhow::bail!("the ground's Mesh {ground} does not have one Primitive");
        };
        anyhow::ensure!(
            placement.material.is_none(),
            "a ground tile's Placement overrides its Mesh's Material"
        );
        ground_material = *material;
        let first_vertex = ground_geometry.positions.len() as u32;
        for (position, normal) in geometry.positions.iter().zip(&geometry.normals) {
            let position = from_pivot(placement, -pivot, *position);
            ground_geometry.positions.push(position.into());
            let normal = placement.rotation * Vec3::from(*normal);
            ground_geometry.normals.push(normal.into());
        }
        ground_geometry.uvs.extend(&geometry.uvs);
        let indices = geometry.indices.iter().map(|index| index + first_vertex);
        ground_geometry.indices.extend(indices);
    }

    let [chunk_lowest, chunk_highest] = chunk.bounds;
    let mut shell = Geometry {
        bounds: chunk_lowest.cmple(chunk_highest).all().then(|| {
            chunk
                .bounds
                .map(|corner| (corner - pivot).as_vec3().to_array())
        }),
        ..faceted(&positions, &finest)
    };
    let atlas = bake::Atlas::chart(&mut shell, pixel_metres)?;
    let mut proxy = mesh::lod_0(&[(shell, None), (ground_geometry.clone(), ground_material)])?;
    let positions_metres = proxy
        .vertices
        .iter()
        .map(|vertex| Vec3::from(vertex.position));
    let farthest_metres = positions_metres.map(Vec3::length).fold(0.0, f32::max);
    anyhow::ensure!(
        farthest_metres <= CHUNK_EDGE_METRES as f32,
        "a vertex of the Far proxy lies {farthest_metres:.1} m from its pivot, past a Chunk's edge"
    );
    let mut finer_indices = finest.len();
    for doublings in 1..MESH_MAX_LODS as i32 {
        let (coarser, error_metres) =
            simplified(&positions, &finest, pixel_metres * 2f32.powi(doublings));
        if coarser.is_empty()
            || coarser.len() as f32 > finer_indices as f32 * mesh::LOD_KEPT_INDICES_CEILING
        {
            break;
        }
        let mut shell = faceted(&positions, &coarser);
        if let Some(atlas) = &atlas {
            atlas.map(&mut shell);
        }
        mesh::push_lod(&mut proxy, error_metres, &[&shell, &ground_geometry])?;
        finer_indices = coarser.len();
    }
    Ok(proxy)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::f32::consts::{FRAC_1_SQRT_2, FRAC_PI_4};
    use std::path::Path;

    use glam::{Quat, dvec3};

    use super::*;
    use crate::ReferenceScene;
    use crate::cook::{ASSETS_DIRECTORY, read_sources};
    use crate::scenes::ground_plane;

    const PIXEL_METRES: f32 = 0.1473;
    const DENSEST_CHUNK: [i32; 2] = [-19, -3];

    fn upright_quad(width: f32, height: f32, forward: f32) -> Geometry {
        let half_width = width / 2.0;
        Geometry {
            positions: vec![
                [-half_width, 0.0, forward],
                [half_width, 0.0, forward],
                [half_width, height, forward],
                [-half_width, height, forward],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            ..Geometry::default()
        }
    }

    fn at(mesh: AssetId, translation: DVec3) -> CookedPlacement {
        CookedPlacement {
            mesh,
            material: None,
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    fn millimetres(position: Vec3) -> [i64; 3] {
        position
            .to_array()
            .map(|metres| (metres * 1000.0).round() as i64)
    }

    fn corners(mesh: &CookedMesh, primitive: usize) -> Vec<Vec3> {
        let primitive = &mesh.primitives[primitive];
        let indices =
            &mesh.indices[primitive.first_index as usize..][..primitive.index_count as usize];
        let corners = indices.iter();
        corners
            .map(|&index| Vec3::from(mesh.vertices[index as usize].position))
            .collect()
    }

    #[test]
    fn a_point_is_as_far_from_a_triangle_as_from_its_nearest_face_edge_or_corner() {
        let triangle = [Vec3::ZERO, Vec3::X * 4.0, Vec3::Z * 4.0];
        let distance = |x, y, z| distance_to_triangle(Vec3::new(x, y, z), triangle);
        assert_eq!(distance(1.0, 0.25, 1.0), 0.25);
        assert_eq!(distance(2.0, 0.0, -3.0), 3.0);
        assert_eq!(distance(7.0, 4.0, 0.0), 5.0);
        assert!((distance(3.0, 0.0, 3.0) - 2f32.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn a_placement_within_a_pixel_of_anothers_surface_is_left_out_and_no_other_is() {
        let (wall, pane, ground, plate) = (AssetId(1), AssetId(2), AssetId(3), AssetId(4));
        let sources = [
            (wall, vec![(upright_quad(4.0, 3.0, 0.0), None)]),
            (pane, vec![(upright_quad(1.0, 1.0, 0.06), None)]),
            (ground, vec![(ground_plane(128.0, 1.0), None)]),
            (plate, vec![(ground_plane(0.5, 1.0), None)]),
        ];
        let primitives_of = |mesh: AssetId| {
            let source = sources.iter().find(|(id, _)| *id == mesh);
            source.map(|(_, primitives)| &primitives[..])
        };
        let chunk = CookedChunk {
            placements: vec![
                at(ground, dvec3(128.0, 0.0, 128.0)),
                at(wall, dvec3(100.0, 0.0, 100.0)),
                at(pane, dvec3(100.0, 1.0, 100.0)),
                at(pane, dvec3(100.0, 1.0, 100.2)),
                at(pane, dvec3(110.0, 1.0, 100.0)),
                at(pane, dvec3(101.8, 1.0, 100.0)),
                at(pane, dvec3(0.5, 1.0, 50.0)),
                CookedPlacement {
                    rotation: Quat::from_rotation_y(FRAC_PI_4),
                    ..at(wall, dvec3(60.0, 0.0, 60.0))
                },
                at(pane, dvec3(60.5, 1.0, 59.94)),
                at(plate, dvec3(20.0, 0.05, 20.0)),
            ],
            lights: Vec::new(),
            bounds: [DVec3::ZERO, DVec3::splat(256.0)],
        };
        let west = CookedChunk {
            placements: vec![at(wall, dvec3(255.0, 0.0, 50.0))],
            ..chunk.clone()
        };
        let neighbours = [(dvec3(-256.0, 0.0, 0.0), &west)];

        let proxy = cook(&chunk, &neighbours, &primitives_of, ground, PIXEL_METRES).unwrap();

        let quad = |[left, bottom]: [f32; 2], [right, top]: [f32; 2], forward: f32| {
            [[left, bottom], [right, bottom], [right, top], [left, top]]
                .map(|[x, y]| millimetres(Vec3::new(x - 128.0, y, forward - 128.0)))
        };
        let reach = 2.0 * FRAC_1_SQRT_2;
        let turned_wall = [(-reach, 0.0), (-reach, 3.0), (reach, 0.0), (reach, 3.0)]
            .map(|(along, y)| millimetres(Vec3::new(along - 68.0, y, -along - 68.0)));
        let kept: BTreeSet<_> = [
            quad([98.0, 0.0], [102.0, 3.0], 100.0),
            quad([99.5, 1.0], [100.5, 2.0], 100.26),
            quad([109.5, 1.0], [110.5, 2.0], 100.06),
            quad([101.3, 1.0], [102.3, 2.0], 100.06),
            turned_wall,
            quad([60.0, 1.0], [61.0, 2.0], 60.0),
            [(19.5, 19.5), (19.5, 20.5), (20.5, 19.5), (20.5, 20.5)]
                .map(|(x, z)| millimetres(Vec3::new(x - 128.0, 0.05, z - 128.0))),
        ]
        .into_iter()
        .flatten()
        .collect();
        let cooked: BTreeSet<_> = corners(&proxy, 0).into_iter().map(millimetres).collect();
        assert_eq!(cooked, kept);
    }

    fn densest_chunks_proxy() -> (CookedMesh, CookedChunk, Vec<u8>) {
        let scene = ReferenceScene::World;
        let mut sources = read_sources(Path::new(ASSETS_DIRECTORY), &[scene], Some(&[])).unwrap();
        let blob = sources.far_proxy_blob(scene, DENSEST_CHUNK).unwrap();
        let chunk = sources.chunk(scene, DENSEST_CHUNK).unwrap().chunk;
        (CookedMesh::from_blob(&blob).unwrap(), chunk, blob)
    }

    #[test]
    fn the_densest_chunks_proxy_keeps_the_cap_a_pixel_of_error_its_content_and_its_ground() {
        let scene = ReferenceScene::World;
        let (proxy, chunk, _) = densest_chunks_proxy();
        let [shell, ground] = &proxy.primitives[..] else {
            panic!("{} Primitives", proxy.primitives.len());
        };
        assert!((pixel_metres(256.0) - PIXEL_METRES).abs() < 1e-4);
        assert!((1..=5832).contains(&(shell.index_count / 3)));
        let atlas_texels_per_metre = shell.texel_density * 4096.0;
        assert!((atlas_texels_per_metre * PIXEL_METRES - 1.0).abs() < 1e-3);
        let flat_roofed = &proxy.coarser_lods[0];
        assert!(flat_roofed.error_metres <= 2.0 * PIXEL_METRES);
        assert!(flat_roofed.primitives[0].index_count * 100 <= shell.index_count * 85);
        assert_eq!(flat_roofed.primitives[1].index_count / 3, 8);
        assert_eq!(ground.material, Some(scene.material_id("grass")));
        assert_eq!(ground.index_count / 3, 8);
        let ground_corners: BTreeSet<_> = corners(&proxy, 1).into_iter().map(millimetres).collect();
        let tile_corners = [-128_000, 0, 128_000];
        let expected = tile_corners
            .into_iter()
            .flat_map(|x| tile_corners.map(|z| [x, 0, z]));
        assert_eq!(ground_corners, expected.collect());

        let positions_of: HashMap<AssetId, Vec<[f32; 3]>> = (scene.meshes().into_iter())
            .map(|(name, primitives)| (scene.mesh_id(name), primitives[0].0.positions.clone()))
            .collect();
        let pivot = dvec3(128.0, 0.0, 128.0);
        let [lowest, highest] = proxy.bounds.map(Vec3::from);
        let (window, ground_tile) = (scene.mesh_id("window"), scene.mesh_id("ground"));
        let mut source_corners = Vec::new();
        for placement in &chunk.placements {
            for position in &positions_of[&placement.mesh] {
                let turned = placement.rotation * (Vec3::from(*position) * placement.scale);
                let corner = (placement.translation - pivot + turned.as_dvec3()).as_vec3();
                assert!(
                    corner.cmpge(lowest - 1e-3).all() && corner.cmple(highest + 1e-3).all(),
                    "the proxy's bounds do not hold {corner}"
                );
                if placement.mesh != window && placement.mesh != ground_tile {
                    source_corners.push(corner);
                }
            }
        }

        let shell_corners = corners(&proxy, 0);
        let source_millimetres: BTreeSet<_> =
            source_corners.iter().copied().map(millimetres).collect();
        let shell_triangles = shell_corners.as_chunks::<3>().0;
        for corner in source_corners.iter().step_by(7) {
            let nearest_metres = shell_triangles
                .iter()
                .map(|&triangle| distance_to_triangle(*corner, triangle))
                .fold(f32::INFINITY, f32::min);
            assert!(
                nearest_metres <= PIXEL_METRES,
                "{corner}: {nearest_metres} m"
            );
        }
        for corner in &shell_corners {
            assert!(
                source_millimetres.contains(&millimetres(*corner)),
                "the proxy's vertex {corner} is no vertex of its walls and roofs"
            );
        }
    }

    #[test]
    fn the_same_chunk_cooks_the_same_blob_twice() {
        let (_, _, first) = densest_chunks_proxy();
        let (_, _, second) = densest_chunks_proxy();
        assert!(first == second);
    }
}
