#version 450

// Fragment shader for Gaussian blur post-processing effect
// Implements a simple 9-tap box blur (can be optimized with separable blur)

layout(location = 0) in vec2 in_uv;     // UV coordinates from vertex shader

layout(set = 0, binding = 0) uniform sampler2D input_texture;

// Push constants for blur parameters
layout(push_constant) uniform BlurParams {
    vec2 texel_size;    // 1.0 / texture_dimensions
    float blur_radius;  // Blur strength multiplier
} blur_params;

layout(location = 0) out vec4 out_color;

void main() {
    // Calculate the offset for sampling neighboring pixels
    vec2 offset = blur_params.texel_size * blur_params.blur_radius;
    
    // 9-tap box blur kernel
    vec4 color = vec4(0.0);
    
    // Sample 3x3 grid around the current pixel
    color += texture(input_texture, in_uv + vec2(-offset.x, -offset.y));
    color += texture(input_texture, in_uv + vec2(0.0, -offset.y));
    color += texture(input_texture, in_uv + vec2(offset.x, -offset.y));
    
    color += texture(input_texture, in_uv + vec2(-offset.x, 0.0));
    color += texture(input_texture, in_uv + vec2(0.0, 0.0));
    color += texture(input_texture, in_uv + vec2(offset.x, 0.0));
    
    color += texture(input_texture, in_uv + vec2(-offset.x, offset.y));
    color += texture(input_texture, in_uv + vec2(0.0, offset.y));
    color += texture(input_texture, in_uv + vec2(offset.x, offset.y));
    
    // Average the samples
    out_color = color / 9.0;
}
