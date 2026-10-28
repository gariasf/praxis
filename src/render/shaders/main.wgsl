// The main pass's entry points, the depth pre-pass's and the lit pass's,
// after the shared parts; its bindings are declared from their bind groups'
// definitions (src/render/bind_groups.rs).

@vertex
fn vs_depth_prepass(@builtin(vertex_index) packed_index: u32) -> @builtin(position) @invariant vec4<f32> {
    return vertex_output(packed_index).clip_position;
}

// With shadow rays on, the depth passes' vertex output: the lit pass's
// position and normal, so their depths meet its, and its flat Material ID,
// which their fragment stage reads, as the lit pass's does, so that the
// M4 Pro merges neither's fragments and each sample's normals come from the
// triangle the lit pass shades there (ADR 0019).
struct DepthNormalsOutput {
    @builtin(position) @invariant clip_position: vec4<f32>,
    @location(0) camera_relative_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(flat) material_id: u32,
}

@vertex
fn vs_depth_normals(@builtin(vertex_index) packed_index: u32) -> DepthNormalsOutput {
    let out = vertex_output(packed_index);
    return DepthNormalsOutput(
        out.clip_position,
        out.camera_relative_position,
        out.normal,
        out.material_id
    );
}

// The surface normals the shadow mask pass reads (SURFACE_NORMALS_FORMAT,
// src/render/targets.rs): the triangle's own normal, facing the camera, from
// the position's screen-space derivatives, both in the triangle's plane, and
// the normal interpolated over it. A triangle seen edge-on, whose derivatives
// span no plane, faces the camera, and a normal interpolated to nothing is
// the triangle's.
fn surface_normals(in: DepthNormalsOutput) -> vec4<f32> {
    let crossed = cross(dpdx(in.camera_relative_position), dpdy(in.camera_relative_position));
    let crossed_length_squared = dot(crossed, crossed);
    let toward_camera = -normalize(in.camera_relative_position);
    let unit_crossed = crossed * inverseSqrt(crossed_length_squared);
    let triangle_normal = select(
        toward_camera,
        select(unit_crossed, -unit_crossed, dot(unit_crossed, toward_camera) < 0.0),
        crossed_length_squared > 0.0
    );
    let normal_length_squared = dot(in.normal, in.normal);
    let interpolated_normal = select(
        triangle_normal,
        in.normal * inverseSqrt(normal_length_squared),
        normal_length_squared > 0.0
    );
    return vec4<f32>(octahedral_encode(triangle_normal), octahedral_encode(interpolated_normal))
        * 0.5 + 0.5;
}

@fragment
fn fs_surface_normals(in: DepthNormalsOutput) -> @location(0) vec4<f32> {
    // Reads the flat input, so the stage has one the compiler cannot drop,
    // to no effect: Material IDs index the Material table, far below
    // NO_MATERIAL - 1.
    return select(surface_normals(in), vec4<f32>(0.0), in.material_id == NO_MATERIAL - 1u);
}

// With shadow rays off, the lit pass draws phase two's Instances into depth
// before it shades, with this and `vs_depth_prepass`: it writes no colour.
@fragment
fn fs_depth() {}

@vertex
fn vs_main(@builtin(vertex_index) packed_index: u32) -> VertexOutput {
    return vertex_output(packed_index);
}

// Every Texture is sampled first, and the shading normal differentiated, so
// neither happens under non-uniform control flow; `lit_colour` does the
// rest.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];
    let albedo = textureSample(albedo_textures[material.layers.x], colour_sampler, in.uv);
    let metallic_roughness =
        textureSample(metallic_roughness_textures[material.layers.z], data_sampler, in.uv).rg;
    let occlusion = textureSample(occlusion_textures[material.layers.w], data_sampler, in.uv).r;
    let emissive = textureSample(emissive_textures[material.extra.x], colour_sampler, in.uv).rgb;
    let normal_sample = textureSample(normal_textures[material.layers.y], data_sampler, in.uv).rg;

    let normal = shading_normal(
        in.normal,
        in.tangent_world,
        normal_sample,
        material.metallic_roughness.z
    );
    // The normal's change across a pixel widens roughness. The lit pass
    // shades once per pixel under 4x MSAA, not per sample, so these
    // derivatives are per pixel, the footprint the filter assumes.
    let surface = decode_surface(
        material,
        albedo,
        metallic_roughness,
        occlusion,
        emissive,
        dpdx(normal),
        dpdy(normal)
    );

    return lit_colour(surface, normal, in.camera_relative_position, in.clip_position.xy);
}
