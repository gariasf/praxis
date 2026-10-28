//! Which triangle each MSAA sample shows, drawn beside the triangles that
//! share its vertices and without them, and what the fragment stage reads
//! (ADR 0019).
//!
//! Each sample's owner is read back from a multisampled ID target. The flat
//! scene's quads do not overlap, so an analytic oracle gives each sample's
//! owner; the tilted scene is the one the visibility pre-pass prototype
//! lost samples on.

use std::collections::HashSet;
use std::io::Write;

use wgpu::util::DeviceExt;

use crate::camera::{Camera, projection_matrix};
use crate::gpu::GpuContext;
use crate::preflight::Machine;
use crate::render::renderer::tests::gpu_context;

/// The ID target's width and height, in pixels.
const TARGET_SIDE: u32 = 256;

/// A quad's corners in its own plane, in the order `Quad` names them.
const UNIT_QUAD: [[f32; 2]; 4] = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];

/// A quad's corners in clip space, `q0 q1 q2 q3`, drawn as the cooker's
/// Meshes draw a quad: triangle `q0 q1 q2`, then `q0 q2 q3`.
type Quad = [[f32; 4]; 4];

/// A triangle's ID, even for a quad's first triangle and odd for the
/// partner sharing its edge `q0 q2`, and the vertex indices it is drawn
/// with.
type Triangle = (u32, [u32; 3]);

/// How a quad's two triangles index their vertices.
#[derive(Clone, Copy, PartialEq)]
enum Indexing {
    /// Both index the quad's four vertices, `0 1 2` and `0 2 3`, as within
    /// a Meshlet.
    Shared,
    /// The partner indexes `2 3 0`: the same edge, another first vertex.
    SharedRotated,
    /// The partner indexes its own copies of the same positions, as a
    /// triangle of another Meshlet or Instance does: their packed indices
    /// differ in the Meshlet record.
    Own,
}

/// Every triangle of `quads` quads, in quad order.
fn every_triangle(quads: u32, indexing: Indexing) -> Vec<Triangle> {
    (0..quads * 2)
        .map(|id| {
            let first = id / 2 * 4;
            let indices = match (id % 2, indexing) {
                (0, _) => [first, first + 1, first + 2],
                (_, Indexing::Shared) => [first, first + 2, first + 3],
                (_, Indexing::SharedRotated) => [first + 2, first + 3, first],
                (_, Indexing::Own) => [first, first + 2, first + 3].map(|index| index + 4 * quads),
            };
            (id, indices)
        })
        .collect()
}

/// Whether two owners, triangle IDs plus one, are triangles of one quad.
fn same_quad(owner: u32, other: u32) -> bool {
    owner != 0 && other != 0 && (owner - 1) / 2 == (other - 1) / 2
}

/// The vertex buffer every draw indexes: each quad's corners, then a copy
/// of them for `Indexing::Own`. A vertex carries weights only its
/// triangles interpolate, `q1`'s for the first and `q3`'s for the partner,
/// and the ID of the triangle it leads in `Indexing::SharedRotated`.
fn vertices(quads: &[Quad]) -> Vec<[u32; 8]> {
    let corners = quads.iter().enumerate().flat_map(|(quad, corners)| {
        let first = quad as u32 * 2;
        let leads = [first, 0, first + 1, 0];
        let weights = [[0.0, 0.0], [1.0, 0.0], [0.0, 0.0], [0.0, 1.0f32]];
        (0..4).map(move |corner| {
            let [x, y, z, w] = corners[corner].map(f32::to_bits);
            let [first_weight, partner_weight] = weights[corner].map(f32::to_bits);
            [
                x,
                y,
                z,
                w,
                first_weight,
                partner_weight,
                leads[corner],
                (quad as f32).to_bits(),
            ]
        })
    });
    corners.clone().chain(corners).collect()
}

