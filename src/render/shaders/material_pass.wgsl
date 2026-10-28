// The material pass's entry point, after the parts the lit pass shades with:
// with the visibility_buffer Render switch on, it shades each pixel from the
// IDs the depth passes wrote, in place of the lit pass, and writes the HDR
// target. Each triangle some sample of a pixel shows is shaded once, at the
// pixel's centre, as the lit pass shades it under 4x MSAA, and weighted by
// the samples it covers, the samples no triangle covers taking the
// background, as the lit pass's resolve averages them. Its bindings are
// declared from their bind groups' definitions (src/render/bind_groups.rs).

// The pass's uniform: the clear colour, exposed, as the CPU writes it, and
// the first index of phase two's draw, which the GPU copies from the frame's
// draw.
struct MaterialPassUniform {
    background: vec4<f32>,
    phase_two_first_index: u32,
}

// The perspective-correct barycentric weights of the point whose device
// coordinates are `point`, (x, y, 1), on the triangle whose edge vectors are
// the columns of `edges`. Row i of M holding corner i's clip x, y and w, a
// point of the triangle with weights λ projects to `point` exactly when λᵀM
// is a multiple of `point`, so λ is a multiple of `point` times M's inverse,
// whose column i is the cross product of rows i + 1 and i + 2 over M's
// determinant; normalising the weights to sum to 1 drops both factors. No
// corner is divided by its w, so a corner behind the camera needs no
// clipping.
fn barycentric_weights(edges: mat3x3<f32>, point: vec3<f32>) -> vec3<f32> {
    let unnormalised = point * edges;
    return unnormalised / (unnormalised.x + unnormalised.y + unnormalised.z);
}

// The attribute whose values at the triangle's corners are `a`, `b` and `c`,
// at the point whose weights are `weights`.
fn interpolate2(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, weights: vec3<f32>) -> vec2<f32> {
    return a * weights.x + b * weights.y + c * weights.z;
}

fn interpolate3(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, weights: vec3<f32>) -> vec3<f32> {
    return a * weights.x + b * weights.y + c * weights.z;
}

fn interpolate4(a: vec4<f32>, b: vec4<f32>, c: vec4<f32>, weights: vec3<f32>) -> vec4<f32> {
    return a * weights.x + b * weights.y + c * weights.z;
}

// The device coordinates, (x, y, 1), of the point `point`, in pixels from
// the top left corner of a target of `size`.
fn device_point(point: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    return vec3<f32>(point.x / size.x * 2.0 - 1.0, 1.0 - point.y / size.y * 2.0, 1.0);
}

