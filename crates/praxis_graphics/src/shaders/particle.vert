#version 450

// Vertex attributes (quad geometry)
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec3 in_color;
layout(location = 3) in vec2 in_uv;

// Instance attributes
layout(location = 4) in vec3 instance_position;
layout(location = 5) in vec4 instance_color;
layout(location = 6) in float instance_size;
layout(location = 7) in float instance_rotation;
layout(location = 8) in float instance_atlas_index;

// View/Projection uniforms
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} vp;

// Output to fragment shader
layout(location = 0) out vec2 frag_uv;
layout(location = 1) out vec4 frag_color;
layout(location = 2) out vec3 frag_world_pos;
layout(location = 3) out float frag_view_depth;

void main() {
    // Extract camera basis vectors from view matrix
    vec3 camera_right = vec3(vp.view[0][0], vp.view[1][0], vp.view[2][0]);
    vec3 camera_up = vec3(vp.view[0][1], vp.view[1][1], vp.view[2][1]);
    
    // Apply rotation
    float cos_rot = cos(instance_rotation);
    float sin_rot = sin(instance_rotation);
    vec3 right = camera_right * cos_rot - camera_up * sin_rot;
    vec3 up = camera_right * sin_rot + camera_up * cos_rot;
    
    // Billboard transformation
    vec3 world_pos = instance_position + 
                     right * in_position.x * instance_size +
                     up * in_position.y * instance_size;
    
    frag_world_pos = world_pos;
    
    // Transform to clip space
    vec4 view_pos = vp.view * vec4(world_pos, 1.0);
    gl_Position = vp.proj * view_pos;
    frag_view_depth = -view_pos.z; // Positive depth in view space
    
    // Pass through UVs and color
    frag_uv = in_uv;
    frag_color = instance_color;
}
