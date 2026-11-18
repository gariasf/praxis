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
fn surface_normals(camera_relative_position: vec3<f32>, normal: vec3<f32>, material_id: u32) -> vec4<f32> {
    let crossed = cross(dpdx(camera_relative_position), dpdy(camera_relative_position));
    let crossed_length_squared = dot(crossed, crossed);
    let toward_camera = -normalize(camera_relative_position);
    let unit_crossed = crossed * inverseSqrt(crossed_length_squared);
    let triangle_normal = select(
        toward_camera,
        select(unit_crossed, -unit_crossed, dot(unit_crossed, toward_camera) < 0.0),
        crossed_length_squared > 0.0
    );
    let normal_length_squared = dot(normal, normal);
    let interpolated_normal = select(
        triangle_normal,
        normal * inverseSqrt(normal_length_squared),
        normal_length_squared > 0.0
    );
    let encoded_normals = vec4<f32>(octahedral_encode(triangle_normal), octahedral_encode(interpolated_normal))
        * 0.5 + 0.5;
    // Reads the flat input, so the stage has one the compiler cannot drop,
    // to no effect: Material IDs index the Material table, far below
    // NO_MATERIAL - 1.
    return select(encoded_normals, vec4<f32>(0.0), material_id == NO_MATERIAL - 1u);
}

@fragment
fn fs_surface_normals(in: DepthNormalsOutput) -> @location(0) vec4<f32> {
    return surface_normals(in.camera_relative_position, in.normal, in.material_id);
}

// With shadow rays off, the lit pass draws phase two's Instances into depth
// before it shades, with this and `vs_depth_prepass`: it writes no colour.
@fragment
fn fs_depth() {}

@vertex
fn vs_main(@builtin(vertex_index) packed_index: u32) -> VertexOutput {
    return vertex_output(packed_index);
}

// A vertex of a Thin Primitive or of a small Instance in each of their
// pipelines: the lit pass's, with the tube's radius as its Instance scales
// it, 0 for any other vertex, what a pixel spans at the tube's centre line
// there, and its Instance's coverage.
struct ThinVertexOutput {
    @builtin(position) @invariant clip_position: vec4<f32>,
    @location(0) camera_relative_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material_id: u32,
    @location(4) tangent_world: vec4<f32>,
    @location(5) @interpolate(flat) radius_metres: f32,
    // At the centroid: read outside its triangle, a tube past its end distance would keep a sample.
    @location(6) @interpolate(perspective, centroid) pixel_metres: f32,
    @location(7) @interpolate(flat) instance_coverage: f32,
}

