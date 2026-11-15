//! The Mesh cook (ADR 0018): welds each Primitive on position, UV and
//! normal, regenerates MikkTSpace tangents, re-splits the vertices whose
//! tangents differ, packs them as 28-byte vertices with a QTangent, orders
//! them for vertex cache, overdraw and vertex fetch, and cuts each
//! Primitive's triangles, in that order, into Meshlets; then simplifies the
//! Mesh into the rest of its LOD chain, each LOD ordered and cut alike.

use std::collections::{HashMap, HashSet};

use anyhow::Context as _;
use glam::{Mat3, Quat, Vec2, Vec3};
use sha2::{Digest, Sha256};

use crate::cooked::{
    AssetId, AssetKind, AssetName, AssetSource, CookedLod, CookedMesh, CookedMeshlet,
    CookedPrimitive, CookedVertex, MESH_FORMAT_VERSION, MESH_MAGIC, MESH_MAX_LODS,
    MESHLET_MAX_TRIANGLES, MESHLET_MAX_VERTICES, QTANGENT_SMALLEST_W, hex,
};
use crate::scenes::Geometry;

/// A Primitive's geometry and the Asset ID of its Material, if it names one.
pub(crate) type SourcePrimitive = (Geometry, Option<AssetId>);

/// How far the overdraw pass may worsen the vertex cache order: the
/// threshold the welded helmet's size was measured at
/// (`docs/research/asset-compression-and-cooked-data.md`, item 6).
const OVERDRAW_THRESHOLD: f32 = 1.05;

/// The share of the previous LOD's indices past which a LOD is dropped and
/// its chain ends.
pub(super) const LOD_KEPT_INDICES_CEILING: f32 = 0.85;

/// The simplifier's weight of each component of a vertex's normal, as in
/// its README's example [ref:meshoptimizer-0.25-attribute-weights].
const LOD_NORMAL_WEIGHT: f32 = 0.5;

/// The simplifier's weight of each component of a vertex's UV, once that is
/// in metres of Texture over the Mesh's extent.
const LOD_UV_WEIGHT: f32 = 1.0;

/// Every encoder version and setting a Mesh Blob depends on, for its Cook key.
/// The meshoptimizer version is the one `meshopt` 0.6.2 vendors, whose vertex
/// encoder writes codec 0 by default [ref:meshoptimizer-0.25-vertex-codec]:
/// update them and the `bevy_mikktspace` version with their dependencies.
pub(super) fn encoder_settings() -> String {
    format!(
        "bevy_mikktspace 1.0.0, meshoptimizer 0.25, vertex codec 0, overdraw {OVERDRAW_THRESHOLD}, meshlets of {MESHLET_MAX_VERTICES} vertices and {MESHLET_MAX_TRIANGLES} triangles cut in index order, at most {MESH_MAX_LODS} LODs each simplified to half the one before with normals weighted {LOD_NORMAL_WEIGHT} and UVs {LOD_UV_WEIGHT} and kept up to {LOD_KEPT_INDICES_CEILING} of it, zstd {} level {}",
        zstd::zstd_safe::version_number(),
        CookedMesh::ZSTD_LEVEL
    )
}

/// A Mesh's Cook key: SHA-256 over its format, its encoders' versions and
/// settings, and each Primitive's geometry and Material, which are all
/// `cook` reads, each part length-prefixed so no two Meshes share an
/// encoding.
pub(super) fn cook_key(primitives: &[SourcePrimitive]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MESH_MAGIC);
    hasher.update(MESH_FORMAT_VERSION.to_le_bytes());
    let mut hash_part = |part: &[u8]| {
        hasher.update((part.len() as u64).to_le_bytes());
        hasher.update(part);
    };
    hash_part(encoder_settings().as_bytes());
    for (geometry, material) in primitives {
        // Every field by name, so that one added to `Geometry` fails to
        // compile here until the key covers it.
        let Geometry {
            positions,
            normals,
            uvs,
            indices,
            bounds,
            thin_radius_metres,
        } = geometry;
        hash_part(bytemuck::cast_slice(positions));
        hash_part(bytemuck::cast_slice(normals));
        hash_part(bytemuck::cast_slice(uvs));
        hash_part(bytemuck::cast_slice(indices));
        if let Some(bounds) = bounds {
            hash_part(bytemuck::cast_slice(bounds));
        }
        if let Some(thin_radius_metres) = thin_radius_metres {
            hash_part(&thin_radius_metres.to_le_bytes());
        }
        hash_part(&material.map_or(0, |material| material.0).to_be_bytes());
    }
    hex(&hasher.finalize())
}

/// The farthest a vertex `cook` takes may lie from its Mesh's pivot: ADR
/// 0014's bound, within which f32 rotation keeps its precision bar.
const MAX_PIVOT_DISTANCE_METRES: f32 = 128.0;

/// How far a Thin Primitive's radius may lie from the one declared, as a
/// share of it.
const THIN_RADIUS_TOLERANCE: f32 = 0.25;

/// Refuses a Primitive declared thin whose triangles are neither the side of
/// a tube of `radius_metres` nor a cap that closes one.
fn check_thin_primitive(geometry: &Geometry, radius_metres: f32) -> anyhow::Result<()> {
    anyhow::ensure!(
        radius_metres.is_finite() && radius_metres > 0.0,
        "it is declared thin with a radius of {radius_metres} m"
    );
    let position = |vertex: u32| Vec3::from(geometry.positions[vertex as usize]);
    let normal = |vertex: u32| Vec3::from(geometry.normals[vertex as usize]).normalize_or_zero();
    let position_bits = |vertex: u32| geometry.positions[vertex as usize].map(f32::to_bits);
    let on_one_ring = |[first, second]: [u32; 2]| {
        let normals_apart = normal(first) - normal(second);
        let centres_apart = position(first) - position(second) - normals_apart * radius_metres;
        normals_apart != Vec3::ZERO
            && centres_apart.length()
                <= THIN_RADIUS_TOLERANCE * radius_metres * normals_apart.length()
    };
    let edges =
        |&[first, second, third]: &[u32; 3]| [[first, second], [second, third], [third, first]];
    let triangles = geometry.indices.as_chunks::<3>().0;
    let ring_positions: HashSet<[u32; 3]> = triangles
        .iter()
        .flat_map(edges)
        .filter(|&edge| on_one_ring(edge))
        .flatten()
        .map(position_bits)
        .collect();
    let widest_cap_edge_metres = 2.0 * radius_metres * (1.0 + THIN_RADIUS_TOLERANCE);
    let stray = triangles.iter().find(|&triangle| {
        let on_rings =
            (triangle.iter()).all(|&vertex| ring_positions.contains(&position_bits(vertex)));
        let is_side = edges(triangle).into_iter().any(on_one_ring);
        let is_cap = edges(triangle).into_iter().all(|[first, second]| {
            position(first).distance(position(second)) <= widest_cap_edge_metres
        });
        !(on_rings && (is_side || is_cap))
    });
    if let Some(&[first, second, third]) = stray {
        anyhow::bail!(
            "it is declared thin with a radius of {radius_metres} m, and its triangle at {}, {}, {} is neither on a tube of that radius nor a cap of one",
            position(first),
            position(second),
            position(third)
        );
    }
    Ok(())
}

