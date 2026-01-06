#version 450

// SSR composite fragment shader
// Blends SSR reflections with environment probe fallback

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outComposite;

layout(set = 0, binding = 0) uniform sampler2D ssrTexture;
layout(set = 0, binding = 1) uniform samplerCube environmentProbe;
layout(set = 0, binding = 2) uniform sampler2D gBufferNormal;
layout(set = 0, binding = 3) uniform sampler2D gBufferMetallicRoughness;

void main() {
    // Sample SSR result
    vec4 ssrSample = texture(ssrTexture, fragTexCoord);
    vec3 ssrColor = ssrSample.rgb;
    float ssrConfidence = ssrSample.a;
    
    // Sample G-buffer
    vec3 normal = normalize(texture(gBufferNormal, fragTexCoord).xyz);
    vec3 metallicRoughness = texture(gBufferMetallicRoughness, fragTexCoord).rgb;
    float roughness = metallicRoughness.g;
    float metallic = metallicRoughness.r;
    
    // Check if pixel has valid reflection data
    if (length(normal) < 0.01 || metallic < 0.1) {
        outComposite = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }
    
    // If SSR has high confidence, use it directly
    if (ssrConfidence > 0.8) {
        outComposite = vec4(ssrColor, 1.0);
        return;
    }
    
    // Otherwise, blend with environment probe
    // Calculate view direction (approximation from screen-space)
    vec3 viewDir = normalize(vec3(
        (fragTexCoord.x - 0.5) * 2.0,
        (fragTexCoord.y - 0.5) * 2.0,
        -1.0
    ));
    
    // Calculate reflection direction
    vec3 reflectionDir = reflect(viewDir, normal);
    
    // Sample environment probe with roughness-based mip level
    float mipLevel = roughness * 4.0; // Assuming 5 mip levels (0-4)
    vec3 envColor = textureLod(environmentProbe, reflectionDir, mipLevel).rgb;
    
    // Blend SSR and environment probe based on confidence
    vec3 finalColor = mix(envColor, ssrColor, ssrConfidence);
    
    // Apply Fresnel effect
    float cosTheta = max(dot(-viewDir, normal), 0.0);
    float fresnel = metallic + (1.0 - metallic) * pow(1.0 - cosTheta, 5.0);
    
    outComposite = vec4(finalColor * fresnel, 1.0);
}
