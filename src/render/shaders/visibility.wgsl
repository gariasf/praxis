// The visibility pass's entry points, after the shared scan: with the
// visibility pre-pass Render switch on, once the visibility ID pass has
// written the triangle each sample shows, they mark those triangles of the
// frame's draw, one bit each, or with Meshlet granularity every triangle of
// each such triangle's Meshlet, and compact the marked triangles' packed
// indices into the compacted draw, which the lit pass draws instead of the
// frame's draw. Its bindings are declared from their bind group's
// definition (src/render/bind_groups.rs).
//
// Meshlet granularity is what reproduces the frame's draw's image on the M4
// Pro: there, which samples a triangle under a pixel covers depends on the
// triangles drawn beside it, so a draw of the triangles some sample shows,
// and nothing else, loses samples of such triangles; a draw of their whole
// Meshlets does not, on the Reference scenes checked.
//
// The compaction keeps the frame's draw's order, a scan over the bit words,
// never the order atomics land: which triangles precede others in a draw
// changes the derivatives some pixels sample their Textures with.

// The indirect arguments of one indexed draw, as the GPU writes them here,
// without the culling pass's atomics.
struct DrawArguments {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}

// The frame's draw as the culling pass lays it out (`FrameDraw`,
// culling.wgsl), read here for its index count.
struct FrameDrawCounts {
    draw: DrawArguments,
    instances_drawn: u32,
    instances_overflowed: u32,
    phase_two_draw: DrawArguments,
    phase_two_offsets: vec2<u32>,
}

// An ID of a sample no triangle covers: the attachment's clear value.
const NO_TRIANGLE: u32 = 0xffffffffu;

// Whether marks are of Meshlet records rather than triangles, as the
// Render switch sets it: every triangle of a Meshlet some sample shows is
// compacted.
override MESHLET_GRANULARITY: bool;

// The Meshlet record triangle `triangle` of the frame's draw is drawn with.
fn record_of(triangle: u32) -> u32 {
    return packed_indices[3u * triangle] >> MESHLET_VERTEX_BITS;
}

// Marks the triangle `id` names, if any, or its Meshlet record.
fn mark(id: u32) {
    if id == NO_TRIANGLE {
        return;
    }
    if MESHLET_GRANULARITY {
        let record = record_of(id);
        atomicOr(&record_bits[record / 32u], 1u << (record % 32u));
    } else {
        atomicOr(&triangle_bits[id / 32u], 1u << (id % 32u));
    }
}

// With MESHLET_GRANULARITY, turns the record marks of `word`'s triangles
// into triangle marks, which the scan and compaction then read.
fn mark_word_from_records(word: u32) {
    let triangles = frame_draw.draw.index_count / 3u;
    var bits = 0u;
    for (var bit = 0u; bit < 32u; bit++) {
        let triangle = 32u * word + bit;
        if triangle < triangles {
            let record = record_of(triangle);
            if (atomicLoad(&record_bits[record / 32u]) >> (record % 32u) & 1u) == 1u {
                bits |= 1u << bit;
            }
        }
    }
    atomicStore(&triangle_bits[word], bits);
}

// The ID the visibility IDs hold at `sample` of `pixel`.
fn id_at(pixel: vec2<u32>, sample: u32) -> u32 {
    let halves = textureLoad(visibility_ids, pixel, sample).rg;
    return halves.x | (halves.y << 16u);
}

// A workgroup per tile of pixels, an invocation per pixel: marks the
// triangle of each of its four samples, which Rust asserts it has
// (SAMPLE_COUNT), loaded together since Metal's compiler does not unroll a
// loop over them. A sample whose triangle the sample before it shows marks
// nothing again.
@compute @workgroup_size(VISIBILITY_TILE_SIZE, VISIBILITY_TILE_SIZE)
fn mark_triangles(@builtin(global_invocation_id) pixel: vec3<u32>) {
    if any(pixel.xy >= textureDimensions(visibility_ids)) {
        return;
    }
    let ids = vec4<u32>(
        id_at(pixel.xy, 0u),
        id_at(pixel.xy, 1u),
        id_at(pixel.xy, 2u),
        id_at(pixel.xy, 3u)
    );
    mark(ids.x);
    if ids.y != ids.x {
        mark(ids.y);
    }
    if ids.z != ids.y {
        mark(ids.z);
    }
    if ids.w != ids.z {
        mark(ids.w);
    }
}

