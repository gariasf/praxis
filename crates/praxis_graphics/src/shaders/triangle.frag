#version 450

// Fragment Shader for 3D Textured Rendering
//
// This shader samples a texture using UV coordinates and multiplies it with
// the vertex color to produce the final pixel color.
//
// Input (from vertex shader):
//   location 0: vec3 v_color - interpolated vertex color (RGB)
//   location 1: vec2 v_uv    - interpolated UV coordinates
//
// Texture Sampler (set 0, binding 1):
//   sampler2D albedo_texture - the base color texture
//
// Output:
//   location 0: vec4 f_color - final pixel color (RGBA)
//
// Rendering Equation:
//   final_color = vertex_color * texture_color
//
// This allows for:
//   - Textured rendering when texture is provided
//   - Tinting textures via vertex colors
//   - Pure color rendering when using a white texture

layout(location = 0) in vec3 v_color;
layout(location = 1) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler2D albedo_texture;

void main() {
    // Sample the texture at the interpolated UV coordinates
    vec4 tex_color = texture(albedo_texture, v_uv);
    
    // Multiply vertex color with texture color
    // This allows vertex colors to tint/modulate the texture
    f_color = vec4(v_color, 1.0) * tex_color;
}
