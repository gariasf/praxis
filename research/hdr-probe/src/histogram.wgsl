// Throwaway probe for #189 (never merged): a 256-bin histogram of log2
// luminance over the resolved HDR target, then one workgroup that averages it
// and moves the exposure toward the result. Bin 0 holds pixels too dark to
// meter.

const BIN_COUNT: u32 = 256u;
const MINIMUM_LOG2_LUMINANCE: f32 = -10.0;
const LOG2_LUMINANCE_RANGE: f32 = 22.0;
const REC709_LUMINANCE = vec3<f32>(0.2126, 0.7152, 0.0722);

struct Exposure {
    // Adapted log2 luminance, then the exposure that maps it to middle grey.
    adapted_log2_luminance: f32,
    exposure: f32,
    adaptation_rate: f32,
    _padding: f32,
}

@group(0) @binding(0) var hdr_texture: texture_2d<f32>;
@group(0) @binding(1) var<storage, read_write> histogram: array<atomic<u32>, BIN_COUNT>;
@group(0) @binding(2) var<storage, read_write> exposure: Exposure;

var<workgroup> local_bins: array<atomic<u32>, BIN_COUNT>;
var<workgroup> weighted_sums: array<f32, BIN_COUNT>;

@compute @workgroup_size(16, 16)
fn build_histogram(
    @builtin(global_invocation_id) id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
) {
    atomicStore(&local_bins[local_index], 0u);
    workgroupBarrier();
    let size = textureDimensions(hdr_texture);
    if all(id.xy < size) {
        let luminance = dot(textureLoad(hdr_texture, vec2<i32>(id.xy), 0).rgb, REC709_LUMINANCE);
        var bin = 0u;
        if luminance > exp2(MINIMUM_LOG2_LUMINANCE) {
            let position = clamp((log2(luminance) - MINIMUM_LOG2_LUMINANCE) / LOG2_LUMINANCE_RANGE, 0.0, 1.0);
            bin = u32(position * 254.0 + 1.0);
        }
        atomicAdd(&local_bins[bin], 1u);
    }
    workgroupBarrier();
    let count = atomicLoad(&local_bins[local_index]);
    if count > 0u {
        atomicAdd(&histogram[local_index], count);
    }
}

@compute @workgroup_size(256)
fn average_histogram(@builtin(local_invocation_index) index: u32) {
    let count = atomicExchange(&histogram[index], 0u);
    weighted_sums[index] = f32(count) * f32(index);
    workgroupBarrier();
    for (var stride = BIN_COUNT / 2u; stride > 0u; stride >>= 1u) {
        if index < stride {
            weighted_sums[index] += weighted_sums[index + stride];
        }
        workgroupBarrier();
    }
    if index == 0u {
        let size = textureDimensions(hdr_texture);
        let metered = max(f32(size.x * size.y) - f32(count), 1.0);
        let mean_bin = weighted_sums[0] / metered - 1.0;
        let mean_log2_luminance = mean_bin / 254.0 * LOG2_LUMINANCE_RANGE + MINIMUM_LOG2_LUMINANCE;
        exposure.adapted_log2_luminance += (mean_log2_luminance - exposure.adapted_log2_luminance) * exposure.adaptation_rate;
        exposure.exposure = 0.18 / exp2(exposure.adapted_log2_luminance);
    }
}
