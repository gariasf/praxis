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
@fragment
fn fs_surface_normals(in: DepthNormalsOutput) -> @location(0) vec4<f32> {
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
    let encoded_normals = vec4<f32>(octahedral_encode(triangle_normal), octahedral_encode(interpolated_normal))
        * 0.5 + 0.5;
    // Reads the flat input, so the stage has one the compiler cannot drop,
    // to no effect: Material IDs index the Material table, far below
    // NO_MATERIAL - 1.
    return select(encoded_normals, vec4<f32>(0.0), in.material_id == NO_MATERIAL - 1u);
}

// With shadow rays off, the lit pass draws phase two's Instances into depth
// before it shades, with this and `vs_depth_prepass`: it writes no colour.
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

// A bit past every bit the shadow mask holds for lights.
const UNMASKED: u32 = 0xffffffffu;

// Whether the shadow mask's word for a pixel, `mask`, hides the light whose
// bit is `bit` (src/render/shadow_mask_pass.rs): a light past its bits casts
// no shadow there.
fn masked(mask: vec2<u32>, bit: u32) -> bool {
    let half = select(mask.x, mask.y, bit >= 32u);
    return bit < SHADOW_MASK_LIGHTS && ((half >> (bit & 31u)) & 1u) == 1u;
}

// The bit of the point light at `index` of the light buffer in a shadow mask
// whose bits follow the lights of `froxel`: after the directional lights',
// at its entry in the Froxel's list, which is in light order, so a binary
// search finds it; UNMASKED where the list lacks it.
fn froxel_mask_bit(froxel: u32, index: u32) -> u32 {
    let count = froxel_lists[froxel].light_count;
    var low = 0u;
    var high = count;
    while low < high {
        let middle = (low + high) / 2u;
        if froxel_lists[froxel].lights[middle] < index {
            low = middle + 1u;
        } else {
            high = middle;
        }
    }
    let listed = low < count && froxel_lists[froxel].lights[low] == index;
    return select(UNMASKED, light.directional_light_count + low, listed);
}

// Every Texture is sampled first, and the shading normal differentiated, so
// neither happens under non-uniform control flow; the rest sums what each
// light reflects, unless the shadow mask hides it, with the ambient term and
// the emissive.
@fragment
fn fs_main(in: VertexOutput, @builtin(sample_mask) covered_samples: u32) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];
    let albedo = textureSample(albedo_textures[material.layers.x], colour_sampler, in.uv);
    let metallic_roughness =
        textureSample(metallic_roughness_textures[material.layers.z], data_sampler, in.uv).rg;
    let occlusion = textureSample(occlusion_textures[material.layers.w], data_sampler, in.uv).r;
    let emissive = textureSample(emissive_textures[material.extra.x], colour_sampler, in.uv).rgb;
    let normal_sample = textureSample(normal_textures[material.layers.y], data_sampler, in.uv).rg;

    // Tangent frame. Interpolation bends the tangent off the normal, so
    // re-orthogonalise it (Gram-Schmidt) before rebuilding the bitangent with
    // the glTF handedness sign [ref:gltf-2.0-bitangent].
    let geometric_normal = normalize(in.normal);
    let interpolated_tangent = in.tangent_world.xyz;
    let tangent = normalize(
        interpolated_tangent - geometric_normal * dot(geometric_normal, interpolated_tangent)
    );
    let bitangent = cross(geometric_normal, tangent) * in.tangent_world.w;
    let tangent_to_world = mat3x3<f32>(tangent, bitangent, geometric_normal);
    let normal = normalize(
        tangent_to_world * tangent_space_normal(normal_sample, material.metallic_roughness.z)
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

    let view_direction = normalize(-in.camera_relative_position);

    // The first directional light is shaded whether or not the light buffer
    // holds one, since a branch around it cost the lit pass about 10% on
    // the M4 Pro: without one, its illuminance is zero and a fixed direction
    // keeps the term finite. Any others follow.
    let has_directional_light = light.directional_light_count > 0u;
    let first = lights[0];
    let first_direction = select(
        vec3<f32>(0.0, 1.0, 0.0),
        normalize(-first.offset),
        has_directional_light
    );
    // The shadow mask's word for this pixel's surface that this fragment
    // shades: none with shadow rays off. Its second surface's where the
    // fragment covers a sample that shows it, as surface-based
    // antialiasing shades each of a pixel's surfaces once
    // [ref:salvi-2012-sbaa-surfaces]; the nearest one's otherwise. The
    // coverage may precede the depth test, so a fragment of the nearest
    // surface that runs under another at a sample takes that one's. Its bits
    // follow the pixel's lights in the order they are summed here, the
    // directional lights first.
    var mask = vec2<u32>(0u);
    if SHADOW_RAYS {
        let pixel = vec2<u32>(in.clip_position.xy);
        let second_samples = textureLoad(shadow_mask_samples, pixel, 0).r;
        let surface = select(0u, 1u, (covered_samples & second_samples) != 0u);
        mask = textureLoad(shadow_mask, pixel, surface, 0).rg;
    }
    let first_illuminance = first.intensity * f32(has_directional_light && !masked(mask, 0u));
    var outgoing_luminance = reflected_luminance(
        surface,
        normal,
        view_direction,
        first_direction,
        first_illuminance
    );
    for (var index = 1u; index < light.directional_light_count; index++) {
        let directional = lights[index];
        let direction = normalize(-directional.offset);
        let illuminance = directional.intensity * f32(!masked(mask, index));
        outgoing_luminance += reflected_luminance(
            surface,
            normal,
            view_direction,
            direction,
            illuminance
        );
    }
    // The point lights the pixel's Froxel lists, in light order, or, with
    // FROXEL_LIGHTS off, every point light: brute force. The mask's bits
    // follow the lights of the Froxel at this pixel's rectangle and the
    // slice that the sample its word was traced from shows, which the top
    // bits of the word's second half hold: this fragment's Froxel but where
    // a slice ends between the sample and the pixel's centre, or at an edge
    // of more than two surfaces.
    var froxel = 0u;
    var mask_froxel = 0u;
    var point_light_count = light.point_light_count;
    if FROXEL_LIGHTS {
        let view_depth = (camera.view_proj * vec4<f32>(in.camera_relative_position, 1.0)).w;
        let size = vec2<f32>(textureDimensions(shadow_mask));
        froxel = froxel_at(in.clip_position.xy, size, view_depth);
        mask_froxel = froxel - froxel % FROXEL_SLICES + (mask.y >> (SHADOW_MASK_LIGHTS - 32u));
        point_light_count = froxel_lists[froxel].light_count;
    }
    for (var entry = 0u; entry < point_light_count; entry++) {
        var index = light.directional_light_count + entry;
        if FROXEL_LIGHTS {
            index = froxel_lists[froxel].lights[entry];
        }
        let point = lights[index];
        let position = point_light_position(point, camera.anchor, camera.offset);
        var bit = light.directional_light_count + entry;
        if SHADOW_RAYS && FROXEL_LIGHTS && mask_froxel != froxel {
            bit = froxel_mask_bit(mask_froxel, index);
        }
        let intensity = point.intensity * f32(!masked(mask, bit));
        outgoing_luminance += point_light_luminance(
            surface,
            normal,
            view_direction,
            position - in.camera_relative_position,
            intensity,
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
