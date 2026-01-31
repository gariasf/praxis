#version 450

// Forward rendering fragment shader with PBR lighting
// Implements physically-based rendering with Cook-Torrance BRDF

// Constants
const float PI = 3.14159265359;
const float MIN_ROUGHNESS = 0.04;

// Input from vertex shader
layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_color;
layout(location = 3) in vec2 v_uv;
layout(location = 4) in vec3 v_tangent;
layout(location = 5) in vec3 v_bitangent;

// Output
layout(location = 0) out vec4 f_color;

// Uniforms
layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

layout(set = 0, binding = 2) uniform sampler2D albedo_texture;
layout(set = 0, binding = 9) uniform sampler2D normal_map;

layout(set = 1, binding = 0) uniform MaterialProperties {
    vec4 base_color;
    float metallic;
    float roughness;
    float emissive_strength;
    float _padding;
} material;

// Lighting structures (simplified for forward rendering)
struct DirectionalLight {
    vec4 direction;
    vec4 color;
    float intensity;
    float _padding[3];
};

layout(set = 0, binding = 3) uniform LightingData {
    DirectionalLight directional_lights[8];
    vec4 ambient_color;
    uint directional_light_count;
    uint _padding[3];
} lighting;

// PBR Functions

// Normal Distribution Function (GGX/Trowbridge-Reitz)
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return a2 / max(denom, 0.0001);
}

// Geometry Function (Schlick-GGX)
float geometry_schlick_ggx(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    
    float denom = NdotV * (1.0 - k) + k;
    
    return NdotV / max(denom, 0.0001);
}

// Smith's method for geometry obstruction and shadowing
float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_ggx(NdotV, roughness);
    float ggx1 = geometry_schlick_ggx(NdotL, roughness);
    
    return ggx1 * ggx2;
}

// Fresnel equation (Schlick approximation)
vec3 fresnel_schlick(float cos_theta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

void main() {
    // Sample textures
    vec4 albedo_sample = texture(albedo_texture, v_uv);
    vec3 albedo = albedo_sample.rgb * material.base_color.rgb * v_color;
    float alpha = albedo_sample.a * material.base_color.a;
    
    // Sample and decode normal map
    vec3 tangent_normal = texture(normal_map, v_uv).rgb * 2.0 - 1.0;
    
    // Build TBN matrix for normal mapping
    vec3 T = normalize(v_tangent);
    vec3 B = normalize(v_bitangent);
    vec3 N = normalize(v_normal);
    mat3 TBN = mat3(T, B, N);
    
    // Transform normal from tangent space to world space
    vec3 normal = normalize(TBN * tangent_normal);
    
    // View direction
    vec3 V = normalize(view_proj.camera_position - v_world_pos);
    
    // Material properties
    float metallic = clamp(material.metallic, 0.0, 1.0);
    float roughness = max(material.roughness, MIN_ROUGHNESS);
    
    // Calculate F0 (surface reflection at zero incidence)
    // Dielectrics: ~0.04, Metals: use albedo color
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);
    
    // Reflectance equation
    vec3 Lo = vec3(0.0);
    
    // Process directional lights
    for (uint i = 0; i < lighting.directional_light_count && i < 8; i++) {
        DirectionalLight light = lighting.directional_lights[i];
        
        vec3 L = normalize(-light.direction.xyz);
        vec3 H = normalize(V + L);
        vec3 radiance = light.color.rgb * light.intensity;
        
        // Cook-Torrance BRDF
        float NDF = distribution_ggx(normal, H, roughness);
        float G = geometry_smith(normal, V, L, roughness);
        vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);
        
        // Specular component
        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(normal, V), 0.0) * max(dot(normal, L), 0.0);
        vec3 specular = numerator / max(denominator, 0.001);
        
        // Energy conservation: kD * diffuse + specular = 1.0
        vec3 kS = F; // Specular contribution
        vec3 kD = vec3(1.0) - kS; // Diffuse contribution
        kD *= 1.0 - metallic; // Metals have no diffuse
        
        // Lambert diffuse
        float NdotL = max(dot(normal, L), 0.0);
        
        // Add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }
    
    // Ambient lighting (simplified IBL)
    vec3 ambient = lighting.ambient_color.rgb * albedo;
    
    // Emissive
    vec3 emissive = material.base_color.rgb * material.emissive_strength;
    
    // Final color
    vec3 color = ambient + Lo + emissive;
    
    // HDR tonemapping (Reinhard)
    color = color / (color + vec3(1.0));
    
    // Gamma correction
    color = pow(color, vec3(1.0 / 2.2));
    
    f_color = vec4(color, alpha);
}