/// The standard sample positions, in pixels
/// [ref:vulkan-standard-sample-locations].
fn sample_positions(sample_count: u32) -> &'static [[f64; 2]] {
    match sample_count {
        1 => &[[0.5, 0.5]],
        2 => &[[0.75, 0.75], [0.25, 0.25]],
        4 => &[
            [0.375, 0.125],
            [0.875, 0.375],
            [0.125, 0.625],
            [0.625, 0.875],
        ],
        _ => unreachable!("1, 2 or 4 samples"),
    }
}

/// A deterministic sequence in [0, 1).
fn sequence() -> impl FnMut() -> f64 {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    move || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Quads with sides of 0.3 to 1.5 pixels flat on the screen, one per
/// 4-pixel cell, each turned and sheared by a fixed pseudo-random sequence.
fn flat_quads() -> Vec<Quad> {
    let mut next = sequence();
    let cells = TARGET_SIDE / 4;
    let side_pixels = f64::from(TARGET_SIDE);
    (0..cells * cells)
        .map(|cell| {
            let centre = [
                f64::from(cell % cells) * 4.0 + 1.0 + next() * 2.0,
                f64::from(cell / cells) * 4.0 + 1.0 + next() * 2.0,
            ];
            let (side, angle) = (0.3 + 1.2 * next(), next() * std::f64::consts::TAU);
            let (shear, squash) = (next() * 0.8 - 0.4, 0.3 + 0.7 * next());
            let (sin, cos) = angle.sin_cos();
            UNIT_QUAD.map(|[u, v]| {
                let (u, v) = (f64::from(u), f64::from(v));
                let (u, v) = ((u + shear * v) * side, v * side * squash);
                let pixel = [centre[0] + u * cos - v * sin, centre[1] + u * sin + v * cos];
                [
                    (pixel[0] / side_pixels * 2.0 - 1.0) as f32,
                    (1.0 - pixel[1] / side_pixels * 2.0) as f32,
                    0.5,
                    1.0,
                ]
            })
        })
        .collect()
}

/// The visibility pre-pass prototype's scene, term for term: a 100 by 100
/// grid of quads 0.3 m wide, tilted, 40 m ahead, of which 7,871 are in
/// view.
fn tilted_quads() -> Vec<Quad> {
    let view_projection = projection_matrix(1.0) * Camera::new().view_matrix();
    let rotation = glam::Mat3::from_rotation_x(1.0) * glam::Mat3::from_rotation_y(0.3);
    let rotation_scale = rotation * glam::Mat3::from_diagonal(glam::vec3(0.3, 0.3, 1.0));
    (0..100 * 100)
        .map(|cell: u32| {
            let (column, row) = (cell % 100, cell / 100);
            let at = |index: u32| (index as f32 + 0.5) * 0.4 - 20.0;
            let offset = glam::vec3(at(column), at(row) * 0.5 - 2.0, -40.0 + row as f32 * 0.05);
            UNIT_QUAD.map(|[x, y]| {
                let world = rotation_scale * glam::vec3(x, y, 0.0) + offset;
                (view_projection * world.extend(1.0)).to_array()
            })
        })
        .filter(|quad| {
            quad.iter()
                .any(|[x, y, _, w]| *w > 0.0 && x.abs() < *w && y.abs() < *w)
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Shading {
    Pixel,
    Sample,
}

/// Where the fragment stage takes the ID of the triangle it writes.
#[derive(Clone, Copy, PartialEq)]
enum IdSource {
    /// `primitive_index`, looked up in the draw's list of triangle IDs.
    Primitive,
    /// A flat-interpolated vertex output: each triangle's first vertex's.
    Flat,
    /// The weights the triangle interpolates, `Indexing::Shared` only: a
    /// first triangle's partner weight is 0 everywhere, a partner's is not.
    Interpolated,
    /// The quad each vertex carries and the weights, all smoothly
    /// interpolated: no flat input and no `primitive_index`.
    Smooth,
    /// `primitive_index`, beside a flat input read to no effect.
    PrimitiveBesideFlat,
    /// `primitive_index`, beside a smooth input read to no effect.
    PrimitiveBesideSmooth,
    /// One fragment stage writes both `primitive_index`'s ID and the
    /// weights', each in a half of the sample; this reads the first.
    BothKeepingPrimitive,
    /// The same fragment stage; this reads the weights' half.
    BothKeepingWeights,
}

struct Raster<'a> {
    vertices: &'a [[u32; 8]],
    draws: Vec<Vec<Triangle>>,
    sample_count: u32,
    shading: Shading,
    id: IdSource,
    /// A depth-only pass draws these first, cleared to 0 and compared with
    /// `Greater`; the draws then test `Equal`, as the lit pass does.
    prepass: Option<Vec<Triangle>>,
}

/// Each sample's triangle ID plus one, 0 where none covers it.
fn rasterise(context: &GpuContext, raster: &Raster) -> Vec<u32> {
    let device = &context.device;
    let sample_count = raster.sample_count;
    let sample_input = match raster.shading {
        Shading::Pixel => "",
        Shading::Sample => ", @builtin(sample_index) sample: u32",
    };
    let id = match raster.id {
        IdSource::Primitive => "ids[primitive] + 1u",
        IdSource::Flat => "in.leads + 1u",
        IdSource::Interpolated => "in.leads + u32(in.weights.y != 0.0) + 1u",
        IdSource::Smooth => "u32(round(in.quad)) * 2u + u32(in.weights.y != 0.0) + 1u",
        IdSource::PrimitiveBesideFlat => "select(ids[primitive] + 1u, 0u, in.leads == 0xffffffffu)",
        IdSource::PrimitiveBesideSmooth => {
            "select(ids[primitive] + 1u, 0u, in.weights.y < -1.0e30)"
        }
        IdSource::BothKeepingPrimitive | IdSource::BothKeepingWeights => {
            "(ids[primitive] + 1u) | ((in.leads + u32(in.weights.y != 0.0) + 1u) << 16u)"
        }
    };
    let texture = match sample_count {
        1 => "texture_2d<f32>",
        _ => "texture_multisampled_2d<f32>",
    };
    let source = format!(
        "enable primitive_index;
struct Vertex {{ position: vec4<f32>, weights: vec2<f32>, leads: u32, quad: f32 }}
struct Varyings {{
    @builtin(position) @invariant position: vec4<f32>,
    @location(0) weights: vec2<f32>,
    @location(1) @interpolate(flat) leads: u32,
    @location(2) quad: f32,
}}
@group(0) @binding(0) var<storage, read> vertices: array<Vertex>;
@group(0) @binding(1) var<storage, read> ids: array<u32>;
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> Varyings {{
    let vertex = vertices[index];
    return Varyings(vertex.position, vertex.weights, vertex.leads, vertex.quad);
}}
@fragment
fn fs_main(in: Varyings, @builtin(primitive_index) primitive: u32{sample_input}) -> @location(0) vec4<f32> {{
    return unpack4x8unorm({id});
}}
@group(0) @binding(0) var target_ids: {texture};
@group(0) @binding(1) var<storage, read_write> owners: array<u32>;
@compute @workgroup_size(8, 8)
fn cs_read(@builtin(global_invocation_id) pixel: vec3<u32>) {{
    for (var sample = 0u; sample < {sample_count}u; sample++) {{
        owners[(pixel.y * {TARGET_SIDE}u + pixel.x) * {sample_count}u + sample] =
            pack4x8unorm(textureLoad(target_ids, pixel.xy, sample));
    }}
}}"
    );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Coverage"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let target_formats = [Some(wgpu::TextureFormat::Rgba8Unorm.into())];
    // The ID pipeline, at depth equal behind a pre-pass, or the pre-pass's.
    let pipeline = |writes_ids: bool| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Coverage"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: writes_ids.then(|| wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                targets: &target_formats,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: raster.prepass.is_some().then(|| wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(!writes_ids),
                depth_compare: Some(if writes_ids {
                    wgpu::CompareFunction::Equal
                } else {
                    wgpu::CompareFunction::Greater
                }),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: sample_count,
                ..Default::default()
            },
            multiview_mask: None,
            cache: None,
        })
    };
    let id_pipeline = pipeline(true);
    let target = |format, usage| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: TARGET_SIDE,
                    height: TARGET_SIDE,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
            .create_view(&Default::default())
    };
    let ids = target(
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    );
    let depth = target(
        wgpu::TextureFormat::Depth32Float,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    let buffer = |contents: &[u8], usage| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents,
            usage,
        })
    };
    let vertices = buffer(
        bytemuck::cast_slice(raster.vertices),
        wgpu::BufferUsages::STORAGE,
    );
    // Each draw's bind group, index buffer and index count.
    let prepare = |triangles: &[Triangle], pipeline: &wgpu::RenderPipeline| {
        let triangle_ids: Vec<u32> = triangles.iter().map(|(id, _)| *id).collect();
        let indices: Vec<u32> = triangles.iter().flat_map(|(_, indices)| *indices).collect();
        let triangle_ids = buffer(
            bytemuck::cast_slice(&triangle_ids),
            wgpu::BufferUsages::STORAGE,
        );
        let mut entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: vertices.as_entire_binding(),
        }];
        let reads_ids = matches!(
            raster.id,
            IdSource::Primitive
                | IdSource::PrimitiveBesideFlat
                | IdSource::PrimitiveBesideSmooth
                | IdSource::BothKeepingPrimitive
                | IdSource::BothKeepingWeights
        );
        if std::ptr::eq(pipeline, &id_pipeline) && reads_ids {
            entries.push(wgpu::BindGroupEntry {
                binding: 1,
                resource: triangle_ids.as_entire_binding(),
            });
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &entries,
        });
        let index_buffer = buffer(bytemuck::cast_slice(&indices), wgpu::BufferUsages::INDEX);
        (bind_group, index_buffer, indices.len() as u32)
    };
    let record = |pass: &mut wgpu::RenderPass, draws: &[(wgpu::BindGroup, wgpu::Buffer, u32)]| {
        for (bind_group, index_buffer, count) in draws {
            pass.set_bind_group(0, bind_group, &[]);
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..*count, 0, 0..1);
        }
    };
    let draws: Vec<_> = raster
        .draws
        .iter()
        .map(|triangles| prepare(triangles, &id_pipeline))
        .collect();

    let mut encoder = device.create_command_encoder(&Default::default());
    let depth_attachment = |load| wgpu::RenderPassDepthStencilAttachment {
        view: &depth,
        depth_ops: Some(wgpu::Operations {
            load,
            store: wgpu::StoreOp::Store,
        }),
        stencil_ops: None,
    };
    if let Some(prepass) = &raster.prepass {
        let prepass_pipeline = pipeline(false);
        let prepass_draw = [prepare(prepass, &prepass_pipeline)];
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Coverage Pre-pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(depth_attachment(wgpu::LoadOp::Clear(0.0))),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&prepass_pipeline);
        record(&mut pass, &prepass_draw);
    }
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Coverage IDs"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &ids,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: raster
                .prepass
                .is_some()
                .then(|| depth_attachment(wgpu::LoadOp::Load)),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&id_pipeline);
        record(&mut pass, &draws);
    }
    let samples = u64::from(TARGET_SIDE * TARGET_SIDE * sample_count) * 4;
    let owners = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: samples,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: samples,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let reader = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Coverage Reader"),
        layout: None,
        module: &module,
        entry_point: Some("cs_read"),
        compilation_options: Default::default(),
        cache: None,
    });
    let reader_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &reader.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&ids),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: owners.as_entire_binding(),
            },
        ],
    });
    {
        let mut pass = encoder.begin_compute_pass(&Default::default());
        pass.set_pipeline(&reader);
        pass.set_bind_group(0, &reader_group, &[]);
        pass.dispatch_workgroups(TARGET_SIDE / 8, TARGET_SIDE / 8, 1);
    }
    encoder.copy_buffer_to_buffer(&owners, 0, &readback, 0, samples);
    context.queue.submit([encoder.finish()]);
    readback.map_async(wgpu::MapMode::Read, .., |result| {
        result.expect("the read-back maps");
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the device polls");
    let mapped = readback
        .get_mapped_range(..)
        .expect("the read-back is mapped");
    let owners: &[u32] = bytemuck::cast_slice(&mapped[..]);
    owners
        .iter()
        .map(|&owner| match raster.id {
            IdSource::BothKeepingPrimitive => owner & 0xFFFF,
            IdSource::BothKeepingWeights => owner >> 16,
            _ => owner,
        })
        .collect()
}

/// Each sample's triangle ID plus one as the geometry places it, for quads
/// that do not overlap; `None` within a hundredth of a pixel of an edge,
/// which vertex snapping may decide either way.
fn analytic(quads: &[Quad], ids: impl Iterator<Item = u32>, sample_count: u32) -> Vec<Option<u32>> {
    let positions = sample_positions(sample_count);
    let mut owners = vec![Some(0); (TARGET_SIDE * TARGET_SIDE) as usize * positions.len()];
    let side_pixels = f64::from(TARGET_SIDE);
    for id in ids {
        let [q0, q1, q2, q3] = quads[id as usize / 2];
        let corners = if id % 2 == 0 {
            [q0, q1, q2]
        } else {
            [q0, q2, q3]
        }
        .map(|[x, y, _, w]| {
            let (x, y, w) = (f64::from(x), f64::from(y), f64::from(w));
            [
                (x / w + 1.0) * side_pixels / 2.0,
                (1.0 - y / w) * side_pixels / 2.0,
            ]
        });
        let span = |axis: usize| {
            let values = corners.map(|corner| corner[axis]);
            let low = values.into_iter().fold(f64::MAX, f64::min).floor().max(0.0);
            let high = values.into_iter().fold(f64::MIN, f64::max).max(0.0);
            low as u32..=(high as u32).min(TARGET_SIDE - 1)
        };
        let orientation = ((corners[1][0] - corners[0][0]) * (corners[2][1] - corners[0][1])
            - (corners[1][1] - corners[0][1]) * (corners[2][0] - corners[0][0]))
            .signum();
        for y in span(1) {
            for x in span(0) {
                for (sample, offset) in positions.iter().enumerate() {
                    let point = [f64::from(x) + offset[0], f64::from(y) + offset[1]];
                    // Each edge's signed distance to the sample, positive
                    // inside.
                    let distances = [0, 1, 2].map(|edge| {
                        let (from, to) = (corners[edge], corners[(edge + 1) % 3]);
                        let (along_x, along_y) = (to[0] - from[0], to[1] - from[1]);
                        orientation
                            * (along_x * (point[1] - from[1]) - along_y * (point[0] - from[0]))
                            / along_x.hypot(along_y)
                    });
                    let slot = (y * TARGET_SIDE + x) as usize * positions.len() + sample;
                    if distances.iter().any(|distance| distance.abs() < 0.01) {
                        owners[slot] = None;
                    } else if distances.iter().all(|&distance| distance > 0.0)
                        && owners[slot].is_some()
                    {
                        owners[slot] = Some(id + 1);
                    }
                }
            }
        }
    }
    owners
}

/// A device beside the Capability floor with `primitive_index`, and 2x
/// MSAA of the ID format where the adapter has it, or `None` without them.
fn coverage_context() -> Option<GpuContext> {
    let context = gpu_context()?;
    let features = context.adapter.features();
    if !features.contains(wgpu::Features::PRIMITIVE_INDEX) {
        writeln!(std::io::stderr(), "skipped: no PRIMITIVE_INDEX").ok();
        return None;
    }
    let (device, queue) =
        pollster::block_on(context.adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::PRIMITIVE_INDEX
                | (features & wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES),
            ..Default::default()
        }))
        .expect("the adapter gives a device");
    Some(GpuContext {
        adapter: context.adapter,
        device,
        queue,
    })
}