// The words of triangle bits the frame's draw reaches: past them, every
// bit is clear.
fn words_drawn() -> u32 {
    return (frame_draw.draw.index_count / 3u + 31u) / 32u;
}

// A workgroup per block of SCAN_BLOCK_SIZE words of triangle bits: each
// word's marked triangles, summed over the block's earlier words into its
// offset, and the block's sum after every word's.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn offset_words(
    @builtin(workgroup_id) block: vec3<u32>,
    @builtin(local_invocation_index) invocation: u32
) {
    let word_capacity = arrayLength(&triangle_bits);
    let word = block.x * SCAN_BLOCK_SIZE + invocation;
    var count = 0u;
    if word < min(word_capacity, words_drawn()) {
        if MESHLET_GRANULARITY {
            mark_word_from_records(word);
        }
        count = countOneBits(atomicLoad(&triangle_bits[word]));
    }
    let offset = exclusive_scan(invocation, vec2<u32>(count, 0u)).x;
    if word < word_capacity {
        triangle_offsets[word] = offset;
    }
    if invocation == SCAN_BLOCK_SIZE - 1u {
        triangle_offsets[word_capacity + block.x] = offset + count;
    }
}

// One workgroup: each block's offset, the sum of the earlier blocks', in
// place of its own sum, as the culling pass's `offset_blocks` sums them; and
// the compacted draw's arguments, its index count three per marked triangle.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn offset_blocks(@builtin(local_invocation_index) invocation: u32) {
    let word_capacity = arrayLength(&triangle_bits);
    let block_count = (word_capacity + SCAN_BLOCK_SIZE - 1u) / SCAN_BLOCK_SIZE;
    let blocks_per_invocation = (block_count + SCAN_BLOCK_SIZE - 1u) / SCAN_BLOCK_SIZE;
    let first_block = invocation * blocks_per_invocation;
    var sum = 0u;
    for (var step = 0u; step < blocks_per_invocation; step++) {
        if first_block + step < block_count {
            sum += triangle_offsets[word_capacity + first_block + step];
        }
    }
    var offset = exclusive_scan(invocation, vec2<u32>(sum, 0u)).x;
    for (var step = 0u; step < blocks_per_invocation; step++) {
        let block = word_capacity + first_block + step;
        if first_block + step < block_count {
            let block_sum = triangle_offsets[block];
            triangle_offsets[block] = offset;
            offset += block_sum;
        }
    }
    if invocation == SCAN_BLOCK_SIZE - 1u {
        compacted_draw = DrawArguments(3u * offset, 1u, 0u, 0, 0u);
    }
}

// An invocation per word of triangle bits: copies each marked triangle's
// three packed indices to its place in the compacted draw, in triangle
// order.
@compute @workgroup_size(SCAN_BLOCK_SIZE)
fn compact_triangles(
    @builtin(workgroup_id) block: vec3<u32>,
    @builtin(local_invocation_index) invocation: u32
) {
    let word_capacity = arrayLength(&triangle_bits);
    let word = block.x * SCAN_BLOCK_SIZE + invocation;
    if word >= min(word_capacity, words_drawn()) {
        return;
    }
    var bits = atomicLoad(&triangle_bits[word]);
    var slot = triangle_offsets[word_capacity + block.x] + triangle_offsets[word];
    while bits != 0u {
        let triangle = 32u * word + firstTrailingBit(bits);
        for (var corner = 0u; corner < 3u; corner++) {
            compacted_indices[3u * slot + corner] = packed_indices[3u * triangle + corner];
        }
        slot += 1u;
        bits &= bits - 1u;
    }
}
