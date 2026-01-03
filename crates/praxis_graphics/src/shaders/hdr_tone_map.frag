#version 450

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D hdr_texture;

layout(push_constant) uniform ToneMapParams {
    float exposure;
    float gamma;
    uint operator;
    uint padding;
} params;

layout(location = 0) out vec4 out_color;

// Reinhard tone mapping
vec3 reinhard_tone_mapping(vec3 color) {
    return color / (color + vec3(1.0));
}

// ACES Filmic tone mapping
// Reference: https://knarkowicz.wordpress.com/2016/01/06/aces-filmic-tone-mapping-curve/
vec3 aces_tone_mapping(vec3 color) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((color * (a * color + b)) / (color * (c * color + d) + e), 0.0, 1.0);
}

// Uncharted 2 tone mapping (Hable)
// Reference: http://filmicworlds.com/blog/filmic-tonemapping-operators/
vec3 uncharted2_partial(vec3 x) {
    const float A = 0.15;
    const float B = 0.50;
    const float C = 0.10;
    const float D = 0.20;
    const float E = 0.02;
    const float F = 0.30;
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

vec3 uncharted2_tone_mapping(vec3 color) {
    const float W = 11.2; // White point
    vec3 curr = uncharted2_partial(color);
    vec3 white_scale = vec3(1.0) / uncharted2_partial(vec3(W));
    return curr * white_scale;
}

// Gamma correction
vec3 gamma_correction(vec3 color, float gamma) {
    return pow(color, vec3(1.0 / gamma));
}

void main() {
    // Sample HDR color
    vec3 hdr_color = texture(hdr_texture, in_uv).rgb;
    
    // Apply exposure
    vec3 color = hdr_color * params.exposure;
    
    // Apply tone mapping operator
    vec3 tone_mapped;
    if (params.operator == 0u) {
        // Reinhard
        tone_mapped = reinhard_tone_mapping(color);
    } else if (params.operator == 1u) {
        // ACES
        tone_mapped = aces_tone_mapping(color);
    } else if (params.operator == 2u) {
        // Uncharted 2
        tone_mapped = uncharted2_tone_mapping(color);
    } else {
        // Default to ACES
        tone_mapped = aces_tone_mapping(color);
    }
    
    // Apply gamma correction
    vec3 final_color = gamma_correction(tone_mapped, params.gamma);
    
    out_color = vec4(final_color, 1.0);
}
