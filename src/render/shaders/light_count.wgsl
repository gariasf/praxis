// The light count pass (src/render/light_count_pass.rs), with the
// count_point_lights Render switch on: an invocation per pixel of the depth
// target, where the frame drew a surface at its first sample, counts the
// point lights the surface's Froxel lists and those whose range reaches it,
// which its workgroup sums before one invocation adds them to the light
// counts.

// Where a pixel's first sample lies within it: WebGPU's standard 4x
// pattern puts sample 0 at (0.375, 0.125).
const FIRST_SAMPLE: vec2<f32> = vec2<f32>(0.375, 0.125);

var<workgroup> lit_pixels: atomic<u32>;
var<workgroup> listed_point_lights: atomic<u32>;
var<workgroup> reaching_point_lights: atomic<u32>;

// The point relative to the camera at clip-space position `clip`: the
// solution of view_proj's x, y and w rows, which hold no translation (ADR
// 0014), by Cramer's rule in cross products.
fn camera_relative_point(clip: vec4<f32>) -> vec3<f32> {
    let rows = transpose(camera.view_proj);
    let x_row = rows[0].xyz;
    let y_row = rows[1].xyz;
    let w_row = rows[3].xyz;
    let y_cross_w = cross(y_row, w_row);
    return (clip.x * y_cross_w + clip.y * cross(w_row, x_row) + clip.w * cross(x_row, y_row))
        / dot(x_row, y_cross_w);
}

@compute @workgroup_size(LIGHT_COUNT_TILE_SIZE, LIGHT_COUNT_TILE_SIZE)
fn count_point_lights(
    @builtin(global_invocation_id) pixel: vec3<u32>,
    @builtin(local_invocation_index) invocation: u32
) {
    let size = textureDimensions(depth);
    if all(pixel.xy < size) {
        // The near plane's distance over the view depth (ADR 0015), and 0
        // where the frame drew nothing; clip z is the near plane's distance.
        let pixel_depth = textureLoad(depth, pixel.xy, 0);
        if pixel_depth > 0.0 {
            let near = camera.view_proj[3].z;
            let view_depth = near / pixel_depth;
            let sample = vec2<f32>(pixel.xy) + FIRST_SAMPLE;
            let normalised = vec2<f32>(
                sample.x / f32(size.x) * 2.0 - 1.0,
                1.0 - sample.y / f32(size.y) * 2.0
            );
            let clip = vec4<f32>(normalised * view_depth, near, view_depth);
            let position = camera_relative_point(clip);
            let froxel = froxel_at(sample, vec2<f32>(size), view_depth);
            let listed = froxel_lists[froxel].light_count;
            var reaching = 0u;
            for (var entry = 0u; entry < listed; entry++) {
                let point = lights[froxel_lists[froxel].lights[entry]];
                let to_light = point_light_position(point, camera.anchor, camera.offset) - position;
                reaching += u32(dot(to_light, to_light) < point.range * point.range);
            }
            atomicAdd(&lit_pixels, 1u);
            atomicAdd(&listed_point_lights, listed);
            atomicAdd(&reaching_point_lights, reaching);
        }
    }
    workgroupBarrier();
    if invocation == 0u {
        atomicAdd(&light_counts.lit_pixels, atomicLoad(&lit_pixels));
        atomicAdd(&light_counts.listed_point_lights, atomicLoad(&listed_point_lights));
        atomicAdd(&light_counts.reaching_point_lights, atomicLoad(&reaching_point_lights));
    }
}
