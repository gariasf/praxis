#version 450

// ============================================================================
// Shadow Map Fragment Shader
// ============================================================================
//
// This shader is used during the shadow pass. It's extremely simple because
// we only need to write depth values to the shadow map. The GPU automatically
// writes the fragment's depth to the depth buffer, so this shader is
// essentially empty.
//
// In Vulkan/GLSL, if a fragment shader has no outputs and the render pass
// only has a depth attachment, the GPU will still write depth values.

void main() {
    // Nothing to do - depth is written automatically by the GPU
    // The fragment's depth (gl_FragCoord.z) is written to the depth buffer
}
