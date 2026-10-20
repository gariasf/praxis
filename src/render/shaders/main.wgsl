// The main pass's bindings and entry points, after the shared parts
// (src/render/shaders/mod.rs).

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0) var<storage, read> materials: array<MaterialData>;
// Each Channel is a binding array indexed by Layer (ADR 0017).
@group(1) @binding(1) var albedo_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(2) var normal_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(3) var metallic_roughness_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(4) var occlusion_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(5) var emissive_textures: binding_array<texture_2d<f32>>;
@group(1) @binding(6) var pbr_sampler: sampler;

@group(2) @binding(0)
var<uniform> light: LightUniform;

@group(3) @binding(0)
var<storage, read> instances: array<InstanceData>;

// The vertex Arena, which the vertex shader pulls from (ADR 0019).
@group(3) @binding(1)
var<storage, read> vertices: array<CookedVertex>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) camera_relative_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material_id: u32,
    @location(4) tangent_world: vec4<f32>, // xyz = world tangent, w = bitangent sign
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    var out: VertexOutput;
    let instance = instances[instance_index];
    // An indexed draw's vertex index counts its base vertex in, so it
    // indexes the whole Arena.
    let vertex = decode_vertex(vertices[vertex_index]);
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
    out.material_id = instance.material_id;
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

    var outgoing_radiance = vec3<f32>(0.0);
    if light.light_counts.y > 0u {
        outgoing_radiance += cook_torrance(
            normal,
            view_direction,
            normalize(-light.direction.xyz),
            base_color,
            metallic,
            roughness,
            light.color.rgb * light.color.a
        );
    }

    for (var light_index = 0u; light_index < light.light_counts.x; light_index++) {
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
