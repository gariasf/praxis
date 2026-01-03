#version 450

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(push_constant) uniform BrightnessParams {
    float threshold;
} params;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(input_texture, in_uv);
    
    float brightness = dot(color.rgb, vec3(0.2126, 0.7152, 0.0722));
    
    if (brightness > params.threshold) {
        out_color = color;
    } else {
        out_color = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
