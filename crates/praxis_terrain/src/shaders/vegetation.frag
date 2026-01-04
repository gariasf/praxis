#version 450

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;
layout(location = 3) in vec3 frag_color;

layout(set = 0, binding = 1) uniform sampler2D albedo_texture;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 tex_color = texture(albedo_texture, frag_uv);
    
    // Alpha testing for foliage
    if (tex_color.a < 0.5) {
        discard;
    }
    
    // Simple lighting
    vec3 light_dir = normalize(vec3(0.5, 1.0, 0.3));
    float ndotl = max(dot(frag_normal, light_dir), 0.0);
    vec3 ambient = vec3(0.3);
    vec3 lighting = ambient + vec3(0.7) * ndotl;
    
    vec3 final_color = tex_color.rgb * frag_color * lighting;
    out_color = vec4(final_color, tex_color.a);
}
