#version 450

layout(location = 0) in vec3 v_color;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D albedo_texture;

void main() {
    vec4 tex_color = texture(albedo_texture, v_uv);
    f_color = vec4(v_color, 1.0) * tex_color;
}