// A Thin Primitive's vertex, moved out from its tube's centre line, which
// lies the radius behind it along its normal, until the tube draws a pixel
// wide where it would draw narrower; or any other vertex, of a small
// Instance, moved out from the centre of its Mesh bounds across the view
// until they cover a pixel's area. Every pipeline that draws one runs this
// stage, so the lit pass meets the depths the depth passes kept.
@vertex
fn vs_thin(@builtin(vertex_index) packed_index: u32) -> ThinVertexOutput {
    var out = vertex_output(packed_index);
    let index = unpack_index(packed_index);
    let record = meshlet_records[index.record];
    let meshlet = meshlets[record.meshlet];
    let instance = instances[record.instance_slot];
    var radius_metres = 0.0;
    var pixel_metres = 0.0;
    var instance_coverage = 1.0;
    if meshlet.thin_radius_metres > 0.0 {
        let normal =
            decode_vertex(vertices[meshlet_vertices[meshlet.first_vertex + index.vertex]]).tangent_frame.normal;
        let radial = instance.rotation_scale * (normal * meshlet.thin_radius_metres);
        radius_metres = length(radial);
        let centre = out.camera_relative_position - radial;
        pixel_metres = camera.pixel_size_per_metre * (camera.view_proj * vec4<f32>(centre, 1.0)).w;
        // At or behind the camera's plane a pixel spans nothing: the vertex stays.
        if pixel_metres > 0.0 && radius_metres > 0.0 {
            let drawn_radius_metres = thin_drawn_radius(radius_metres, pixel_metres);
            out.camera_relative_position = centre + radial * (drawn_radius_metres / radius_metres);
            out.clip_position = camera.view_proj * vec4<f32>(out.camera_relative_position, 1.0);
        }
    } else {
        let stored_min = meshes[instance.mesh].bounds_min;
        let stored_max = meshes[instance.mesh].bounds_max;
        let bounds_min = vec3<f32>(stored_min[0], stored_min[1], stored_min[2]);
        let bounds_max = vec3<f32>(stored_max[0], stored_max[1], stored_max[2]);
        let bounds_centre = camera_relative_position(
            instance.anchor,
            instance.offset,
            camera.anchor,
            camera.offset,
            instance.rotation_scale * (0.5 * (bounds_min + bounds_max))
        );
        instance_coverage = small_instance_coverage(
            bounds_centre,
            instance.rotation_scale,
            0.5 * (bounds_max - bounds_min),
            (camera.view_proj * vec4<f32>(bounds_centre, 1.0)).w,
            camera.pixel_size_per_metre
        );
        // Across the view only: each vertex keeps its depth, as the culling pass's box does.
        let view_axis = vec3<f32>(camera.view_proj[0].w, camera.view_proj[1].w, camera.view_proj[2].w);
        let from_centre = out.camera_relative_position - bounds_centre;
        let along_view = view_axis * (dot(from_centre, view_axis) / max(dot(view_axis, view_axis), 1e-30));
        out.camera_relative_position = bounds_centre + along_view
            + (from_centre - along_view) * small_instance_scale(instance_coverage);
        out.clip_position = camera.view_proj * vec4<f32>(out.camera_relative_position, 1.0);
    }
    return ThinVertexOutput(
        out.clip_position,
        out.camera_relative_position,
        out.normal,
        out.uv,
        out.material_id,
        out.tangent_world,
        radius_metres,
        pixel_metres,
        instance_coverage
    );
}

// The samples of a pixel's four that a fragment of a Thin Primitive keeps:
// the tube's true width over the width it draws, of them, to the nearest;
// any other fragment keeps its Instance's coverage of them. The depth passes
// write them, and the lit pass shades what covers them. The radius is a flat
// input, which keeps the M4 Pro from merging the fragments of triangles that
// share indices (ADR 0019).
fn thin_fragment_samples(in: ThinVertexOutput) -> u32 {
    let width_share = in.radius_metres / thin_drawn_radius(in.radius_metres, max(in.pixel_metres, 1e-9));
    return thin_kept_samples(select(in.instance_coverage, width_share, in.radius_metres > 0.0));
}

@fragment
fn fs_thin_depth(in: ThinVertexOutput) -> @builtin(sample_mask) u32 {
    return thin_fragment_samples(in);
}

struct ThinSurfaceNormals {
    @location(0) normals: vec4<f32>,
    @builtin(sample_mask) kept_samples: u32,
}

@fragment
fn fs_thin_surface_normals(in: ThinVertexOutput) -> ThinSurfaceNormals {
    return ThinSurfaceNormals(
        surface_normals(in.camera_relative_position, in.normal, in.material_id),
        thin_fragment_samples(in)
    );
}

