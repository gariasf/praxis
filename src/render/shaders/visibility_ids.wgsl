// The depth passes' entry points with the visibility_buffer Render switch
// on, after the main pass's parts: beside depth, and with shadow rays on the
// surface normals, they write the triangle each sample shows, which the
// material pass reads (src/render/material_pass.rs).
//
// Each fragment stage reads a flat input, the Material ID, so the M4 Pro
// merges no fragments of triangles sharing an edge and each sample's ID
// names the triangle that covers it (ADR 0019).

// Whether the pipeline draws phase two's draw, as the depth pass sets it.
override PHASE_TWO: bool;

// The ID of the triangle `primitive` of the phase's draw, as the attachment
// stores it; `material_id` is read to no effect, since Material IDs index
// the Material table, far below NO_MATERIAL - 1.
fn visibility_id(primitive: u32, material_id: u32) -> vec2<u32> {
    let id = select(primitive, primitive | PHASE_TWO_BIT, PHASE_TWO);
    return encode_visibility_id(select(id, 0u, material_id == NO_MATERIAL - 1u));
}

// With shadow rays off, the depth passes' vertex output: the clip position
// and the flat Material ID the fragment stage reads.
struct VisibilityOutput {
    @builtin(position) @invariant clip_position: vec4<f32>,
    @location(0) @interpolate(flat) material_id: u32,
}

@vertex
fn vs_visibility(@builtin(vertex_index) packed_index: u32) -> VisibilityOutput {
    let out = vertex_output(packed_index);
    return VisibilityOutput(out.clip_position, out.material_id);
}

@fragment
fn fs_visibility(
    in: VisibilityOutput,
    @builtin(primitive_index) primitive: u32
) -> @location(0) vec2<u32> {
    return visibility_id(primitive, in.material_id);
}

// With shadow rays on, the surface normals and the ID.
struct NormalsAndId {
    @location(0) normals: vec4<f32>,
    @location(1) id: vec2<u32>,
}

@fragment
fn fs_visibility_normals(
    in: DepthNormalsOutput,
    @builtin(primitive_index) primitive: u32
) -> NormalsAndId {
    return NormalsAndId(surface_normals(in), visibility_id(primitive, in.material_id));
}
