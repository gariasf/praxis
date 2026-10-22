// The shading terms, shared by every pass that lights a surface
// (src/render/shaders/mod.rs).

const PI: f32 = 3.14159265;

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

// F term. base_reflectance is the reflectance at normal incidence (textbook F0).
fn fresnel_schlick(cos_incidence_angle: f32, base_reflectance: vec3<f32>) -> vec3<f32> {
    return base_reflectance
        + (vec3<f32>(1.0) - base_reflectance) * pow(1.0 - cos_incidence_angle, 5.0);
}

// D term. Fraction of microfacets whose normal points along the half vector.
fn distribution_ggx(normal_dot_half: f32, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let denominator = normal_dot_half * normal_dot_half * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / (PI * denominator * denominator);
}

// G term. Fraction of microfacets neither masked from the viewer nor shadowed from the light.
fn geometry_smith(normal_dot_view: f32, normal_dot_light: f32, roughness: f32) -> f32 {
    let roughness_plus_one = roughness + 1.0;
    let remapped_roughness = (roughness_plus_one * roughness_plus_one) / 8.0;
    let view_masking = normal_dot_view
        / (normal_dot_view * (1.0 - remapped_roughness) + remapped_roughness);
    let light_shadowing = normal_dot_light
        / (normal_dot_light * (1.0 - remapped_roughness) + remapped_roughness);
    return view_masking * light_shadowing;
}

// A cooked normal Texture stores a unit normal's x and y in [0, 1], in BC5's
// two channels (ADR 0018); z is rebuilt as the positive root of
// x² + y² + z² = 1, and is 0 where block error carries x and y past the
// unit circle.
fn tangent_space_normal(encoded: vec2<f32>) -> vec3<f32> {
    let x_and_y = encoded * 2.0 - vec2<f32>(1.0);
    return vec3<f32>(x_and_y, sqrt(max(1.0 - dot(x_and_y, x_and_y), 0.0)));
}

// Outgoing radiance toward view_direction from one light arriving along light_direction.
fn cook_torrance(
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    base_color: vec3<f32>,
    metallic: f32,
    roughness: f32,
    radiance: vec3<f32>
) -> vec3<f32> {
    let half_vector = normalize(view_direction + light_direction);
    let normal_dot_view = max(dot(normal, view_direction), 0.0);
    let normal_dot_light = max(dot(normal, light_direction), 0.0);
    let normal_dot_half = max(dot(normal, half_vector), 0.0);
    let half_dot_view = max(dot(half_vector, view_direction), 0.0);

    let base_reflectance = mix(vec3<f32>(0.04), base_color, metallic);
    let fresnel = fresnel_schlick(half_dot_view, base_reflectance);
    let normal_distribution = distribution_ggx(normal_dot_half, roughness);
    let geometry_shadowing = geometry_smith(normal_dot_view, normal_dot_light, roughness);
    let specular = (normal_distribution * fresnel * geometry_shadowing)
        / (4.0 * normal_dot_view * normal_dot_light + 0.001);

    // Light that is not reflected refracts in and re-emerges as diffuse; metals absorb it.
    let diffuse_weight = (vec3<f32>(1.0) - fresnel) * (1.0 - metallic);
    let diffuse = diffuse_weight * base_color / PI;
    return (diffuse + specular) * radiance * normal_dot_light;
}
