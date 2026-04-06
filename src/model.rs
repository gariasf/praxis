use crate::vertex::Vertex;

pub struct LoadedPrimitive {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub base_color_image: Option<(Vec<u8>, u32, u32)>, // rgba pixels, width, height
}

pub struct LoadedModel {
    pub primitives: Vec<LoadedPrimitive>,
}

pub fn load_glb(path: &str) -> anyhow::Result<LoadedModel> {
    let (document, buffers, images) = gltf::import(path)?;

    let mesh = document
        .meshes()
        .next()
        .ok_or(anyhow::anyhow!("No meshes found"))?;

    let mut primitives = Vec::new();

    for primitive in mesh.primitives() {
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
            .ok_or(anyhow::anyhow!("Mesh has no indices"))?
            .into_u32()
            .collect();

        let vertices: Vec<Vertex> = positions
            .iter()
            .enumerate()
            .map(|(i, &pos)| Vertex {
                position: pos,
                normal: normals[i],
                uv: uvs[i],
            })
            .collect();

        let base_color_image = primitive
            .material()
            .pbr_metallic_roughness()
            .base_color_texture()
            .map(|tex_info| {
                let image = &images[tex_info.texture().source().index()];
                let pixels = match image.format {
                    gltf::image::Format::R8G8B8A8 => image.pixels.clone(),
                    gltf::image::Format::R8G8B8 => image
                        .pixels
                        .chunks(3)
                        .flat_map(|rgb| [rgb[0], rgb[1], rgb[2], 255])
                        .collect(),
                    other => panic!("Unsupported texture format: {:?}", other),
                };
                (pixels, image.width, image.height)
            });

        primitives.push(LoadedPrimitive {
            vertices,
            indices,
            base_color_image,
        });
    }

    Ok(LoadedModel { primitives })
}
