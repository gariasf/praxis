#version 450

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(push_constant) uniform BlurParams {
    vec2 texel_size;
} params;

layout(location = 0) out vec4 out_color;

void main() {
    float weights[5] = float[](0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);
    
    vec3 result = texture(input_texture, in_uv).rgb * weights[0];
    
    for (int i = 1; i < 5; i++) {
        float offset = float(i) * params.texel_size.y;
        result += texture(input_texture, in_uv + vec2(0.0, offset)).rgb * weights[i];
        result += texture(input_texture, in_uv - vec2(0.0, offset)).rgb * weights[i];
    }
    
    out_color = vec4(result, 1.0);
}
