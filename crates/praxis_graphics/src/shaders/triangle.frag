#version 450

// Fragment Shader for 3D Textured Rendering with Blinn-Phong Lighting
//
// This shader implements Blinn-Phong lighting model with support for:
// - Directional lights (sun-like, infinite distance)
// - Point lights (omnidirectional, distance-based attenuation)
// - Ambient lighting (global base illumination)
// - Texture sampling and color modulation
//
// Input (from vertex shader):
//   location 0: vec3 v_world_pos - vertex position in world space
//   location 1: vec3 v_normal    - normal in world space
//   location 2: vec3 v_color     - interpolated vertex color (RGB)
//   location 3: vec2 v_uv        - interpolated UV coordinates
//
// Texture Sampler (set 0, binding 1):
//   sampler2D albedo_texture - the base color texture
//
// Output:
//   location 0: vec4 f_color - final pixel color (RGBA)
//
// Lighting Model (Blinn-Phong):
//   final_color = (ambient + diffuse + specular) * albedo
//   - Ambient: constant base lighting
//   - Diffuse: Lambert (N·L) term
//   - Specular: Blinn-Phong (N·H)^shininess term

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_color;
layout(location = 3) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D albedo_texture;

// Lighting constants
// TODO: These should be passed via uniform buffers from the ECS DirectionalLight
// and PointLight components. For now, they are hardcoded for demonstration.
const vec3 AMBIENT_LIGHT = vec3(0.1, 0.1, 0.1);
const float SHININESS = 32.0;
const vec3 CAMERA_POS = vec3(0.0, 5.0, 10.0); // Temporary fixed camera position

// Directional light (simulating sun)
const vec3 DIR_LIGHT_DIRECTION = normalize(vec3(0.5, -1.0, 0.3));
const vec3 DIR_LIGHT_COLOR = vec3(1.0, 0.95, 0.8);
const float DIR_LIGHT_INTENSITY = 1.0;

// Point light
const vec3 POINT_LIGHT_POS = vec3(0.0, 5.0, 0.0);
const vec3 POINT_LIGHT_COLOR = vec3(1.0, 0.9, 0.7);
const float POINT_LIGHT_INTENSITY = 10.0;
const float POINT_LIGHT_RANGE = 20.0;

// Calculate diffuse lighting using Lambert's cosine law
float calculate_diffuse(vec3 normal, vec3 light_dir) {
    return max(dot(normal, light_dir), 0.0);
}

// Calculate specular lighting using Blinn-Phong model
float calculate_specular(vec3 normal, vec3 light_dir, vec3 view_dir) {
    vec3 halfway_dir = normalize(light_dir + view_dir);
    return pow(max(dot(normal, halfway_dir), 0.0), SHININESS);
}

// Calculate attenuation for point lights based on distance
float calculate_attenuation(float distance, float range) {
    // Inverse square falloff with smooth cutoff at range
    float attenuation = 1.0 / (1.0 + distance * distance);
    float range_factor = max(1.0 - (distance / range), 0.0);
    return attenuation * range_factor;
}

void main() {
    // Sample the texture at the interpolated UV coordinates
    vec4 tex_color = texture(albedo_texture, v_uv);
    
    // Base albedo color (vertex color * texture color)
    vec3 albedo = v_color * tex_color.rgb;
    
    // Normalize the interpolated normal
    vec3 normal = normalize(v_normal);
    
    // Calculate view direction (from fragment to camera)
    vec3 view_dir = normalize(CAMERA_POS - v_world_pos);
    
    // Initialize lighting accumulator with ambient light
    vec3 lighting = AMBIENT_LIGHT;
    
    // === Directional Light ===
    {
        vec3 light_dir = -DIR_LIGHT_DIRECTION; // Light direction points toward source
        
        // Diffuse component
        float diffuse = calculate_diffuse(normal, light_dir);
        
        // Specular component
        float specular = calculate_specular(normal, light_dir, view_dir);
        
        // Combine diffuse and specular with light properties
        vec3 dir_light_contrib = DIR_LIGHT_COLOR * DIR_LIGHT_INTENSITY * (diffuse + specular * 0.5);
        lighting += dir_light_contrib;
    }
    
    // === Point Light ===
    {
        vec3 light_vec = POINT_LIGHT_POS - v_world_pos;
        float distance = length(light_vec);
        vec3 light_dir = light_vec / distance; // Normalize
        
        // Calculate attenuation
        float attenuation = calculate_attenuation(distance, POINT_LIGHT_RANGE);
        
        // Diffuse component
        float diffuse = calculate_diffuse(normal, light_dir);
        
        // Specular component
        float specular = calculate_specular(normal, light_dir, view_dir);
        
        // Combine with attenuation and light properties
        vec3 point_light_contrib = POINT_LIGHT_COLOR * POINT_LIGHT_INTENSITY * 
                                   (diffuse + specular * 0.3) * attenuation;
        lighting += point_light_contrib;
    }
    
    // Apply lighting to albedo
    vec3 final_color = lighting * albedo;
    
    // Output final color with alpha from texture
    f_color = vec4(final_color, tex_color.a);
}
