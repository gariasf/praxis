//! The bend pass: a compute pass that bends Meshes on the GPU each frame from
//! their rest Meshes, a stand-in for skinning (#208). Each of Frame data's
//! bends names a rest Mesh, whose +Y axis runs up its middle from y = 0, and
//! a bent Mesh of the same vertices: the middle bends into a circular arc,
//! and each vertex turns with the middle at its height, its tangent frame
//! too. The main pass then pulls the bent Mesh's vertices from the vertex
//! Arena as it pulls any Mesh's.

use crate::render::bind_groups::{BENDS, pipeline_layout};
use crate::render::shaders;

/// The bend pass's label, and its name in frame stats.
pub(crate) const BEND_PASS: &str = "bend_pass";

/// Bends a frame applies at most: past it, those of the bent Meshes
/// highest in the vertex Arena are left out, so those Meshes keep the shape
/// they last took, and are counted and warned about. Provisional until M4's
/// split of the 8 GB Budget sizes it.
pub const BEND_CAPACITY: u32 = 4096;

/// Vertices a workgroup of the pass bends, one per invocation.
pub(crate) const BEND_WORKGROUP_SIZE: u32 = 64;

/// One bend as the pass reads it: where its rest Mesh's vertices and its
/// bent Mesh's start in the vertex Arena, how many each holds, and the bend
/// (`Bend`), its curvature blended between the two newest Ticks.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BendData {
    pub rest_first_vertex: u32,
    pub first_vertex: u32,
    pub vertex_count: u32,
    /// The direction the middle bends towards, about +Y from +X towards +Z,
    /// in radians.
    pub azimuth: f32,
    /// Radians the middle turns through per metre of its height.
    pub curvature: f32,
}

/// The pass's pipeline, and the bends it reads.
pub struct BendPass {
    pipeline: wgpu::ComputePipeline,
    /// Its group 3 (`BENDS`).
    bind_group: wgpu::BindGroup,
    /// The frame's bends, which the CPU writes: created at its fixed
    /// capacity, it never grows.
    bends: wgpu::Buffer,
}

