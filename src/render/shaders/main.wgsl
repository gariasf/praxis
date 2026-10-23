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
    // The position again, interpolated within the samples the triangle
    // covers: at an edge pixel the centre can lie off the triangle, where
    // its plane, extended past a crease, runs inside the mesh, so a shadow
    // ray starting there would find the mesh in its way.
    @location(5) @interpolate(perspective, centroid) centroid_position: vec3<f32>,
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
    out.centroid_position = position;
    out.material_id = record.material_id;
    return out;
}

// A directional light's shadow ray runs to f32's largest distance, past
// anything a TLAS holds.
const UNBOUNDED_RAY_DISTANCE: f32 = 3.40282347e38;

// A shadow ray's origin leaves its surface along the normal by this share
// of the point's largest coordinate, relative to the camera or in the TLAS's
// frame, and of 1 m at least. Rasterising the point, composing it into the
// TLAS's frame and intersecting the TLAS each round a coordinate by at most
// 2^-24 of its size, so the origin clears the surface by 256 roundings.
const SHADOW_RAY_OFFSET_SCALE: f32 = 1.52587890625e-5; // 2^-16

// Whether either TLAS holds geometry within `distance` of the surface point
// at `position`, relative to the camera, along `direction`, a unit vector:
// one hard shadow ray (ADR 0020), traced against the static TLAS, then,
// where that finds nothing, against the dynamic TLAS unless it is empty: a
// query of an empty TLAS still costs the lit pass. It starts off the
// surface along `normal`, its unit geometric normal, in the frame both TLAS
// share, whose origin is the corner of the camera's `tlas_anchor`, composed
// as an Instance's position is (ADR 0014).
fn shadowed(position: vec3<f32>, normal: vec3<f32>, direction: vec3<f32>, distance: f32) -> bool {
    // The camera stands where an Instance does in this composition, and the
    // TLAS's frame, with no offset from its Anchor, where the camera does.
    let tlas_position = camera_relative_position(
        camera.anchor,
        camera.offset,
        camera.tlas_anchor,
        vec3<f32>(0.0),
        position
    );
    let extent = max(abs(tlas_position), abs(position));
    let offset = SHADOW_RAY_OFFSET_SCALE * max(max(extent.x, extent.y), max(extent.z, 1.0));
    // Opaque, so one rayQueryProceed completes a traversal, and any hit
    // shadows the point.
    let ray = RayDesc(
        RAY_FLAG_FORCE_OPAQUE | RAY_FLAG_TERMINATE_ON_FIRST_HIT,
        0xffu,
        0.0,
        distance,
        tlas_position + normal * offset,
        direction
    );
    var query: ray_query;
    rayQueryInitialize(&query, static_tlas, ray);
    rayQueryProceed(&query);
    let hidden = rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE;
    if hidden || camera.dynamic_tlas_entries == 0u {
        return hidden;
    }
    rayQueryInitialize(&query, dynamic_tlas, ray);
    rayQueryProceed(&query);
    return rayQueryGetCommittedIntersection(&query).kind != RAY_QUERY_INTERSECTION_NONE;
}

// Every Texture is sampled first, so none is sampled under non-uniform
// control flow; the rest decodes the surface and sums what each light
// reflects, unless a shadow ray finds it hidden, with the ambient term and
// the emissive.
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
    let first_direction = select(
        vec3<f32>(0.0, 1.0, 0.0),
        normalize(-first.offset),
        has_directional_light
    );
    var first_illuminance = first.intensity * f32(has_directional_light);
    // Each directional light's shadow ray is traced only where the light
    // adds something.
    if SHADOW_RAYS && has_directional_light && dot(normal, first_direction) > 0.0
        && shadowed(
            in.centroid_position,
            geometric_normal,
            first_direction,
            UNBOUNDED_RAY_DISTANCE
        ) {
        first_illuminance = vec3<f32>(0.0);
    }
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
        var illuminance = directional.intensity;
        if SHADOW_RAYS && dot(normal, direction) > 0.0
            && shadowed(in.centroid_position, geometric_normal, direction, UNBOUNDED_RAY_DISTANCE) {
            illuminance = vec3<f32>(0.0);
        }
        outgoing_luminance += reflected_luminance(
            surface,
            normal,
            view_direction,
            direction,
            illuminance
        );
    }
    // The point lights the pixel's Froxel lists, in light order, or, with
    // FROXEL_LIGHTS off, every point light: brute force.
    var froxel = 0u;
    var point_light_count = light.point_light_count;
    if FROXEL_LIGHTS {
        froxel = froxel_at(camera.view_proj * vec4<f32>(in.camera_relative_position, 1.0));
        point_light_count = froxel_lists[froxel].light_count;
    }
    for (var entry = 0u; entry < point_light_count; entry++) {
        var index = light.directional_light_count + entry;
        if FROXEL_LIGHTS {
            index = froxel_lists[froxel].lights[entry];
        }
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