/// What the compacted draw keeps of the triangles some sample shows.
#[derive(Clone, Copy)]
enum Granularity {
    /// Those triangles alone.
    Triangles,
    /// Their whole quads, the groups that share vertex indices.
    Quads,
}

/// The prototype's compaction on the tilted scene: a draw of every
/// triangle at depth equal after a pre-pass of them, then a draw of the
/// triangles some sample showed at `granularity`. Returns the samples whose
/// owner changed: to none, to its quad's other triangle, and to any other.
fn compaction_changes(
    context: &GpuContext,
    granularity: Granularity,
    indexing: Indexing,
) -> [usize; 3] {
    let quads = tilted_quads();
    let vertices = vertices(&quads);
    let every = every_triangle(quads.len() as u32, indexing);
    let raster = |draw: Vec<Triangle>| Raster {
        vertices: &vertices,
        draws: vec![draw],
        sample_count: 4,
        shading: Shading::Pixel,
        id: IdSource::Primitive,
        prepass: Some(every.clone()),
    };
    let full = rasterise(context, &raster(every.clone()));
    let shown: HashSet<u32> = full
        .iter()
        .filter(|&&owner| owner != 0)
        .map(|owner| owner - 1)
        .collect();
    let kept = every
        .iter()
        .copied()
        .filter(|(id, _)| match granularity {
            Granularity::Triangles => shown.contains(id),
            Granularity::Quads => shown.contains(id) || shown.contains(&(id ^ 1)),
        })
        .collect();
    let compacted = rasterise(context, &raster(kept));
    let mut changes = [0; 3];
    for (&full, &compacted) in full.iter().zip(&compacted) {
        if full != compacted {
            changes[match compacted {
                0 => 0,
                _ if same_quad(full, compacted) => 1,
                _ => 2,
            }] += 1;
        }
    }
    changes
}

