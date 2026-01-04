#version 450

layout(location = 0) in vec2 frag_uv;
layout(location = 1) in vec4 frag_color;
layout(location = 2) in vec4 frag_world_pos;
layout(location = 3) in vec4 frag_screen_pos;

layout(set = 0, binding = 1) uniform sampler2D particle_texture;
layout(set = 0, binding = 2) uniform sampler2D depth_texture;

layout(push_constant) uniform PushConstants {
    float soft_particle_distance;
    float depth_fade_power;
} pc;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 tex_color = texture(particle_texture, frag_uv);
    vec4 particle_color = frag_color * tex_color;
    
    vec2 screen_uv = (frag_screen_pos.xy / frag_screen_pos.w) * 0.5 + 0.5;
    
    float particle_depth = frag_screen_pos.z / frag_screen_pos.w;
    float scene_depth = texture(depth_texture, screen_uv).r;
    
    float depth_diff = scene_depth - particle_depth;
    float soft_factor = smoothstep(0.0, pc.soft_particle_distance, depth_diff);
    soft_factor = pow(soft_factor, pc.depth_fade_power);
    
    particle_color.a *= soft_factor;
    
    float dist_from_center = length(frag_uv - vec2(0.5));
    float edge_fade = smoothstep(0.5, 0.3, dist_from_center);
    particle_color.a *= edge_fade;
    
    if (particle_color.a < 0.01) {
        discard;
    }
    
    out_color = particle_color;
}
