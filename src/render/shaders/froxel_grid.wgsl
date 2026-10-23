// The Froxel grid, which the Froxel pass builds each Froxel's light list for
// (src/render/froxel_pass.rs) and the lit pass reads a pixel's lights from:
// FROXEL_COLUMNS × FROXEL_ROWS rectangles of equal size on screen, each cut
// into FROXEL_SLICES slices of view depth,
// the first nearer than FIRST_FROXEL_SLICE_DEPTH, then
// FROXEL_SLICES_PER_OCTAVE to each doubling of depth, the last reaching to
// infinity (ADR 0015). The constants are declared from Rust's
// (src/render/shaders/mod.rs).

const FROXEL_GRID_SIZE: vec2<f32> = vec2<f32>(f32(FROXEL_COLUMNS), f32(FROXEL_ROWS));

// The index of the Froxel in `slice` of the screen rectangle at column and
// row `rectangle`, counted from the lowest normalised device coordinates.
fn froxel_index(rectangle: vec2<u32>, slice: u32) -> u32 {
    return (rectangle.y * FROXEL_COLUMNS + rectangle.x) * FROXEL_SLICES + slice;
}

// The view depth where `slice` starts, in metres along the view axis.
fn froxel_slice_start(slice: u32) -> f32 {
    let octaves = (f32(slice) - 1.0) / FROXEL_SLICES_PER_OCTAVE;
    return select(FIRST_FROXEL_SLICE_DEPTH * exp2(octaves), 0.0, slice == 0u);
}

// The slice that holds view depth `depth`.
fn froxel_slice(depth: f32) -> u32 {
    let slice = floor(log2(depth / FIRST_FROXEL_SLICE_DEPTH) * FROXEL_SLICES_PER_OCTAVE) + 1.0;
    return u32(clamp(slice, 0.0, f32(FROXEL_SLICES - 1u)));
}

// The index of the Froxel that holds the point at clip-space position
// `clip`: its screen rectangle from its normalised device coordinates, its
// slice from clip w, its depth along the view axis.
fn froxel_at(clip: vec4<f32>) -> u32 {
    let rectangle = clamp(
        floor((clip.xy / clip.w * 0.5 + 0.5) * FROXEL_GRID_SIZE),
        vec2<f32>(0.0),
        FROXEL_GRID_SIZE - 1.0
    );
    return froxel_index(vec2<u32>(rectangle), froxel_slice(clip.w));
}
