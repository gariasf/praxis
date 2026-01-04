#version 450

// Advanced material fragment shader with:
// - Parallax occlusion mapping
// - Extended PBR (clearcoat, sheen, transmission)
// - Material layers blending

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_color;
layout(location = 3) in vec2 v_uv;
layout(location = 4) in vec3 v_tangent;
layout(location = 5) in vec3 v_bitangent;

layout(location = 0) out vec4 f_color;

// View projection uniforms
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

// Textures
layout(set = 0, binding = 2) uniform sampler2D albedo_texture;
layout(set = 0, binding = 9) uniform sampler2D normal_map;
layout(set = 0, binding = 11) uniform sampler2D metallic_roughness_map;
layout(set = 0, binding = 12) uniform sampler2D height_map;
layout(set = 0, binding = 13) uniform sampler2D ao_map;
layout(set = 0, binding = 14) uniform sampler2D emissive_map;

// Lighting data
struct DirectionalLight {
    vec4 direction;
    vec4 color;
    float intensity;
    float _padding[3];
};

struct PointLight {
    vec4 position;
    vec4 color;
    float intensity;
    float range;
    float _padding[2];
};

layout(set = 0, binding = 3, std140) uniform LightingData {
    DirectionalLight directional_lights[8];
    PointLight point_lights[16];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
} lighting;

// Base material properties
layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;
    float metallic;
    float roughness;
    float emissive_strength;
    float _padding;
} material;

// Extended PBR properties
layout(set = 1, binding = 1, std140) uniform ExtendedPbrProperties {
    float clearcoat;
    float clearcoat_roughness;
    float sheen;
    float sheen_tint;
    float transmission;
    float ior;
    float anisotropy;
    float anisotropy_rotation;
} extended;

// Parallax properties
layout(set = 1, binding = 2, std140) uniform ParallaxProperties {
    float height_scale;
    uint min_samples;
    uint max_samples;
    uint enabled;
} parallax;

const float PI = 3.14159265359;
const float MIN_SHININESS = 2.0;
const float MAX_SHININESS = 256.0;

// Parallax occlusion mapping
vec2 parallax_occlusion_mapping(vec2 tex_coords, vec3 view_dir_tangent) {
    if (parallax.enabled == 0u) {
        return tex_coords;
    }

    // Calculate number of layers based on view angle
    float num_layers = mix(float(parallax.max_samples), float(parallax.min_samples), 
                           abs(dot(vec3(0.0, 0.0, 1.0), view_dir_tangent)));
    
    float layer_depth = 1.0 / num_layers;
    float current_layer_depth = 0.0;
    
    vec2 delta_tex_coords = view_dir_tangent.xy * parallax.height_scale / num_layers;
    
    vec2 current_tex_coords = tex_coords;
    float current_depth_map_value = texture(height_map, current_tex_coords).r;
    
    // Steep parallax mapping with occlusion
    while (current_layer_depth < current_depth_map_value) {
        current_tex_coords -= delta_tex_coords;
        current_depth_map_value = texture(height_map, current_tex_coords).r;
        current_layer_depth += layer_depth;
    }
    
    // Parallax occlusion mapping interpolation
    vec2 prev_tex_coords = current_tex_coords + delta_tex_coords;
    float after_depth = current_depth_map_value - current_layer_depth;
    float before_depth = texture(height_map, prev_tex_coords).r - (current_layer_depth - layer_depth);
    
    float weight = after_depth / (after_depth - before_depth);
    vec2 final_tex_coords = mix(current_tex_coords, prev_tex_coords, weight);
    
    return final_tex_coords;
}

// Fresnel-Schlick approximation
vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}

// GGX/Trowbridge-Reitz normal distribution function
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    
    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return num / denom;
}

// Smith's geometry function for GGX
float geometry_schlick_ggx(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    
    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    
    return num / denom;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_ggx(NdotV, roughness);
    float ggx1 = geometry_schlick_ggx(NdotL, roughness);
    
    return ggx1 * ggx2;
}

// Cook-Torrance BRDF
vec3 cook_torrance_brdf(vec3 N, vec3 V, vec3 L, vec3 albedo, float metallic, float roughness) {
    vec3 H = normalize(V + L);
    
    // Base reflectivity (F0) - lerp between dielectric (0.04) and albedo for metals
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);
    
    // Cook-Torrance components
    float NDF = distribution_ggx(N, H, roughness);
    float G = geometry_smith(N, V, L, roughness);
    vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    
    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    vec3 specular = numerator / denominator;
    
    // Energy conservation: diffuse and specular
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;
    
    float NdotL = max(dot(N, L), 0.0);
    return (kD * albedo / PI + specular) * NdotL;
}

// Clearcoat layer (secondary specular)
vec3 clearcoat_brdf(vec3 N, vec3 V, vec3 L, float clearcoat, float clearcoat_roughness) {
    if (clearcoat <= 0.0) {
        return vec3(0.0);
    }
    
    vec3 H = normalize(V + L);
    
    // Clearcoat uses fixed F0 for a clear coating (like car paint)
    vec3 F0 = vec3(0.04);
    
    float NDF = distribution_ggx(N, H, clearcoat_roughness);
    float G = geometry_smith(N, V, L, clearcoat_roughness);
    vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    
    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    
    return (numerator / denominator) * clearcoat;
}

