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

// The pixels along each side of the tile level 0's workgroups reduce: twice
// its texels, which each overlap the two pixels from twice their index, and a
// third, the next texel's first, where the depth target's side is odd.
const DEPTH_PYRAMID_PIXEL_TILE_SIZE = 2u * DEPTH_PYRAMID_TILE_SIZE;

// The minimum depth of each pixel of a workgroup's tile and, where a side of
// the depth target is odd, of the column or row past it, which the tile's
// last texel along that side overlaps.
var<workgroup> pixel_minima: array<array<f32, DEPTH_PYRAMID_PIXEL_TILE_SIZE + 1u>, DEPTH_PYRAMID_PIXEL_TILE_SIZE + 1u>;

// The minimum depth of `pixel`'s samples, or 1, the nearest, which leaves a
// minimum as it is, past the depth target's edge.
fn pixel_minimum_within(pixel: vec2<u32>, depth_size: vec2<u32>) -> f32 {
    if any(pixel >= depth_size) {
        return 1.0;
    }
    return pixel_minimum(pixel);
}

// The minimum of the four pixel minima from `corner` of the workgroup's.
fn shared_block_minimum(corner: vec2<u32>) -> f32 {
    return min(
        min(pixel_minima[corner.y][corner.x], pixel_minima[corner.y][corner.x + 1u]),
        min(pixel_minima[corner.y + 1u][corner.x], pixel_minima[corner.y + 1u][corner.x + 1u])
    );
}

// Level 0: an invocation per pixel of a tile, which shares the minimum of
// its samples, then a texel per invocation of the tile's first quarter, the
// minimum of the shared minima of the pixels it overlaps: as the later
// levels' overlapped_texels counts them, from twice its index, two along a
// side of the depth target that is even and three along one that is odd,
// none of them past the edge, which 1 leaves out.
@compute @workgroup_size(DEPTH_PYRAMID_PIXEL_TILE_SIZE, DEPTH_PYRAMID_PIXEL_TILE_SIZE)
fn reduce_depth(
    @builtin(global_invocation_id) pixel: vec3<u32>,
    @builtin(local_invocation_id) local: vec3<u32>,
    @builtin(workgroup_id) workgroup: vec3<u32>,
) {
    let depth_size = textureDimensions(depth);
    let odd = depth_size % 2u == vec2<u32>(1u);
    pixel_minima[local.y][local.x] = pixel_minimum_within(pixel.xy, depth_size);
    let past_tile = (workgroup.xy + 1u) * DEPTH_PYRAMID_PIXEL_TILE_SIZE;
    if odd.x && local.x == 0u {
        pixel_minima[local.y][DEPTH_PYRAMID_PIXEL_TILE_SIZE] =
            pixel_minimum_within(vec2<u32>(past_tile.x, pixel.y), depth_size);
    }
    if odd.y && local.y == 0u {
        pixel_minima[DEPTH_PYRAMID_PIXEL_TILE_SIZE][local.x] =
            pixel_minimum_within(vec2<u32>(pixel.x, past_tile.y), depth_size);
    }
    if all(odd) && all(local.xy == vec2<u32>(0u)) {
        pixel_minima[DEPTH_PYRAMID_PIXEL_TILE_SIZE][DEPTH_PYRAMID_PIXEL_TILE_SIZE] =
            pixel_minimum_within(past_tile, depth_size);
    }
    workgroupBarrier();
    let texel = workgroup.xy * DEPTH_PYRAMID_TILE_SIZE + local.xy;
    let size = textureDimensions(level);
    if any(local.xy >= vec2<u32>(DEPTH_PYRAMID_TILE_SIZE)) || any(texel >= size) {
        return;
    }
    let first = 2u * local.xy;
    let last = first + 2u;
    var minimum = shared_block_minimum(first);
    if odd.x {
        minimum = min(minimum, min(pixel_minima[first.y][last.x], pixel_minima[first.y + 1u][last.x]));
    }
    if odd.y {
        minimum = min(minimum, min(pixel_minima[last.y][first.x], pixel_minima[last.y][first.x + 1u]));
    }
    if all(odd) {
        minimum = min(minimum, pixel_minima[last.y][last.x]);
    }
    textureStore(level, texel, vec4<f32>(minimum));
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
// level below it overlaps: its first two columns and rows, the second the
// first again where it overlaps one alone, then a third where the level
// below is odd.
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
