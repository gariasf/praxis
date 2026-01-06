#version 450

// SSR roughness-aware blur fragment shader
// Applies variable-strength blur based on surface roughness

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outBlurred;

layout(set = 0, binding = 0) uniform sampler2D ssrTexture;
layout(set = 0, binding = 1) uniform sampler2D gBufferMetallicRoughness;

layout(push_constant) uniform PushConstants {
    vec2 texelSize;
    vec2 blurDirection;
} pushConstants;

// Gaussian blur kernel weights (5 samples)
const float weights[5] = float[](0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main() {
    vec3 metallicRoughness = texture(gBufferMetallicRoughness, fragTexCoord).rgb;
    float roughness = metallicRoughness.g;
    
    // Sample center
    vec4 result = texture(ssrTexture, fragTexCoord) * weights[0];
    
    // Calculate blur radius based on roughness
    // Rougher surfaces get more blur
    float blurRadius = roughness * 4.0;
    
    // Apply Gaussian blur
    for (int i = 1; i < 5; ++i) {
        vec2 offset = pushConstants.blurDirection * pushConstants.texelSize * float(i) * blurRadius;
        
        result += texture(ssrTexture, fragTexCoord + offset) * weights[i];
        result += texture(ssrTexture, fragTexCoord - offset) * weights[i];
    }
    
    outBlurred = result;
}