impl BendPass {
    /// The pass, bending Meshes within `vertex_arena`, the Mesh pool's.
    pub fn new(device: &wgpu::Device, vertex_arena: &wgpu::Buffer) -> BendPass {
        let bends = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bend Buffer"),
            size: u64::from(BEND_CAPACITY) * size_of::<BendData>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = BENDS.layout(device);
        let bind_group = BENDS.bind_group(
            device,
            &layout,
            [
                ("bends", bends.as_entire_binding()),
                ("vertices", vertex_arena.as_entire_binding()),
            ],
        );
        let shader = device.create_shader_module(shaders::BEND.descriptor());
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("bend_vertices"),
            layout: Some(&pipeline_layout(
                device,
                "Bend Pass Pipeline Layout",
                &[(&BENDS, &layout)],
            )),
            module: &shader,
            entry_point: Some("bend_vertices"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        BendPass {
            pipeline,
            bind_group,
            bends,
        }
    }

    /// Records the pass into `encoder`, timed into `timestamp_writes`, for
    /// `bends`, as uploaded into its buffer: a row of workgroups per bend,
    /// an invocation per vertex of the longest. Returns whether it did: with
    /// no bends there is nothing to record.
    pub fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bends: &[BendData],
        timestamp_writes: Option<wgpu::ComputePassTimestampWrites<'_>>,
    ) -> bool {
        let Some(longest) = bends.iter().map(|bend| bend.vertex_count).max() else {
            return false;
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(BEND_PASS),
            timestamp_writes,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(BENDS.index, &self.bind_group, &[]);
        pass.dispatch_workgroups(longest.div_ceil(BEND_WORKGROUP_SIZE), bends.len() as u32, 1);
        true
    }

    /// The frame's bends, `BendData` that the CPU writes.
    pub fn bends(&self) -> &wgpu::Buffer {
        &self.bends
    }

    /// Bytes of the bends' buffer.
    pub fn gpu_memory_bytes(&self) -> u64 {
        self.bends.size()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::ops::Range;
    use std::time::Duration;

    use super::*;
    use crate::assets::MeshUse;
    use crate::assets::pools::tests::read_storage;
    use crate::cooked::{
        CookedMesh, CookedMeshlet, CookedPrimitive, CookedVertex, MESHLET_MAX_VERTICES,
    };
    use crate::render::renderer::tests::{gpu_context, render_frame};
    use crate::render::{FrameData, RenderSwitches, Renderer, TIMED_PASSES};

    /// Vertices of each Mesh: more than a workgroup bends, not a multiple of
    /// it.
    const VERTEX_COUNT: u32 = 67;

    /// The rest Mesh: a strip of triangles over vertices from its foot up
    /// its middle to 2 m, each a quarter metre out from the middle its own
    /// way, its tangent frame turned its own way by under 2.8 radians, so
    /// its w, at least cos 1.4, carries the bitangent sign, which every
    /// other vertex has negative.
    fn rest_mesh() -> CookedMesh {
        let vertices = (0..VERTEX_COUNT)
            .map(|index| {
                let bitangent_sign = if index % 2 == 0 { 1.0 } else { -1.0 };
                let index = f64::from(index);
                let around = 2.4 * index;
                let axis = glam::dvec3((1.3 * index).cos(), 0.6, (1.3 * index).sin());
                let angle = 2.8 * (0.37 * index).fract();
                let rotation = glam::DQuat::from_axis_angle(axis.normalize(), angle);
                let height = 2.0 * index / f64::from(VERTEX_COUNT - 1);
                CookedVertex {
                    position: [0.25 * around.cos(), height, 0.25 * around.sin()]
                        .map(|coordinate| coordinate as f32),
                    uv: [index as f32, 0.5],
                    qtangent: (rotation * bitangent_sign)
                        .to_array()
                        .map(|component| (component * f64::from(i16::MAX)).round() as i16),
                }
            })
            .collect();
        let triangles: Vec<[u32; 3]> = (0..VERTEX_COUNT - 2)
            .map(|first| [first, first + 1, first + 2])
            .collect();
        let counts = triangles.chunks(MESHLET_MAX_VERTICES - 2).map(<[_]>::len);
        CookedMesh {
            vertices,
            indices: triangles.as_flattened().to_vec(),
            primitives: vec![CookedPrimitive {
                first_index: 0,
                index_count: 3 * triangles.len() as u32,
                material: None,
                meshlets: CookedMeshlet::split(&triangles, counts).expect("within the limits"),
            }],
            bounds: [[-0.25, 0.0, -0.25], [0.25, 2.0, 0.25]],
        }
    }

    /// Where bending by `curvature` towards `azimuth` takes the rest Mesh's
    /// point at `position`, and how it turns the point's tangent frame: the
    /// middle's direction, turned from +Y towards the azimuth by the
    /// curvature times the height, summed in 4,096 steps up to the point's
    /// height, then the point's offset from the middle, turned as the middle
    /// turns there.
    pub(crate) fn bent(
        position: glam::DVec3,
        azimuth: f64,
        curvature: f64,
    ) -> (glam::DVec3, glam::DQuat) {
        const STEPS: u32 = 4096;
        let toward = glam::dvec3(azimuth.cos(), 0.0, azimuth.sin());
        let direction = |height: f64| {
            glam::DVec3::Y * (curvature * height).cos() + toward * (curvature * height).sin()
        };
        let step = position.y / f64::from(STEPS);
        let middle: glam::DVec3 = (0..STEPS)
            .map(|index| direction((f64::from(index) + 0.5) * step) * step)
            .sum();
        let turn = glam::DQuat::from_rotation_arc(glam::DVec3::Y, direction(position.y));
        (
            middle + turn * glam::dvec3(position.x, 0.0, position.z),
            turn,
        )
    }

    /// A QTangent's normal, tangent and bitangent sign, decoded as the main
    /// pass decodes it.
    fn tangent_frame(qtangent: [i16; 4]) -> (glam::DVec3, glam::DVec3, f64) {
        let components =
            qtangent.map(|component| (f64::from(component) / f64::from(i16::MAX)).max(-1.0));
        let rotation = glam::DQuat::from_array(components).normalize();
        let sign = if rotation.w < 0.0 { -1.0 } else { 1.0 };
        (rotation * glam::DVec3::Z, rotation * glam::DVec3::X, sign)
    }

    /// Of five Meshes loaded as the rest Mesh, three bent from the first,
    /// straight, forward along a slanted azimuth and back the other way,
    /// take their bent shapes, tangent frames included, within f32's and
    /// Snorm16's rounding, while the first and the one after them keep
    /// theirs; the pass's GPU time is read back within the frame's.
    #[test]
    fn the_bend_pass_bends_each_mesh_from_its_rest_mesh() {
        let Some(context) = gpu_context() else {
            return;
        };
        let target_format = wgpu::TextureFormat::Rgba8Unorm;
        let switches = RenderSwitches::default();
        let mut renderer = Renderer::new(&context, target_format, 4, 4, switches);
        let rest_mesh = rest_mesh();
        let [rest, straight, forward, back, after] = [(); 5].map(|()| {
            let staging = &mut renderer.load_staging;
            let handle =
                renderer
                    .mesh_pool
                    .load(staging, &rest_mesh, &[None], MeshUse::DrawnAndTraced);
            let mesh = renderer.mesh_pool.get(handle.expect("fits"));
            let vertices = &mesh.expect("loaded").vertices;
            vertices.start as usize..vertices.end as usize
        });
        let bend = |mesh: &Range<usize>, azimuth, curvature| BendData {
            rest_first_vertex: rest.start as u32,
            first_vertex: mesh.start as u32,
            vertex_count: VERTEX_COUNT,
            azimuth,
            curvature,
        };
        let bends = [
            bend(&straight, 1.0, 0.0),
            bend(&forward, 2.2, 0.35),
            bend(&back, -0.4, -0.6),
        ];
        let frame = FrameData {
            bends: bends.to_vec(),
            ..FrameData::default()
        };
        let (_, _, gpu) = render_frame(&context, &mut renderer, &frame, 4, 4);

        let arena: Vec<CookedVertex> =
            read_storage(&context, renderer.mesh_pool.vertex_arena(), after.end);
        for mesh in [rest, after] {
            assert!(
                arena[mesh] == rest_mesh.vertices[..],
                "a Mesh not bent changed"
            );
        }
        for (mesh, bend) in [straight, forward, back].into_iter().zip(bends) {
            let vertices = rest_mesh.vertices.iter().zip(&arena[mesh]);
            for (index, (rest_vertex, bent_vertex)) in vertices.enumerate() {
                let context = format!("curvature {}, vertex {index}", bend.curvature);
                let rest_position = glam::Vec3::from(rest_vertex.position).as_dvec3();
                let azimuth = f64::from(bend.azimuth);
                let (expected, turn) = bent(rest_position, azimuth, bend.curvature.into());
                // Unbent, nothing moves at all.
                if bend.curvature == 0.0 {
                    assert_eq!(bent_vertex.position, rest_vertex.position, "{context}");
                }
                let position = glam::Vec3::from(bent_vertex.position).as_dvec3();
                assert!(
                    (position - expected).length() <= 1e-5,
                    "{context}: at {position}, expected {expected}"
                );
                let (normal, tangent, sign) = tangent_frame(rest_vertex.qtangent);
                let (bent_normal, bent_tangent, bent_sign) = tangent_frame(bent_vertex.qtangent);
                // Each component rounds to a Snorm16 step, 3e-5, once.
                let frame_error = (bent_normal - turn * normal)
                    .length()
                    .max((bent_tangent - turn * tangent).length());
                assert!(frame_error <= 2e-4, "{context}: frame off by {frame_error}");
                let (sign_and_uv, rest_sign_and_uv) =
                    ((bent_sign, bent_vertex.uv), (sign, rest_vertex.uv));
                assert_eq!(sign_and_uv, rest_sign_and_uv, "{context}");
            }
        }
        let pass = TIMED_PASSES.iter().position(|&pass| pass == BEND_PASS);
        let (_, time) = gpu.pass_times[pass.expect("a timed pass")];
        assert!(
            Duration::ZERO < time && time <= gpu.frame_time,
            "the bend pass took {time:?} of {:?}",
            gpu.frame_time
        );
    }
}
