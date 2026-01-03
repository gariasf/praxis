#version 450

// Fragment shader for deferred rendering lighting pass
// Reads G-buffer and accumulates lighting from all lights

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

// G-buffer inputs
layout(set = 0, binding = 0) uniform sampler2D gbuffer_albedo;
layout(set = 0, binding = 1) uniform sampler2D gbuffer_normal;
layout(set = 0, binding = 2) uniform sampler2D gbuffer_metallic_roughness;
layout(set = 0, binding = 3) uniform sampler2D gbuffer_depth;

// View/projection uniforms
layout(set = 0, binding = 4, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

// Lighting data structures
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

layout(set = 0, binding = 5, std140) uniform LightingData {
    DirectionalLight directional_lights[8];
    PointLight point_lights[16];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
} lighting;

// Lighting constants
const float MIN_SHININESS = 2.0;
const float MAX_SHININESS = 256.0;

// Calculate diffuse lighting using Lambert's cosine law
float calculate_diffuse(vec3 normal, vec3 light_dir) {
    return max(dot(normal, light_dir), 0.0);
}

// Calculate specular lighting using Blinn-Phong model
float calculate_specular(vec3 normal, vec3 light_dir, vec3 view_dir, float shininess) {
    vec3 halfway_dir = normalize(light_dir + view_dir);
    return pow(max(dot(normal, halfway_dir), 0.0), shininess);
}

// Calculate attenuation for point lights
float calculate_attenuation(float distance, float range) {
    float attenuation = 1.0 / (1.0 + distance * distance);
    float range_factor = max(1.0 - (distance / range), 0.0);
    return attenuation * range_factor;
}

// Reconstruct world position from depth
vec3 reconstruct_world_position(vec2 uv, float depth) {
    // Convert UV and depth to NDC
    vec4 ndc = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
    
    // Transform to view space
    vec4 view_pos = inverse(view_proj.proj) * ndc;
    view_pos /= view_pos.w;
    
    // Transform to world space
    vec4 world_pos = inverse(view_proj.view) * view_pos;
    return world_pos.xyz;
}

void main() {
    // Sample G-buffer
    vec3 albedo = texture(gbuffer_albedo, v_uv).rgb;
    vec3 normal = texture(gbuffer_normal, v_uv).rgb;
    vec4 material_data = texture(gbuffer_metallic_roughness, v_uv);
    float depth = texture(gbuffer_depth, v_uv).r;
    
    // Extract material properties
    float metallic = material_data.r;
    float roughness = material_data.g;
    float emissive_strength = material_data.b;
    
    // Early exit if no geometry (depth = 1.0 = far plane)
    if (depth >= 1.0) {
        f_color = vec4(0.1, 0.2, 0.3, 1.0);
        return;
    }
    
    // Reconstruct world position from depth
    vec3 world_pos = reconstruct_world_position(v_uv, depth);
    
    // Calculate view direction
    vec3 view_dir = normalize(view_proj.camera_position - world_pos);
    
    // Convert roughness to shininess
    float shininess = mix(MAX_SHININESS, MIN_SHININESS, roughness);
    
    // Calculate diffuse and specular factors based on metallic
    float diffuse_factor = 1.0 - metallic;
    vec3 specular_color = mix(vec3(1.0), albedo, metallic);
    
    // Start with ambient lighting
    vec3 lighting_result = lighting.ambient_color.rgb;
    
    // Process directional lights
    for (uint i = 0; i < lighting.directional_light_count; i++) {
        DirectionalLight light = lighting.directional_lights[i];
        vec3 light_dir = -light.direction.xyz;
        
        float diffuse = calculate_diffuse(normal, light_dir) * diffuse_factor;
        float specular = calculate_specular(normal, light_dir, view_dir, shininess);
        
        vec3 diffuse_contrib = light.color.rgb * light.intensity * diffuse;
        vec3 specular_contrib = light.color.rgb * light.intensity * specular * specular_color * 0.5;
        lighting_result += diffuse_contrib + specular_contrib;
    }
    
    // Process point lights
    for (uint i = 0; i < lighting.point_light_count; i++) {
        PointLight light = lighting.point_lights[i];
        
        vec3 light_vec = light.position.xyz - world_pos;
        float distance = length(light_vec);
        vec3 light_dir = light_vec / distance;
        
        float attenuation = calculate_attenuation(distance, light.range);
        
        float diffuse = calculate_diffuse(normal, light_dir) * diffuse_factor;
        float specular = calculate_specular(normal, light_dir, view_dir, shininess);
        
        vec3 diffuse_contrib = light.color.rgb * light.intensity * diffuse * attenuation;
        vec3 specular_contrib = light.color.rgb * light.intensity * specular * specular_color * 0.3 * attenuation;
        lighting_result += diffuse_contrib + specular_contrib;
    }
    
    // Apply lighting to albedo
    vec3 lit_color = lighting_result * albedo;
    
    // Add emissive
    vec3 emissive = albedo * emissive_strength;
    vec3 final_color = lit_color + emissive;
    
    f_color = vec4(final_color, 1.0);
}
