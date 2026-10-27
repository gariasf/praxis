// The visibility ID pass's fragment entry point, after the main pass's
// parts: with the visibility pre-pass Render switch on, a pass after both
// phases' depth passes draws the frame's draw again at depth equal, as the
// lit pass does, and writes the triangle each sample shows, which the
// visibility pass reads (src/render/visibility_pass.rs). The ID is the
// triangle's place in the frame's draw: the draw's primitive index. The ID's
// halves fill an Rg16Uint sample: R32Uint has no MSAA among WebGPU's
// guaranteed format features.
//
// The IDs are not written in the depth passes themselves: on the M4 Pro, at
// a sample two coplanar triangles sharing an edge both reach, the ID a
// depth-writing pass keeps is not the triangle whose fragment the lit pass's
// equal test then passes. Written at depth equal, in the lit pass's own
// depth state and draw order, the IDs name the triangles the lit pass shows.

@fragment
fn fs_visibility(@builtin(primitive_index) triangle: u32) -> @location(0) vec2<u32> {
    return vec2<u32>(triangle & 0xffffu, triangle >> 16u);
}
