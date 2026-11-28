// The direction at the centre of `texel`: +Z at the map's centre, -Z at its corners.
fn probe_map_direction(texel: vec2<u32>, side: u32) -> vec3<f32> {
    return octahedral_decode((vec2<f32>(texel) + 0.5) / f32(side) * 2.0 - 1.0);
}
