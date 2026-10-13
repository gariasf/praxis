@group(0) @binding(0)
var<uniform> camera: CameraUniform;
// Positions reach the shaders relative to the camera (ADR 0014), so the
// camera sits at the origin and view_proj holds no translation. The camera's
// World position arrives as the anchor of its grid cell plus an offset.
struct CameraUniform {
    view_proj: mat4x4<f32>,
    anchor: vec3<i32>,
    offset: vec3<f32>,
}

// Edge of the anchor grid in metres; ANCHOR_CELL_SIZE in Rust matches it.
const ANCHOR_CELL_SIZE: f32 = 64.0;

// A point's position relative to the camera, from its Instance's anchor and
// offset, the camera's, and its position relative to the Instance's pivot
// (ADR 0014). The anchor difference is exact in integers, so f32 only ever
// holds magnitudes near the camera.
fn camera_relative_position(
    anchor: vec3<i32>,
    offset: vec3<f32>,
    camera_anchor: vec3<i32>,
    camera_offset: vec3<f32>,
    local_position: vec3<f32>
) -> vec3<f32> {
    let anchor_position = vec3<f32>(anchor - camera_anchor) * ANCHOR_CELL_SIZE;
    return anchor_position + (offset - camera_offset) + local_position;
}

struct MaterialData {
    base_color: vec4<f32>,
    metallic_roughness: vec4<f32>,
    emissive: vec4<f32>,
    texture_indices: vec4<u32>,
    extra: vec4<u32>,
};

@group(1) @binding(0) var<storage, read> materials: array<MaterialData>;
@group(1) @binding(1) var albedo_array: texture_2d_array<f32>;
@group(1) @binding(2) var normal_array: texture_2d_array<f32>;
@group(1) @binding(3) var metallic_roughness_array: texture_2d_array<f32>;
@group(1) @binding(4) var occlusion_array: texture_2d_array<f32>;
@group(1) @binding(5) var emissive_array: texture_2d_array<f32>;
@group(1) @binding(6) var pbr_sampler: sampler;

@group(2) @binding(0)
var<uniform> light: LightUniform;
struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,

    // Point light
    point_positions: array<vec4<f32>, 4>,
    point_colors: array<vec4<f32>, 4>,
    num_point_lights: vec4<f32>,
}

// An Instance's pivot as anchor plus offset, so camera motion changes no
// Instance, and its rotation-scale.
struct InstanceData {
    rotation_scale: mat3x3<f32>,
    normal_matrix: mat3x3<f32>,
    anchor: vec3<i32>,
    material_id: u32,
    offset: vec3<f32>,
}

@group(3) @binding(0)
var<storage, read> instances: array<InstanceData>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec4<f32>, // xyz = tangent, w = bitangent sign
}

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
in: VertexInput, @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    var out: VertexOutput;
    let instance = instances[instance_idx];
    let position = camera_relative_position(
        instance.anchor,
        instance.offset,
        camera.anchor,
        camera.offset,
        instance.rotation_scale * in.position
    );
    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    out.uv = in.uv;
    out.normal = instance.normal_matrix * in.normal;
    // Tangents are directions along the surface, so they follow the
    // rotation-scale; only normals need the inverse-transpose.
    out.tangent_world = vec4<f32>(instance.rotation_scale * in.tangent.xyz, in.tangent.w);
    out.camera_relative_position = position;
    out.material_id = instance.material_id;
    return out;
}

const PI: f32 = 3.14159265;

// F term. base_reflectance is the reflectance at normal incidence (textbook F0).
fn fresnel_schlick(cos_incidence_angle: f32, base_reflectance: vec3<f32>) -> vec3<f32> {
    return base_reflectance
        + (vec3<f32>(1.0) - base_reflectance) * pow(1.0 - cos_incidence_angle, 5.0);
}

// D term. Fraction of microfacets whose normal points along the half vector.
fn distribution_ggx(normal_dot_half: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let denominator = normal_dot_half * normal_dot_half * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / (PI * denominator * denominator);
}

