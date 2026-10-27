// The shading terms, shared by every pass that lights a surface
// (src/render/shaders/mod.rs): the surface decode, glTF 2.0's
// metallic-roughness BRDF, written from its Appendix B (Khronos, CC BY 4.0),
// and KHR_lights_punctual's falloff. Appendix B is the specification
// source's, which revised the published 2.0.1's informative sample
// implementation: 2.0.1 gives a separable Smith G and a simplified mix of
// the two lobes [ref:gltf-2.0.1-sample-brdf].
// https://github.com/KhronosGroup/glTF/blob/7cee0fcddd5730e90d488bfb5aa2e52171483dde/specification/2.0/Specification.adoc
// https://github.com/KhronosGroup/glTF/blob/854a757a61e6bc0dde91631579b9d3d2e1743df9/extensions/2.0/Khronos/KHR_lights_punctual/README.md

const PI: f32 = 3.14159265;

// Appendix B's reflectance of a dielectric at normal incidence,
// ((1 − ior) / (1 + ior))² at its index of refraction of 1.5
// [ref:gltf-2.0-metal-and-dielectric-brdf].
const DIELECTRIC_BASE_REFLECTANCE: f32 = 0.04;

// As roughness, and α = roughness² with it, falls to 0, D collapses to a
// delta and the BRDF becomes singular, dividing by zero, so Appendix B's
// source says never to use α = 0 but to clamp it to a small positive value
// [ref:gltf-2.0-zero-roughness].
const MINIMUM_ROUGHNESS: f32 = 0.04;

// A surface point, as the shading terms read it.
struct Surface {
    base_color: vec3<f32>,
    // The base colour's alpha, which the lit pass writes out.
    opacity: f32,
    metallic: f32,
    roughness: f32,
    occlusion: f32,
    emissive: vec3<f32>,
}

// A surface point from its Material's factors and the texels of its Layers,
// each factor scaling its texel (glTF 2.0, "Metallic-Roughness Material"
// [ref:gltf-2.0-metallic-roughness]): cooked metallic-roughness holds roughness
// in red and metallic in green (BC5, ADR 0018). Roughness keeps to
// MINIMUM_ROUGHNESS or above.
fn decode_surface(
    material: MaterialData,
    albedo: vec4<f32>,
    metallic_roughness: vec2<f32>,
    occlusion: f32,
    emissive: vec3<f32>
) -> Surface {
    return Surface(
        albedo.rgb * material.base_color.rgb,
        albedo.a * material.base_color.a,
        metallic_roughness.g * material.metallic_roughness.x,
        max(metallic_roughness.r * material.metallic_roughness.y, MINIMUM_ROUGHNESS),
        occlusion,
        emissive * material.emissive.rgb,
    );
}

// Schlick's Fresnel weight, (1 − |V·H|)⁵ (Appendix B, "Fresnel"
// [ref:gltf-2.0-fresnel]): how far F climbs from the reflectance at normal
// incidence, F0, towards 1 as V turns from H. Multiplied out, not pow, which
// has no value for the negative base that a |V·H| rounded past 1 gives.
fn fresnel_weight(view_dot_half: f32) -> f32 {
    let complement = 1.0 - abs(view_dot_half);
    let complement_squared = complement * complement;
    return complement_squared * complement_squared * complement;
}

// D: the Trowbridge-Reitz (GGX) distribution of microfacet normals at width
// alpha (Appendix B, "Specular BRDF" [ref:gltf-2.0-specular-brdf]),
//   α² χ⁺(N·H) / (π ((N·H)² (α² − 1) + 1)²),
// its (N·H)² (α² − 1) + 1 summed as (N·H)² α² + (1 − N·H)(1 + N·H), terms
// f32 keeps to a few ulps where the sum falls to α², at N·H = 1.
fn distribution_ggx(normal_dot_half: f32, alpha: f32) -> f32 {
    let alpha_squared = alpha * alpha;
    let denominator = normal_dot_half * normal_dot_half * alpha_squared
        + (1.0 - normal_dot_half) * (1.0 + normal_dot_half);
    let distribution = alpha_squared / (PI * denominator * denominator);
    return select(0.0, distribution, normal_dot_half > 0.0);
}

