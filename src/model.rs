use crate::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelUniform {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

pub struct LoadedPrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub base_color_image: Option<(Vec<u8>, u32, u32)>, // rgba pixels, width, height
}

pub struct LoadedModel {
    pub primitives: Vec<LoadedPrimitive>,
}

pub fn load_model(path: &str) -> anyhow::Result<LoadedModel> {
    let (document, buffers, images) = gltf::import(path)?;

    let meshes = document.meshes();
    if meshes.len() == 0 {
        return Err(anyhow::anyhow!("No meshes found"));
    }

    let mut primitives = Vec::new();

    for mesh in meshes {
        for primitive in mesh.primitives() {
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

            let vertices: Vec<Vertex> = positions
                .iter()
                .enumerate()
                .map(|(i, &pos)| Vertex {
                    position: pos,
                    normal: normals[i],
                    uv: uvs[i],
                })
                .collect();

            let base_color_image = if let Some(tex_info) = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
            {
                let image = &images[tex_info.texture().source().index()];
                let pixels = match image.format {
                    gltf::image::Format::R8G8B8A8 => image.pixels.clone(),
                    gltf::image::Format::R8G8B8 => image
                        .pixels
                        .chunks(3)
                        .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                        .collect(),
                    other => anyhow::bail!("Unsupported texture format: {:?}", other),
                };
                Some((pixels, image.width, image.height))
            } else {
                None
            };

            primitives.push(LoadedPrimitive {
                vertices,
                indices,
                base_color_image,
            });
        }
    }

    Ok(LoadedModel { primitives })
}
