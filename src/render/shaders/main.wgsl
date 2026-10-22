// The main pass's entry points, the depth pre-pass's and the lit pass's,
// after the shared parts; its bindings are declared from their bind groups'
// definitions (src/render/bind_groups.rs).

struct VertexOutput {
    // Invariant in both passes, which compute it alike, so the lit pass's
    // depth-equal test meets exactly the depths the pre-pass kept.
    @builtin(position) @invariant clip_position: vec4<f32>,
    @location(0) camera_relative_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material_id: u32,
    @location(4) tangent_world: vec4<f32>, // xyz = world tangent, w = bitangent sign
}

@vertex
fn vs_depth_prepass(@builtin(vertex_index) packed_index: u32) -> @builtin(position) @invariant vec4<f32> {
    return vertex_output(packed_index).clip_position;
}

// The lit pass draws phase two's Instances into depth before it shades,
// with this and `vs_depth_prepass`: it writes no colour.
@fragment
fn fs_depth() {}

@vertex
fn vs_main(@builtin(vertex_index) packed_index: u32) -> VertexOutput {
    return vertex_output(packed_index);
}

fn vertex_output(packed_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // The frame's draw has no base vertex, so the vertex index is the packed
    // index: a Meshlet record, which names the Instance, the Meshlet and the
    // Material, and a vertex of that Meshlet (ADR 0019).
    let index = unpack_index(packed_index);
    let record = meshlet_records[index.record];
    let instance = instances[record.instance_slot];
    let first_vertex = meshlets[record.meshlet].first_vertex;
    let vertex = decode_vertex(vertices[meshlet_vertices[first_vertex + index.vertex]]);
    let position = camera_relative_position(
        instance.anchor,
        instance.offset,
        camera.anchor,
        camera.offset,
        instance.rotation_scale * vertex.position
    );
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    out.uv = vertex.uv;
    let tangent_frame = vertex.tangent_frame;
    out.normal = instance.normal_matrix * tangent_frame.normal;
    // Tangents are directions along the surface, so they follow the
    // rotation-scale; only normals need the inverse-transpose.
    out.tangent_world = vec4<f32>(
        instance.rotation_scale * tangent_frame.tangent,
        tangent_frame.bitangent_sign
    );
    out.camera_relative_position = position;
    out.material_id = record.material_id;
    return out;
}

// Every Texture is sampled first, so none is sampled under non-uniform
// control flow; the rest decodes the surface and sums what each light
// reflects with the ambient term and the emissive.
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];
    let surface = decode_surface(
        material,
        textureSample(albedo_textures[material.layers.x], colour_sampler, in.uv),
        textureSample(metallic_roughness_textures[material.layers.z], data_sampler, in.uv).rg,
        textureSample(occlusion_textures[material.layers.w], data_sampler, in.uv).r,
        textureSample(emissive_textures[material.extra.x], colour_sampler, in.uv).rgb
    );
    let normal_sample = textureSample(normal_textures[material.layers.y], data_sampler, in.uv).rg;

    // Tangent frame. Interpolation bends the tangent off the normal, so
    // re-orthogonalise it (Gram-Schmidt) before rebuilding the bitangent with
    // the glTF handedness sign.
    let geometric_normal = normalize(in.normal);
    let interpolated_tangent = in.tangent_world.xyz;
    let tangent = normalize(
        interpolated_tangent - geometric_normal * dot(geometric_normal, interpolated_tangent)
    );
    let bitangent = cross(geometric_normal, tangent) * in.tangent_world.w;
    let tangent_to_world = mat3x3<f32>(tangent, bitangent, geometric_normal);
    let normal = normalize(tangent_to_world * tangent_space_normal(normal_sample));

    let view_direction = normalize(-in.camera_relative_position);

    // The first directional light is shaded whether or not the light buffer
    // holds one, since a branch around it cost the lit pass about 10% on
    // the M4 Pro: without one, its illuminance is zero and a fixed direction
    // keeps the term finite. Any others follow.
    let has_directional_light = light.directional_light_count > 0u;
    let first = lights[0];
    var outgoing_luminance = reflected_luminance(
        surface,
        normal,
        view_direction,
        select(vec3<f32>(0.0, 1.0, 0.0), normalize(-first.offset), has_directional_light),
        first.intensity * f32(has_directional_light)
    );
    for (var index = 1u; index < light.directional_light_count; index++) {
        let directional = lights[index];
        outgoing_luminance += reflected_luminance(
            surface,
            normal,
            view_direction,
            normalize(-directional.offset),
            directional.intensity
        );
    }
    let light_count = light.directional_light_count + light.point_light_count;
    for (var index = light.directional_light_count; index < light_count; index++) {
        let point = lights[index];
        let position = point_light_position(point, camera.anchor, camera.offset);
        outgoing_luminance += point_light_luminance(
            surface,
            normal,
            view_direction,
            position - in.camera_relative_position,
            point.intensity,
            point.range
        );
    }

    let luminance = ambient_luminance(surface, light.ambient_illuminance.rgb) + outgoing_luminance
        + surface.emissive;
    // Pre-exposed in f32 and clamped to fp16's range before the HDR target
    // stores it (ADR 0023), weighted for the resolve.
    let exposed = min(luminance * camera.exposure, vec3<f32>(LARGEST_HALF));
    return vec4<f32>(resolve_weighted(exposed), surface.opacity);
}
