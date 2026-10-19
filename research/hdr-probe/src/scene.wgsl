// Throwaway probe for #189 (never merged): flat-shaded triangles standing in
// for the lit pass, written to a 4x MSAA colour attachment that the hardware
// box-resolves. WEIGHTING stores color / (1 + luminance), the reversible
// weighting `post.wgsl` undoes.

override WEIGHTING: bool = false;

const REC709_LUMINANCE = vec3<f32>(0.2126, 0.7152, 0.0722);

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) radiance: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) radiance: vec3<f32>,
}

@vertex
fn vs_scene(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.5, 1.0);
    output.radiance = input.radiance;
    return output;
}

@fragment
fn fs_scene(input: VertexOutput) -> @location(0) vec4<f32> {
    if WEIGHTING {
        return vec4<f32>(input.radiance / (1.0 + dot(input.radiance, REC709_LUMINANCE)), 1.0);
    }
    return vec4<f32>(input.radiance, 1.0);
}
