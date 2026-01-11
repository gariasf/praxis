#version 450

// Input from vertex shader
layout(location = 0) in vec2 frag_uv;
layout(location = 1) in vec4 frag_color;
layout(location = 2) in vec3 frag_world_pos;
layout(location = 3) in float frag_view_depth;

// Textures
layout(set = 1, binding = 0) uniform sampler2D particle_texture;
layout(set = 1, binding = 1) uniform sampler2D depth_texture;

// Push constants for soft particles
layout(push_constant) uniform PushConstants {
    float soft_particle_distance;
    float depth_fade_power;
} pc;

// Output
layout(location = 0) out vec4 out_color;

void main() {
    // Sample particle texture
    vec4 tex_color = texture(particle_texture, frag_uv);
    vec4 particle_color = frag_color * tex_color;
    
    // Calculate screen-space UV for depth buffer lookup
    vec2 screen_uv = gl_FragCoord.xy / textureSize(depth_texture, 0);
    
    // Get scene depth
    float scene_depth = texture(depth_texture, screen_uv).r;
    
    // Convert scene depth from [0, 1] to view space
    // Note: This assumes a standard perspective projection
    // For accurate soft particles, we'd need to linearize depth properly
    float particle_depth = gl_FragCoord.z;
    
    // Soft particle fade
    float depth_diff = scene_depth - particle_depth;
    float soft_factor = clamp(depth_diff / pc.soft_particle_distance, 0.0, 1.0);
    soft_factor = pow(soft_factor, pc.depth_fade_power);
    
    particle_color.a *= soft_factor;
    
    // Additional radial fade from center for softer appearance
    float dist_from_center = length(frag_uv - vec2(0.5));
    float edge_fade = smoothstep(0.5, 0.3, dist_from_center);
    particle_color.a *= edge_fade;
    
    // Discard fully transparent fragments
    if (particle_color.a < 0.01) {
        discard;
    }
    
    out_color = particle_color;
}
