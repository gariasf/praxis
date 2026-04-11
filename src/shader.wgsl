@group(0) @binding(0)
var<uniform> camera: CameraUniform;
struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos:  vec4<f32>,
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(0)
var<uniform> light: LightUniform;
struct LightUniform {
    direction: vec4<f32>,
    color: vec4<f32>,
    ambient: vec4<f32>,
}

@group(3) @binding(0)
var<uniform> model: ModelUniform;
struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

// Vertex shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model.model * vec4<f32>(position, 1.0);
    out.uv = uv;
    out.normal = (model.normal_matrix * vec4<f32>(normal, 0.0)).xyz;
    out.world_pos = (model.model * vec4<f32>(position, 1.0)).xyz;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.normal);
    let ambient_color = light.ambient.rgb * light.ambient.a;
    let light_dir = normalize(-light.direction.xyz);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_color = light.color.rgb * light.color.a * diffuse_strength;
    let texture_color = textureSample(t_diffuse, s_diffuse, in.uv);

    let view_dir = normalize(camera.camera_pos.xyz - in.world_pos);
    let half_vec = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_vec), 0.0), 200.0);
    return vec4<f32>((ambient_color + diffuse_color + specular) * texture_color.rgb, texture_color.a);
}
