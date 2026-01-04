//! Terrain mesh generation from heightmaps.

use crate::heightmap::TerrainHeightmap;
use praxis_graphics::{MeshData, Vertex3D};
use praxis_math::Vec3;
use praxis_utils::Result;

/// Terrain mesh generator.
pub struct TerrainMesh;

impl TerrainMesh {
    /// Generates vertices and indices for a terrain chunk at a specific LOD level.
    ///
    /// # Arguments
    ///
    /// * `heightmap` - The heightmap containing elevation data
    /// * `chunk_x` - X coordinate of the chunk in the grid
    /// * `chunk_z` - Z coordinate of the chunk in the grid
    /// * `chunk_size` - Size of the chunk in world units
    /// * `vertices_per_side` - Number of vertices per side (controls LOD)
    /// * `world_scale` - Scale factor for converting grid to world coordinates
    pub fn generate_chunk(
        heightmap: &TerrainHeightmap,
        chunk_x: i32,
        chunk_z: i32,
        chunk_size: f32,
        vertices_per_side: u32,
        world_scale: f32,
    ) -> Result<MeshData> {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut colors = Vec::new();
        let mut tangents = Vec::new();

        let grid_start_x = (chunk_x as f32 * chunk_size / world_scale) as u32;
        let grid_start_z = (chunk_z as f32 * chunk_size / world_scale) as u32;
        let grid_step = (chunk_size / world_scale / (vertices_per_side - 1) as f32).max(1.0);

        for z in 0..vertices_per_side {
            for x in 0..vertices_per_side {
                let grid_x = grid_start_x + (x as f32 * grid_step) as u32;
                let grid_z = grid_start_z + (z as f32 * grid_step) as u32;

                let world_x = chunk_x as f32 * chunk_size
                    + x as f32 * chunk_size / (vertices_per_side - 1) as f32;
                let world_z = chunk_z as f32 * chunk_size
                    + z as f32 * chunk_size / (vertices_per_side - 1) as f32;
                let world_y = heightmap.get_height(grid_x, grid_z);

                positions.push([world_x, world_y, world_z]);

                let normal = heightmap.calculate_normal(grid_x, grid_z, world_scale);
                normals.push([normal.x, normal.y, normal.z]);

                let u = x as f32 / (vertices_per_side - 1) as f32;
                let v = z as f32 / (vertices_per_side - 1) as f32;
                uvs.push([u * chunk_size / 10.0, v * chunk_size / 10.0]);

                colors.push([1.0, 1.0, 1.0]);

                tangents.push([1.0, 0.0, 0.0, 1.0]);
            }
        }

        let mut indices = Vec::new();
        for z in 0..vertices_per_side - 1 {
            for x in 0..vertices_per_side - 1 {
                let top_left = (z * vertices_per_side + x) as u16;
                let top_right = top_left + 1;
                let bottom_left = ((z + 1) * vertices_per_side + x) as u16;
                let bottom_right = bottom_left + 1;

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        let mut mesh_data = MeshData {
            positions,
            colors: Some(colors),
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: Some(tangents),
            indices,
        };

        Self::calculate_tangents(&mut mesh_data);

        Ok(mesh_data)
    }

    /// Generates skirt vertices to prevent gaps between LOD levels.
    ///
    /// Skirts are vertical quads around the chunk edges that extend downward
    /// to hide any gaps that might appear when adjacent chunks use different LOD levels.
    pub fn generate_skirt(
        vertices: &[Vertex3D],
        vertices_per_side: u32,
        skirt_depth: f32,
    ) -> (Vec<Vertex3D>, Vec<u32>) {
        let mut skirt_vertices = Vec::new();
        let mut skirt_indices = Vec::new();

        let base_index = vertices.len() as u32;
        let mut current_index = base_index;

        for side in 0..4 {
            let edge_vertices = match side {
                0 => (0..vertices_per_side).collect::<Vec<_>>(),
                1 => (0..vertices_per_side)
                    .map(|i| i * vertices_per_side + vertices_per_side - 1)
                    .collect::<Vec<_>>(),
                2 => (0..vertices_per_side)
                    .map(|i| (vertices_per_side - 1) * vertices_per_side + i)
                    .collect::<Vec<_>>(),
                _ => (0..vertices_per_side)
                    .map(|i| i * vertices_per_side)
                    .collect::<Vec<_>>(),
            };

            for &idx in &edge_vertices {
                let v = &vertices[idx as usize];
                skirt_vertices.push(*v);

                let mut skirt_bottom = *v;
                skirt_bottom.position[1] -= skirt_depth;
                skirt_vertices.push(skirt_bottom);
            }

            for i in 0..edge_vertices.len() - 1 {
                let offset = current_index + (i * 2) as u32;

                skirt_indices.push(offset);
                skirt_indices.push(offset + 2);
                skirt_indices.push(offset + 1);

                skirt_indices.push(offset + 1);
                skirt_indices.push(offset + 2);
                skirt_indices.push(offset + 3);
            }

            current_index += (edge_vertices.len() * 2) as u32;
        }

        (skirt_vertices, skirt_indices)
    }

    /// Calculates tangents for normal mapping support.
    pub fn calculate_tangents(mesh_data: &mut MeshData) {
        if mesh_data.tangents.is_none() {
            mesh_data.tangents = Some(vec![[1.0, 0.0, 0.0, 1.0]; mesh_data.positions.len()]);
        }

        let tangents = mesh_data.tangents.as_mut().unwrap();
        let positions = &mesh_data.positions;
        let uvs = mesh_data.uvs.as_ref();
        let indices = &mesh_data.indices;

        if uvs.is_none() {
            return;
        }

        let uvs = uvs.unwrap();

        for i in (0..indices.len()).step_by(3) {
            let i0 = indices[i] as usize;
            let i1 = indices[i + 1] as usize;
            let i2 = indices[i + 2] as usize;

            let v0 = positions[i0];
            let v1 = positions[i1];
            let v2 = positions[i2];

            let uv0 = uvs[i0];
            let uv1 = uvs[i1];
            let uv2 = uvs[i2];

            let edge1 = Vec3::new(v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]);
            let edge2 = Vec3::new(v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);

            let delta_uv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
            let delta_uv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];

            let f = 1.0 / (delta_uv1[0] * delta_uv2[1] - delta_uv2[0] * delta_uv1[1] + 0.0001);

            let tangent = Vec3::new(
                f * (delta_uv2[1] * edge1.x - delta_uv1[1] * edge2.x),
                f * (delta_uv2[1] * edge1.y - delta_uv1[1] * edge2.y),
                f * (delta_uv2[1] * edge1.z - delta_uv1[1] * edge2.z),
            )
            .normalize();

            for &idx in &[i0, i1, i2] {
                tangents[idx] = [tangent.x, tangent.y, tangent.z, 1.0];
            }
        }
    }

