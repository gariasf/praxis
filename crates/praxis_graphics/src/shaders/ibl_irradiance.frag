#version 450

// Fragment shader for precomputing diffuse irradiance from environment map
// This shader performs convolution to compute the ambient lighting contribution

layout(location = 0) in vec3 v_local_pos;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform samplerCube u_environment_map;

const float PI = 3.14159265359;

void main() {
    vec3 N = normalize(v_local_pos);
    
    vec3 irradiance = vec3(0.0);
    
    // Tangent space coordinate system
    vec3 up = vec3(0.0, 1.0, 0.0);
    vec3 right = normalize(cross(up, N));
    up = normalize(cross(N, right));
    
    float sample_delta = 0.025;
    float nr_samples = 0.0;
    
    // Convolve the environment map over the hemisphere
    for (float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            // Spherical to cartesian (in tangent space)
            vec3 tangent_sample = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
            
            // Tangent space to world space
            vec3 sample_vec = tangent_sample.x * right + tangent_sample.y * up + tangent_sample.z * N;
            
            irradiance += texture(u_environment_map, sample_vec).rgb * cos(theta) * sin(theta);
            nr_samples++;
        }
    }
    
    irradiance = PI * irradiance * (1.0 / float(nr_samples));
    
    f_color = vec4(irradiance, 1.0);
}
