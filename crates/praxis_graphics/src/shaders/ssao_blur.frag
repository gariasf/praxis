#version 450

// SSAO blur fragment shader
// Blurs the SSAO texture to reduce noise

layout(location = 0) in vec2 fragTexCoord;

layout(location = 0) out float outOcclusion;

layout(set = 0, binding = 0) uniform sampler2D ssaoInput;

layout(push_constant) uniform PushConstants {
    vec2 texelSize;
} pc;

void main() {
    vec2 texelSize = pc.texelSize;
    float result = 0.0;
    
    // 4x4 box blur
    for (int x = -2; x < 2; ++x) {
        for (int y = -2; y < 2; ++y) {
            vec2 offset = vec2(float(x), float(y)) * texelSize;
            result += texture(ssaoInput, fragTexCoord + offset).r;
        }
    }
    
    outOcclusion = result / 16.0;
}