// V: Smith's height-correlated masking-shadowing G over 4 |N·L| |N·V|, at
// width alpha (Appendix B, "Specular BRDF"),
//   χ⁺(H·L) χ⁺(H·V) / (2 (|N·V| √(α² + (1 − α²)(N·L)²) + |N·L| √(α² + (1 − α²)(N·V)²))),
// where H·L equals H·V, H being halfway between L and V. Where N·L and N·V
// are both 0, its denominator is 0 and it is 0, so that a light along the
// surface, which reflects none, gives none.
fn smith_visibility(
    normal_dot_light: f32,
    normal_dot_view: f32,
    view_dot_half: f32,
    alpha: f32
) -> f32 {
    let alpha_squared = alpha * alpha;
    // The denominator's two terms hold the light's shadowing and the view's
    // masking, each under its root.
    let shadowing_term = abs(normal_dot_view)
        * sqrt(alpha_squared + (1.0 - alpha_squared) * normal_dot_light * normal_dot_light);
    let masking_term = abs(normal_dot_light)
        * sqrt(alpha_squared + (1.0 - alpha_squared) * normal_dot_view * normal_dot_view);
    let denominator = 2.0 * (shadowing_term + masking_term);
    return select(0.0, 1.0 / denominator, view_dot_half > 0.0 && denominator > 0.0);
}

// A cooked normal Texture stores a unit normal's x and y in [0, 1], in BC5's
// two channels (ADR 0018); z is rebuilt as the positive root of
// x² + y² + z² = 1, and is 0 where block error carries x and y past the
// unit circle. x and y are then multiplied by the Material's normal scale,
// (sample · 2 − 1) ⊙ ⟨scale, scale, 1⟩ (glTF 2.0, "Normal Texture"
// [ref:gltf-2.0-normal-texture-scale]), which glTF normalises last: the lit
// pass does, once its orthonormal tangent frame has turned the result into
// World space. At a scale of 0, x and y vanish and z is 1, so a texel
// without z flattens like the rest instead of normalising a zero vector.
fn tangent_space_normal(encoded: vec2<f32>, scale: f32) -> vec3<f32> {
    let x_and_y = encoded * 2.0 - vec2<f32>(1.0);
    let z = sqrt(max(1.0 - dot(x_and_y, x_and_y), 0.0));
    return vec3<f32>(x_and_y * scale, select(z, 1.0, scale == 0.0));
}

// The BRDF, mix(dielectric_brdf, metal_brdf, metallic), per steradian, from
// the cosines between the normal N, the directions to the light L and to
// the viewer V, and H halfway between them (Appendix B, "Metal BRDF and
// Dielectric BRDF" [ref:gltf-2.0-metal-and-dielectric-brdf]).
fn metallic_roughness_brdf(
    surface: Surface,
    normal_dot_light: f32,
    normal_dot_view: f32,
    normal_dot_half: f32,
    view_dot_half: f32
) -> vec3<f32> {
    let alpha = surface.roughness * surface.roughness;
    let specular = distribution_ggx(normal_dot_half, alpha)
        * smith_visibility(normal_dot_light, normal_dot_view, view_dot_half, alpha);
    // The mix, gathered by lobe. A dielectric's surface reflects F of the
    // light, from its fixed F0, and the rest enters, scatters and leaves
    // diffuse; a metal's F0 is its base colour. Schlick's F is linear in F0,
    // so metallic's mix of the two specular lobes is one lobe whose F is at
    // the mixed F0, which no light changes. Mixed as Appendix B writes it,
    // the lit pass took up to 25 µs (5%) longer at 1440p on the M4 Pro.
    let weight = fresnel_weight(view_dot_half);
    let dielectric_fresnel = DIELECTRIC_BASE_REFLECTANCE
        + (1.0 - DIELECTRIC_BASE_REFLECTANCE) * weight;
    let diffuse_share = (1.0 - surface.metallic) * (1.0 - dielectric_fresnel) / PI;
    let base_reflectance = mix(
        vec3<f32>(DIELECTRIC_BASE_REFLECTANCE),
        surface.base_color,
        surface.metallic
    );
    let specular_fresnel = base_reflectance + (vec3<f32>(1.0) - base_reflectance) * weight;
    return diffuse_share * surface.base_color + specular_fresnel * specular;
}

