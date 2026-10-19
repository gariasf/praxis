//! The Mesh cook (ADR 0018): welds each Primitive on position, UV and
//! normal, regenerates MikkTSpace tangents, re-splits the vertices whose
//! tangents differ, packs them as 28-byte vertices with a QTangent, and orders
//! them for vertex cache, overdraw and vertex fetch.

use glam::{Mat3, Quat, Vec3};
use sha2::{Digest, Sha256};

use crate::cooked::{
    AssetId, AssetKind, AssetSource, CookedMesh, CookedPrimitive, CookedVertex,
    MESH_FORMAT_VERSION, MESH_MAGIC, hex,
};
use crate::scenes::Geometry;

/// A Primitive's geometry and the Asset ID of its Material, if it names one.
pub(crate) type SourcePrimitive = (Geometry, Option<AssetId>);

/// How far the overdraw pass may worsen the vertex cache order, as #96
/// measured it.
const OVERDRAW_THRESHOLD: f32 = 1.05;

/// Every encoder version and setting a Mesh Blob depends on, for its Cook key.
/// The meshoptimizer version is the one `meshopt` 0.6.2 vendors: update it
/// and the `bevy_mikktspace` version with their dependencies.
pub(super) fn encoder_settings() -> String {
    format!(
        "bevy_mikktspace 1.0.0, meshoptimizer 0.25, vertex codec 0, overdraw {OVERDRAW_THRESHOLD}, zstd {} level {}",
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
        } = geometry;
        hash_part(bytemuck::cast_slice(positions));
        hash_part(bytemuck::cast_slice(normals));
        hash_part(bytemuck::cast_slice(uvs));
        hash_part(bytemuck::cast_slice(indices));
        hash_part(&material.map_or(0, |material| material.0).to_be_bytes());
    }
    hex(&hasher.finalize())
}

/// Cooks one Mesh from its Primitives' geometry and Material Asset IDs.
pub(crate) fn cook(primitives: &[SourcePrimitive]) -> CookedMesh {
    let mut mesh = CookedMesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        primitives: Vec::new(),
    };
    for (geometry, material) in primitives {
        let (vertices, indices) = cook_primitive(geometry);
        let first_vertex = mesh.vertices.len() as u32;
        mesh.primitives.push(CookedPrimitive {
            first_index: mesh.indices.len() as u32,
            index_count: indices.len() as u32,
            material: *material,
        });
        mesh.indices
            .extend(indices.iter().map(|index| index + first_vertex));
        mesh.vertices.extend(vertices);
    }
    mesh.vertices = meshopt::optimize_vertex_fetch(&mut mesh.indices, &mesh.vertices);
    mesh
}

/// One Primitive's vertices and indices, welded, re-split and ordered for
/// vertex cache and overdraw.
fn cook_primitive(geometry: &Geometry) -> (Vec<CookedVertex>, Vec<u32>) {
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
    fn vertex_index(&self, face: usize, vert: usize) -> usize {
        self.indices[face * 3 + vert] as usize
    }
}

impl bevy_mikktspace::Geometry for TangentGeometry<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.positions[self.vertex_index(face, vert)]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.normals[self.vertex_index(face, vert)]
    }

    /// MikkTSpace puts the bitangent along +v. glTF's `v` runs down the
    /// image while its normal maps take +Y as up, so `v` is flipped.
    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let [u, v] = self.uvs[self.vertex_index(face, vert)];
        [u, 1.0 - v]
    }

    fn set_tangent(
        &mut self,
        tangent_space: Option<bevy_mikktspace::TangentSpace>,
        face: usize,
        vert: usize,
    ) {
        // `None` means a degenerate face with no neighbour to borrow from.
        self.tangents[face * 3 + vert] = tangent_space.unwrap_or_default().tangent_encoded();
    }
}

/// MikkTSpace tangents in glTF's convention, one per index: each corner's own.
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

/// The smallest QTangent w, one Snorm16 step, so the w that carries the
/// bitangent sign never rounds to zero.
const SMALLEST_W: f32 = 1.0 / i16::MAX as f32;

