#version 450

// Vertex shader for equirectangular to cubemap conversion

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 v_local_pos;

layout(set = 0, binding = 1) uniform CaptureMatrices {
    mat4 view;
    mat4 projection;
} u_capture;

void main() {
    v_local_pos = position;
    gl_Position = u_capture.projection * u_capture.view * vec4(position, 1.0);
}
