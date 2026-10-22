// The shading terms, shared by every pass that lights a surface
// (src/render/shaders/mod.rs): the surface decode and glTF 2.0's
// metallic-roughness BRDF, written from its Appendix B (Khronos, CC BY 4.0):
// https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#appendix-b-brdf-implementation

const PI: f32 = 3.14159265;

// Appendix B's reflectance of a dielectric at normal incidence,
// ((1 − ior) / (1 + ior))² at its index of refraction of 1.5.
const DIELECTRIC_BASE_REFLECTANCE: f32 = 0.04;

// As roughness, and α = roughness² with it, falls to 0, D collapses to a
// delta and the BRDF divides by zero, so glTF 2.0's Appendix B keeps α above
// 0 (https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#specular-brdf).
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
// each factor scaling its texel (glTF 2.0, "Metallic-Roughness Material"):
// cooked metallic-roughness holds roughness in red and metallic in green
// (BC5, ADR 0018). Roughness keeps to MINIMUM_ROUGHNESS or above.
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

// F: Schlick's approximation (Appendix B, "Fresnel"). base_reflectance is
// the reflectance at normal incidence (textbook F0), and the angle is
// between V and H.
fn fresnel_schlick(cos_incidence_angle: f32, base_reflectance: vec3<f32>) -> vec3<f32> {
    return base_reflectance
        + (vec3<f32>(1.0) - base_reflectance) * pow(1.0 - cos_incidence_angle, 5.0);
}

// D: the Trowbridge-Reitz (GGX) distribution of microfacet normals at width
// alpha (Appendix B, "Specular BRDF"),
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
// unit circle.
fn tangent_space_normal(encoded: vec2<f32>) -> vec3<f32> {
    let x_and_y = encoded * 2.0 - vec2<f32>(1.0);
    return vec3<f32>(x_and_y, sqrt(max(1.0 - dot(x_and_y, x_and_y), 0.0)));
}

// The BRDF, mix(dielectric_brdf, metal_brdf, metallic), per steradian, from
// the cosines between the normal N, the directions to the light L and to
// the viewer V, and H halfway between them (Appendix B, "Metal BRDF and
// Dielectric BRDF").
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
    let cos_incidence_angle = abs(view_dot_half);
    // A dielectric's surface reflects F of the light; the rest enters,
    // scatters and leaves diffuse.
    let dielectric = mix(
        surface.base_color / PI,
        vec3<f32>(specular),
        fresnel_schlick(cos_incidence_angle, vec3<f32>(DIELECTRIC_BASE_REFLECTANCE))
    );
    let metal = fresnel_schlick(cos_incidence_angle, surface.base_color) * specular;
    return mix(dielectric, metal, surface.metallic);
}

// The radiance a surface point reflects along view_direction from a
// punctual light along light_direction, which gives `illuminance` on a
// surface facing it: f(L, V) E (N·L)⁺, the directions unit vectors from the
// point. H is left zero where L and V are opposite, so χ⁺ zeroes the
// specular lobe there.
fn reflected_radiance(
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
