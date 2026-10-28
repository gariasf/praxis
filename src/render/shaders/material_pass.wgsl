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
// clipping. The sum is 0 only where the point's ray runs parallel to the
// triangle's plane, which a quad's neighbour of a pixel seen at a grazing
// angle can reach: the weights are then left unnormalised, finite.
fn barycentric_weights(edges: mat3x3<f32>, point: vec3<f32>) -> vec3<f32> {
    let unnormalised = point * edges;
    let sum = unnormalised.x + unnormalised.y + unnormalised.z;
    return unnormalised / select(sum, 1.0, sum == 0.0);
}

// The attribute whose values at the triangle's corners are `first`,
// `second` and `third`, at the point whose weights are `weights`.
fn interpolate2(
    first: vec2<f32>,
    second: vec2<f32>,
    third: vec2<f32>,
    weights: vec3<f32>
) -> vec2<f32> {
    return first * weights.x + second * weights.y + third * weights.z;
}

fn interpolate3(
    first: vec3<f32>,
    second: vec3<f32>,
    third: vec3<f32>,
    weights: vec3<f32>
) -> vec3<f32> {
    return first * weights.x + second * weights.y + third * weights.z;
}

fn interpolate4(
    first: vec4<f32>,
    second: vec4<f32>,
    third: vec4<f32>,
    weights: vec3<f32>
) -> vec4<f32> {
    return first * weights.x + second * weights.y + third * weights.z;
}

// The shading normal of the triangle whose corners are `first_corner`,
// `second_corner` and `third_corner`, at the point whose weights are
// `weights` and whose UV is `uv`, `material`'s normal Texture sampled there
// with the UV's gradients.
fn corner_shading_normal(
    first_corner: VertexOutput,
    second_corner: VertexOutput,
    third_corner: VertexOutput,
    weights: vec3<f32>,
    uv: vec2<f32>,
    uv_gradient_x: vec2<f32>,
    uv_gradient_y: vec2<f32>,
    material: MaterialData
) -> vec3<f32> {
    let normal_sample = textureSampleGrad(
        normal_textures[material.layers.y],
        data_sampler,
        uv,
        uv_gradient_x,
        uv_gradient_y
    ).rg;
    return shading_normal(
        interpolate3(first_corner.normal, second_corner.normal, third_corner.normal, weights),
        interpolate4(
            first_corner.tangent_world,
            second_corner.tangent_world,
            third_corner.tangent_world,
            weights
        ),
        normal_sample,
        material.metallic_roughness.z
    );
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
    // Phase one's draw starts at index 0, phase two's where it ends.
    var triangle = id & ~PHASE_TWO_BIT;
    if (id & PHASE_TWO_BIT) != 0u {
        triangle += material_pass.phase_two_first_index / 3u;
    }
    let first_index = 3u * triangle;
    let first_corner = vertex_output(packed_indices[first_index]);
    let second_corner = vertex_output(packed_indices[first_index + 1u]);
    let third_corner = vertex_output(packed_indices[first_index + 2u]);
    let rows = mat3x3<f32>(
        first_corner.clip_position.xyw,
        second_corner.clip_position.xyw,
        third_corner.clip_position.xyw
    );
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

    let uv = interpolate2(first_corner.uv, second_corner.uv, third_corner.uv, weights);
    let uv_origin = interpolate2(first_corner.uv, second_corner.uv, third_corner.uv, origin);
    let uv_right = interpolate2(first_corner.uv, second_corner.uv, third_corner.uv, right);
    let uv_down = interpolate2(first_corner.uv, second_corner.uv, third_corner.uv, down);
    let uv_gradient_x = uv_right - uv_origin;
    let uv_gradient_y = uv_down - uv_origin;

    // The flat Material ID comes from the first corner, as the lit pass's.
    let material = materials[first_corner.material_id];
    let albedo = textureSampleGrad(
        albedo_textures[material.layers.x],
        colour_sampler,
        uv,
        uv_gradient_x,
        uv_gradient_y
    );
    let metallic_roughness = textureSampleGrad(
        metallic_roughness_textures[material.layers.z],
        data_sampler,
        uv,
        uv_gradient_x,
        uv_gradient_y
    ).rg;
    let occlusion = textureSampleGrad(
        occlusion_textures[material.layers.w],
        data_sampler,
        uv,
        uv_gradient_x,
        uv_gradient_y
    ).r;
    let emissive = textureSampleGrad(
        emissive_textures[material.extra.x],
        colour_sampler,
        uv,
        uv_gradient_x,
        uv_gradient_y
    ).rgb;
    let normal = corner_shading_normal(
        first_corner,
        second_corner,
        third_corner,
        weights,
        uv,
        uv_gradient_x,
        uv_gradient_y,
        material
    );
    let normal_origin = corner_shading_normal(
        first_corner,
        second_corner,
        third_corner,
        origin,
        uv_origin,
        uv_gradient_x,
        uv_gradient_y,
        material
    );
    let normal_right = corner_shading_normal(
        first_corner,
        second_corner,
        third_corner,
        right,
        uv_right,
        uv_gradient_x,
        uv_gradient_y,
        material
    );
    let normal_down = corner_shading_normal(
        first_corner,
        second_corner,
        third_corner,
        down,
        uv_down,
        uv_gradient_x,
        uv_gradient_y,
        material
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
        first_corner.camera_relative_position,
        second_corner.camera_relative_position,
        third_corner.camera_relative_position,
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
        // The bits of the samples that show this ID.
        let sample_bits = select(vec4<u32>(0u), vec4<u32>(1u, 2u, 4u, 8u), ids == vec4<u32>(id));
        let matching = dot(sample_bits, vec4<u32>(1u));
        var shaded = cleared;
        if id != NO_TRIANGLE {
            shaded = shade_triangle(id, pixel_centre, vec2<f32>(size));
        }
        colour += shaded * f32(countOneBits(matching));
        unshaded &= ~matching;
    }
    textureStore(hdr_target, pixel, colour / 4.0);
}
