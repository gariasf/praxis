// A visibility ID, as the depth passes write it with the visibility_buffer
// Render switch on and the material pass reads it: a triangle's place in its
// phase's draw, the draw's primitive index, with PHASE_TWO_BIT set for phase
// two's, whose draw starts where phase one's ends. Its halves fill an
// Rg16Uint sample: R32Uint has no MSAA among WebGPU's guaranteed format
// features (VISIBILITY_ID_FORMAT, src/render/targets.rs).

// The bit that marks an ID of phase two's draw.
const PHASE_TWO_BIT: u32 = 0x80000000u;

// The ID of a sample no triangle covers: the attachment's clear value.
const NO_TRIANGLE: u32 = 0xffffffffu;

fn encode_visibility_id(id: u32) -> vec2<u32> {
    return vec2<u32>(id & 0xffffu, id >> 16u);
}

fn decode_visibility_id(halves: vec4<u32>) -> u32 {
    return halves.x | (halves.y << 16u);
}
