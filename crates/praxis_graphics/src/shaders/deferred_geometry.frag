#version 450

// Fragment shader for deferred rendering geometry pass
// Outputs geometry data to G-buffer (multiple render targets)

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_color;
layout(location = 3) in vec2 v_uv;
layout(location = 4) in vec4 v_current_pos;
layout(location = 5) in vec4 v_previous_pos;

// G-buffer outputs
layout(location = 0) out vec4 out_albedo;           // RGB: albedo, A: unused
layout(location = 1) out vec4 out_normal;           // RGB: normal (world space), A: unused
layout(location = 2) out vec4 out_metallic_roughness; // R: metallic, G: roughness, B: emissive, A: unused
layout(location = 3) out vec2 out_velocity;         // RG: screen-space velocity

layout(set = 0, binding = 4) uniform sampler2D albedo_texture;

layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;
    float metallic;
    float roughness;
    float emissive_strength;
    float _padding;
} material;

void main() {
    // Sample texture and combine with vertex color and material tint
    vec4 tex_color = texture(albedo_texture, v_uv);
    vec3 albedo = v_color * tex_color.rgb * material.base_color.rgb;
    
    // Write to G-buffer
    out_albedo = vec4(albedo, 1.0);
    
    // Normalize and pack normal (world space)
    vec3 normal = normalize(v_normal);
    out_normal = vec4(normal, 1.0);
    
    // Pack material properties
    out_metallic_roughness = vec4(
        material.metallic,
        material.roughness,
        material.emissive_strength,
        1.0
    );
    
    // Calculate screen-space velocity
    // Convert to NDC by dividing by w
    vec2 current_ndc = v_current_pos.xy / v_current_pos.w;
    vec2 previous_ndc = v_previous_pos.xy / v_previous_pos.w;
    
    // Calculate velocity (motion vector)
    out_velocity = current_ndc - previous_ndc;
}