/// Cooks one Mesh from its Primitives' geometry and Material Asset IDs, or
/// refuses one with a vertex past `MAX_PIVOT_DISTANCE_METRES` from its pivot
/// or a Thin Primitive without radial normals.
pub(crate) fn cook(primitives: &[SourcePrimitive]) -> anyhow::Result<CookedMesh> {
    let farthest_metres = primitives
        .iter()
        .flat_map(|(geometry, _)| &geometry.positions)
        .map(|position| Vec3::from(*position).length())
        .fold(0.0, f32::max);
    anyhow::ensure!(
        farthest_metres <= MAX_PIVOT_DISTANCE_METRES,
        "a vertex lies {farthest_metres:.1} m from the Mesh's pivot, past ADR 0014's {MAX_PIVOT_DISTANCE_METRES} m"
    );
    let mut mesh = lod_0(primitives)?;
    mesh.coarser_lods = coarser_lods(&mesh)?;
    Ok(mesh)
}

pub(super) fn lod_0(primitives: &[SourcePrimitive]) -> anyhow::Result<CookedMesh> {
    let mut mesh = CookedMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        primitives: Vec::new(),
        coarser_lods: Vec::new(),
        bounds: [[0.0; 3]; 2],
    };
    let mut triangle_counts = Vec::new();
    for (primitive_index, (geometry, material)) in primitives.iter().enumerate() {
        if let Some(radius_metres) = geometry.thin_radius_metres {
            check_thin_primitive(geometry, radius_metres)
                .with_context(|| format!("Primitive {primitive_index}"))?;
        }
        let (vertices, indices) = cook_primitive(geometry);
        let triangles = indices.as_chunks::<3>().0;
        triangle_counts.push(meshlet_triangle_counts(triangles, vertices.len()));
        let first_vertex = mesh.vertices.len() as u32;
        mesh.primitives.push(CookedPrimitive {
            first_index: mesh.indices.len() as u32,
            index_count: indices.len() as u32,
            material: *material,
            texel_density: texel_density(geometry),
            thin_radius_metres: geometry.thin_radius_metres,
            meshlets: Vec::new(),
        });
        mesh.indices
            .extend(indices.iter().map(|index| index + first_vertex));
        mesh.vertices.extend(vertices);
    }
    // Renumbering the vertices keeps each run, so the cuts above hold.
    mesh.vertices = meshopt::optimize_vertex_fetch(&mut mesh.indices, &mesh.vertices);
    for (primitive, triangle_counts) in mesh.primitives.iter_mut().zip(triangle_counts) {
        let first_index = primitive.first_index as usize;
        let indices = &mesh.indices[first_index..][..primitive.index_count as usize];
        primitive.meshlets = CookedMeshlet::split(indices.as_chunks::<3>().0, triangle_counts)
            .expect("the cut keeps to the Meshlet limits");
    }
    let positions = mesh
        .vertices
        .iter()
        .map(|vertex| Vec3::from(vertex.position));
    let held = primitives
        .iter()
        .filter_map(|(geometry, _)| geometry.bounds)
        .flat_map(|[lowest, highest]| [Vec3::from(lowest), Vec3::from(highest)]);
    let corners = positions.chain(held);
    mesh.bounds = [
        corners.clone().fold(Vec3::INFINITY, Vec3::min).into(),
        corners.fold(Vec3::NEG_INFINITY, Vec3::max).into(),
    ];
    Ok(mesh)
}

/// `geometries` are the Mesh's Primitives in order. A vertex an earlier LOD
/// holds in the same Primitive is shared, never one of another Primitive.
pub(super) fn push_lod(
    mesh: &mut CookedMesh,
    error_metres: f32,
    geometries: &[&Geometry],
) -> anyhow::Result<()> {
    let mut lod = CookedLod {
        error_metres,
        indices: Vec::new(),
        primitives: Vec::new(),
    };
    for (primitive_index, geometry) in geometries.iter().enumerate() {
        let held = mesh.lods().flat_map(|(_, indices, primitives)| {
            let primitive = &primitives[primitive_index];
            &indices[primitive.first_index as usize..][..primitive.index_count as usize]
        });
        let mut index_of_vertex: HashMap<[u8; size_of::<CookedVertex>()], u32> = held
            .map(|&index| (bytemuck::cast(mesh.vertices[index as usize]), index))
            .collect();
        let (vertices, indices) = cook_primitive(geometry);
        let mesh_indices: Vec<u32> = vertices
            .iter()
            .map(|vertex| {
                *index_of_vertex
                    .entry(bytemuck::cast(*vertex))
                    .or_insert_with(|| {
                        mesh.vertices.push(*vertex);
                        mesh.vertices.len() as u32 - 1
                    })
            })
            .collect();
        let indices: Vec<u32> = indices
            .iter()
            .map(|&index| mesh_indices[index as usize])
            .collect();
        let triangles = indices.as_chunks::<3>().0;
        let finest = &mesh.primitives[primitive_index];
        lod.primitives.push(CookedPrimitive {
            first_index: lod.indices.len() as u32,
            index_count: indices.len() as u32,
            material: finest.material,
            texel_density: finest.texel_density,
            thin_radius_metres: finest.thin_radius_metres,
            meshlets: CookedMeshlet::split(
                triangles,
                meshlet_triangle_counts(triangles, mesh.vertices.len()),
            )?,
        });
        lod.indices.extend(indices);
    }
    mesh.coarser_lods.push(lod);
    Ok(())
}

