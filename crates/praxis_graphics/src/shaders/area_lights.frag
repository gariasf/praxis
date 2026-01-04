#version 450

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec3 v_color;
layout(location = 3) in vec2 v_uv;
layout(location = 4) in vec3 v_tangent;
layout(location = 5) in vec3 v_bitangent;

layout(location = 0) out vec4 f_color;

struct AreaLight {
    mat4 transform;
    vec4 color;
    float intensity;
    uint light_type;
    float param1;
    float param2;
    uint two_sided;
};

layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

layout(set = 0, binding = 1, std140) uniform AreaLightsData {
    AreaLight lights[16];
    uint light_count;
} area_lights;

layout(set = 0, binding = 2) uniform sampler2D ltc_matrix_1;
layout(set = 0, binding = 3) uniform sampler2D ltc_matrix_2;

layout(set = 0, binding = 4) uniform sampler2D albedo_texture;

layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;
    float metallic;
    float roughness;
    float emissive_strength;
    float _padding;
} material;

const float PI = 3.14159265359;

vec3 integrate_edge(vec3 v1, vec3 v2) {
    float x = dot(v1, v2);
    float y = abs(x);
    
    float a = 0.8543985 + (0.4965155 + 0.0145206 * y) * y;
    float b = 3.4175940 + (4.1616724 + y) * y;
    float v = a / b;
    
    float theta_sintheta = (x > 0.0) ? v : 0.5 * inversesqrt(max(1.0 - x * x, 1e-7)) - v;
    
    return cross(v1, v2) * theta_sintheta;
}

vec3 ltc_evaluate(vec3 N, vec3 V, vec3 P, mat3 Minv, vec3 points[4]) {
    vec3 T1, T2;
    T1 = normalize(V - N * dot(V, N));
    T2 = cross(N, T1);
    
    mat3 TBN = mat3(T1, T2, N);
    Minv = Minv * transpose(TBN);
    
    vec3 L[4];
    for (int i = 0; i < 4; i++) {
        L[i] = normalize(Minv * (points[i] - P));
    }
    
    vec3 vsum = vec3(0.0);
    vsum += integrate_edge(L[0], L[1]);
    vsum += integrate_edge(L[1], L[2]);
    vsum += integrate_edge(L[2], L[3]);
    vsum += integrate_edge(L[3], L[0]);
    
    float len = length(vsum);
    float z = vsum.z / len;
    
    return vec3(max(0.0, z) * len);
}

vec3 evaluate_rectangle_light(AreaLight light, vec3 P, vec3 N, vec3 V, float roughness) {
    float width = light.param1;
    float height = light.param2;
    
    vec3 light_pos = vec3(light.transform[3]);
    vec3 light_right = normalize(vec3(light.transform[0]));
    vec3 light_up = normalize(vec3(light.transform[1]));
    vec3 light_forward = normalize(vec3(light.transform[2]));
    
    float ndotv = clamp(dot(N, V), 0.0, 1.0);
    vec2 uv = vec2(roughness, sqrt(1.0 - ndotv));
    uv = uv * (63.0 / 64.0) + 0.5 / 64.0;
    
    vec4 t1 = texture(ltc_matrix_1, uv);
    vec4 t2 = texture(ltc_matrix_2, uv);
    
    mat3 Minv = mat3(
        vec3(t1.x, 0.0, t1.y),
        vec3(0.0, 1.0, 0.0),
        vec3(t1.z, 0.0, t1.w)
    );
    
    vec3 half_width = light_right * width * 0.5;
    vec3 half_height = light_up * height * 0.5;
    
    vec3 points[4];
    points[0] = light_pos - half_width - half_height;
    points[1] = light_pos + half_width - half_height;
    points[2] = light_pos + half_width + half_height;
    points[3] = light_pos - half_width + half_height;
    
    vec3 diffuse = ltc_evaluate(N, V, P, mat3(1.0), points);
    vec3 specular = ltc_evaluate(N, V, P, Minv, points);
    specular *= t2.x * material.metallic + (1.0 - material.metallic) * 0.04;
    
    float dist = length(light_pos - P);
    float attenuation = 1.0 / (1.0 + dist * dist);
    
    return (diffuse + specular) * light.color.rgb * light.intensity * attenuation;
}

vec3 evaluate_sphere_light(AreaLight light, vec3 P, vec3 N, vec3 V) {
    vec3 light_pos = vec3(light.transform[3]);
    float radius = light.param1;
    
    vec3 L = light_pos - P;
    float dist = length(L);
    L /= dist;
    
    vec3 closest_point = light_pos - L * radius;
    vec3 L_closest = normalize(closest_point - P);
    
    float ndotl = max(dot(N, L_closest), 0.0);
    
    float sphere_angle = asin(radius / dist);
    float solid_angle = 2.0 * PI * (1.0 - cos(sphere_angle));
    
    float attenuation = solid_angle / (4.0 * PI);
    
    return light.color.rgb * light.intensity * ndotl * attenuation;
}

void main() {
    vec4 tex_color = texture(albedo_texture, v_uv);
    vec3 albedo = v_color * tex_color.rgb * material.base_color.rgb;
    float alpha = tex_color.a * material.base_color.a;
    
    vec3 N = normalize(v_normal);
    vec3 V = normalize(view_proj.camera_position - v_world_pos);
    
    vec3 lighting = vec3(0.0);
    
    for (uint i = 0u; i < area_lights.light_count && i < 16u; i++) {
        AreaLight light = area_lights.lights[i];
        
        if (light.light_type == 0u) {
            lighting += evaluate_rectangle_light(light, v_world_pos, N, V, material.roughness);
        } else if (light.light_type == 2u) {
            lighting += evaluate_sphere_light(light, v_world_pos, N, V);
        }
    }
    
    vec3 final_color = albedo * lighting;
    vec3 emissive = material.base_color.rgb * material.emissive_strength;
    final_color += emissive;
    
    f_color = vec4(final_color, alpha);
}
