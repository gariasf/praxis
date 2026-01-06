#version 450

// Fragment shader for Temporal Anti-Aliasing (TAA)
// Implements temporal reprojection with velocity-based sampling and neighborhood clamping

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

// Input textures
layout(set = 0, binding = 0) uniform sampler2D current_frame;
layout(set = 0, binding = 1) uniform sampler2D history_frame;
layout(set = 0, binding = 2) uniform sampler2D velocity_buffer;
layout(set = 0, binding = 3) uniform sampler2D depth_buffer;

layout(set = 0, binding = 4, std140) uniform TaaConfig {
    vec2 jitter_offset;
    float blend_factor;
    float _padding;
} config;

// Convert RGB to YCoCg color space for better color clamping
vec3 rgb_to_ycocg(vec3 rgb) {
    float Y = dot(rgb, vec3(0.25, 0.5, 0.25));
    float Co = dot(rgb, vec3(0.5, 0.0, -0.5));
    float Cg = dot(rgb, vec3(-0.25, 0.5, -0.25));
    return vec3(Y, Co, Cg);
}

vec3 ycocg_to_rgb(vec3 ycocg) {
    float Y = ycocg.x;
    float Co = ycocg.y;
    float Cg = ycocg.z;
    
    float tmp = Y - Cg;
    float r = tmp + Co;
    float g = Y + Cg;
    float b = tmp - Co;
    
    return vec3(r, g, b);
}

// Sample 3x3 neighborhood for min/max and variance
void sample_neighborhood(vec2 uv, out vec3 color_min, out vec3 color_max, out vec3 color_avg) {
    vec2 texel_size = 1.0 / textureSize(current_frame, 0);
    
    // Sample 3x3 neighborhood
    vec3 samples[9];
    int index = 0;
    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            vec2 offset = vec2(x, y) * texel_size;
            samples[index] = texture(current_frame, uv + offset).rgb;
            index++;
        }
    }
    
    // Convert to YCoCg for better color space
    for (int i = 0; i < 9; i++) {
        samples[i] = rgb_to_ycocg(samples[i]);
    }
    
    // Calculate min, max, and average
    color_min = samples[0];
    color_max = samples[0];
    color_avg = vec3(0.0);
    
    for (int i = 0; i < 9; i++) {
        color_min = min(color_min, samples[i]);
        color_max = max(color_max, samples[i]);
        color_avg += samples[i];
    }
    
    color_avg /= 9.0;
}

// Clip history color to neighborhood AABB
vec3 clip_aabb(vec3 aabb_min, vec3 aabb_max, vec3 history_color) {
    vec3 center = 0.5 * (aabb_max + aabb_min);
    vec3 extents = 0.5 * (aabb_max - aabb_min);
    
    // Calculate distance from center to history color
    vec3 offset = history_color - center;
    vec3 unit_offset = offset / max(extents, vec3(0.0001));
    
    float max_component = max(max(abs(unit_offset.x), abs(unit_offset.y)), abs(unit_offset.z));
    
    if (max_component > 1.0) {
        // Clip to AABB
        return center + offset / max_component;
    }
    
    return history_color;
}

void main() {
    // Read velocity for reprojection
    vec2 velocity = texture(velocity_buffer, v_uv).rg;
    
    // Calculate reprojected UV coordinate
    vec2 history_uv = v_uv - velocity;
    
    // Sample current frame
    vec3 current_color = texture(current_frame, v_uv).rgb;
    
    // Check if reprojected coordinate is valid
    bool valid_history = history_uv.x >= 0.0 && history_uv.x <= 1.0 && 
                        history_uv.y >= 0.0 && history_uv.y <= 1.0;
    
    if (!valid_history) {
        // No valid history, use current frame
        f_color = vec4(current_color, 1.0);
        return;
    }
    
    // Sample history frame with bilinear filtering
    vec3 history_color = texture(history_frame, history_uv).rgb;
    
    // Neighborhood clamping for history rejection
    vec3 color_min, color_max, color_avg;
    sample_neighborhood(v_uv, color_min, color_max, color_avg);
    
    // Convert history to YCoCg for clamping
    vec3 history_ycocg = rgb_to_ycocg(history_color);
    
    // Clip history to neighborhood AABB
    vec3 clamped_history = clip_aabb(color_min, color_max, history_ycocg);
    
    // Convert back to RGB
    history_color = ycocg_to_rgb(clamped_history);
    
    // Blend current and history
    // Use adaptive blend factor based on velocity magnitude
    float velocity_length = length(velocity);
    float adaptive_blend = mix(config.blend_factor, 0.5, clamp(velocity_length * 10.0, 0.0, 1.0));
    
    vec3 final_color = mix(history_color, current_color, adaptive_blend);
    
    f_color = vec4(final_color, 1.0);
}
