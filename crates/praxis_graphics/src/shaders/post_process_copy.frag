#version 450

// Fragment shader for simple texture copy (passthrough)
// This is the most basic post-processing effect - it just samples and outputs the texture

layout(location = 0) in vec2 in_uv;     // UV coordinates from vertex shader

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(location = 0) out vec4 out_color;

void main() {
    // Simply sample the input texture and output
    out_color = texture(input_texture, in_uv);
}
