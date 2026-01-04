#version 450

// Material layer blending fragment shader
// Supports multiple blend modes: Replace, Add, Multiply, Overlay

layout(location = 0) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

// Layer 0 (base)
layout(set = 0, binding = 0) uniform sampler2D base_albedo;
layout(set = 0, binding = 1) uniform sampler2D base_normal;
layout(set = 0, binding = 2) uniform sampler2D base_metallic_roughness;

// Layer 1
layout(set = 0, binding = 3) uniform sampler2D layer1_albedo;
layout(set = 0, binding = 4) uniform sampler2D layer1_normal;
layout(set = 0, binding = 5) uniform sampler2D layer1_metallic_roughness;
layout(set = 0, binding = 6) uniform sampler2D layer1_mask;

// Layer 2
layout(set = 0, binding = 7) uniform sampler2D layer2_albedo;
layout(set = 0, binding = 8) uniform sampler2D layer2_normal;
layout(set = 0, binding = 9) uniform sampler2D layer2_metallic_roughness;
layout(set = 0, binding = 10) uniform sampler2D layer2_mask;

// Layer 3
layout(set = 0, binding = 11) uniform sampler2D layer3_albedo;
layout(set = 0, binding = 12) uniform sampler2D layer3_normal;
layout(set = 0, binding = 13) uniform sampler2D layer3_metallic_roughness;
layout(set = 0, binding = 14) uniform sampler2D layer3_mask;

// Layer parameters
layout(set = 1, binding = 0, std140) uniform LayerParams {
    // Layer 1
    vec2 layer1_uv_scale;
    float layer1_opacity;
    uint layer1_blend_mode; // 0=Replace, 1=Add, 2=Multiply, 3=Overlay
    
    // Layer 2
    vec2 layer2_uv_scale;
    float layer2_opacity;
    uint layer2_blend_mode;
    
    // Layer 3
    vec2 layer3_uv_scale;
    float layer3_opacity;
    uint layer3_blend_mode;
    
    // Flags
    uint layer1_enabled;
    uint layer2_enabled;
    uint layer3_enabled;
    uint _padding;
} layers;

// Blend modes
vec3 blend_replace(vec3 base, vec3 layer, float mask) {
    return mix(base, layer, mask);
}

vec3 blend_add(vec3 base, vec3 layer, float mask) {
    return base + layer * mask;
}

vec3 blend_multiply(vec3 base, vec3 layer, float mask) {
    return mix(base, base * layer, mask);
}

float overlay_channel(float base, float layer) {
    if (base < 0.5) {
        return 2.0 * base * layer;
    } else {
        return 1.0 - 2.0 * (1.0 - base) * (1.0 - layer);
    }
}

vec3 blend_overlay(vec3 base, vec3 layer, float mask) {
    vec3 overlaid = vec3(
        overlay_channel(base.r, layer.r),
        overlay_channel(base.g, layer.g),
        overlay_channel(base.b, layer.b)
    );
    return mix(base, overlaid, mask);
}

vec3 blend_layer(vec3 base, vec3 layer, float mask, uint blend_mode) {
    if (blend_mode == 0u) {
        return blend_replace(base, layer, mask);
    } else if (blend_mode == 1u) {
        return blend_add(base, layer, mask);
    } else if (blend_mode == 2u) {
        return blend_multiply(base, layer, mask);
    } else {
        return blend_overlay(base, layer, mask);
    }
}

// Normal blending (in tangent space)
vec3 blend_normals(vec3 base, vec3 layer, float mask) {
    // Unpack normals from [0,1] to [-1,1]
    base = base * 2.0 - 1.0;
    layer = layer * 2.0 - 1.0;
    
    // Blend using RNM (Reoriented Normal Mapping)
    vec3 blended = normalize(vec3(
        base.xy + layer.xy * mask,
        base.z * layer.z
    ));
    
    // Pack back to [0,1]
    return blended * 0.5 + 0.5;
}

void main() {
    // Sample base layer
    vec3 albedo = texture(base_albedo, v_uv).rgb;
    vec3 normal = texture(base_normal, v_uv).rgb;
    vec2 metallic_roughness = texture(base_metallic_roughness, v_uv).rg;
    
    // Layer 1
    if (layers.layer1_enabled != 0u) {
        vec2 uv1 = v_uv * layers.layer1_uv_scale;
        float mask1 = texture(layer1_mask, v_uv).r * layers.layer1_opacity;
        
        if (mask1 > 0.0) {
            vec3 layer1_alb = texture(layer1_albedo, uv1).rgb;
            vec3 layer1_norm = texture(layer1_normal, uv1).rgb;
            vec2 layer1_mr = texture(layer1_metallic_roughness, uv1).rg;
            
            albedo = blend_layer(albedo, layer1_alb, mask1, layers.layer1_blend_mode);
            normal = blend_normals(normal, layer1_norm, mask1);
            metallic_roughness = mix(metallic_roughness, layer1_mr, mask1);
        }
    }
    
    // Layer 2
    if (layers.layer2_enabled != 0u) {
        vec2 uv2 = v_uv * layers.layer2_uv_scale;
        float mask2 = texture(layer2_mask, v_uv).r * layers.layer2_opacity;
        
        if (mask2 > 0.0) {
            vec3 layer2_alb = texture(layer2_albedo, uv2).rgb;
            vec3 layer2_norm = texture(layer2_normal, uv2).rgb;
            vec2 layer2_mr = texture(layer2_metallic_roughness, uv2).rg;
            
            albedo = blend_layer(albedo, layer2_alb, mask2, layers.layer2_blend_mode);
            normal = blend_normals(normal, layer2_norm, mask2);
            metallic_roughness = mix(metallic_roughness, layer2_mr, mask2);
        }
    }
    
    // Layer 3
    if (layers.layer3_enabled != 0u) {
        vec2 uv3 = v_uv * layers.layer3_uv_scale;
        float mask3 = texture(layer3_mask, v_uv).r * layers.layer3_opacity;
        
        if (mask3 > 0.0) {
            vec3 layer3_alb = texture(layer3_albedo, uv3).rgb;
            vec3 layer3_norm = texture(layer3_normal, uv3).rgb;
            vec2 layer3_mr = texture(layer3_metallic_roughness, uv3).rg;
            
            albedo = blend_layer(albedo, layer3_alb, mask3, layers.layer3_blend_mode);
            normal = blend_normals(normal, layer3_norm, mask3);
            metallic_roughness = mix(metallic_roughness, layer3_mr, mask3);
        }
    }
    
    // Output blended material
    // Note: This is typically rendered to a texture that becomes the input
    // to the main material shader
    f_color = vec4(albedo, 1.0);
}