thread_local! {
    static PROBE_SCALAR_NORMAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `mesh`'s LODs past LOD 0: each simplified from the one before to half its
/// triangles, the whole Mesh in one call so that a border between Primitives
/// collapses on both sides alike, until `MESH_MAX_LODS` or a LOD that keeps
/// more than `LOD_KEPT_INDICES_CEILING` of the one before, or nothing. A
/// LOD's error is the sum of the simplifier's errors down to it
/// [ref:meshoptimizer-0.25-lod-chain], each relative to the Mesh's extent
/// and so times `simplify_scale` in metres
/// [ref:meshoptimizer-0.25-simplify-scale].
fn coarser_lods(mesh: &CookedMesh) -> anyhow::Result<Vec<CookedLod>> {
    let positions = meshopt::VertexDataAdapter::new(
        bytemuck::cast_slice(&mesh.vertices),
        size_of::<CookedVertex>(),
        0,
    )
    .expect("a whole number of vertices, position first");
    let extent_metres = meshopt::simplify_scale(&positions);
    // No two Primitives share a vertex, so each vertex names its Primitive.
    let mut primitive_of_vertex = vec![0; mesh.vertices.len()];
    for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
        for &vertex in
            &mesh.indices[primitive.first_index as usize..][..primitive.index_count as usize]
        {
            primitive_of_vertex[vertex as usize] = primitive_index;
        }
    }
    let attributes: Vec<[f32; 5]> = mesh
        .vertices
        .iter()
        .zip(&primitive_of_vertex)
        .map(|(vertex, &primitive_index)| {
            let normal = if PROBE_SCALAR_NORMAL.get() {
                let [x, y, z, w] = vertex.qtangent.map(f32::from);
                let length_squared = x * x + y * y + z * z + w * w;
                Vec3::new(
                    2.0 * (x * z + w * y) / length_squared,
                    2.0 * (y * z - w * x) / length_squared,
                    1.0 - 2.0 * (x * x + y * y) / length_squared,
                )
            } else {
                Quat::from_array(vertex.qtangent.map(f32::from)).normalize() * Vec3::Z
            };
            let uv_units_across_extent =
                mesh.primitives[primitive_index].texel_density * extent_metres;
            let [u, v] = vertex.uv.map(|coordinate| {
                if uv_units_across_extent > 0.0 {
                    coordinate / uv_units_across_extent
                } else {
                    0.0
                }
            });
            [normal.x, normal.y, normal.z, u, v]
        })
        .collect();
    let weights = [
        LOD_NORMAL_WEIGHT,
        LOD_NORMAL_WEIGHT,
        LOD_NORMAL_WEIGHT,
        LOD_UV_WEIGHT,
        LOD_UV_WEIGHT,
    ];
    let unlocked = vec![false; mesh.vertices.len()];
    let mut lods = Vec::new();
    let mut finer = mesh.indices.clone();
    let mut error_metres = 0.0;
    while lods.len() + 1 < MESH_MAX_LODS && !finer.is_empty() {
        let mut relative_error = 0.0;
        let simplified = meshopt::simplify_with_attributes_and_locks(
            &finer,
            &positions,
            attributes.as_flattened(),
            &weights,
            size_of::<[f32; 5]>(),
            &unlocked,
            finer.len() / 6 * 3,
            // No error limit [ref:meshoptimizer-0.25-simplify-error].
            f32::MAX,
            meshopt::SimplifyOptions::None,
            Some(&mut relative_error),
        );
        if simplified.is_empty()
            || simplified.len() as f32 > finer.len() as f32 * LOD_KEPT_INDICES_CEILING
        {
            break;
        }
        error_metres += relative_error * extent_metres;
        let mut lod = CookedLod {
            error_metres,
            indices: Vec::new(),
            primitives: Vec::new(),
        };
        for (primitive_index, primitive) in mesh.primitives.iter().enumerate() {
            // The simplifier keeps a triangle on its Primitive's vertices
            // [ref:meshoptimizer-0.25-simplify-materials].
            let mut indices: Vec<u32> = simplified
                .as_chunks::<3>()
                .0
                .iter()
                .filter(|triangle| primitive_of_vertex[triangle[0] as usize] == primitive_index)
                .flatten()
                .copied()
                .collect();
            anyhow::ensure!(
                indices
                    .iter()
                    .all(|&vertex| primitive_of_vertex[vertex as usize] == primitive_index),
                "the simplifier gave a triangle vertices of two Primitives"
            );
            if !indices.is_empty() {
                indices = meshopt::optimize_vertex_cache(&indices, mesh.vertices.len());
                meshopt::optimize_overdraw_in_place(&mut indices, &positions, OVERDRAW_THRESHOLD);
            }
            let triangles = indices.as_chunks::<3>().0;
            lod.primitives.push(CookedPrimitive {
                first_index: lod.indices.len() as u32,
                index_count: indices.len() as u32,
                material: primitive.material,
                texel_density: primitive.texel_density,
                thin_radius_metres: primitive.thin_radius_metres,
                meshlets: CookedMeshlet::split(
                    triangles,
                    meshlet_triangle_counts(triangles, mesh.vertices.len()),
                )?,
            });
            lod.indices.extend(indices);
        }
        lods.push(lod);
        finer = simplified;
    }
    Ok(lods)
}

/// `geometry`'s UV units per metre, as `CookedPrimitive::texel_density`
/// defines it.
fn texel_density(geometry: &Geometry) -> f32 {
    let (mut uv_area, mut mesh_area) = (0.0f64, 0.0f64);
    for triangle in geometry.indices.as_chunks::<3>().0 {
        let [first, second, third] = triangle.map(|index| index as usize);
        let position = |vertex: usize| Vec3::from(geometry.positions[vertex]).as_dvec3();
        let uv = |vertex: usize| Vec2::from(geometry.uvs[vertex]).as_dvec2();
        mesh_area += (position(second) - position(first))
            .cross(position(third) - position(first))
            .length();
        uv_area += (uv(second) - uv(first))
            .perp_dot(uv(third) - uv(first))
            .abs();
    }
    if mesh_area > 0.0 {
        (uv_area / mesh_area).sqrt() as f32
    } else {
        0.0
    }
}

/// One Primitive's vertices and indices, welded, re-split and ordered for
/// vertex cache and overdraw.
fn cook_primitive(geometry: &Geometry) -> (Vec<CookedVertex>, Vec<u32>) {
    if geometry.indices.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let welded = weld(geometry);
    let tangents = corner_tangents(
        &welded.positions,
        &welded.normals,
        &welded.uvs,
        &welded.indices,
    );
    let corners: Vec<CookedVertex> = welded
        .indices
        .iter()
        .zip(tangents)
        .map(|(&index, tangent)| {
            let index = index as usize;
            CookedVertex {
                position: welded.positions[index],
                uv: welded.uvs[index],
                qtangent: qtangent(welded.normals[index], tangent),
            }
        })
        .collect();
    // Corners of one welded vertex stay apart where their QTangents differ.
    let (vertex_count, remap) = meshopt::generate_vertex_remap(&corners, None);
    let vertices = meshopt::remap_vertex_buffer(&corners, vertex_count, &remap);
    let indices = meshopt::remap_index_buffer(None, corners.len(), &remap);
    let mut indices = meshopt::optimize_vertex_cache(&indices, vertex_count);
    let positions = meshopt::VertexDataAdapter::new(
        bytemuck::cast_slice(&vertices),
        size_of::<CookedVertex>(),
        0,
    )
    .expect("a whole number of vertices, position first");
    meshopt::optimize_overdraw_in_place(&mut indices, &positions, OVERDRAW_THRESHOLD);
    (vertices, indices)
}

/// Each Meshlet's triangle count when `triangles`, indexing `vertex_count`
/// vertices, are cut in order into the longest runs the Meshlet limits
/// allow, like meshoptimizer's `meshopt_buildMeshletsScan`, which `meshopt`
/// binds only as an `unsafe` call and which counts a degenerate triangle's
/// repeated corner twice [ref:meshoptimizer-0.25-scan-meshlets]. Kept in
/// order, the indices stay as the vertex cache and overdraw passes left
/// them, and so does every image.
fn meshlet_triangle_counts(triangles: &[[u32; 3]], vertex_count: usize) -> Vec<usize> {
    let mut in_run = vec![false; vertex_count];
    let mut run_vertices = Vec::new();
    let mut counts: Vec<usize> = Vec::new();
    for triangle in triangles {
        let mut corners = *triangle;
        corners.sort_unstable();
        let new_vertices = (0..3)
            .filter(|&corner| {
                !in_run[corners[corner] as usize]
                    && (corner == 0 || corners[corner] != corners[corner - 1])
            })
            .count();
        if counts
            .last()
            .is_none_or(|&count| count == MESHLET_MAX_TRIANGLES)
            || run_vertices.len() + new_vertices > MESHLET_MAX_VERTICES
        {
            for vertex in run_vertices.drain(..) {
                in_run[vertex as usize] = false;
            }
            counts.push(0);
        }
        for vertex in corners {
            if !in_run[vertex as usize] {
                in_run[vertex as usize] = true;
                run_vertices.push(vertex);
            }
        }
        *counts.last_mut().expect("a run is open") += 1;
    }
    counts
}

