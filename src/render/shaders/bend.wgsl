// The bend pass's entry point, after the shared parts (src/render/bend_pass.rs),
// a stand-in for skinning: a row of workgroups per bend and an
// invocation per vertex. Each vertex of the bend's rest Mesh, whose +Y axis
// runs up its middle from y = 0, keeps its offset from the middle at its
// height, turned as the middle bent into a circular arc turns there: about
// the horizontal axis across the bend's azimuth, by the curvature times the
// height. Its tangent frame turns with it, and it lands in the bent Mesh's
// range of the vertex Arena. Its bindings are declared from their bind
// group's definition (src/render/bind_groups.rs).

// A bend as the CPU writes it: where its rest Mesh's vertices and its bent
// Mesh's start in the vertex Arena, how many each holds, the direction the
// middle bends towards, about +Y from +X towards +Z, and the radians it
// turns through per metre of its height.
struct BendData {
    rest_first_vertex: u32,
    first_vertex: u32,
    vertex_count: u32,
    azimuth: f32,
    curvature: f32,
}

// sin(angle) / angle, sinc, by its Taylor series below 0.5 radians, whose
// first term left out, angle⁸ / 9!, is under 1.1e-8 there: Metal compiles
// with fast math [ref:msl-4.1-fast-math], which leaves sin an absolute
// error [ref:msl-4.1-sin-fast-accuracy] that dividing by a small angle
// would magnify.
fn sinc(angle: f32) -> f32 {
    let squared = angle * angle;
    let series = 1.0 - squared / 6.0 * (1.0 - squared / 20.0 * (1.0 - squared / 42.0));
    return select(sin(angle) / angle, series, abs(angle) < 0.5);
}

// The quaternion product of `outer` and `inner`: `inner`'s rotation, then
// `outer`'s.
fn quaternion_product(outer: vec4<f32>, inner: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        outer.w * inner.xyz + inner.w * outer.xyz + cross(outer.xyz, inner.xyz),
        outer.w * inner.w - dot(outer.xyz, inner.xyz)
    );
}

@compute @workgroup_size(BEND_WORKGROUP_SIZE)
fn bend_vertices(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let bend = bends[invocation.y];
    if invocation.x >= bend.vertex_count {
        return;
    }
    var vertex = vertices[bend.rest_first_vertex + invocation.x];
    let height = vertex.position[1];
    // At this height the middle has turned through twice the half angle,
    // from +Y towards the azimuth, about +Y × toward. The arc's chord from
    // the foot runs at the half angle, its length the height times
    // sin(half) / half.
    let half_angle = 0.5 * bend.curvature * height;
    let toward = vec3<f32>(cos(bend.azimuth), 0.0, sin(bend.azimuth));
    let rotation = vec4<f32>(
        vec3<f32>(toward.z, 0.0, -toward.x) * sin(half_angle),
        cos(half_angle)
    );
    let chord = height * sinc(half_angle);
    let middle = chord * (vec3<f32>(0.0, cos(half_angle), 0.0) + toward * sin(half_angle));
    let offset = vec3<f32>(vertex.position[0], 0.0, vertex.position[2]);
    let position = middle + rotate(rotation, offset);
    vertex.position = array<f32, 3>(position.x, position.y, position.z);

    // The QTangent turns with it, stored as the cooker stores one: its w
    // made positive and kept off zero, then given the bitangent sign
    // (ADR 0018).
    let stored = unpack_qtangent(vertex);
    let bitangent_sign = select(1.0, -1.0, stored.w < 0.0);
    var turned = quaternion_product(rotation, normalize(stored) * bitangent_sign);
    turned *= select(1.0, -1.0, turned.w < 0.0);
    if turned.w < QTANGENT_SMALLEST_W {
        let axis_length = sqrt(1.0 - QTANGENT_SMALLEST_W * QTANGENT_SMALLEST_W);
        turned = vec4<f32>(normalize(turned.xyz) * axis_length, QTANGENT_SMALLEST_W);
    }
    turned *= bitangent_sign;
    vertex.qtangent = array<u32, 2>(pack2x16snorm(turned.xy), pack2x16snorm(turned.zw));
    vertices[bend.first_vertex + invocation.x] = vertex;
}