// The luminance a surface point reflects along view_direction from a
// punctual light along light_direction, which gives `illuminance` on a
// surface facing it: f(L, V) E (N·L)⁺, the directions unit vectors from the
// point (ADR 0023's photometric units). H is left zero where L and V are
// opposite, so χ⁺ zeroes the specular lobe there.
fn reflected_luminance(
    surface: Surface,
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    illuminance: vec3<f32>
) -> vec3<f32> {
    let half_sum = light_direction + view_direction;
    let half_vector = half_sum * inverseSqrt(max(dot(half_sum, half_sum), 1e-30));
    let normal_dot_light = dot(normal, light_direction);
    let brdf = metallic_roughness_brdf(
        surface,
        normal_dot_light,
        dot(normal, view_direction),
        dot(normal, half_vector),
        dot(view_direction, half_vector)
    );
    return brdf * illuminance * max(normal_dot_light, 0.0);
}

// Point lights hold distance² at 1 cm² or more, as if 1 cm in radius, so a
// surface at one is lit finitely; Filament (Apache-2.0) holds the same
// [ref:filament-punctual-light-1cm].
const MINIMUM_LIGHT_DISTANCE_SQUARED: f32 = 1e-4;

// The share of a point light's intensity that reaches `distance_squared` from
// it as illuminance: inverse-square falloff within KHR_lights_punctual's range
// window [ref:khr-lights-punctual-range], which falls to 0 at `range` and stays
// 0 past it,
//   max(min(1 − (distance / range)⁴, 1), 0) / distance²,
// with (distance / range)⁴ taken as (distance² / range²)².
fn point_light_attenuation(distance_squared: f32, range: f32) -> f32 {
    let ratio_squared = distance_squared / (range * range);
    let window = clamp(1.0 - ratio_squared * ratio_squared, 0.0, 1.0);
    return window / distance_squared;
}

// The luminance a surface point reflects along view_direction from a point
// light at light_offset from it, of luminous intensity `intensity` (its
// colour times its candela, ADR 0023) and range `range`. Within 1 cm of
// the light distance² holds at MINIMUM_LIGHT_DISTANCE_SQUARED, and the
// light's direction shortens with the distance, to zero at the light
// itself, where it has none.
fn point_light_luminance(
    surface: Surface,
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_offset: vec3<f32>,
    intensity: vec3<f32>,
    range: f32
) -> vec3<f32> {
    let distance_squared = max(dot(light_offset, light_offset), MINIMUM_LIGHT_DISTANCE_SQUARED);
    let illuminance = intensity * point_light_attenuation(distance_squared, range);
    let light_direction = light_offset * inverseSqrt(distance_squared);
    return reflected_luminance(surface, normal, view_direction, light_direction, illuminance);
}

// The luminance a surface point reflects from a constant ambient
// illuminance E, a stand-in for ADR 0020's irradiance probes (M5), which
// light diffuse and rough specular reflection alike: E/π times its base
// colour, as a white-lit dielectric's diffuse lobe or a metal's specular
// one about reflects, darkened by occlusion, which glTF applies to
// indirect light only [ref:gltf-2.0-occlusion].
fn ambient_luminance(surface: Surface, illuminance: vec3<f32>) -> vec3<f32> {
    return illuminance * surface.base_color / PI * surface.occlusion;
}