/// `geometry` with the vertices whose position, UV and normal are
/// bit-for-bit equal merged into one.
pub(crate) fn weld(geometry: &Geometry) -> Geometry {
    // Position, UV, normal.
    let keys: Vec<[f32; 8]> = (0..geometry.positions.len())
        .map(|vertex| {
            let ([x, y, z], [u, v], [normal_x, normal_y, normal_z]) = (
                geometry.positions[vertex],
                geometry.uvs[vertex],
                geometry.normals[vertex],
            );
            [x, y, z, u, v, normal_x, normal_y, normal_z]
        })
        .collect();
    let (vertex_count, remap) = meshopt::generate_vertex_remap(&keys, Some(&geometry.indices));
    let keys = meshopt::remap_vertex_buffer(&keys, vertex_count, &remap);
    Geometry {
        positions: keys.iter().map(|key| [key[0], key[1], key[2]]).collect(),
        uvs: keys.iter().map(|key| [key[3], key[4]]).collect(),
        normals: keys.iter().map(|key| [key[5], key[6], key[7]]).collect(),
        indices: meshopt::remap_index_buffer(Some(&geometry.indices), vertex_count, &remap),
        bounds: None,
        thin_radius_metres: None,
    }
}

/// Indexed triangle mesh viewed through the MikkTSpace `Geometry` interface.
/// MikkTSpace works per face-vertex, so the tangents land per corner.
struct TangentGeometry<'a> {
    positions: &'a [[f32; 3]],
    normals: &'a [[f32; 3]],
    uvs: &'a [[f32; 2]],
    indices: &'a [u32],
    tangents: Vec<[f32; 4]>,
}

impl TangentGeometry<'_> {
    fn vertex_index(&self, face: usize, vertex: usize) -> usize {
        self.indices[face * 3 + vertex] as usize
    }
}

impl bevy_mikktspace::Geometry for TangentGeometry<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vertex: usize) -> [f32; 3] {
        self.positions[self.vertex_index(face, vertex)]
    }

    fn normal(&self, face: usize, vertex: usize) -> [f32; 3] {
        self.normals[self.vertex_index(face, vertex)]
    }

    /// MikkTSpace puts the bitangent along +v. glTF's `v` runs down the
    /// image [ref:gltf-2.0-texture-origin] while its normal Textures take +Y
    /// as up [ref:gltf-2.0-normal-texture-convention], so `v` is flipped.
    fn tex_coord(&self, face: usize, vertex: usize) -> [f32; 2] {
        let [u, v] = self.uvs[self.vertex_index(face, vertex)];
        [u, 1.0 - v]
    }

    fn set_tangent(
        &mut self,
        tangent_space: Option<bevy_mikktspace::TangentSpace>,
        face: usize,
        vertex: usize,
    ) {
        // `None` means a degenerate face with no neighbour to borrow from.
        self.tangents[face * 3 + vertex] = tangent_space.unwrap_or_default().tangent_encoded();
    }
}

/// MikkTSpace tangents in glTF's convention, one per index: each corner's
/// own. glTF computes a Primitive's missing tangents this way
/// [ref:gltf-2.0-mikktspace-tangents].
fn corner_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let mut geometry = TangentGeometry {
        positions,
        normals,
        uvs,
        indices,
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; indices.len()],
    };
    if bevy_mikktspace::generate_tangents(&mut geometry).is_err() {
        tracing::error!("MikkTSpace tangent generation failed; falling back to +X tangents");
    }
    geometry.tangents
}

/// The QTangent of a glTF normal and tangent (`xyz`, `w` the bitangent
/// sign [ref:gltf-2.0-tangent-bitangent]): the rotation whose columns are
/// the tangent made perpendicular to the normal, the bitangent
/// cross(normal, tangent) and the normal, with its w's sign set to the
/// bitangent sign.
pub(crate) fn qtangent(normal: [f32; 3], tangent: [f32; 4]) -> [i16; 4] {
    let normal = Vec3::from(normal).normalize_or(Vec3::Z);
    let [tangent_x, tangent_y, tangent_z, bitangent_sign] = tangent;
    let tangent = Vec3::new(tangent_x, tangent_y, tangent_z);
    let tangent = (tangent - normal * normal.dot(tangent))
        .try_normalize()
        .unwrap_or_else(|| normal.any_orthonormal_vector());
    let frame = Mat3::from_cols(tangent, normal.cross(tangent), normal);
    let mut rotation = Quat::from_mat3(&frame);
    rotation = if PROBE_SCALAR_NORMAL.get() {
        let [x, y, z, w] = rotation.to_array();
        let length = (x * x + y * y + z * z + w * w).sqrt();
        Quat::from_xyzw(x / length, y / length, z / length, w / length)
    } else {
        rotation.normalize()
    };
    // A quaternion and its negation are the same rotation.
    if rotation.w < 0.0 {
        rotation = -rotation;
    }
    if rotation.w < QTANGENT_SMALLEST_W {
        let axis =
            rotation.xyz().normalize() * (1.0 - QTANGENT_SMALLEST_W * QTANGENT_SMALLEST_W).sqrt();
        rotation = Quat::from_xyzw(axis.x, axis.y, axis.z, QTANGENT_SMALLEST_W);
    }
    if bitangent_sign < 0.0 {
        rotation = -rotation;
    }
    rotation
        .to_array()
        .map(|component| (component * i16::MAX as f32).round() as i16)
}