/// The QTangent of a glTF normal and tangent (`xyz`, `w` the bitangent
/// sign): the rotation whose columns are the tangent made perpendicular to
/// the normal, the bitangent cross(normal, tangent) and the normal, with
/// its w's sign set to the bitangent sign.
pub(crate) fn qtangent(normal: [f32; 3], tangent: [f32; 4]) -> [i16; 4] {
    let normal = Vec3::from(normal).normalize_or(Vec3::Z);
    let [tangent_x, tangent_y, tangent_z, bitangent_sign] = tangent;
    let tangent = Vec3::new(tangent_x, tangent_y, tangent_z);
    let tangent = (tangent - normal * normal.dot(tangent))
        .try_normalize()
        .unwrap_or_else(|| normal.any_orthonormal_vector());
    let frame = Mat3::from_cols(tangent, normal.cross(tangent), normal);
    let mut rotation = Quat::from_mat3(&frame).normalize();
    // A quaternion and its negation are the same rotation.
    if rotation.w < 0.0 {
        rotation = -rotation;
    }
    if rotation.w < SMALLEST_W {
        let axis = rotation.xyz().normalize() * (1.0 - SMALLEST_W * SMALLEST_W).sqrt();
        rotation = Quat::from_xyzw(axis.x, axis.y, axis.z, SMALLEST_W);
    }
    if bitangent_sign < 0.0 {
        rotation = -rotation;
    }
    rotation
        .to_array()
        .map(|component| (component * i16::MAX as f32).round() as i16)
}

/// Every Mesh of a glTF file, in mesh-local space, with each Primitive's
/// geometry and the Asset ID of its Material, named as the Material cook
/// names it; a Primitive on glTF's default Material names none.
pub(super) fn gltf_meshes(
    guid: u128,
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> anyhow::Result<Vec<(AssetId, Vec<SourcePrimitive>)>> {
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
            let geometry = Geometry {
                positions,
                normals,
                uvs,
                indices,
            };
            primitives.push((geometry, material));
        }
        meshes.push((AssetId::new(source, AssetKind::Mesh, name), primitives));
    }
    Ok(meshes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ReferenceScene;

    #[test]
    fn the_cook_key_covers_every_part_a_mesh_cooks_from() {
        let triangle = || {
            let geometry = Geometry {
                positions: vec![[0.0; 3]; 3],
                normals: vec![[0.0, 0.0, 1.0]; 3],
                uvs: vec![[0.0; 2]; 3],
                indices: vec![0, 1, 2],
            };
            (geometry, None)
        };
        let key = cook_key(&[triangle()]);
        let changes: [fn(&mut SourcePrimitive); 5] = [
            |(geometry, _)| geometry.positions[0][0] = 1.0,
            |(geometry, _)| geometry.normals[0][2] = -1.0,
            |(geometry, _)| geometry.uvs[0][0] = 1.0,
            |(geometry, _)| geometry.indices[0] = 2,
            |(_, material)| *material = Some(AssetId(1)),
        ];
        for change in changes {
            let mut primitive = triangle();
            change(&mut primitive);
            assert_ne!(cook_key(&[primitive]), key);
        }
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
        };
        assert_eq!(weld(&soup).positions.len(), 4);
        assert_eq!(cook(&[(soup, None)]).vertices.len(), 4);
    }

    /// The plane's `u` grows along +X and its `v` along +Z, down the image,
    /// so its tangent is +X, its normal +Y, and its bitangent, up the image,
    /// −Z: cross(+Y, +X) times a bitangent sign of +1.
    #[test]
    fn a_plane_has_its_uv_aligned_tangent_frame() {
        let mesh = cook(&[(crate::scenes::ground_plane(2.0, 1.0), None)]);
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
            for (_, primitives) in gltf_meshes(0, &gltf.document, &buffers).unwrap() {
                meshes.push((path, primitives));
            }
        }
        meshes
    }

    /// Each triangle rotated to start at its lowest index: meshopt's index
    /// codec may rotate a triangle, which keeps its winding.
    fn triangles_from_lowest_index(indices: &[u32]) -> Vec<[u32; 3]> {
        indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|&triangle| {
                let mut triangle = triangle;
                let lowest = (0..3).min_by_key(|&corner| triangle[corner]).unwrap();
                triangle.rotate_left(lowest);
                triangle
            })
            .collect()
    }

    #[test]
    fn every_reference_scene_mesh_decodes_to_its_cooked_data() {
        let procedural = ReferenceScene::ALL.into_iter().flat_map(|scene| {
            scene
                .meshes()
                .into_iter()
                .map(move |(name, geometry, _)| (name, vec![(geometry, Some(AssetId(1)))]))
        });
        for (name, primitives) in procedural.chain(reference_gltf_meshes()) {
            let mesh = cook(&primitives);
            let blob = mesh.to_blob().unwrap();
            let mut decoded = CookedMesh::from_blob(&blob).unwrap();
            assert_eq!(
                triangles_from_lowest_index(&decoded.indices),
                triangles_from_lowest_index(&mesh.indices),
                "{name}"
            );
            decoded.indices = mesh.indices.clone();
            assert_eq!(decoded, mesh, "{name}");
        }
    }

    /// #96 counted 14,085 vertices with position, UV and normal welded.
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
