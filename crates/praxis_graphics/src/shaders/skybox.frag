#version 450

// Skybox Fragment Shader
//
// This shader samples a cubemap texture using the interpolated direction vector
// from the vertex shader. The cubemap represents the surrounding environment
// (sky, space, etc.) that appears at infinite distance.

// Input: direction vector for sampling the cubemap
layout(location = 0) in vec3 v_direction;

// Output: final color
layout(location = 0) out vec4 f_color;

// Cubemap texture sampler
layout(set = 0, binding = 1) uniform samplerCube skybox_cubemap;

void main() {
    // Sample the cubemap using the direction vector
    // The direction points from the camera toward the skybox surface
    // The samplerCube automatically handles the cube face selection
    vec3 color = texture(skybox_cubemap, v_direction).rgb;
    
    // Output the color with full opacity
    f_color = vec4(color, 1.0);
}
