#version 450

// Film grain noise fragment shader for cinematic presentation
// Adds procedural grain noise to simulate film stock

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(set = 0, binding = 1) uniform FilmGrainParams {
    float intensity;           // Grain intensity
    float size;                // Grain particle size
    float luminance_impact;    // How much grain intensity varies with luminance
    float time;                // Time for animated grain
} params;

layout(location = 0) out vec4 out_color;

// Pseudo-random noise function
float random(vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898, 78.233))) * 43758.5453123);
}

// Noise function with time variation for animated grain
float noise(vec2 st) {
    vec2 i = floor(st);
    vec2 f = fract(st);
    
    // Add time component for animation
    float time_component = params.time * 0.001;
    
    float a = random(i + time_component);
    float b = random(i + vec2(1.0, 0.0) + time_component);
    float c = random(i + vec2(0.0, 1.0) + time_component);
    float d = random(i + vec2(1.0, 1.0) + time_component);
    
    // Smooth interpolation
    vec2 u = f * f * (3.0 - 2.0 * f);
    
    return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

void main() {
    vec4 color = texture(input_texture, in_uv);
    
    // Calculate luminance for luminance-based grain intensity
    float luminance = dot(color.rgb, vec3(0.299, 0.587, 0.114));
    
    // Adjust grain intensity based on luminance (more visible in darker areas typically)
    float lum_factor = 1.0 + (1.0 - luminance) * params.luminance_impact;
    
    // Calculate grain coordinates based on size parameter
    vec2 grain_coord = in_uv * textureSize(input_texture, 0) / params.size;
    
    // Generate grain noise
    float grain = noise(grain_coord) * 2.0 - 1.0; // Range from -1 to 1
    
    // Apply grain intensity and luminance factor
    grain *= params.intensity * lum_factor;
    
    // Add grain to color
    vec3 final_color = color.rgb + vec3(grain);
    
    // Clamp to valid range
    final_color = clamp(final_color, 0.0, 1.0);
    
    out_color = vec4(final_color, color.a);
}