// Its triangle may reach samples the depth passes did not keep. One that
// keeps none writes nothing: another fragment's depth may equal its own.
@fragment
fn fs_thin_main(in: ThinVertexOutput, @builtin(sample_mask) covered_samples: u32) -> @location(0) vec4<f32> {
    let vertex =
        VertexOutput(in.clip_position, in.camera_relative_position, in.normal, in.uv, in.material_id, in.tangent_world);
    let kept_samples = covered_samples & thin_fragment_samples(in);
    if kept_samples == 0u {
        discard;
    }
    return lit(vertex, kept_samples);
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

// The share of the samples a fragment covers, `covered_samples`, from which
// the directional light at `index` of the light buffer is hidden: where the
// fragment shades the pixel's nearest surface, from `sample_verdicts`, the
// shadow mask samples' verdicts at each sample for the first
// SHADOW_EDGE_LIGHTS directional lights, SAMPLE_COUNT bits a light, as
// MSAA resolves an edge from its samples; otherwise the mask's bit, 0 or 1.
fn directional_hidden_share(
    mask: vec2<u32>,
    sample_verdicts: u32,
    nearest_surface: bool,
    covered_samples: u32,
    index: u32
) -> f32 {
    let hidden = (sample_verdicts >> (SAMPLE_COUNT * index)) & covered_samples;
    let share = f32(countOneBits(hidden)) / f32(max(countOneBits(covered_samples), 1u));
    return select(f32(masked(mask, index)), share, nearest_surface && index < SHADOW_EDGE_LIGHTS);
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
        let middle = low + (high - low) / 2u;
        if froxel_lists[froxel].lights[middle] < index {
            low = middle + 1u;
        } else {
            high = middle;
        }
    }
    let listed = low < count && froxel_lists[froxel].lights[low] == index;
    return select(UNMASKED, light.directional_light_count + low, listed);
}

@fragment
fn fs_main(in: VertexOutput, @builtin(sample_mask) covered_samples: u32) -> @location(0) vec4<f32> {
    return lit(in, covered_samples);
}

// Every Texture is sampled first, and the shading normal differentiated, so
// neither happens under non-uniform control flow; the rest sums what each
// light reflects, unless the shadow mask hides it, with the ambient term and
// the emissive.
fn lit(in: VertexOutput, covered_samples: u32) -> vec4<f32> {
    let material = materials[in.material_id];
    let albedo_class_and_slice = layer_class_and_slice(material.layers.x);
    let normal_class_and_slice = layer_class_and_slice(material.layers.y);
    let metallic_roughness_class_and_slice = layer_class_and_slice(material.layers.z);
    let occlusion_class_and_slice = layer_class_and_slice(material.layers.w);
    let emissive_class_and_slice = layer_class_and_slice(material.extra.x);
    let albedo =
        textureSample(albedo_textures[albedo_class_and_slice.x], colour_sampler, in.uv, albedo_class_and_slice.y);
    let metallic_roughness = textureSample(
        metallic_roughness_textures[metallic_roughness_class_and_slice.x],
        data_sampler,
        in.uv,
        metallic_roughness_class_and_slice.y,
    ).rg;
    let occlusion = textureSample(
        occlusion_textures[occlusion_class_and_slice.x],
        data_sampler,
        in.uv,
        occlusion_class_and_slice.y,
    ).r;
    let emissive = textureSample(
        emissive_textures[emissive_class_and_slice.x],
        colour_sampler,
        in.uv,
        emissive_class_and_slice.y,
    ).rgb;
    let normal_sample =
        textureSample(normal_textures[normal_class_and_slice.x], data_sampler, in.uv, normal_class_and_slice.y).rg;

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
    // holds one, since a branch around it slowed the lit pass on the M4 Pro
    // (measured: `docs/performance.md`, Measurements behind the code):
    // without one, its illuminance is zero and a fixed direction keeps the
    // term finite. Any others follow.
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
    // directional lights first. At the nearest surface, the first
    // directional lights' verdicts at each sample stand in for their bits.
    var mask = vec2<u32>(0u);
    var sample_verdicts = 0u;
    var nearest_surface = false;
    let covered = covered_samples & ALL_SAMPLES;
    if SHADOW_RAYS {
        let pixel = vec2<u32>(in.clip_position.xy);
        let samples = textureLoad(shadow_mask_samples, pixel, 0).r;
        nearest_surface = (covered & samples) == 0u;
        mask = textureLoad(shadow_mask, pixel, select(1u, 0u, nearest_surface), 0).rg;
        sample_verdicts = samples >> SAMPLE_COUNT;
    }
    let first_illuminance = first.intensity * f32(has_directional_light)
        * (1.0 - directional_hidden_share(mask, sample_verdicts, nearest_surface, covered, 0u));
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
        let illuminance = directional.intensity
            * (1.0 - directional_hidden_share(mask, sample_verdicts, nearest_surface, covered, index));
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
