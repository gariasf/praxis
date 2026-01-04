#version 450

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_color;
layout(location = 3) in vec2 v_uv;

layout(location = 0) out vec4 f_color;

struct LightProbe {
    vec4 position;
    vec4 sh_r;
    vec4 sh_g;
    vec4 sh_b;
    vec4 sh_r2;
    vec4 sh_g2;
    vec4 sh_b2;
    vec4 sh_r3;
    vec4 sh_g3;
    vec4 sh_b3;
    float intensity;
    float radius;
};

layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

layout(set = 0, binding = 1, std140) uniform LightProbeData {
    LightProbe probes[64];
    uint probe_count;
} light_probes;

layout(set = 0, binding = 2) uniform sampler2D albedo_texture;

layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;
    float metallic;
    float roughness;
    float emissive_strength;
    float _padding;
} material;

const float C0 = 0.282095;
const float C1 = 0.488603;
const float C2_0 = 1.092548;
const float C2_1 = 0.315392;
const float C2_2 = 0.546274;

vec3 evaluate_sh(vec3 normal, vec3 sh_r, vec3 sh_g, vec3 sh_b, 
                 vec3 sh_r2, vec3 sh_g2, vec3 sh_b2,
                 vec3 sh_r3, vec3 sh_g3, vec3 sh_b3) {
    vec3 result = vec3(0.0);
    
    float y00 = C0;
    result.r += sh_r.x * y00;
    result.g += sh_g.x * y00;
    result.b += sh_b.x * y00;
    
    float y1_1 = C1 * normal.y;
    float y10 = C1 * normal.z;
    float y11 = C1 * normal.x;
    
    result.r += sh_r.y * y1_1 + sh_r.z * y10 + sh_r2.x * y11;
    result.g += sh_g.y * y1_1 + sh_g.z * y10 + sh_g2.x * y11;
    result.b += sh_b.y * y1_1 + sh_b.z * y10 + sh_b2.x * y11;
    
    float y2_2 = C2_1 * normal.x * normal.y;
    float y2_1 = C2_1 * normal.y * normal.z;
    float y20 = C2_2 * (3.0 * normal.z * normal.z - 1.0);
    float y21 = C2_1 * normal.x * normal.z;
    float y22 = C2_0 * (normal.x * normal.x - normal.y * normal.y);
    
    result.r += sh_r2.y * y2_2 + sh_r2.z * y2_1 + sh_r3.x * y20 + 
                sh_r3.y * y21 + sh_r3.z * y22;
    result.g += sh_g2.y * y2_2 + sh_g2.z * y2_1 + sh_g3.x * y20 + 
                sh_g3.y * y21 + sh_g3.z * y22;
    result.b += sh_b2.y * y2_2 + sh_b2.z * y2_1 + sh_b3.x * y20 + 
                sh_b3.y * y21 + sh_b3.z * y22;
    
    return max(result, vec3(0.0));
}

vec3 sample_probes(vec3 position, vec3 normal) {
    vec3 irradiance = vec3(0.0);
    float total_weight = 0.0;
    
    for (uint i = 0u; i < light_probes.probe_count && i < 64u; i++) {
        LightProbe probe = light_probes.probes[i];
        
        vec3 probe_pos = probe.position.xyz;
        float dist = length(probe_pos - position);
        
        if (dist < probe.radius) {
            float weight = 1.0 - (dist / probe.radius);
            weight = weight * weight;
            
            vec3 probe_irradiance = evaluate_sh(
                normal,
                probe.sh_r.xyz, probe.sh_g.xyz, probe.sh_b.xyz,
                probe.sh_r2.xyz, probe.sh_g2.xyz, probe.sh_b2.xyz,
                probe.sh_r3.xyz, probe.sh_g3.xyz, probe.sh_b3.xyz
            );
            
            irradiance += probe_irradiance * probe.intensity * weight;
            total_weight += weight;
        }
    }
    
    if (total_weight > 0.0) {
        irradiance /= total_weight;
    }
    
    return irradiance;
}

void main() {
    vec4 tex_color = texture(albedo_texture, v_uv);
    vec3 albedo = v_color * tex_color.rgb * material.base_color.rgb;
    float alpha = tex_color.a * material.base_color.a;
    
    vec3 N = normalize(v_normal);
    
    vec3 irradiance = sample_probes(v_world_pos, N);
    
    vec3 diffuse_factor = (1.0 - material.metallic) * albedo;
    vec3 lighting = irradiance * diffuse_factor;
    
    vec3 emissive = material.base_color.rgb * material.emissive_strength;
    vec3 final_color = lighting + emissive;
    
    f_color = vec4(final_color, alpha);
}