// The colour the triangle whose visibility ID is `id` shows through the
// pixel whose centre is `pixel_centre`, in a target of `size`, as the lit
// pass's fragment shader shades it there. Where the lit pass differentiates
// across its 2×2 quad, this evaluates the triangle's attributes, on its
// plane, at the centres of the quad's top left pixel and the pixels right of
// and below it, as helper lanes would: the UVs' differences are the
// gradients each Texture is sampled with, and the shading normal's, the
// normal Texture sampled at those UVs too, widen roughness. Differences
// across the quad rather than from each pixel match the lit pass's image
// on the M4 Pro far more closely (docs/performance.md).
fn shade_triangle(id: u32, pixel_centre: vec2<f32>, size: vec2<f32>) -> vec4<f32> {
    var triangle = id & ~PHASE_TWO_BIT;
    if (id & PHASE_TWO_BIT) != 0u {
        triangle += material_pass.phase_two_first_index / 3u;
    }
    let first_index = 3u * triangle;
    let a = vertex_output(packed_indices[first_index]);
    let b = vertex_output(packed_indices[first_index + 1u]);
    let c = vertex_output(packed_indices[first_index + 2u]);
    let rows = mat3x3<f32>(a.clip_position.xyw, b.clip_position.xyw, c.clip_position.xyw);
    let edges = mat3x3<f32>(
        cross(rows[1], rows[2]),
        cross(rows[2], rows[0]),
        cross(rows[0], rows[1])
    );
    let weights = barycentric_weights(edges, device_point(pixel_centre, size));
    let quad_corner = floor(pixel_centre * 0.5) * 2.0 + 0.5;
    let origin = barycentric_weights(edges, device_point(quad_corner, size));
    let right = barycentric_weights(edges, device_point(quad_corner + vec2<f32>(1.0, 0.0), size));
    let down = barycentric_weights(edges, device_point(quad_corner + vec2<f32>(0.0, 1.0), size));

    let uv = interpolate2(a.uv, b.uv, c.uv, weights);
    let uv_origin = interpolate2(a.uv, b.uv, c.uv, origin);
    let uv_right = interpolate2(a.uv, b.uv, c.uv, right);
    let uv_down = interpolate2(a.uv, b.uv, c.uv, down);
    let uv_dx = uv_right - uv_origin;
    let uv_dy = uv_down - uv_origin;

    // The flat Material ID comes from the first corner, as the lit pass's.
    let material = materials[a.material_id];
    let albedo = textureSampleGrad(
        albedo_textures[material.layers.x],
        colour_sampler,
        uv,
        uv_dx,
        uv_dy
    );
    let metallic_roughness = textureSampleGrad(
        metallic_roughness_textures[material.layers.z],
        data_sampler,
        uv,
        uv_dx,
        uv_dy
    ).rg;
    let occlusion = textureSampleGrad(
        occlusion_textures[material.layers.w],
        data_sampler,
        uv,
        uv_dx,
        uv_dy
    ).r;
    let emissive = textureSampleGrad(
        emissive_textures[material.extra.x],
        colour_sampler,
        uv,
        uv_dx,
        uv_dy
    ).rgb;
    let normal_layer = material.layers.y;
    let scale = material.metallic_roughness.z;
    let normal = shading_normal(
        interpolate3(a.normal, b.normal, c.normal, weights),
        interpolate4(a.tangent_world, b.tangent_world, c.tangent_world, weights),
        textureSampleGrad(normal_textures[normal_layer], data_sampler, uv, uv_dx, uv_dy).rg,
        scale
    );
    let normal_right = shading_normal(
        interpolate3(a.normal, b.normal, c.normal, right),
        interpolate4(a.tangent_world, b.tangent_world, c.tangent_world, right),
        textureSampleGrad(normal_textures[normal_layer], data_sampler, uv_right, uv_dx, uv_dy).rg,
        scale
    );
    let normal_down = shading_normal(
        interpolate3(a.normal, b.normal, c.normal, down),
        interpolate4(a.tangent_world, b.tangent_world, c.tangent_world, down),
        textureSampleGrad(normal_textures[normal_layer], data_sampler, uv_down, uv_dx, uv_dy).rg,
        scale
    );
    let normal_origin = shading_normal(
        interpolate3(a.normal, b.normal, c.normal, origin),
        interpolate4(a.tangent_world, b.tangent_world, c.tangent_world, origin),
        textureSampleGrad(normal_textures[normal_layer], data_sampler, uv_origin, uv_dx, uv_dy).rg,
        scale
    );
    let surface = decode_surface(
        material,
        albedo,
        metallic_roughness,
        occlusion,
        emissive,
        normal_right - normal_origin,
        normal_down - normal_origin
    );
    let position = interpolate3(
        a.camera_relative_position,
        b.camera_relative_position,
        c.camera_relative_position,
        weights
    );
    return lit_colour(surface, normal, position, pixel_centre);
}

// An invocation per pixel: shades each distinct ID among the pixel's four
// samples once, in sample order, and writes their average, each weighted by
// the samples it covers, into the HDR target.
@compute @workgroup_size(MATERIAL_TILE_SIZE, MATERIAL_TILE_SIZE)
fn shade_pixels(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let size = textureDimensions(hdr_target);
    if any(invocation.xy >= size) {
        return;
    }
    let pixel = invocation.xy;
    let ids = vec4<u32>(
        decode_visibility_id(textureLoad(visibility_ids, pixel, 0)),
        decode_visibility_id(textureLoad(visibility_ids, pixel, 1)),
        decode_visibility_id(textureLoad(visibility_ids, pixel, 2)),
        decode_visibility_id(textureLoad(visibility_ids, pixel, 3))
    );
    let pixel_centre = vec2<f32>(pixel) + 0.5;
    // The clear colour, clamped and weighted as the lit pass's attachment
    // clears to it.
    let cleared = vec4<f32>(
        resolve_weighted(min(material_pass.background.rgb, vec3<f32>(LARGEST_HALF))),
        material_pass.background.a
    );
    var colour = vec4<f32>(0.0);
    var unshaded = 0xfu;
    while unshaded != 0u {
        let id = ids[firstTrailingBit(unshaded)];
        let matching = dot(select(vec4<u32>(0u), vec4<u32>(1u, 2u, 4u, 8u), ids == vec4<u32>(id)), vec4<u32>(1u));
        var shaded = cleared;
        if id != NO_TRIANGLE {
            shaded = shade_triangle(id, pixel_centre, vec2<f32>(size));
        }
        colour += shaded * f32(countOneBits(matching));
        unshaded &= ~matching;
    }
    textureStore(hdr_target, pixel, colour / 4.0);
}
