#version 450

layout(location = 0) in vec3 frag_position;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec2 frag_uv;
layout(location = 3) in vec3 frag_color;
layout(location = 4) in mat3 frag_tbn;

layout(set = 0, binding = 1) uniform sampler2D splat_map;
layout(set = 0, binding = 2) uniform sampler2D layer0_albedo;
layout(set = 0, binding = 3) uniform sampler2D layer1_albedo;
layout(set = 0, binding = 4) uniform sampler2D layer2_albedo;
layout(set = 0, binding = 5) uniform sampler2D layer3_albedo;
layout(set = 0, binding = 6) uniform sampler2D layer0_normal;
layout(set = 0, binding = 7) uniform sampler2D layer1_normal;
layout(set = 0, binding = 8) uniform sampler2D layer2_normal;
layout(set = 0, binding = 9) uniform sampler2D layer3_normal;

layout(set = 1, binding = 0) uniform TerrainProperties {
    vec4 layer_tiling[4];
    vec4 layer_properties[4]; // metallic, roughness, unused, unused
} terrain;

layout(location = 0) out vec4 out_color;

vec3 blend_normals(vec3 n1, vec3 n2, float blend) {
    return normalize(mix(n1, n2, blend));
}

void main() {
    // Sample splat map to get blend weights
    vec4 splat = texture(splat_map, frag_uv);
    
    // Normalize weights to ensure they sum to 1
    float weight_sum = splat.r + splat.g + splat.b + splat.a;
    if (weight_sum > 0.0) {
        splat /= weight_sum;
    } else {
        splat = vec4(1.0, 0.0, 0.0, 0.0);
    }
    
    // Sample albedo textures with tiling
    vec3 albedo0 = texture(layer0_albedo, frag_uv * terrain.layer_tiling[0].x).rgb;
    vec3 albedo1 = texture(layer1_albedo, frag_uv * terrain.layer_tiling[1].x).rgb;
    vec3 albedo2 = texture(layer2_albedo, frag_uv * terrain.layer_tiling[2].x).rgb;
    vec3 albedo3 = texture(layer3_albedo, frag_uv * terrain.layer_tiling[3].x).rgb;
    
    // Blend albedo textures
    vec3 final_albedo = albedo0 * splat.r + 
                        albedo1 * splat.g + 
                        albedo2 * splat.b + 
                        albedo3 * splat.a;
    
    // Sample normal maps
    vec3 normal0 = texture(layer0_normal, frag_uv * terrain.layer_tiling[0].x).rgb * 2.0 - 1.0;
    vec3 normal1 = texture(layer1_normal, frag_uv * terrain.layer_tiling[1].x).rgb * 2.0 - 1.0;
    vec3 normal2 = texture(layer2_normal, frag_uv * terrain.layer_tiling[2].x).rgb * 2.0 - 1.0;
    vec3 normal3 = texture(layer3_normal, frag_uv * terrain.layer_tiling[3].x).rgb * 2.0 - 1.0;
    
    // Blend normals
    vec3 blended_normal = normal0 * splat.r + 
                          normal1 * splat.g + 
                          normal2 * splat.b + 
                          normal3 * splat.a;
    
    // Transform normal to world space using TBN matrix
    vec3 world_normal = normalize(frag_tbn * blended_normal);
    
    // Simple lighting calculation
    vec3 light_dir = normalize(vec3(0.5, 1.0, 0.3));
    float ndotl = max(dot(world_normal, light_dir), 0.0);
    vec3 ambient = vec3(0.2);
    vec3 lighting = ambient + vec3(0.8) * ndotl;
    
    // Final color
    out_color = vec4(final_albedo * lighting * frag_color, 1.0);
}
