#version 450

// Motion blur fragment shader using velocity buffer and sample accumulation
// Creates realistic motion blur based on per-pixel velocity

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;
layout(set = 0, binding = 1) uniform sampler2D velocity_texture;

layout(set = 0, binding = 2) uniform MotionBlurParams {
    float intensity;           // Blur intensity multiplier
    int sample_count;          // Number of samples along motion vector
    float shutter_angle;       // Simulated camera shutter angle (0-360 degrees)
    float max_blur_radius;     // Maximum blur radius in pixels
} params;

layout(location = 0) out vec4 out_color;

const int MAX_SAMPLES = 32;

void main() {
    // Sample velocity at current pixel (stored as screen-space motion vector)
    vec2 velocity = texture(velocity_texture, in_uv).rg;
    
    // Scale velocity by intensity and shutter angle
    float shutter_scale = params.shutter_angle / 360.0;
    vec2 scaled_velocity = velocity * params.intensity * shutter_scale;
    
    // Clamp velocity to max blur radius
    float velocity_length = length(scaled_velocity);
    if (velocity_length > params.max_blur_radius) {
        scaled_velocity = normalize(scaled_velocity) * params.max_blur_radius;
    }
    
    // If velocity is too small, skip blur
    if (length(scaled_velocity) < 0.5) {
        out_color = texture(input_texture, in_uv);
        return;
    }
    
    // Accumulate samples along motion vector
    vec4 color_sum = vec4(0.0);
    int actual_samples = min(params.sample_count, MAX_SAMPLES);
    
    // Sample along motion vector from current to previous position
    for (int i = 0; i < actual_samples; i++) {
        // Distribute samples evenly along motion vector
        float t = float(i) / float(actual_samples - 1);
        t = t * 2.0 - 1.0; // Range from -1 to 1
        
        vec2 sample_uv = in_uv + scaled_velocity * t * 0.5;
        
        // Clamp to valid texture coordinates
        sample_uv = clamp(sample_uv, vec2(0.0), vec2(1.0));
        
        vec4 sample_color = texture(input_texture, sample_uv);
        color_sum += sample_color;
    }
    
    // Average all samples
    out_color = color_sum / float(actual_samples);
}
