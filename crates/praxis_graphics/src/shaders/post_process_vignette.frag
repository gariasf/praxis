#version 450

// Vignette effect fragment shader
// Darkens the edges of the image for cinematic presentation

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(set = 0, binding = 1) uniform VignetteParams {
    float intensity;           // Darkness intensity at edges
    float smoothness;          // How gradual the vignette transition is
    float roundness;           // Roundness of vignette shape (0 = rectangular, 1 = circular)
    vec2 center;               // Center point of vignette effect
} params;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(input_texture, in_uv);
    
    // Calculate distance from vignette center
    vec2 offset = in_uv - params.center;
    
    // Adjust for aspect ratio and roundness
    offset.x *= 1.0 + (1.0 - params.roundness);
    
    // Calculate vignette factor
    float dist = length(offset);
    
    // Apply smoothstep for smooth transition
    float vignette = smoothstep(params.intensity, params.intensity - params.smoothness, dist);
    
    // Apply vignette by darkening
    out_color = vec4(color.rgb * vignette, color.a);
}
