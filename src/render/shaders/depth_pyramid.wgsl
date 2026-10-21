// The depth pyramid pass's entry points, which build the depth pyramid a
// level at a time: each texel holds the minimum depth, the farthest in
// reverse-Z (ADR 0015), of the texels below whose share of the screen
// overlaps its own, every sample of each pixel of the depth target below
// level 0. Its bindings are declared from their bind group's definition
// (src/render/bind_groups.rs).

// The texels of a level whose share of the screen overlaps that of a texel
// of the level above, from the first to the last along each axis.
struct OverlappedTexels {
    first: vec2<u32>,
    last: vec2<u32>,
}

// The texels of a level `below_size` texels in size that `texel` of a level
// `size` overlaps, the rows and columns of each level sharing the screen
// evenly: from ⌊texel × below / size⌋ to ⌈(texel + 1) × below / size⌉ − 1.
// Two along an axis that halves exactly, three where the level below is odd.
fn overlapped_texels(texel: vec2<u32>, size: vec2<u32>, below_size: vec2<u32>) -> OverlappedTexels {
    return OverlappedTexels(
        texel * below_size / size,
        ((texel + 1u) * below_size + size - 1u) / size - 1u,
    );
}

// The minimum depth of the depth target's four samples at `pixel`, which
// Rust asserts it has (SAMPLE_COUNT), loaded together: Metal's compiler
// does not unroll a loop over them.
fn pixel_minimum(pixel: vec2<u32>) -> f32 {
    return min(
        min(textureLoad(depth, pixel, 0), textureLoad(depth, pixel, 1)),
        min(textureLoad(depth, pixel, 2), textureLoad(depth, pixel, 3))
    );
}

// The minimum depth of the depth target's samples at the four pixels of
// `columns` and `rows`.
fn block_minimum(columns: vec2<u32>, rows: vec2<u32>) -> f32 {
    return min(
        min(pixel_minimum(vec2<u32>(columns.x, rows.x)), pixel_minimum(vec2<u32>(columns.y, rows.x))),
        min(pixel_minimum(vec2<u32>(columns.x, rows.y)), pixel_minimum(vec2<u32>(columns.y, rows.y)))
    );
}

// Level 0: a texel per invocation, the minimum of every sample of the depth
// target's pixels it overlaps: its first two columns and rows, the second
// the first again where it overlaps one alone, then a third where the
// target's side is odd.
@compute @workgroup_size(DEPTH_PYRAMID_TILE_SIZE, DEPTH_PYRAMID_TILE_SIZE)
fn reduce_depth(@builtin(global_invocation_id) texel: vec3<u32>) {
    let size = textureDimensions(level);
    if any(texel.xy >= size) {
        return;
    }
    let overlapped = overlapped_texels(texel.xy, size, textureDimensions(depth));
    let first = overlapped.first;
    let second = min(first + 1u, overlapped.last);
    let last = overlapped.last;
    var minimum = block_minimum(vec2<u32>(first.x, second.x), vec2<u32>(first.y, second.y));
    if last.x > second.x {
        minimum = min(minimum, block_minimum(vec2<u32>(last.x), vec2<u32>(first.y, second.y)));
    }
    if last.y > second.y {
        minimum = min(minimum, block_minimum(vec2<u32>(first.x, second.x), vec2<u32>(last.y)));
    }
    if all(last > second) {
        minimum = min(minimum, block_minimum(vec2<u32>(last.x), vec2<u32>(last.y)));
    }
    textureStore(level, texel.xy, vec4<f32>(minimum));
}

// The minimum depth of the level below at the four texels of `columns`
// and `rows`, loaded together.
fn level_block_minimum(columns: vec2<u32>, rows: vec2<u32>) -> f32 {
    return min(
        min(textureLoad(level_below, vec2<u32>(columns.x, rows.x), 0).r, textureLoad(level_below, vec2<u32>(columns.y, rows.x), 0).r),
        min(textureLoad(level_below, vec2<u32>(columns.x, rows.y), 0).r, textureLoad(level_below, vec2<u32>(columns.y, rows.y), 0).r)
    );
}

// Each later level: a texel per invocation, the minimum of the texels of the
// level below it overlaps, taken as level 0 takes its pixels.
@compute @workgroup_size(DEPTH_PYRAMID_TILE_SIZE, DEPTH_PYRAMID_TILE_SIZE)
fn reduce_level(@builtin(global_invocation_id) texel: vec3<u32>) {
    let size = textureDimensions(level);
    if any(texel.xy >= size) {
        return;
    }
    let overlapped = overlapped_texels(texel.xy, size, textureDimensions(level_below));
    let first = overlapped.first;
    let second = min(first + 1u, overlapped.last);
    let last = overlapped.last;
    var minimum = level_block_minimum(vec2<u32>(first.x, second.x), vec2<u32>(first.y, second.y));
    if last.x > second.x {
        minimum = min(minimum, level_block_minimum(vec2<u32>(last.x), vec2<u32>(first.y, second.y)));
    }
    if last.y > second.y {
        minimum = min(minimum, level_block_minimum(vec2<u32>(first.x, second.x), vec2<u32>(last.y)));
    }
    if all(last > second) {
        minimum = min(minimum, level_block_minimum(vec2<u32>(last.x), vec2<u32>(last.y)));
    }
    textureStore(level, texel.xy, vec4<f32>(minimum));
}