// G term. Fraction of microfacets neither masked from the viewer nor shadowed from the light.
fn geometry_smith(normal_dot_view: f32, normal_dot_light: f32, roughness: f32) -> f32 {
    let roughness_plus_one = roughness + 1.0;
    let remapped_roughness = (roughness_plus_one * roughness_plus_one) / 8.0;
    let view_masking = normal_dot_view
        / (normal_dot_view * (1.0 - remapped_roughness) + remapped_roughness);
    let light_shadowing = normal_dot_light
        / (normal_dot_light * (1.0 - remapped_roughness) + remapped_roughness);
    return view_masking * light_shadowing;
}

// Outgoing radiance toward view_direction from one light arriving along light_direction.
fn cook_torrance(
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    base_color: vec3<f32>,
    metallic: f32,
    roughness: f32,
    radiance: vec3<f32>
) -> vec3<f32> {
    let half_vector = normalize(view_direction + light_direction);
    let normal_dot_view = max(dot(normal, view_direction), 0.0);
    let normal_dot_light = max(dot(normal, light_direction), 0.0);
    let normal_dot_half = max(dot(normal, half_vector), 0.0);
    let half_dot_view = max(dot(half_vector, view_direction), 0.0);

    let base_reflectance = mix(vec3<f32>(0.04), base_color, metallic);
    let fresnel = fresnel_schlick(half_dot_view, base_reflectance);
    let normal_distribution = distribution_ggx(normal_dot_half, roughness);
    let geometry_shadowing = geometry_smith(normal_dot_view, normal_dot_light, roughness);
    let specular = (normal_distribution * fresnel * geometry_shadowing)
        / (4.0 * normal_dot_view * normal_dot_light + 0.001);

    // Light that is not reflected refracts in and re-emerges as diffuse; metals absorb it.
    let diffuse_weight = (vec3<f32>(1.0) - fresnel) * (1.0 - metallic);
    let diffuse = diffuse_weight * base_color / PI;
    return (diffuse + specular) * radiance * normal_dot_light;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let material = materials[in.material_id];
    let albedo_sample = textureSample(albedo_array, pbr_sampler, in.uv, i32(material.texture_indices.x));
    let base_color = albedo_sample.rgb * material.base_color.rgb;

    // glTF packs roughness in green and metallic in blue; factors multiply.
    let metallic_roughness_sample = textureSample(metallic_roughness_array, pbr_sampler, in.uv, i32(material.texture_indices.z)).rgb;
    let metallic = metallic_roughness_sample.b * material.metallic_roughness.x;
    // GGX collapses to 0/0 at roughness 0; clamp keeps the highlight finite.
    let roughness = max(metallic_roughness_sample.g * material.metallic_roughness.y, 0.04);

    let ambient_occlusion = textureSample(occlusion_array, pbr_sampler, in.uv, i32(material.texture_indices.w)).r;
    let emissive = textureSample(emissive_array, pbr_sampler, in.uv, i32(material.extra.x)).rgb
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

    // Normal map stores a tangent-space direction in [0, 1]; expand to [-1, 1].
    let normal_sample = textureSample(normal_array, pbr_sampler, in.uv, i32(material.texture_indices.y)).rgb;
    let tangent_space_normal = normal_sample * 2.0 - vec3<f32>(1.0);
    let normal = normalize(tangent_to_world * tangent_space_normal);

    let view_direction = normalize(-in.camera_relative_position);

    var outgoing_radiance = cook_torrance(
        normal,
        view_direction,
        normalize(-light.direction.xyz),
        base_color,
        metallic,
        roughness,
        light.color.rgb * light.color.a
    );

    for (var light_index = 0u; light_index < u32(light.num_point_lights.x); light_index++) {
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

    // Constant ambient stand-in until Step 6 replaces it with IBL; occlusion darkens it.
    let ambient = 0.03 * base_color * ambient_occlusion;
    return vec4<f32>(
        ambient + outgoing_radiance + emissive,
        albedo_sample.a * material.base_color.a
    );
}
