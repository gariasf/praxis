use crate::render::Vertex;

/// RGBA8 pixels, row-major, top row first.
#[cfg(feature = "cook")]
pub struct LoadedImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct LoadedPrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct LoadedModel {
    pub primitives: Vec<LoadedPrimitive>,
}

/// The vertices and indices of every Primitive of the glTF file at `path`;
/// its images are not read, since Textures load cooked.
pub fn load_model(path: &str) -> anyhow::Result<LoadedModel> {
    let _span = tracing::info_span!("load_model", path = %path).entered();

    tracing::debug!("starting model load");

    let gltf::Gltf { document, blob } = gltf::Gltf::open(path)?;
    let buffers = gltf::import_buffers(&document, std::path::Path::new(path).parent(), blob)?;

    let meshes = document.meshes();
    let mesh_count = meshes.len();
    tracing::debug!(meshes = mesh_count, buffers = buffers.len(), "glTF parsed");

    if mesh_count == 0 {
        return Err(anyhow::anyhow!("No meshes found"));
    }

    let mut primitives = Vec::new();

    for (mesh_index, mesh) in meshes.enumerate() {
        let _mesh_span = tracing::debug_span!("mesh", mesh_index).entered();

        for (primitive_index, primitive) in mesh.primitives().enumerate() {
            let _prim_span = tracing::debug_span!("primitive", primitive_index).entered();

            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err(anyhow::anyhow!(
                    "Unsupported primitive mode: {:?}. Only triangle primitives are supported",
                    primitive.mode()
                ));
            }
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or(anyhow::anyhow!("Mesh has no positions"))?
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let indices: Vec<u32> = reader
                .read_indices()
                .map(|iter| iter.into_u32().collect())
                .unwrap_or_else(|| (0..positions.len() as u32).collect());

            let tangents: Vec<[f32; 4]> = match reader.read_tangents() {
                Some(tangent_iter) => {
                    tracing::debug!("tangents read from TANGENT accessor");
                    tangent_iter.collect()
                }
                None => {
                    tracing::debug!("no TANGENT accessor; generating tangents with MikkTSpace");
                    generate_tangents(&positions, &normals, &uvs, &indices)
                }
            };

            let vertices: Vec<Vertex> = (0..positions.len())
                .map(|vertex_index| Vertex {
                    position: positions[vertex_index],
                    normal: normals[vertex_index],
                    uv: uvs[vertex_index],
                    tangent: tangents[vertex_index],
                })
                .collect();

            tracing::debug!(
                vertices = vertices.len(),
                indices = indices.len(),
                "primitive loaded"
            );

            primitives.push(LoadedPrimitive { vertices, indices });
        }
    }

    tracing::info!(primitives = primitives.len(), "model load complete");

    Ok(LoadedModel { primitives })
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

/// Generates glTF-convention tangents (`xyz` tangent, `w` bitangent sign) with
/// MikkTSpace, the algorithm the glTF spec prescribes when `TANGENT` is absent.
/// For a vertex shared by several faces the last face written wins. Fine for
/// well-formed meshes; a UV seam already splits vertices.
pub(crate) fn generate_tangents(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    uvs: &[[f32; 2]],
    indices: &[u32],
) -> Vec<[f32; 4]> {
    let mut tangents = vec![[1.0, 0.0, 0.0, 1.0]; positions.len()];
    for (index, tangent) in indices
        .iter()
        .zip(corner_tangents(positions, normals, uvs, indices))
    {
        tangents[*index as usize] = tangent;
    }
    tangents
}

/// MikkTSpace tangents in glTF's convention, one per index: each corner's own.
pub(crate) fn corner_tangents(
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

#[cfg(test)]
mod tests {
    use super::generate_tangents;

    /// A quad facing +Z, its texture upright: `u` grows along +X and glTF's `v`
    /// grows down the image, along −Y. glTF normal maps take +Y as up the
    /// image, so the bitangent, cross(normal, tangent) × w, must be +Y.
    #[test]
    fn generated_bitangent_points_up_the_image() {
        let positions = [
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
        ];
        let normals = [[0.0, 0.0, 1.0]; 4];
        let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        let indices = [0, 1, 2, 0, 2, 3];

        for tangent in generate_tangents(&positions, &normals, &uvs, &indices) {
            let [x, y, z, handedness] = tangent;
            let bitangent = glam::Vec3::Z.cross(glam::vec3(x, y, z)) * handedness;
            assert!(
                glam::vec3(x, y, z).abs_diff_eq(glam::Vec3::X, 1e-6)
                    && bitangent.abs_diff_eq(glam::Vec3::Y, 1e-6),
                "tangent {tangent:?} gives bitangent {bitangent}, expected tangent +X and bitangent +Y"
            );
        }
    }
}
