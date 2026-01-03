#version 450

// SSAO fragment shader
// Samples the hemisphere around each pixel using depth and normal information

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out float outOcclusion;

layout(set = 0, binding = 0) uniform sampler2D gBufferNormal;
layout(set = 0, binding = 1) uniform sampler2D gBufferDepth;
layout(set = 0, binding = 2) uniform sampler2D noiseTexture;

layout(set = 0, binding = 3) uniform SsaoParams {
    mat4 projection;
    mat4 view;
    vec4 samples[64]; // xyz = sample position, w = unused
    vec2 noiseScale;
    float radius;
    float bias;
    float power;
    int kernelSize;
} ssao;

// Reconstruct view-space position from depth
vec3 reconstructViewPosition(vec2 texCoord, float depth) {
    // Convert depth to NDC
    vec4 clipSpacePos = vec4(texCoord * 2.0 - 1.0, depth, 1.0);
    
    // Transform to view space
    vec4 viewSpacePos = inverse(ssao.projection) * clipSpacePos;
    viewSpacePos /= viewSpacePos.w;
    
    return viewSpacePos.xyz;
}

void main() {
    // Sample G-buffer
    vec3 normal = texture(gBufferNormal, fragTexCoord).xyz;
    
    // Check if this is a valid fragment (non-zero normal)
    if (length(normal) < 0.01) {
        outOcclusion = 1.0;
        return;
    }
    
    // Normalize the normal (it's already in view space from G-buffer)
    normal = normalize(normal);
    
    float depth = texture(gBufferDepth, fragTexCoord).r;
    vec3 fragPos = reconstructViewPosition(fragTexCoord, depth);
    
    // Sample noise texture for rotation
    vec3 randomVec = texture(noiseTexture, fragTexCoord * ssao.noiseScale).xyz;
    
    // Create TBN matrix to orient sample kernel
    vec3 tangent = normalize(randomVec - normal * dot(randomVec, normal));
    vec3 bitangent = cross(normal, tangent);
    mat3 TBN = mat3(tangent, bitangent, normal);
    
    // Sample hemisphere and accumulate occlusion
    float occlusion = 0.0;
    for (int i = 0; i < ssao.kernelSize; ++i) {
        // Get sample position in view space
        vec3 samplePos = TBN * ssao.samples[i].xyz;
        samplePos = fragPos + samplePos * ssao.radius;
        
        // Project sample position to screen space
        vec4 offset = ssao.projection * vec4(samplePos, 1.0);
        offset.xyz /= offset.w;
        offset.xy = offset.xy * 0.5 + 0.5;
        
        // Get depth at sample position
        float sampleDepth = texture(gBufferDepth, offset.xy).r;
        vec3 sampleViewPos = reconstructViewPosition(offset.xy, sampleDepth);
        
        // Range check
        float rangeCheck = smoothstep(0.0, 1.0, ssao.radius / abs(fragPos.z - sampleViewPos.z));
        
        // Accumulate occlusion if sample is closer than surface
        occlusion += (sampleViewPos.z >= samplePos.z + ssao.bias ? 1.0 : 0.0) * rangeCheck;
    }
    
    // Normalize and invert (1.0 = no occlusion, 0.0 = full occlusion)
    occlusion = 1.0 - (occlusion / float(ssao.kernelSize));
    
    // Apply power for artistic control
    outOcclusion = pow(occlusion, ssao.power);
}
