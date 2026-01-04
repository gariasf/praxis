#version 450

// Depth-of-Field fragment shader with circle of confusion calculation and bokeh blur
// Implements realistic camera lens defocus simulation based on depth

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;
layout(set = 0, binding = 1) uniform sampler2D depth_texture;

layout(set = 0, binding = 2) uniform DofParams {
    float focus_distance;      // Distance to focal plane
    float focus_range;         // Range around focal plane that stays sharp
    float bokeh_radius;        // Maximum blur radius for out-of-focus areas
    float aperture;            // Aperture size (f-number)
} params;

layout(location = 0) out vec4 out_color;

const int SAMPLE_COUNT = 16;

// Poisson disk samples for bokeh blur
const vec2 POISSON_DISK[16] = vec2[](
    vec2(-0.94201624, -0.39906216),
    vec2(0.94558609, -0.76890725),
    vec2(-0.094184101, -0.92938870),
    vec2(0.34495938, 0.29387760),
    vec2(-0.91588581, 0.45771432),
    vec2(-0.81544232, -0.87912464),
    vec2(-0.38277543, 0.27676845),
    vec2(0.97484398, 0.75648379),
    vec2(0.44323325, -0.97511554),
    vec2(0.53742981, -0.47373420),
    vec2(-0.26496911, -0.41893023),
    vec2(0.79197514, 0.19090188),
    vec2(-0.24188840, 0.99706507),
    vec2(-0.81409955, 0.91437590),
    vec2(0.19984126, 0.78641367),
    vec2(0.14383161, -0.14100790)
);

// Calculate circle of confusion based on depth and focus parameters
float calculate_coc(float depth) {
    // Convert depth to linear distance (assuming perspective projection)
    float linear_depth = depth * 100.0; // Scale to world units
    
    // Calculate circle of confusion using thin lens equation
    // CoC = (aperture * focal_length * |distance - focus_distance|) / (distance * (focus_distance - focal_length))
    float focus_diff = abs(linear_depth - params.focus_distance);
    
    // Normalize by focus range for smooth transition
    float coc = smoothstep(0.0, params.focus_range, focus_diff);
    
    return coc * params.bokeh_radius;
}

void main() {
    vec2 texel_size = 1.0 / textureSize(input_texture, 0);
    
    // Sample depth at current pixel
    float center_depth = texture(depth_texture, in_uv).r;
    
    // Calculate circle of confusion for current pixel
    float coc = calculate_coc(center_depth);
    
    // If in focus, return original color
    if (coc < 0.001) {
        out_color = texture(input_texture, in_uv);
        return;
    }
    
    // Accumulate bokeh blur samples
    vec4 color_sum = vec4(0.0);
    float weight_sum = 0.0;
    
    for (int i = 0; i < SAMPLE_COUNT; i++) {
        vec2 offset = POISSON_DISK[i] * coc * texel_size;
        vec2 sample_uv = in_uv + offset;
        
        // Clamp to valid texture coordinates
        if (sample_uv.x < 0.0 || sample_uv.x > 1.0 || sample_uv.y < 0.0 || sample_uv.y > 1.0) {
            continue;
        }
        
        // Sample depth at offset position
        float sample_depth = texture(depth_texture, sample_uv).r;
        float sample_coc = calculate_coc(sample_depth);
        
        // Weight samples based on their CoC (larger CoC = more contribution to blur)
        float weight = 1.0;
        if (sample_coc < coc) {
            // Reduce contribution from sharper areas
            weight = sample_coc / coc;
        }
        
        vec4 sample_color = texture(input_texture, sample_uv);
        color_sum += sample_color * weight;
        weight_sum += weight;
    }
    
    // Normalize accumulated color
    out_color = color_sum / max(weight_sum, 0.001);
}
