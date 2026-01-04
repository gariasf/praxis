#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

// Per-instance data
layout(location = 4) in vec4 instance_model_col0;
layout(location = 5) in vec4 instance_model_col1;
layout(location = 6) in vec4 instance_model_col2;
layout(location = 7) in vec4 instance_model_col3;
layout(location = 8) in vec4 instance_color_and_wind;

layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
} vp;

layout(push_constant) uniform PushConstants {
    float time;
    float wind_strength;
    vec2 wind_direction;
} pc;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_uv;
layout(location = 3) out vec3 frag_color;

void main() {
    // Reconstruct instance model matrix
    mat4 model = mat4(
        instance_model_col0,
        instance_model_col1,
        instance_model_col2,
        instance_model_col3
    );
    
    // Apply wind animation
    float wind_phase = instance_color_and_wind.w;
    float wind_factor = position.y; // Only affect top of vegetation
    vec2 wind_offset = pc.wind_direction * sin(pc.time + wind_phase) * pc.wind_strength * wind_factor;
    vec3 animated_pos = position + vec3(wind_offset.x, 0.0, wind_offset.y);
    
    vec4 world_pos = model * vec4(animated_pos, 1.0);
    gl_Position = vp.proj * vp.view * world_pos;
    
    frag_position = world_pos.xyz;
    frag_normal = normalize(mat3(model) * normal);
    frag_uv = uv;
    frag_color = color * instance_color_and_wind.rgb;
}
