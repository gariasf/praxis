#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0, std140) uniform VolumetricFogData {
    vec4 fog_color;
    float fog_density;
    float max_distance;
    uint num_steps;
    uint density_function_type;
    float density_param1;
    float density_param2;
    float light_scattering;
    float anisotropy;
    float shadow_influence;
} fog;

layout(set = 0, binding = 1, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

layout(set = 0, binding = 2) uniform sampler2D depth_texture;
layout(set = 0, binding = 3) uniform sampler2D color_texture;

layout(set = 0, binding = 4, std140) uniform LightingData {
    vec4 directional_light_dir;
    vec4 directional_light_color;
    float directional_light_intensity;
} lighting;

float calculate_density(vec3 position) {
    float density = fog.fog_density;
    
    if (fog.density_function_type == 0u) {
        return density;
    } else if (fog.density_function_type == 1u) {
        float dist = length(position - view_proj.camera_position);
        return density * exp(-fog.density_param1 * dist);
    } else if (fog.density_function_type == 2u) {
        float height_factor = exp(-fog.density_param2 * max(position.y - fog.density_param1, 0.0));
        return density * height_factor;
    } else if (fog.density_function_type == 3u) {
        float noise = fract(sin(dot(position * fog.density_param1, vec3(12.9898, 78.233, 45.164))) * 43758.5453);
        return density * (0.8 + 0.4 * noise);
    }
    
    return density;
}

float phase_function(float cos_theta) {
    float g = fog.anisotropy;
    float g2 = g * g;
    float denom = 1.0 + g2 - 2.0 * g * cos_theta;
    return (1.0 - g2) / (4.0 * 3.14159265359 * pow(denom, 1.5));
}

vec3 reconstruct_world_position(vec2 uv, float depth) {
    vec4 clip_pos = vec4(uv * 2.0 - 1.0, depth, 1.0);
    vec4 view_pos = inverse(view_proj.proj) * clip_pos;
    view_pos /= view_pos.w;
    vec4 world_pos = inverse(view_proj.view) * view_pos;
    return world_pos.xyz;
}

void main() {
    float depth = texture(depth_texture, v_uv).r;
    vec3 scene_color = texture(color_texture, v_uv).rgb;
    
    vec3 world_pos = reconstruct_world_position(v_uv, depth);
    vec3 ray_origin = view_proj.camera_position;
    vec3 ray_dir = normalize(world_pos - ray_origin);
    
    float ray_length = min(length(world_pos - ray_origin), fog.max_distance);
    float step_size = ray_length / float(fog.num_steps);
    
    vec3 accumulated_fog = vec3(0.0);
    float transmittance = 1.0;
    
    for (uint i = 0u; i < fog.num_steps; i++) {
        float t = (float(i) + 0.5) * step_size;
        vec3 sample_pos = ray_origin + ray_dir * t;
        
        float density = calculate_density(sample_pos);
        
        vec3 light_dir = -lighting.directional_light_dir.xyz;
        float cos_theta = dot(ray_dir, light_dir);
        float phase = phase_function(cos_theta);
        
        vec3 in_scatter = lighting.directional_light_color.rgb * 
                         lighting.directional_light_intensity * 
                         fog.light_scattering * 
                         phase;
        
        float extinction = density * step_size;
        float sample_transmittance = exp(-extinction);
        
        accumulated_fog += transmittance * (1.0 - sample_transmittance) * 
                          (fog.fog_color.rgb + in_scatter);
        
        transmittance *= sample_transmittance;
        
        if (transmittance < 0.01) break;
    }
    
    vec3 final_color = scene_color * transmittance + accumulated_fog;
    f_color = vec4(final_color, 1.0);
}