/// Every Mesh of a glTF file, with its name, in mesh-local space, with each
/// Primitive's geometry and the Asset ID of its Material, named as the
/// Material cook names it; a Primitive on glTF's default Material names none,
/// and one whose `extras` hold `thin_radius_metres` is a Thin Primitive.
pub(super) fn gltf_meshes<'document>(
    guid: u128,
    document: &'document gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> anyhow::Result<Vec<(AssetId, AssetName<'document>, Vec<SourcePrimitive>)>> {
    let source = AssetSource::File(guid);
    let material_names = super::asset_names(document.materials().map(|material| material.name()));
    let mesh_names = super::asset_names(document.meshes().map(|mesh| mesh.name()));
    let mut meshes = Vec::new();
    for (mesh, name) in document.meshes().zip(mesh_names) {
        let mut primitives = Vec::new();
        for primitive in mesh.primitives() {
            anyhow::ensure!(
                primitive.mode() == gltf::mesh::Mode::Triangles,
                "{name:?}: primitive mode {:?} is not triangles",
                primitive.mode()
            );
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| anyhow::anyhow!("{name:?}: a primitive has no positions"))?
                .collect();
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .ok_or_else(|| anyhow::anyhow!("{name:?}: a primitive has no normals"))?
                .collect();
            let uvs = reader.read_tex_coords(0).map_or_else(
                || vec![[0.0; 2]; positions.len()],
                |uvs| uvs.into_f32().collect(),
            );
            let indices = reader.read_indices().map_or_else(
                || (0..positions.len() as u32).collect(),
                |indices| indices.into_u32().collect(),
            );
            let material = primitive
                .material()
                .index()
                .map(|index| AssetId::new(source, AssetKind::Material, material_names[index]));
            let thin_radius_metres = (primitive.extras().as_deref())
                .map(|extras| serde_json::from_str::<serde_json::Value>(extras.get()))
                .transpose()?
                .as_ref()
                .and_then(|extras| extras.get("thin_radius_metres"))
                .map(|radius_metres| {
                    radius_metres.as_f64().with_context(|| {
                        format!("{name:?}: a primitive's thin_radius_metres is {radius_metres}")
                    })
                })
                .transpose()?
                .map(|radius_metres| radius_metres as f32);
            let geometry = Geometry {
                positions,
                normals,
                uvs,
                indices,
                bounds: None,
                thin_radius_metres,
            };
            primitives.push((geometry, material));
        }
        meshes.push((
            AssetId::new(source, AssetKind::Mesh, name),
            name,
            primitives,
        ));
    }
    Ok(meshes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ReferenceScene;
    use crate::cooked::CookedCache;

    #[test]
    fn the_cook_key_covers_every_part_a_mesh_cooks_from() {
        let triangle = || {
            let geometry = Geometry {
                positions: vec![[0.0; 3]; 3],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                uvs: vec![[0.0; 2]; 3],
                indices: vec![0, 1, 2],
                bounds: None,
                thin_radius_metres: None,
            };
            (geometry, None)
        };
        let key = cook_key(&[triangle()]);
        let changes: [fn(&mut SourcePrimitive); 7] = [
            |(geometry, _)| geometry.positions[0][0] = 1.0,
            |(geometry, _)| geometry.normals[0][2] = -1.0,
            |(geometry, _)| geometry.uvs[0][0] = 1.0,
            |(geometry, _)| geometry.indices[0] = 2,
            |(geometry, _)| geometry.bounds = Some([[-1.0; 3], [1.0; 3]]),
            |(geometry, _)| geometry.thin_radius_metres = Some(0.5),
            |(_, material)| *material = Some(AssetId(1)),
        ];
        for change in changes {
            let mut primitive = triangle();
            change(&mut primitive);
            assert_ne!(cook_key(&[primitive]), key);
        }
    }

    #[test]
    fn towns_wire_and_railing_post_cook_as_thin_primitives() {
        let meshes = ReferenceScene::Town.meshes();
        for (name, radius_metres) in [("wire", 0.015), ("railing-post", 0.025)] {
            let (_, primitives) = meshes.iter().find(|(mesh, _)| *mesh == name).unwrap();
            let primitives: Vec<SourcePrimitive> = primitives
                .iter()
                .map(|(geometry, _)| (geometry.clone(), None))
                .collect();
            let cooked = cook(&primitives).unwrap();
            for (_, _, primitives) in cooked.lods() {
                let radii: Vec<_> = primitives
                    .iter()
                    .map(|primitive| primitive.thin_radius_metres)
                    .collect();
                assert_eq!(radii, [Some(radius_metres)], "{name}");
            }
        }
    }

    #[test]
    fn a_gltf_primitives_extras_declare_it_thin() {
        let primitive =
            |extras: &str| format!(r#"{{"attributes": {{"POSITION": 0, "NORMAL": 0}}{extras}}}"#);
        let meshes = |primitives: [String; 3]| {
            format!(
                r#"{{
                    "asset": {{"version": "2.0"}},
                    "buffers": [{{"byteLength": 36}}],
                    "bufferViews": [{{"buffer": 0, "byteLength": 36}}],
                    "accessors": [{{
                        "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3",
                        "min": [0, 0, 0], "max": [0, 0, 0]
                    }}],
                    "meshes": [{{"primitives": [{}]}}]
                }}"#,
                primitives.join(", ")
            )
        };
        let read = |json: String| {
            let gltf = gltf::Gltf::from_slice(json.as_bytes()).unwrap();
            let buffers = [gltf::buffer::Data(vec![0; 36])];
            gltf_meshes(0, &gltf.document, &buffers).map(|meshes| {
                (meshes[0].2.iter())
                    .map(|(geometry, _)| geometry.thin_radius_metres)
                    .collect::<Vec<_>>()
            })
        };
        let radii = read(meshes([
            primitive(r#", "extras": {"thin_radius_metres": 0.015}"#),
            primitive(""),
            primitive(r#", "extras": 3"#),
        ]));
        assert_eq!(radii.unwrap(), [Some(0.015), None, None]);
        let not_a_number = primitive(r#", "extras": {"thin_radius_metres": "thin"}"#);
        let error = read(meshes([not_a_number, primitive(""), primitive("")])).unwrap_err();
        assert!(error.to_string().contains("thin_radius_metres"), "{error}");
    }

    /// One Snorm16 step of a component.
    const STEP: f32 = 1.0 / i16::MAX as f32;

    /// The normal, tangent and bitangent sign a QTangent holds, decoded as
    /// the shader will: the rotation applied to +Z and +X, and w's sign.
    fn decode(qtangent: [i16; 4]) -> (Vec3, Vec3, f32) {
        let rotation =
            Quat::from_array(qtangent.map(|component| (component as f32 * STEP).max(-1.0)))
                .normalize();
        (rotation * Vec3::Z, rotation * Vec3::X, rotation.w.signum())
    }

    /// Frames to encode, as rotations of the identity: about each axis,
    /// about skewed axes, and near the half turns whose w is near zero.
    pub(crate) fn sample_rotations() -> Vec<Quat> {
        let mut rotations: Vec<Quat> = Vec::new();
        for step in 0..=64 {
            let angle = std::f32::consts::TAU * step as f32 / 64.0;
            for axis in [
                Vec3::X,
                Vec3::Y,
                Vec3::Z,
                Vec3::new(1.0, 2.0, -3.0).normalize(),
            ] {
                rotations.push(Quat::from_axis_angle(axis, angle));
            }
        }
        for near_half_turn in [1e-7, 1e-5, 3e-5, 1e-4] {
            let angle = std::f32::consts::PI - near_half_turn;
            rotations.push(Quat::from_axis_angle(Vec3::new(0.6, 0.0, 0.8), angle));
        }
        rotations
    }

    #[test]
    fn qtangents_decode_to_their_frame_within_snorm16_precision() {
        let mut worst = 0.0f32;
        for rotation in sample_rotations() {
            for bitangent_sign in [1.0, -1.0] {
                let (normal, tangent) = (rotation * Vec3::Z, rotation * Vec3::X);
                let (decoded_normal, decoded_tangent, decoded_sign) = decode(qtangent(
                    normal.into(),
                    tangent.extend(bitangent_sign).into(),
                ));
                assert_eq!(decoded_sign, bitangent_sign, "rotation {rotation}");
                worst = worst
                    .max((decoded_normal - normal).abs().max_element())
                    .max((decoded_tangent - tangent).abs().max_element());
            }
        }
        // A rotated axis is a product of two quaternions, each component off
        // by at most half a step, plus the nudge that keeps w a whole step.
        assert!(worst <= 4.0 * STEP, "worst component error {worst}");
    }

    #[test]
    fn a_triangle_soup_plane_welds_to_four_vertices() {
        let plane = crate::scenes::ground_plane(1.0, 1.0);
        let corners = |attribute: &[[f32; 3]]| -> Vec<[f32; 3]> {
            plane
                .indices
                .iter()
                .map(|&index| attribute[index as usize])
                .collect()
        };
        let soup = Geometry {
            positions: corners(&plane.positions),
            normals: corners(&plane.normals),
            uvs: plane
                .indices
                .iter()
                .map(|&index| plane.uvs[index as usize])
                .collect(),
            indices: (0..6).collect(),
            bounds: None,
            thin_radius_metres: None,
        };
        assert_eq!(weld(&soup).positions.len(), 4);
        assert_eq!(cook(&[(soup, None)]).unwrap().vertices.len(), 4);
    }

    #[test]
    fn a_square_metre_quad_has_the_uv_units_per_metre_its_uvs_span() {
        for uv_repeat in [1.0, 2.0] {
            let mesh = cook(&[(crate::scenes::ground_plane(0.5, uv_repeat), None)]).unwrap();
            assert_eq!(mesh.primitives[0].texel_density, uv_repeat);
        }
    }

    /// The plane's `u` grows along +X and its `v` along +Z, down the image,
    /// so its tangent is +X, its normal +Y, and its bitangent, up the image,
    /// −Z: cross(+Y, +X) times a bitangent sign of +1.
    #[test]
    fn a_plane_has_its_uv_aligned_tangent_frame() {
        let mesh = cook(&[(crate::scenes::ground_plane(2.0, 1.0), None)]).unwrap();
        for vertex in &mesh.vertices {
            let (normal, tangent, sign) = decode(vertex.qtangent);
            assert!(
                normal.abs_diff_eq(Vec3::Y, 4.0 * STEP)
                    && tangent.abs_diff_eq(Vec3::X, 4.0 * STEP)
                    && sign == 1.0,
                "normal {normal}, tangent {tangent}, bitangent sign {sign}"
            );
        }
    }

    /// The meshes of the committed and, when fetched, the fetched glTF files
    /// the Reference scenes draw.
    fn reference_gltf_meshes() -> Vec<(&'static str, Vec<SourcePrimitive>)> {
        let mut meshes = Vec::new();
        for path in [
            "assets/NormalTangentTest/NormalTangentTest.glb",
            "assets/SciFiHelmet/SciFiHelmet.gltf",
        ] {
            // SciFiHelmet is fetched, so absent from some checkouts.
            let Ok(gltf) = gltf::Gltf::open(path) else {
                continue;
            };
            let base = std::path::Path::new(path).parent();
            let buffers = gltf::import_buffers(&gltf.document, base, gltf.blob.clone()).unwrap();
            for (_, _, primitives) in gltf_meshes(0, &gltf.document, &buffers).unwrap() {
                meshes.push((path, primitives));
            }
        }
        meshes
    }

    /// `triangle` rotated to its lowest rotation: meshopt's index codec may
    /// rotate a triangle, which keeps its winding.
    fn rotate_to_lowest<T: Ord + Copy>(triangle: &mut [T; 3]) {
        *triangle = (0..3)
            .map(|start| {
                let mut rotated = *triangle;
                rotated.rotate_left(start);
                rotated
            })
            .min()
            .unwrap();
    }

    /// `mesh` with every triangle, of each LOD's index stream and of each
    /// Meshlet, at its lowest rotation.
    pub(crate) fn with_lowest_rotations(mut mesh: CookedMesh) -> CookedMesh {
        let coarser = mesh
            .coarser_lods
            .iter_mut()
            .map(|lod| (&mut lod.indices, &mut lod.primitives));
        for (indices, primitives) in
            std::iter::once((&mut mesh.indices, &mut mesh.primitives)).chain(coarser)
        {
            indices
                .as_chunks_mut::<3>()
                .0
                .iter_mut()
                .for_each(rotate_to_lowest);
            for meshlet in primitives
                .iter_mut()
                .flat_map(|primitive| &mut primitive.meshlets)
            {
                meshlet.triangles.iter_mut().for_each(rotate_to_lowest);
            }
        }
        mesh
    }

    /// `triangles` with each corner as the bits of its position and UV, at
    /// their lowest rotation, sorted: what a cook keeps of a Primitive,
    /// whatever it welds, reorders or rotates.
    fn corner_triangles(
        triangles: impl Iterator<Item = [([f32; 3], [f32; 2]); 3]>,
    ) -> Vec<[[u32; 5]; 3]> {
        let mut triangles: Vec<_> = triangles
            .map(|triangle| {
                let mut corners =
                    triangle.map(|([x, y, z], [u, v])| [x, y, z, u, v].map(f32::to_bits));
                rotate_to_lowest(&mut corners);
                corners
            })
            .collect();
        triangles.sort_unstable();
        triangles
    }

    /// Checks `mesh`, as read back, against the Primitives it cooked from:
    /// each Primitive's Meshlets, of at most 64 vertices and 124 triangles,
    /// hold its triangles in index stream order, each once, through their
    /// vertices; those are the source's triangles; the bounds are the box
    /// around the source's vertices and any box a Primitive adds, so they
    /// hold every vertex.
    fn check_meshlets(name: &str, mesh: &CookedMesh, sources: &[SourcePrimitive]) {
        let corner = |vertex: u32| {
            let CookedVertex { position, uv, .. } = mesh.vertices[vertex as usize];
            (position, uv)
        };
        let mut bounds = [Vec3::INFINITY, Vec3::NEG_INFINITY];
        for (primitive, (geometry, _)) in mesh.primitives.iter().zip(sources) {
            let first_index = primitive.first_index as usize;
            let indices = &mesh.indices[first_index..][..primitive.index_count as usize];
            let mut stream = indices.as_chunks::<3>().0.iter();
            let mut triangles = Vec::new();
            for meshlet in &primitive.meshlets {
                let (vertex_count, triangle_count) =
                    (meshlet.vertices.len(), meshlet.triangles.len());
                assert!(
                    vertex_count <= 64 && (1..=124).contains(&triangle_count),
                    "{name}: a Meshlet of {vertex_count} vertices and {triangle_count} triangles"
                );
                for local in &meshlet.triangles {
                    let triangle = local.map(|vertex| meshlet.vertices[vertex as usize]);
                    assert_eq!(
                        Some(&triangle),
                        stream.next(),
                        "{name}: a Meshlet triangle is not its Primitive's next"
                    );
                    triangles.push(triangle.map(corner));
                }
            }
            assert_eq!(stream.next(), None, "{name}: a triangle in no Meshlet");
            let source: Vec<_> = geometry
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .map(|triangle| {
                    triangle.map(|index| {
                        let index = index as usize;
                        (geometry.positions[index], geometry.uvs[index])
                    })
                })
                .collect();
            let added = geometry.bounds.into_iter().flatten();
            for position in source
                .iter()
                .flatten()
                .map(|&(position, _)| position)
                .chain(added)
            {
                let position = Vec3::from(position);
                bounds = [bounds[0].min(position), bounds[1].max(position)];
            }
            assert_eq!(
                corner_triangles(triangles.into_iter()),
                corner_triangles(source.into_iter()),
                "{name}: the Meshlets hold other triangles than the source's"
            );
        }
        assert_eq!(
            mesh.bounds,
            bounds.map(|corner| corner.to_array()),
            "{name}: the bounds are not the box around the source's vertices and added box"
        );
    }

    /// The cook seam: every Reference scene Mesh cooked into a scratch
    /// Cooked cache and read back by Asset ID is its cooked data, in valid
    /// Meshlets.
    #[test]
    fn every_reference_scene_mesh_reads_back_as_cooked_in_valid_meshlets() {
        let directory =
            std::env::temp_dir().join(format!("praxis-mesh-read-back-{}", std::process::id()));
        let cache = CookedCache::new(&directory);
        let procedural = ReferenceScene::ALL.into_iter().flat_map(|scene| {
            scene.meshes().into_iter().map(|(name, primitives)| {
                let primitives = primitives
                    .into_iter()
                    .map(|(geometry, _)| (geometry, Some(AssetId(1))))
                    .collect();
                (name, primitives)
            })
        });
        for (id, (name, primitives)) in procedural.chain(reference_gltf_meshes()).enumerate() {
            let mesh = cook(&primitives).unwrap();
            let id = AssetId(id as u128);
            cache.write(id, "key", &mesh.to_blob().unwrap()).unwrap();
            let decoded = cache.mesh(id).unwrap();
            check_meshlets(name, &decoded, &primitives);
            assert_eq!(
                with_lowest_rotations(decoded),
                with_lowest_rotations(mesh),
                "{name}"
            );
        }
        std::fs::remove_dir_all(&directory).ok();
    }

    #[test]
    fn probe_lod_chain_hashes() {
        use sha2::{Digest, Sha256};
        fn short_hex(hasher: Sha256) -> String {
            let digest = hasher.finalize();
            let mut text = String::new();
            for byte in digest.iter().take(6) {
                text.push_str(&format!("{byte:02x}"));
            }
            text
        }
        let mut lines = vec![format!("PROBE arch {}", std::env::consts::ARCH)];
        for scalar_normal in [false, true] {
            PROBE_SCALAR_NORMAL.set(scalar_normal);
            for scene in ReferenceScene::ALL {
                for (name, primitives) in scene.meshes() {
                    let primitives: Vec<SourcePrimitive> = primitives
                        .into_iter()
                        .map(|(geometry, _)| (geometry, Some(AssetId(1))))
                        .collect();
                    let mut source = Sha256::new();
                    for (geometry, _) in &primitives {
                        source.update(bytemuck::cast_slice::<[f32; 3], u8>(&geometry.positions));
                        source.update(bytemuck::cast_slice::<[f32; 3], u8>(&geometry.normals));
                        source.update(bytemuck::cast_slice::<[f32; 2], u8>(&geometry.uvs));
                        source.update(bytemuck::cast_slice::<u32, u8>(&geometry.indices));
                    }
                    let mesh = cook(&primitives).unwrap();
                    let mut positions = Sha256::new();
                    let mut qtangents = Sha256::new();
                    let mut uvs = Sha256::new();
                    for vertex in &mesh.vertices {
                        positions.update(bytemuck::bytes_of(&vertex.position));
                        qtangents.update(bytemuck::bytes_of(&vertex.qtangent));
                        uvs.update(bytemuck::bytes_of(&vertex.uv));
                    }
                    let mut indices = Sha256::new();
                    indices.update(bytemuck::cast_slice::<u32, u8>(&mesh.indices));
                    lines.push(format!(
                        "FIELDS scalar_normal={scalar_normal} {}/{name} source {} positions {} qtangents {} uvs {} indices {}",
                        scene.name(),
                        short_hex(source),
                        short_hex(positions),
                        short_hex(qtangents),
                        short_hex(uvs),
                        short_hex(indices),
                    ));
                    let mut finest = Sha256::new();
                    finest.update(bytemuck::cast_slice::<CookedVertex, u8>(&mesh.vertices));
                    finest.update(bytemuck::cast_slice::<u32, u8>(&mesh.indices));
                    let mut coarser = Sha256::new();
                    let mut index_counts = Vec::new();
                    for lod in &mesh.coarser_lods {
                        coarser.update(lod.error_metres.to_le_bytes());
                        coarser.update(bytemuck::cast_slice::<u32, u8>(&lod.indices));
                        index_counts.push(lod.indices.len());
                    }
                    lines.push(format!(
                        "PROBE scalar_normal={scalar_normal} {}/{name} lod0 {} coarser {} {index_counts:?}",
                        scene.name(),
                        short_hex(finest),
                        short_hex(coarser),
                    ));
                }
            }
        }
        PROBE_SCALAR_NORMAL.set(false);
        for (path, primitives) in reference_gltf_meshes() {
            let mesh = cook(&primitives).unwrap();
            let mut vertices = Sha256::new();
            vertices.update(bytemuck::cast_slice::<CookedVertex, u8>(&mesh.vertices));
            vertices.update(bytemuck::cast_slice::<u32, u8>(&mesh.indices));
            let mut coarser = Sha256::new();
            for lod in &mesh.coarser_lods {
                coarser.update(bytemuck::cast_slice::<u32, u8>(&lod.indices));
            }
            lines.push(format!(
                "GLTF {path} lod0 {} coarser {}",
                short_hex(vertices),
                short_hex(coarser)
            ));
        }
        panic!("\n{}\n", lines.join("\n"));
    }

    /// The scenes' sphere, of 2,208 triangles that collapse, with
    /// `tetrahedra` tetrahedra of 1 m edges beside it whose faces share no
    /// vertex: each face has its own normal, so every corner is three
    /// vertices and none of their four triangles collapses.
    fn sphere_beside_faceted_tetrahedra(tetrahedra: u32) -> Geometry {
        let corners = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 0.0, 0.866],
            [0.5, 0.816, 0.289],
        ]
        .map(Vec3::from);
        let mut geometry = crate::scenes::uv_sphere(1.0, 48, 24);
        for tetrahedron in 0..tetrahedra {
            let cell = [tetrahedron % 20, tetrahedron / 20 % 20, tetrahedron / 400];
            let origin = Vec3::from(cell.map(|step| 2.0 + 1.5 * step as f32));
            for face in [[0, 1, 2], [0, 3, 1], [1, 3, 2], [2, 3, 0]] {
                let [first, second, third] = face.map(|corner| corners[corner]);
                let normal = (second - first).cross(third - first).normalize();
                for corner in [first, second, third] {
                    geometry.indices.push(geometry.positions.len() as u32);
                    geometry.positions.push((origin + corner).into());
                    geometry.normals.push(normal.into());
                    geometry.uvs.push([corner.x, corner.z]);
                }
            }
        }
        geometry
    }

    /// A sphere of 65,024 triangles halves seven times and could halve
    /// again; one ten times its size strays about ten times as far. Where four
    /// triangles in five cannot collapse, the LOD that keeps them and little
    /// else, some 80% of LOD 0, ends the chain as its LOD 1; where nine in
    /// ten cannot, that LOD, of some 90%, is dropped.
    #[test]
    fn a_lod_chain_halves_until_its_stop_rule_within_eight_lods_and_its_errors_never_decrease() {
        let chain = |radius: f32| {
            let mesh = cook(&[(crate::scenes::uv_sphere(radius, 256, 128), None)]).unwrap();
            let lods: Vec<(usize, f32)> = mesh
                .lods()
                .map(|(error_metres, indices, _)| (indices.len() / 3, error_metres))
                .collect();
            lods
        };
        let lods = chain(1.0);
        assert_eq!(lods.len(), 8, "{lods:?}");
        assert_eq!(lods[0], (65_024, 0.0));
        for pair in lods.windows(2) {
            let [(finer_triangles, finer_error), (triangles, error)] = [pair[0], pair[1]];
            // A collapse takes two triangles, so a LOD may pass its target by one.
            assert!(
                (finer_triangles / 2 - 1..=finer_triangles * 85 / 100).contains(&triangles),
                "{lods:?}"
            );
            assert!(error > 0.0 && error >= finer_error, "{lods:?}");
        }
        // The coarsest LOD, of some 500 triangles, strays less than the
        // sphere's radius and more than a hundredth of it.
        assert!((0.01..1.0).contains(&lods[7].1), "{lods:?}");
        let larger = chain(10.0);
        assert_eq!(larger.len(), lods.len());
        // Collapses of equal cost fall in another order at another size, so
        // the errors are ten times apart only within a factor of two.
        for ((_, error), (_, larger_error)) in lods.iter().zip(larger).skip(1) {
            assert!(
                (5.0..20.0).contains(&(larger_error / error)),
                "{error} m at 1 m, {larger_error} m at 10 m"
            );
        }

        let triangles = |tetrahedra: u32| -> Vec<usize> {
            cook(&[(sphere_beside_faceted_tetrahedra(tetrahedra), None)])
                .unwrap()
                .lods()
                .map(|(_, indices, _)| indices.len() / 3)
                .collect()
        };
        let four_in_five = triangles(2_208);
        assert_eq!((four_in_five.len(), four_in_five[0]), (2, 11_040));
        assert!(
            (8_832..8_832 + 552).contains(&four_in_five[1]),
            "{four_in_five:?}"
        );
        assert_eq!(triangles(4_968), [22_080]);
    }

    /// A sphere whose northern and southern halves are two Primitives, which
    /// share no vertex: along the equator each has its own. Its positions
    /// are snapped to a grid, a negative zero to zero, so that those the
    /// sphere's sines leave a rounding error apart, at its poles and where
    /// its UVs wrap, are equal.
    fn sphere_of_two_primitives() -> Vec<SourcePrimitive> {
        let mut sphere = crate::scenes::uv_sphere(1.0, 64, 32);
        for position in &mut sphere.positions {
            *position = position.map(|coordinate| (coordinate * 4096.0).round() / 4096.0 + 0.0);
        }
        [true, false]
            .map(|northern| {
                let mut half = sphere.clone();
                half.indices = sphere
                    .indices
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .filter(|triangle| {
                        let height: f32 = triangle
                            .iter()
                            .map(|&vertex| sphere.positions[vertex as usize][1])
                            .sum();
                        (height > 0.0) == northern
                    })
                    .flatten()
                    .copied()
                    .collect();
                (weld(&half), None)
            })
            .into()
    }

    /// A closed surface stays closed: in every LOD each triangle edge,
    /// between positions, is matched by one running the other way, so the
    /// equator's two Primitives collapsed the same edges; the equator did
    /// lose vertices, and each Primitive kept to its own half.
    #[test]
    fn no_lod_has_a_crack_along_a_border_between_primitives() {
        let mesh = cook(&sphere_of_two_primitives()).unwrap();
        assert!(mesh.coarser_lods.len() >= 4);
        let position = |vertex: u32| mesh.vertices[vertex as usize].position.map(f32::to_bits);
        let mut equator_vertices = Vec::new();
        for (lod, (_, indices, primitives)) in mesh.lods().enumerate() {
            let mut unmatched: std::collections::HashMap<[[u32; 3]; 2], i32> = Default::default();
            for triangle in indices.as_chunks::<3>().0 {
                for corner in 0..3 {
                    let (from, to) = (
                        position(triangle[corner]),
                        position(triangle[(corner + 1) % 3]),
                    );
                    *unmatched.entry([from.min(to), from.max(to)]).or_default() +=
                        if from < to { 1 } else { -1 };
                }
            }
            let open_edges = unmatched.values().filter(|&&count| count != 0).count();
            assert_eq!(open_edges, 0, "LOD {lod}");
            for (primitive, northern) in primitives.iter().zip([true, false]) {
                let run =
                    &indices[primitive.first_index as usize..][..primitive.index_count as usize];
                assert!(!run.is_empty());
                let side = if northern { 1.0 } else { -1.0 };
                assert!(
                    run.iter()
                        .all(|&vertex| mesh.vertices[vertex as usize].position[1] * side >= 0.0),
                    "LOD {lod}"
                );
            }
            let mut on_equator: Vec<_> = indices
                .iter()
                .filter(|&&vertex| mesh.vertices[vertex as usize].position[1].abs() < 1e-6)
                .map(|&vertex| position(vertex))
                .collect();
            on_equator.sort_unstable();
            on_equator.dedup();
            equator_vertices.push(on_equator.len());
        }
        assert_eq!(equator_vertices[0], 64);
        assert!(
            equator_vertices.last().unwrap() * 4 <= 64,
            "{equator_vertices:?}"
        );
    }

    /// `docs/research/asset-compression-and-cooked-data.md` (item 6) counted
    /// 14,085 vertices with position, UV and normal welded.
    #[test]
    fn sci_fi_helmet_welds_to_14085_vertices() {
        let meshes: Vec<_> = reference_gltf_meshes()
            .into_iter()
            .filter(|(path, _)| path.contains("SciFiHelmet"))
            .collect();
        if meshes.is_empty() {
            return;
        }
        let welded: usize = meshes
            .iter()
            .flat_map(|(_, primitives)| primitives)
            .map(|(geometry, _)| weld(geometry).positions.len())
            .sum();
        assert_eq!(welded, 14_085);
    }
}
