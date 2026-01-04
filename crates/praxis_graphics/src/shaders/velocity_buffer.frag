#version 450

// Velocity buffer generation fragment shader
// Calculates per-pixel screen-space motion vectors for motion blur

layout(location = 0) in vec4 current_pos;
layout(location = 1) in vec4 previous_pos;

layout(location = 0) out vec4 out_velocity;

void main() {
    // Convert from clip space to normalized device coordinates (NDC)
    vec2 current_ndc = current_pos.xy / current_pos.w;
    vec2 previous_ndc = previous_pos.xy / previous_pos.w;
    
    // Convert from NDC [-1,1] to screen space [0,1]
    vec2 current_screen = current_ndc * 0.5 + 0.5;
    vec2 previous_screen = previous_ndc * 0.5 + 0.5;
    
    // Calculate velocity as difference between current and previous positions
    vec2 velocity = current_screen - previous_screen;
    
    // Store velocity in RG channels (BA unused)
    out_velocity = vec4(velocity, 0.0, 1.0);
}