    /// Generates a flat terrain mesh for testing.
    pub fn generate_flat_plane(
        size: f32,
        subdivisions: u32,
    ) -> Result<MeshData> {
        let vertices_per_side = subdivisions + 1;
        let step = size / subdivisions as f32;

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut colors = Vec::new();

        for z in 0..vertices_per_side {
            for x in 0..vertices_per_side {
                let world_x = x as f32 * step - size / 2.0;
                let world_z = z as f32 * step - size / 2.0;

                positions.push([world_x, 0.0, world_z]);
                normals.push([0.0, 1.0, 0.0]);
                uvs.push([x as f32 / subdivisions as f32, z as f32 / subdivisions as f32]);
                colors.push([1.0, 1.0, 1.0]);
            }
        }

        let mut indices = Vec::new();
        for z in 0..subdivisions {
            for x in 0..subdivisions {
                let top_left = (z * vertices_per_side + x) as u16;
                let top_right = top_left + 1;
                let bottom_left = ((z + 1) * vertices_per_side + x) as u16;
                let bottom_right = bottom_left + 1;

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        let mut mesh_data = MeshData {
            positions,
            colors: Some(colors),
            normals: Some(normals),
            uvs: Some(uvs),
            tangents: None,
        indices,
        };

        Self::calculate_tangents(&mut mesh_data);

        Ok(mesh_data)
    }
}
