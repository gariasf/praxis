#version 450

// Vertex shader for full-screen quad post-processing
// This shader simply passes through clip-space positions and UV coordinates

layout(location = 0) in vec2 position;  // Clip space position [-1, 1]
layout(location = 1) in vec2 uv;        // UV coordinates [0, 1]

layout(location = 0) out vec2 out_uv;   // Output UV to fragment shader

void main() {
    // Pass position directly to clip space (no transformation needed)
    gl_Position = vec4(position, 0.0, 1.0);
    
    // Pass UV coordinates to fragment shader
    out_uv = uv;
}
