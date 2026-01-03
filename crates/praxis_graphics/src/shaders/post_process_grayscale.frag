#version 450

// Fragment shader for grayscale post-processing effect
// Converts color image to grayscale using luminance formula

layout(location = 0) in vec2 in_uv;     // UV coordinates from vertex shader

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(location = 0) out vec4 out_color;

void main() {
    // Sample the input texture
    vec4 color = texture(input_texture, in_uv);
    
    // Calculate luminance using the standard formula
    // These weights account for human eye sensitivity to different colors
    float luminance = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    
    // Output grayscale color (luminance in all channels, preserve alpha)
    out_color = vec4(vec3(luminance), color.a);
}
