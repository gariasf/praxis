#version 450

// Chromatic aberration fragment shader for lens distortion simulation
// Simulates color fringing caused by imperfect lenses

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(set = 0, binding = 1) uniform ChromaticAberrationParams {
    float intensity;           // Aberration intensity
    float radial_falloff;      // How much effect increases toward edges
    vec2 direction;            // Direction of aberration (for directional distortion)
    float red_offset;          // Red channel offset multiplier
    float blue_offset;         // Blue channel offset multiplier
} params;

layout(location = 0) out vec4 out_color;

void main() {
    // Calculate distance from center (0.5, 0.5)
    vec2 center = vec2(0.5);
    vec2 offset_from_center = in_uv - center;
    float dist_from_center = length(offset_from_center);
    
    // Apply radial falloff - more aberration near edges
    float radial_factor = pow(dist_from_center * 2.0, params.radial_falloff);
    
    // Calculate aberration direction (radial by default)
    vec2 aberration_dir = normalize(offset_from_center);
    
    // If custom direction is provided, blend it in
    if (length(params.direction) > 0.01) {
        aberration_dir = mix(aberration_dir, params.direction, 0.5);
    }
    
    // Calculate offset amount based on intensity and radial factor
    vec2 offset = aberration_dir * params.intensity * radial_factor;
    
    // Sample each color channel with different offsets
    float r = texture(input_texture, in_uv + offset * params.red_offset).r;
    float g = texture(input_texture, in_uv).g; // Green stays centered
    float b = texture(input_texture, in_uv - offset * params.blue_offset).b;
    
    // Combine channels
    out_color = vec4(r, g, b, 1.0);
}
