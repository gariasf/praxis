use crate::render::Vertex;

/// RGBA8 pixels, row-major, top row first.
pub struct LoadedImage {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct LoadedPrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub base_color_image: Option<LoadedImage>,
    pub normal_image: Option<LoadedImage>,
    pub metallic_roughness_image: Option<LoadedImage>,
    pub occlusion_image: Option<LoadedImage>,
    pub emissive_image: Option<LoadedImage>,
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub emissive_factor: [f32; 3],
}

pub struct LoadedModel {
    pub primitives: Vec<LoadedPrimitive>,
}

pub fn load_model(path: &str) -> anyhow::Result<LoadedModel> {
    let _span = tracing::info_span!("load_model", path = %path).entered();

    tracing::debug!("starting model load");

    let (document, buffers, images) = gltf::import(path)?;

    let meshes = document.meshes();
    let mesh_count = meshes.len();
    tracing::debug!(
        meshes = mesh_count,
        buffers = buffers.len(),
        images = images.len(),
        "glTF parsed"
    );

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

            let material = primitive.material();
            let pbr_metallic_roughness = material.pbr_metallic_roughness();
            let base_color_factor = pbr_metallic_roughness.base_color_factor();
            let metallic_factor = pbr_metallic_roughness.metallic_factor();
            let roughness_factor = pbr_metallic_roughness.roughness_factor();
            let emissive_factor = material.emissive_factor();

            let base_color_image = pbr_metallic_roughness
                .base_color_texture()
                .map(|info| decode_rgba(&images, info.texture()))
                .transpose()?;
            let metallic_roughness_image = pbr_metallic_roughness
                .metallic_roughness_texture()
                .map(|info| decode_rgba(&images, info.texture()))
                .transpose()?;
            let normal_image = material
                .normal_texture()
                .map(|normal_texture| decode_rgba(&images, normal_texture.texture()))
                .transpose()?;
            let occlusion_image = material
                .occlusion_texture()
                .map(|occlusion_texture| decode_rgba(&images, occlusion_texture.texture()))
                .transpose()?;
            let emissive_image = material
                .emissive_texture()
                .map(|info| decode_rgba(&images, info.texture()))
                .transpose()?;

            tracing::debug!(
                vertices = vertices.len(),
                indices = indices.len(),
                has_base_color = base_color_image.is_some(),
                has_normal = normal_image.is_some(),
                has_metallic_roughness = metallic_roughness_image.is_some(),
                has_occlusion = occlusion_image.is_some(),
                has_emissive = emissive_image.is_some(),
                "primitive loaded"
            );

            primitives.push(LoadedPrimitive {
                vertices,
                indices,
                base_color_image,
                normal_image,
                metallic_roughness_image,
                occlusion_image,
                emissive_image,
                base_color_factor,
                metallic_factor,
                roughness_factor,
                emissive_factor,
            });
        }
    }

    tracing::info!(primitives = primitives.len(), "model load complete");

    Ok(LoadedModel { primitives })
}

/// Decodes a glTF texture's image into RGBA8, expanding RGB sources.
fn decode_rgba(
    images: &[gltf::image::Data],
    texture: gltf::Texture,
) -> anyhow::Result<LoadedImage> {
    let image = &images[texture.source().index()];
    let pixels = match image.format {
        gltf::image::Format::R8G8B8A8 => image.pixels.clone(),
        gltf::image::Format::R8G8B8 => image
            .pixels
            .chunks(3)
            .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
            .collect(),
        other => anyhow::bail!("Unsupported texture format: {:?}", other),
    };
    Ok(LoadedImage {
        pixels,
        width: image.width,
        height: image.height,
    })
}

/// Indexed triangle mesh viewed through the MikkTSpace `Geometry` interface.
/// MikkTSpace works per face-vertex; results land on the shared vertex through
/// the index buffer, so for a vertex shared by several faces the last face
/// written wins. Fine for well-formed meshes; a UV seam already splits vertices.
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

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.uvs[self.vertex_index(face, vert)]
    }

    fn set_tangent(
        &mut self,
        tangent_space: Option<bevy_mikktspace::TangentSpace>,
        face: usize,
        vert: usize,
    ) {
        // `None` means a degenerate face with no neighbour to borrow from.
        let vertex_index = self.vertex_index(face, vert);
        self.tangents[vertex_index] = tangent_space.unwrap_or_default().tangent_encoded();
    }
}

/// Generates glTF-convention tangents (`xyz` tangent, `w` bitangent sign) with
/// MikkTSpace, the algorithm the glTF spec prescribes when `TANGENT` is absent.
fn generate_tangents(
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
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; positions.len()],
    };
    if bevy_mikktspace::generate_tangents(&mut geometry).is_err() {
        tracing::error!("MikkTSpace tangent generation failed; falling back to +X tangents");
    }
    geometry.tangents
}
