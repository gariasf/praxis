#version 450

// SSR fragment shader with hierarchical ray marching
// Traces reflection rays through screen-space depth buffer

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out vec4 outReflection; // RGB = reflection color, A = confidence

layout(set = 0, binding = 0) uniform sampler2D gBufferNormal;
layout(set = 0, binding = 1) uniform sampler2D gBufferDepth;
layout(set = 0, binding = 2) uniform sampler2D gBufferMetallicRoughness;
layout(set = 0, binding = 3) uniform sampler2D sceneColor;

layout(set = 0, binding = 4) uniform SsrParams {
    mat4 projection;
    mat4 view;
    mat4 invProjection;
    mat4 invView;
    vec3 cameraPosition;
    uint maxSteps;
    uint maxBinarySearchSteps;
    float stepSize;
    float thickness;
    float maxRoughness;
    float minHitConfidence;
    float edgeFadeFactor;
} ssr;

// Reconstruct view-space position from depth
vec3 reconstructViewPosition(vec2 texCoord, float depth) {
    vec4 clipSpacePos = vec4(texCoord * 2.0 - 1.0, depth, 1.0);
    vec4 viewSpacePos = ssr.invProjection * clipSpacePos;
    return viewSpacePos.xyz / viewSpacePos.w;
}

// Reconstruct world-space position from depth
vec3 reconstructWorldPosition(vec2 texCoord, float depth) {
    vec3 viewPos = reconstructViewPosition(texCoord, depth);
    vec4 worldPos = ssr.invView * vec4(viewPos, 1.0);
    return worldPos.xyz;
}

// Project view-space position to screen space
vec3 projectToScreenSpace(vec3 viewPos) {
    vec4 clipPos = ssr.projection * vec4(viewPos, 1.0);
    clipPos.xyz /= clipPos.w;
    vec3 screenPos;
    screenPos.xy = clipPos.xy * 0.5 + 0.5;
    screenPos.z = clipPos.z;
    return screenPos;
}

// Calculate screen-space edge fade
float calculateEdgeFade(vec2 screenPos) {
    vec2 fade = max(vec2(0.0), abs(screenPos - 0.5) * 2.0 - (1.0 - ssr.edgeFadeFactor));
    return 1.0 - dot(fade, fade);
}

// Hierarchical ray marching with binary search refinement
bool traceScreenSpaceRay(
    vec3 rayOrigin,
    vec3 rayDirection,
    out vec2 hitUV,
    out float hitConfidence
) {
    vec3 rayPos = rayOrigin;
    float rayStep = ssr.stepSize;
    
    // Adaptive step size based on distance
    for (uint i = 0; i < ssr.maxSteps; ++i) {
        rayPos += rayDirection * rayStep;
        
        // Project to screen space
        vec3 screenPos = projectToScreenSpace(rayPos);
        
        // Check if ray is off screen
        if (screenPos.x < 0.0 || screenPos.x > 1.0 ||
            screenPos.y < 0.0 || screenPos.y > 1.0 ||
            screenPos.z < 0.0 || screenPos.z > 1.0) {
            return false;
        }
        
        // Sample depth at current screen position
        float sampledDepth = texture(gBufferDepth, screenPos.xy).r;
        vec3 sampledViewPos = reconstructViewPosition(screenPos.xy, sampledDepth);
        
        // Check for intersection
        float depthDiff = rayPos.z - sampledViewPos.z;
        
        if (depthDiff > 0.0 && depthDiff < ssr.thickness) {
            // Binary search refinement
            vec3 refinedRayPos = rayPos;
            vec3 refinementStep = rayDirection * rayStep * 0.5;
            
            for (uint j = 0; j < ssr.maxBinarySearchSteps; ++j) {
                vec3 testScreenPos = projectToScreenSpace(refinedRayPos);
                float testDepth = texture(gBufferDepth, testScreenPos.xy).r;
                vec3 testViewPos = reconstructViewPosition(testScreenPos.xy, testDepth);
                
                if (refinedRayPos.z > testViewPos.z) {
                    refinedRayPos -= refinementStep;
                } else {
                    refinedRayPos += refinementStep;
                }
                
                refinementStep *= 0.5;
            }
            
            // Final hit position
            vec3 finalScreenPos = projectToScreenSpace(refinedRayPos);
            hitUV = finalScreenPos.xy;
            
            // Calculate confidence based on edge proximity and depth difference
            float edgeFade = calculateEdgeFade(hitUV);
            float depthFade = 1.0 - clamp(abs(depthDiff) / ssr.thickness, 0.0, 1.0);
            hitConfidence = edgeFade * depthFade;
            
            return hitConfidence >= ssr.minHitConfidence;
        }
        
        // Hierarchical step size adjustment
        // Increase step size when far from intersections
        rayStep *= 1.05;
    }
    
    return false;
}

void main() {
    // Sample G-buffer
    vec3 normal = texture(gBufferNormal, fragTexCoord).xyz;
    float depth = texture(gBufferDepth, fragTexCoord).r;
    vec3 metallicRoughness = texture(gBufferMetallicRoughness, fragTexCoord).rgb;
    float roughness = metallicRoughness.g;
    float metallic = metallicRoughness.r;
    
    // Check if this pixel should have reflections
    if (length(normal) < 0.01 || roughness > ssr.maxRoughness || metallic < 0.1) {
        outReflection = vec4(0.0, 0.0, 0.0, 0.0);
        return;
    }
    
    normal = normalize(normal);
    
    // Reconstruct view-space position
    vec3 viewPos = reconstructViewPosition(fragTexCoord, depth);
    vec3 worldPos = reconstructWorldPosition(fragTexCoord, depth);
    
    // Calculate view direction in view space
    vec3 viewDir = normalize(viewPos);
    
    // Calculate reflection direction in view space
    vec3 reflectionDir = reflect(viewDir, (ssr.view * vec4(normal, 0.0)).xyz);
    
    // Trace ray through screen space
    vec2 hitUV;
    float hitConfidence;
    bool hit = traceScreenSpaceRay(viewPos, reflectionDir, hitUV, hitConfidence);
    
    if (hit) {
        // Sample scene color at hit position
        vec3 reflectionColor = texture(sceneColor, hitUV).rgb;
        
        // Apply Fresnel effect (Schlick's approximation)
        float cosTheta = max(dot(-viewDir, normal), 0.0);
        float fresnel = metallic + (1.0 - metallic) * pow(1.0 - cosTheta, 5.0);
        
        // Output reflection with confidence
        outReflection = vec4(reflectionColor * fresnel, hitConfidence);
    } else {
        // No hit - mark for environment probe fallback
        outReflection = vec4(0.0, 0.0, 0.0, 0.0);
    }
}