/// Drawing the triangles `primitive_index` names in whole quads, the
/// groups that share vertex indices, keeps every sample the draw of every
/// triangle covers, each named by its own triangle or that triangle's
/// partner. On the M4 Pro, drawing the named triangles alone loses
/// samples, since fragments that read no flat input merge (ADR 0019); if
/// it stops losing them, that line of the ADR is due for review.
#[test]
fn a_draw_of_whole_index_sharing_groups_keeps_every_sample() {
    let Some(context) = coverage_context() else {
        return;
    };
    let [lost, _, others] = compaction_changes(&context, Granularity::Quads, Indexing::Shared);
    assert_eq!((lost, others), (0, 0));
    let info = context.adapter.get_info();
    if Machine::classify(&info.name, info.backend, info.device_type) == Machine::M4Pro {
        let [lost, ..] = compaction_changes(&context, Granularity::Triangles, Indexing::Shared);
        assert!(
            lost > 0,
            "the M4 Pro no longer merges index-sharing fragments"
        );
    }
}

/// The experiment table in `docs/performance.md` (2026-10-28), printed by
/// `cargo test --release --lib coverage_experiments -- --ignored
/// --nocapture`: per draw of the flat scene and sample count, the samples
/// whose owner departs from the oracle, to the other triangle of the same
/// quad and to any other.
#[test]
#[ignore = "prints a table and asserts nothing"]
fn coverage_experiments() {
    let Some(context) = coverage_context() else {
        return;
    };
    let quads = flat_quads();
    let vertices = vertices(&quads);
    let all = |indexing| every_triangle(quads.len() as u32, indexing);
    let shared = all(Indexing::Shared);
    let pick = |keep: fn(u32) -> bool| -> Vec<Triangle> {
        shared.iter().copied().filter(|(id, _)| keep(*id)).collect()
    };
    // Two quads' first triangles, then their partners: one triangle
    // between each pair.
    let interleaved = shared
        .chunks(4)
        .flat_map(|four| [0, 2, 1, 3].map(|at| four[at]))
        .collect();
    let (pixel, primitive) = (Shading::Pixel, IdSource::Primitive);
    let row = |name, draws: Vec<Vec<Triangle>>| (name, draws, pixel, primitive);
    let rows = [
        row(
            "Partner with its own indices (another Meshlet or Instance)",
            vec![all(Indexing::Own)],
        ),
        row(
            "Partner sharing two indices, as within a Meshlet",
            vec![shared.clone()],
        ),
        (
            "The same, sample-rate shading",
            vec![shared.clone()],
            Shading::Sample,
            primitive,
        ),
        (
            "The same, IDs from the weights each interpolates",
            vec![shared.clone()],
            pixel,
            IdSource::Interpolated,
        ),
        (
            "The same, IDs from smooth inputs alone",
            vec![shared.clone()],
            pixel,
            IdSource::Smooth,
        ),
        (
            "The same, primitive_index beside a smooth input",
            vec![shared.clone()],
            pixel,
            IdSource::PrimitiveBesideSmooth,
        ),
        (
            "The same, primitive_index beside a flat input",
            vec![shared.clone()],
            pixel,
            IdSource::PrimitiveBesideFlat,
        ),
        (
            "The same, both IDs from one fragment: primitive_index's",
            vec![shared.clone()],
            pixel,
            IdSource::BothKeepingPrimitive,
        ),
        (
            "The same, both IDs from one fragment: the weights'",
            vec![shared.clone()],
            pixel,
            IdSource::BothKeepingWeights,
        ),
        row(
            "Partner sharing them, led by another vertex",
            vec![all(Indexing::SharedRotated)],
        ),
        (
            "The same, IDs from each first vertex's flat output",
            vec![all(Indexing::SharedRotated)],
            pixel,
            IdSource::Flat,
        ),
        row("Partner removed", vec![pick(|id| id % 2 == 0)]),
        row("Every other quad removed", vec![pick(|id| id / 2 % 2 == 0)]),
        row(
            "Triangle 1 removed, so later pairs shift by one",
            vec![pick(|id| id != 1)],
        ),
        row("One triangle between partners", vec![interleaved]),
        row(
            "Every partner after every first triangle",
            vec![[pick(|id| id % 2 == 0), pick(|id| id % 2 == 1)].concat()],
        ),
        row(
            "Partners in a later draw call",
            vec![pick(|id| id % 2 == 0), pick(|id| id % 2 == 1)],
        ),
        row(
            "Partners in an earlier draw call",
            vec![pick(|id| id % 2 == 1), pick(|id| id % 2 == 0)],
        ),
    ];
    let has_2x = context
        .device
        .features()
        .contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
    let mut table =
        String::from("| Draw | 4x: partner's, other | 2x | 1x |\n|---|---:|---:|---:|\n");
    for (name, draws, shading, id) in rows {
        let cells = [4, 2, 1].map(|sample_count| {
            if sample_count == 2 && !has_2x {
                return "–".to_owned();
            }
            let raster = Raster {
                vertices: &vertices,
                draws: draws.clone(),
                sample_count,
                shading,
                id,
                prepass: None,
            };
            let got = rasterise(&context, &raster);
            let ids = draws.iter().flatten().map(|(id, _)| *id);
            let expected = analytic(&quads, ids, sample_count);
            let (mut partners, mut others) = (0, 0);
            for (&got, expected) in got.iter().zip(&expected) {
                match *expected {
                    Some(expected) if got != expected && same_quad(got, expected) => partners += 1,
                    Some(expected) if got != expected => others += 1,
                    _ => {}
                }
            }
            format!("{partners}, {others}")
        });
        table += &format!("| {name} | {} |\n", cells.join(" | "));
    }
    let covered = analytic(&quads, shared.iter().map(|(id, _)| *id), 4)
        .iter()
        .filter(|owner| matches!(owner, Some(owner) if *owner != 0))
        .count();
    table += &format!("\nflat scene, 4x: {covered} samples covered outside the edge band\n");
    let compaction = |granularity, indexing| compaction_changes(&context, granularity, indexing);
    table += &format!(
        "\ntilted, 4x, compacted after IDs at depth equal: samples lost, to the partner, other\n\
         triangles shown, shared indices: {:?}\n\
         triangles shown with their quads, shared indices: {:?}\n\
         triangles shown, own indices: {:?}\n",
        compaction(Granularity::Triangles, Indexing::Shared),
        compaction(Granularity::Quads, Indexing::Shared),
        compaction(Granularity::Triangles, Indexing::Own),
    );
    writeln!(std::io::stderr(), "{table}").ok();
}
