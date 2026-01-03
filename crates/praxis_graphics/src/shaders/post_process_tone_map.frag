#version 450

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D scene_texture;
layout(set = 0, binding = 1) uniform sampler2D bloom_texture;

layout(push_constant) uniform ToneMapParams {
    float exposure;
    float bloom_intensity;
} params;

layout(location = 0) out vec4 out_color;

vec3 reinhard_tone_mapping(vec3 hdr_color, float exposure) {
    vec3 color = hdr_color * exposure;
    color = color / (color + vec3(1.0));
    return color;
}

vec3 gamma_correction(vec3 color, float gamma) {
    return pow(color, vec3(1.0 / gamma));
}

void main() {
    vec3 scene_color = texture(scene_texture, in_uv).rgb;
    vec3 bloom_color = texture(bloom_texture, in_uv).rgb;
    
    vec3 hdr_color = scene_color + bloom_color * params.bloom_intensity;
    
    vec3 tone_mapped = reinhard_tone_mapping(hdr_color, params.exposure);
    
    vec3 gamma_corrected = gamma_correction(tone_mapped, 2.2);
    
    out_color = vec4(gamma_corrected, 1.0);
}
