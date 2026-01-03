#version 450

// Skybox Vertex Shader
// 
// This shader renders a skybox cube that always appears at infinite distance.
// The skybox uses reversed depth to ensure it's always rendered behind all other
// geometry, even when the camera is inside the skybox.
//
// Key techniques:
// - No model matrix: skybox is always centered on camera
// - Remove translation from view matrix to keep skybox at camera origin
// - Set depth to 1.0 (farthest) in reversed depth mode

// Vertex input: position of the skybox cube vertices
layout(location = 0) in vec3 position;

// Output: direction vector for sampling the cubemap
layout(location = 0) out vec3 v_direction;

// View and projection matrices
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

void main() {
    // Remove translation from view matrix to keep skybox centered on camera
    // We take the 3x3 upper-left rotation/scale part and construct a 4x4 matrix
    mat4 view_no_translation = mat4(mat3(view_proj.view));
    
    // Transform position to clip space
    vec4 clip_pos = view_proj.proj * view_no_translation * vec4(position, 1.0);
    
    // Set depth to 1.0 (farthest) by setting z = w
    // After perspective division (z/w), this gives depth = 1.0
    // With reversed depth, this ensures the skybox is always behind everything
    gl_Position = clip_pos.xyww;
    
    // Use the local position as the direction vector for cubemap sampling
    // The direction points from the camera origin toward the skybox surface
    v_direction = position;
}
