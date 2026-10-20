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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];
    let albedo_sample = textureSample(albedo_textures[material.texture_indices.x], pbr_sampler, in.uv);
    let base_color = albedo_sample.rgb * material.base_color.rgb;

    // Cooked metallic-roughness holds roughness in red and metallic in green
    // (BC5, ADR 0018); factors multiply.
    let metallic_roughness_sample = textureSample(metallic_roughness_textures[material.texture_indices.z], pbr_sampler, in.uv).rg;
    let metallic = metallic_roughness_sample.g * material.metallic_roughness.x;
    // GGX collapses to 0/0 at roughness 0; clamp keeps the highlight finite.
    let roughness = max(metallic_roughness_sample.r * material.metallic_roughness.y, 0.04);

    let ambient_occlusion = textureSample(occlusion_textures[material.texture_indices.w], pbr_sampler, in.uv).r;
    let emissive = textureSample(emissive_textures[material.extra.x], pbr_sampler, in.uv).rgb
        * material.emissive.rgb;

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

    let normal_sample = textureSample(normal_textures[material.texture_indices.y], pbr_sampler, in.uv).rg;
    let normal = normalize(tangent_to_world * tangent_space_normal(normal_sample));

    let view_direction = normalize(-in.camera_relative_position);

    // Shaded whether or not the uniform holds a directional light, since a
    // branch around it cost the main pass about 10% on the M4 Pro: without
    // one, its radiance is zero and a fixed direction keeps the term finite.
    let has_directional_light = light.directional_light_count > 0u;
    var outgoing_radiance = cook_torrance(
        normal,
        view_direction,
        select(vec3<f32>(0.0, 1.0, 0.0), normalize(-light.direction.xyz), has_directional_light),
        base_color,
        metallic,
        roughness,
        light.color.rgb * light.color.a * f32(light.directional_light_count)
    );

    for (var light_index = 0u; light_index < light.point_light_count; light_index++) {
        let light_vector = light.point_positions[light_index].xyz - in.camera_relative_position;
        let distance = length(light_vector);
        let attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
        outgoing_radiance += cook_torrance(
            normal,
            view_direction,
            normalize(light_vector),
            base_color,
            metallic,
            roughness,
            light.point_colors[light_index].rgb * light.point_colors[light_index].a * attenuation
        );
    }

    // Constant ambient stand-in until ADR 0020's irradiance probes (M5); occlusion darkens it.
    let ambient = 0.03 * base_color * ambient_occlusion;
    return vec4<f32>(
        ambient + outgoing_radiance + emissive,
        albedo_sample.a * material.base_color.a
    );
}
