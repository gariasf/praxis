// The scan the culling and visibility passes compact with, over a
// workgroup of SCAN_BLOCK_SIZE invocations, which is declared from Rust
// (src/render/shaders/mod.rs).

var<workgroup> scan: array<vec2<u32>, SCAN_BLOCK_SIZE>;

// `sum + addend`, or the most a u32 holds where that would pass it: past
// every capacity, as the sum it stands for is.
fn saturating_add(sum: vec2<u32>, addend: vec2<u32>) -> vec2<u32> {
    return select(sum + addend, vec2<u32>(0xffffffffu), sum > vec2<u32>(0xffffffffu) - addend);
}

// The sum of `value` over the workgroup's invocations before this one, which
// they all call together: a Hillis and Steele scan
// [ref:hillis-steele-partial-sums], read one place back.
fn exclusive_scan(invocation: u32, value: vec2<u32>) -> vec2<u32> {
    scan[invocation] = value;
    for (var stride = 1u; stride < SCAN_BLOCK_SIZE; stride *= 2u) {
        workgroupBarrier();
        var before = vec2<u32>(0u);
        if invocation >= stride {
            before = scan[invocation - stride];
        }
        workgroupBarrier();
        scan[invocation] = saturating_add(scan[invocation], before);
    }
    workgroupBarrier();
    var before = vec2<u32>(0u);
    if invocation > 0u {
        before = scan[invocation - 1u];
    }
    return before;
}
