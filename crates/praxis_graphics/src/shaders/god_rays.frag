#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0, std140) uniform GodRaysData {
    vec4 light_position_screen;
    uint num_samples;
    float density;
    float weight;
    float decay;
    float exposure;
    float threshold;
} god_rays;

layout(set = 0, binding = 1) uniform sampler2D scene_texture;
layout(set = 0, binding = 2) uniform sampler2D occlusion_texture;

void main() {
    vec2 light_screen_pos = god_rays.light_position_screen.xy;
    
    vec2 delta_uv = (v_uv - light_screen_pos);
    delta_uv *= god_rays.density / float(god_rays.num_samples);
    
    vec2 sample_uv = v_uv;
    float illumination_decay = 1.0;
    vec3 accumulated_light = vec3(0.0);
    
    for (uint i = 0u; i < god_rays.num_samples; i++) {
        sample_uv -= delta_uv;
        
        vec3 occlusion = texture(occlusion_texture, sample_uv).rgb;
        float luminance = dot(occlusion, vec3(0.299, 0.587, 0.114));
        
        if (luminance > god_rays.threshold) {
            accumulated_light += occlusion * illumination_decay * god_rays.weight;
        }
        
        illumination_decay *= god_rays.decay;
    }
    
    accumulated_light *= god_rays.exposure;
    
    vec3 scene_color = texture(scene_texture, v_uv).rgb;
    vec3 final_color = scene_color + accumulated_light;
    
    f_color = vec4(final_color, 1.0);
}