// Sheen (fabric-like reflectance at grazing angles)
vec3 sheen_brdf(vec3 N, vec3 V, vec3 L, vec3 albedo, float sheen, float sheen_tint) {
    if (sheen <= 0.0) {
        return vec3(0.0);
    }
    
    vec3 H = normalize(V + L);
    float VdotH = max(dot(V, H), 0.0);
    
    // Sheen color: lerp between white and albedo based on tint
    vec3 sheen_color = mix(vec3(1.0), albedo, sheen_tint);
    
    // Simplified sheen model (Charlie sheen)
    float sheen_distribution = pow(1.0 - VdotH, 5.0);
    
    return sheen_color * sheen * sheen_distribution;
}

// Simple transmission approximation
vec3 transmission_color(vec3 view_dir, vec3 albedo, float transmission, float ior) {
    if (transmission <= 0.0) {
        return vec3(0.0);
    }
    
    // Simple transmission: allow some light through based on transmission factor
    // In a full implementation, this would refract through the surface
    return albedo * transmission * 0.5;
}

void main() {
    // Build TBN matrix for tangent space
    vec3 T = normalize(v_tangent);
    vec3 B = normalize(v_bitangent);
    vec3 N = normalize(v_normal);
    mat3 TBN = mat3(T, B, N);
    
    // Calculate view direction in tangent space for parallax
    vec3 view_dir = normalize(view_proj.camera_position - v_world_pos);
    vec3 view_dir_tangent = normalize(transpose(TBN) * view_dir);
    
    // Apply parallax occlusion mapping
    vec2 tex_coords = parallax_occlusion_mapping(v_uv, view_dir_tangent);
    
    // Discard fragments outside texture bounds (from steep parallax)
    if (tex_coords.x < 0.0 || tex_coords.x > 1.0 || 
        tex_coords.y < 0.0 || tex_coords.y > 1.0) {
        discard;
    }
    
    // Sample textures
    vec4 tex_color = texture(albedo_texture, tex_coords);
    vec3 tangent_normal = texture(normal_map, tex_coords).rgb * 2.0 - 1.0;
    vec2 metallic_roughness = texture(metallic_roughness_map, tex_coords).rg;
    float ao = texture(ao_map, tex_coords).r;
    vec3 emissive = texture(emissive_map, tex_coords).rgb;
    
    // Combine with material properties
    vec3 albedo = v_color * tex_color.rgb * material.base_color.rgb;
    float alpha = tex_color.a * material.base_color.a;
    float metallic = metallic_roughness.r * material.metallic;
    float roughness = metallic_roughness.g * material.roughness;
    
    // Transform normal to world space
    vec3 normal = normalize(TBN * tangent_normal);
    
    // Lighting accumulation
    vec3 Lo = vec3(0.0);
    
    // Directional lights
    for (uint i = 0; i < lighting.directional_light_count; i++) {
        DirectionalLight light = lighting.directional_lights[i];
        vec3 light_dir = -light.direction.xyz;
        
        // Base PBR
        vec3 radiance = light.color.rgb * light.intensity;
        Lo += cook_torrance_brdf(normal, view_dir, light_dir, albedo, metallic, roughness) * radiance;
        
        // Clearcoat
        Lo += clearcoat_brdf(normal, view_dir, light_dir, extended.clearcoat, extended.clearcoat_roughness) * radiance;
        
        // Sheen
        Lo += sheen_brdf(normal, view_dir, light_dir, albedo, extended.sheen, extended.sheen_tint) * radiance;
    }
    
    // Point lights
    for (uint i = 0; i < lighting.point_light_count; i++) {
        PointLight light = lighting.point_lights[i];
        vec3 light_vec = light.position.xyz - v_world_pos;
        float distance = length(light_vec);
        vec3 light_dir = light_vec / distance;
        
        float attenuation = 1.0 / (1.0 + distance * distance);
        attenuation *= max(1.0 - (distance / light.range), 0.0);
        
        vec3 radiance = light.color.rgb * light.intensity * attenuation;
        
        // Base PBR
        Lo += cook_torrance_brdf(normal, view_dir, light_dir, albedo, metallic, roughness) * radiance;
        
        // Clearcoat
        Lo += clearcoat_brdf(normal, view_dir, light_dir, extended.clearcoat, extended.clearcoat_roughness) * radiance;
        
        // Sheen
        Lo += sheen_brdf(normal, view_dir, light_dir, albedo, extended.sheen, extended.sheen_tint) * radiance;
    }
    
    // Ambient lighting with AO
    vec3 ambient = lighting.ambient_color.rgb * albedo * ao;
    
    // Transmission
    vec3 transmitted = transmission_color(view_dir, albedo, extended.transmission, extended.ior);
    
    // Final color
    vec3 color = ambient + Lo + transmitted;
    
    // Emissive
    color += emissive * material.emissive_strength;
    
    f_color = vec4(color, alpha);
}
