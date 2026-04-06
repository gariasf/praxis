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
    let (document, buffers, _images) = gltf::import(path)?;

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

        primitives.push(LoadedPrimitive {
            vertices,
            indices,
            base_color_image: None, // texture extraction later
        });
    }

    Ok(LoadedModel { primitives })
}
