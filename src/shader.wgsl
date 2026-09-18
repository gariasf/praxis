@group(0) @binding(0)
var<uniform> camera: CameraUniform;
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position:  vec4<f32>,
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
@group(1) @binding(3) var mr_array: texture_2d_array<f32>;
@group(1) @binding(4) var ao_array: texture_2d_array<f32>;
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

struct InstanceData {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
    material_id: u32,
}

@group(3) @binding(0)
var<storage, read> instances: array<InstanceData>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material_id: u32,
}

@vertex
fn vs_main(
in: VertexInput, @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    var out: VertexOutput;
    let instance = instances[instance_idx];
    out.clip_position = camera.view_proj * instance.model * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.normal = (instance.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    out.world_pos = (instance.model * vec4<f32>(in.position, 1.0)).xyz;
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
    let metallic = material.metallic_roughness.x;
    // GGX collapses to 0/0 at roughness 0; clamp keeps the highlight finite.
    let roughness = max(material.metallic_roughness.y, 0.04);

    let normal = normalize(in.normal);
    let view_direction = normalize(camera.position.xyz - in.world_pos);

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
        let light_vector = light.point_positions[light_index].xyz - in.world_pos;
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

    // Constant ambient stand-in until Step 6 replaces it with IBL.
    let ambient = 0.03 * base_color;
    return vec4<f32>(ambient + outgoing_radiance, albedo_sample.a * material.base_color.a);
}
