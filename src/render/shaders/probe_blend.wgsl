struct ProbeTexel {
    first_ray: u32,
    direction: vec3<f32>,
}

fn probe_texel(position: vec2<f32>, side: u32) -> ProbeTexel {
    let texel = vec2<u32>(position);
    let column = texel.x / side;
    let level = column / PROBE_COUNTS.x;
    let slot = ((level % PROBE_LEVELS_PER_FRAME) * PROBE_COUNTS.z + texel.y / side) * PROBE_COUNTS.x
        + column % PROBE_COUNTS.x;
    return ProbeTexel(slot * PROBE_RAYS, probe_map_direction(texel % side, side));
}

// Each ray weighs by its cosine to the texel's direction [ref:majercik-2019-ddgi-probe-update].
@fragment
fn blend_irradiance(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let texel = probe_texel(position.xy, PROBE_IRRADIANCE_SIDE);
    var weighted = vec3<f32>(0.0);
    var weight_sum = 0.0;
    for (var ray = 0u; ray < PROBE_RAYS; ray++) {
        let weight = max(dot(texel.direction, probe_ray_directions[ray].xyz), 0.0);
        weighted += weight * probe_rays[texel.first_ray + ray].rgb;
        weight_sum += weight;
    }
    let irradiance = PI * PROBE_LIGHT_SCALE * weighted / max(weight_sum, 1e-6);
    return vec4<f32>(min(irradiance, vec3<f32>(LARGEST_HALF)), 0.0);
}

@fragment
fn blend_depth_moments(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let texel = probe_texel(position.xy, PROBE_DEPTH_MOMENTS_SIDE);
    var moments = vec2<f32>(0.0);
    var weight_sum = 0.0;
    for (var ray = 0u; ray < PROBE_RAYS; ray++) {
        // cos¹⁶ falls to half 16.8° off the texel's direction, about the 18° between neighbouring rays.
        let facing = max(dot(texel.direction, probe_ray_directions[ray].xyz), 0.0);
        let squared = facing * facing;
        let fourth = squared * squared;
        let weight = fourth * fourth * fourth * fourth;
        let distance = probe_rays[texel.first_ray + ray].w;
        moments += weight * vec2<f32>(distance, distance * distance);
        weight_sum += weight;
    }
    return vec4<f32>(moments / max(weight_sum, 1e-6), 0.0, 0.0);
}
