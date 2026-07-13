use bevy_ecs::prelude::*;

use crate::{
    assets::{Mesh, MeshHandle},
    render::Vertex,
};

#[derive(Resource)]
pub struct MeshPool {
    meshes: Vec<Mesh>,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    vertex_capacity: u64,
    index_capacity: u64,
    vertex_used: u64,
    index_used: u64,
    vertex_data: Vec<Vertex>,
    index_data: Vec<u32>,
}

impl MeshPool {
    pub fn new(device: &wgpu::Device, vertex_capacity: u64, index_capacity: u64) -> Self {
        Self {
            meshes: Vec::new(),
            vertex_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vertex_buffer"),
                size: vertex_capacity,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            index_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("index_buffer"),
                size: index_capacity,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            vertex_capacity,
            index_capacity,
            vertex_used: 0,
            index_used: 0,
            vertex_data: Vec::new(),
            index_data: Vec::new(),
        }
    }

    pub fn insert(&mut self, mesh: Mesh) -> MeshHandle {
        let handle = MeshHandle(self.meshes.len() as u32);
        self.meshes.push(mesh);
        handle
    }

    /// Writes one primitive's geometry into the shared buffers.
    /// Returns (vertex_offset, index_offset, index_count) in ELEMENTS.
    pub fn push_primitive(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> (u32, u32, u32) {
        let vertex_bytes = std::mem::size_of_val(vertices) as u64;
        let index_bytes = std::mem::size_of_val(indices) as u64;

        // grow if this write would overflow
        if self.vertex_used + vertex_bytes > self.vertex_capacity {
            self.grow_vertex(device, queue, self.vertex_used + vertex_bytes);
        }
        if self.index_used + index_bytes > self.index_capacity {
            self.grow_index(device, queue, self.index_used + index_bytes);
        }

        let vertex_offset = (self.vertex_used / std::mem::size_of::<Vertex>() as u64) as u32;
        let index_offset = (self.index_used / std::mem::size_of::<u32>() as u64) as u32;

        queue.write_buffer(
            &self.vertex_buffer,
            self.vertex_used,
            bytemuck::cast_slice(vertices),
        );
        queue.write_buffer(
            &self.index_buffer,
            self.index_used,
            bytemuck::cast_slice(indices),
        );

        self.vertex_data.extend_from_slice(vertices); // shadow, for growth
        self.index_data.extend_from_slice(indices);
        self.vertex_used += vertex_bytes;
        self.index_used += index_bytes;

        (vertex_offset, index_offset, indices.len() as u32)
    }

    /// Allocates a larger vertex buffer (at least `needed` bytes), replays the
    /// shadow copy into it, and swaps the handle. Nothing references the old
    /// buffer past this frame; the render loop re-slices the live handle.
    fn grow_vertex(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, needed: u64) {
        let new_capacity = (self.vertex_capacity * 2).max(needed);
        let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("vertex_buffer"),
            size: new_capacity,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&new_buffer, 0, bytemuck::cast_slice(&self.vertex_data));
        self.vertex_buffer = new_buffer;
        self.vertex_capacity = new_capacity;
        tracing::info!(new_capacity, "vertex buffer grown");
    }

    /// Index-buffer counterpart of [`Self::grow_vertex`].
    fn grow_index(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, needed: u64) {
        let new_capacity = (self.index_capacity * 2).max(needed);
        let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("index_buffer"),
            size: new_capacity,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&new_buffer, 0, bytemuck::cast_slice(&self.index_data));
        self.index_buffer = new_buffer;
        self.index_capacity = new_capacity;
        tracing::info!(new_capacity, "index buffer grown");
    }

    pub fn get(&self, handle: MeshHandle) -> Option<&Mesh> {
        self.meshes.get(handle.0 as usize)
    }
}
