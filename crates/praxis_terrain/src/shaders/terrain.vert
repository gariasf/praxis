#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec4 tangent;

layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
} vp;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pc;

layout(location = 0) out vec3 frag_position;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec2 frag_uv;
layout(location = 3) out vec3 frag_color;
layout(location = 4) out mat3 frag_tbn;

void main() {
    vec4 world_pos = pc.model * vec4(position, 1.0);
    gl_Position = vp.proj * vp.view * world_pos;
    
    frag_position = world_pos.xyz;
    frag_normal = normalize(mat3(pc.model) * normal);
    frag_uv = uv;
    frag_color = color;
    
    // Compute TBN matrix for normal mapping
    vec3 T = normalize(mat3(pc.model) * tangent.xyz);
    vec3 N = frag_normal;
    vec3 B = cross(N, T) * tangent.w;
    frag_tbn = mat3(T, B, N);
}
