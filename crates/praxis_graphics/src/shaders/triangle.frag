#version 450

// ============================================================================
// Fragment Shader for 3D Textured Rendering with Blinn-Phong Lighting
// ============================================================================
//
// This shader implements the Blinn-Phong lighting model, a widely-used real-time
// lighting algorithm that balances visual quality with performance. It processes
// each pixel (fragment) to compute its final color based on material properties,
// lighting conditions, and textures.
//
// # Lighting Model Overview
//
// Blinn-Phong lighting consists of three components:
//
// 1. **Ambient**: Constant base illumination that prevents objects from being
//    completely black in shadow. Simulates indirect/scattered light.
//    Formula: ambient_color
//
// 2. **Diffuse**: Light scattered equally in all directions from a surface.
//    Follows Lambert's cosine law: intensity proportional to cos(θ) where θ
//    is the angle between the surface normal and light direction.
//    Formula: light_color * max(N·L, 0)
//
// 3. **Specular**: Glossy highlights from light reflecting in a specific direction.
//    Uses the half-vector between light and view directions for efficiency.
//    Formula: light_color * (N·H)^shininess
//
// Final color = (ambient + diffuse + specular) * albedo
//
// # Data Flow: CPU to GPU
//
// The rendering pipeline passes data from CPU (Rust) to GPU (GLSL) through
// several mechanisms:
//
// ## 1. Uniform Buffers (Set 0, Binding 0)
//    - Source: `Uniforms` struct in Rust (lib.rs)
//    - Contains: model, view, projection matrices
//    - Updated: Per-object, every frame
//    - Purpose: Transform vertices from model space to clip space
//    - Memory: Host-visible buffer, ~192 bytes per object
//
// ## 2. Texture Sampler (Set 0, Binding 1)
//    - Source: `TextureManager` in Rust
//    - Contains: Texture image and sampler configuration
//    - Updated: On texture load/change
//    - Purpose: Sample albedo (base) color at each fragment
//    - Memory: Device-local image, size varies by texture
//
// ## 3. Lighting Uniform Buffer (Set 0, Binding 2)
//    - Source: `LightingUniforms` struct in Rust (lighting.rs)
//    - Contains: Arrays of directional/point lights, counts, ambient color
//    - Updated: Every frame when lighting changes
//    - Purpose: Provide lighting data for illumination calculations
//    - Memory: Host-visible buffer, 1184 bytes (fixed size)
//    - Layout: std140 (specific alignment rules for compatibility)
//
// ## 4. Vertex Attributes (from vertex shader)
//    - v_world_pos: Fragment position in world space
//    - v_normal: Interpolated surface normal in world space
//    - v_color: Interpolated vertex color
//    - v_uv: Interpolated texture coordinates
//    - Updated: Per-vertex, interpolated per-fragment
//    - Purpose: Provide per-fragment data for lighting and texturing
//
// # Memory Layout (std140)
//
// The lighting buffer uses std140 layout, which has specific alignment rules:
// - vec3 is treated as vec4 (16-byte aligned) - the 4th component is padding
// - Array elements have 16-byte stride minimum
// - Struct size must be multiple of largest member alignment
//
// This is why DirectionalLight and PointLight structures have explicit
// padding fields - to ensure correct memory layout between CPU and GPU.
//
// # Lighting Calculations
//
// For each fragment, we:
// 1. Sample the texture to get base color (albedo)
// 2. Start with ambient lighting
// 3. Loop through directional lights:
//    - Calculate diffuse contribution based on N·L
//    - Calculate specular contribution based on N·H
//    - Accumulate light contribution
// 4. Loop through point lights:
//    - Calculate light direction from fragment position
//    - Calculate distance attenuation (inverse square + range cutoff)
//    - Calculate diffuse and specular contributions
//    - Accumulate attenuated light contribution
// 5. Multiply total lighting by albedo
// 6. Output final color with alpha from texture
//
// # Performance Notes
//
// - Loop counts are dynamic (based on light counts), but hardware unrolls small loops
// - Maximum 8 directional + 16 point lights to keep iteration count reasonable
// - Specular calculations use Blinn-Phong (half-vector) instead of Phong (reflection)
//   for better performance
// - Attenuation uses simplified formula (no linear/quadratic constants)

// ============================================================================
// Input Variables (from Vertex Shader)
// ============================================================================
// These values are interpolated across the triangle from the vertex shader

layout(location = 0) in vec3 v_world_pos;  // Fragment position in world space
layout(location = 1) in vec3 v_normal;     // Interpolated normal in world space
layout(location = 2) in vec3 v_color;      // Interpolated vertex color
layout(location = 3) in vec2 v_uv;         // Interpolated UV coordinates

// ============================================================================
// Output Variables
// ============================================================================

layout(location = 0) out vec4 f_color;     // Final pixel color (RGBA)

// ============================================================================
// Uniform Bindings
// ============================================================================

// Texture sampler at binding 1
// Samples the albedo (base color) texture at the given UV coordinates
layout(set = 0, binding = 1) uniform sampler2D albedo_texture;

// ============================================================================
// Material Properties Uniform Buffer
// ============================================================================
// This uniform buffer at set 1, binding 0 contains material-specific properties
// that control how the surface responds to light (PBR-style parameters).
//
// Data Flow:
//   1. CPU: Application sets material properties in MaterialProperties struct
//   2. CPU: Properties written to host-visible uniform buffer per draw call
//   3. GPU: Bound to descriptor set at set 1, binding 0
//   4. GPU: Read by fragment shader to control lighting behavior
//
// Memory layout (std140, 32 bytes):
//   Offset 0:  vec4 base_color (16 bytes) - rgba tint multiplier
//   Offset 16: float metallic (4 bytes) - metallic factor [0,1]
//   Offset 20: float roughness (4 bytes) - roughness factor [0,1]
//   Offset 24: float emissive_strength (4 bytes) - emissive intensity
//   Offset 28: float _padding (4 bytes) - alignment padding
//
// # PBR Material Properties Explained
//
// ## Metallic [0.0, 1.0]
// Controls whether the surface behaves like a metal or a dielectric (non-metal).
//
// - **Metallic = 0.0** (Dielectric): Surface like wood, plastic, stone, cloth
//   * Reflects only a small amount of light (4-8%)
//   * Keeps most of its base color in diffuse reflection
//   * Specular highlights are white/gray (reflect light color, not surface color)
//   * Subsurface scattering can occur (light penetrates slightly)
//
// - **Metallic = 1.0** (Metal): Surface like iron, gold, copper, chrome
//   * Reflects most light (60-90%+)
//   * No diffuse reflection (metals don't scatter light internally)
//   * Specular highlights are colored (reflect surface color tinted by light)
//   * No subsurface scattering (light doesn't penetrate)
//
// - **In-between values** (0.0 < metallic < 1.0): Blend between behaviors
//   * Can simulate oxidized metals, metal dust, or layered materials
//   * Most real materials are either fully metallic (1.0) or fully dielectric (0.0)
//
// **How it affects lighting in this shader:**
// - High metallic reduces diffuse contribution (metals don't scatter light)
// - High metallic tints specular highlights with base_color (colored reflections)
// - Low metallic keeps diffuse high and specular white/neutral
//
// ## Roughness [0.0, 1.0]
// Controls how rough or smooth the surface appears, affecting reflection sharpness.
//
// - **Roughness = 0.0** (Smooth/Glossy): Mirror-like, polished surface
//   * Tight, sharp specular highlights
//   * Clear reflections (in a full PBR model with environment mapping)
//   * High specular intensity concentrated in small area
//   * Examples: polished metal, glass, water, glossy plastic
//
// - **Roughness = 1.0** (Rough/Matte): Diffuse, rough surface
//   * Wide, soft specular highlights (or none at all)
//   * Blurry reflections scattered over larger area
//   * Lower specular intensity spread across wide area
//   * Examples: rough stone, unpolished wood, rubber, cloth
//
// - **In-between values**: Varying degrees of polish
//   * Most real-world materials fall in 0.3-0.7 range
//   * Worn surfaces might have varying roughness across the surface
//
// **How it affects lighting in this shader:**
// - Low roughness = high shininess power (tight, sharp highlights)
// - High roughness = low shininess power (soft, spread highlights)
// - Roughness modulates specular intensity and spread
//
// ## Emissive Strength [0.0, ∞]
// Controls how much the surface glows (emits light) independent of external lighting.
//
// - **Emissive = 0.0**: No self-illumination (most surfaces)
//   * Surface only visible due to external lights
//   * Completely black in darkness
//
// - **Emissive > 0.0**: Surface emits light
//   * base_color * emissive_strength added to final color
//   * Independent of lighting - always visible even in darkness
//   * Doesn't actually illuminate other surfaces (not a real light source)
//   * Examples: LEDs, screens, neon signs, hot metals, magic effects
//
// - **Emissive > 1.0**: Very bright emission
//   * Can exceed typical color range (HDR effect)
//   * Creates "bloom" effect if post-processing is enabled
//   * Useful for light sources, fire, lasers, etc.
//
// **How it affects lighting in this shader:**
// - Added directly to final color (after all lighting calculations)
// - Multiplied by base_color to tint the emission
// - Not affected by external lights (constant addition)
//
// # PBR (Physically Based Rendering) Concepts
//
// PBR aims to model light-matter interaction based on physical principles:
//
// 1. **Energy Conservation**: Reflected light ≤ incoming light
//    - As metallic increases, diffuse must decrease
//    - Total reflection cannot exceed 100%
//
// 2. **Fresnel Effect**: Reflectivity increases at grazing angles
//    - Looking at water head-on: see through (little reflection)
//    - Looking at water at shallow angle: mirror-like (high reflection)
//    - This shader uses simplified Blinn-Phong; full PBR would model this
//
// 3. **Microfacet Theory**: Rough surfaces have tiny random-oriented facets
//    - Smooth surface: all facets aligned, sharp reflection
//    - Rough surface: facets random, scattered reflection
//    - Roughness controls statistical distribution of facet orientations
//
// This shader uses a **simplified Blinn-Phong approximation** of PBR concepts
// rather than full physically-based BRDF (Cook-Torrance, GGX, etc.). It's fast
// and artist-friendly while capturing the essential behavior of metallic/roughness.
//
layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;         // Base color tint (rgba) - multiplied with texture
    float metallic;          // Metallic factor [0,1] - 0=dielectric, 1=metal
    float roughness;         // Roughness factor [0,1] - 0=smooth, 1=rough
    float emissive_strength; // Emissive intensity - makes surface glow
    float _padding;          // Padding for std140 alignment
} material;

// ============================================================================
// Lighting Data Structures
// ============================================================================
// These must match the Rust structures EXACTLY, including padding

// Directional light structure (48 bytes, matches DirectionalLightData in Rust)
//
// Memory layout:
//   Offset 0:  vec4 direction  (16 bytes) - xyz=direction, w=padding
//   Offset 16: vec4 color      (16 bytes) - rgb=color, a=padding  
//   Offset 32: float intensity (4 bytes)  - brightness multiplier
//   Offset 36: float[3] _padding (12 bytes) - align to 48 bytes
//
// Direction points FROM the light source (i.e., the direction light travels)
// This is used directly in lighting calculations: fragment_to_light = -direction
struct DirectionalLight {
    vec4 direction;     // Light direction (xyz) + padding (w)
    vec4 color;         // Light color (rgb) + padding (a)
    float intensity;    // Brightness multiplier (typically 0.0-2.0)
    float _padding[3];  // Padding to align struct to 16-byte boundary
};

// Point light structure (48 bytes, matches PointLightData in Rust)
//
// Memory layout:
//   Offset 0:  vec4 position   (16 bytes) - xyz=position, w=padding
//   Offset 16: vec4 color      (16 bytes) - rgb=color, a=padding
//   Offset 32: float intensity (4 bytes)  - brightness multiplier
//   Offset 36: float range     (4 bytes)  - maximum effective distance
//   Offset 40: float[2] _padding (8 bytes) - align to 48 bytes
//
// Point lights have position and radiate in all directions with attenuation
struct PointLight {
    vec4 position;      // Light position in world space (xyz) + padding (w)
    vec4 color;         // Light color (rgb) + padding (a)
    float intensity;    // Brightness multiplier
    float range;        // Maximum range in world units
    float _padding[2];  // Padding to align struct to 16-byte boundary
};

// Lighting uniform buffer at binding 2 (1184 bytes total)
//
// This buffer is uploaded from the CPU every frame with updated lighting data.
// The buffer is host-visible (CPU-writable) and device-visible (GPU-readable).
//
// Data Flow:
//   1. CPU: Application updates LightingUniforms struct in Rust
//   2. CPU: RenderContext writes to host-visible buffer via buffer.write()
//   3. GPU: Shader reads from buffer during fragment processing
//   4. GPU: Loops use count fields to process only active lights
//
// Memory layout:
//   Offset 0:    DirectionalLight[8] directional_lights (384 bytes)
//   Offset 384:  PointLight[16] point_lights (768 bytes)
//   Offset 1152: vec4 ambient_color (16 bytes)
//   Offset 1168: uint directional_light_count (4 bytes)
//   Offset 1172: uint point_light_count (4 bytes)
//   Offset 1176: uint[2] _padding (8 bytes)
//
layout(set = 0, binding = 2, std140) uniform LightingData {
    DirectionalLight directional_lights[8];   // Array of directional lights
    PointLight point_lights[16];               // Array of point lights
    vec4 ambient_color;                        // Global ambient light (rgb) + padding
    uint directional_light_count;              // Number of active directional lights (0-8)
    uint point_light_count;                    // Number of active point lights (0-16)
} lighting;

// ============================================================================
// Lighting Constants
// ============================================================================

// Camera position in world space (temporary fixed position)
// TODO: Pass this via uniform buffer for dynamic camera
const vec3 CAMERA_POS = vec3(0.0, 5.0, 10.0);

// Minimum and maximum shininess values for roughness mapping
// Roughness 1.0 (rough) → shininess 2.0 (very wide highlights)
// Roughness 0.0 (smooth) → shininess 256.0 (very tight highlights)
const float MIN_SHININESS = 2.0;
const float MAX_SHININESS = 256.0;

// ============================================================================
// Lighting Calculation Functions
// ============================================================================

// Calculate diffuse lighting using Lambert's cosine law
//
// The diffuse component represents light scattered equally in all directions
// from a surface. The intensity is proportional to the cosine of the angle
// between the surface normal and the light direction.
//
// Formula: diffuse = max(N·L, 0)
//
// Arguments:
//   normal: Normalized surface normal
//   light_dir: Normalized direction from fragment to light source
//
// Returns:
//   Diffuse intensity [0, 1] where 0 = perpendicular/away, 1 = directly facing
float calculate_diffuse(vec3 normal, vec3 light_dir) {
    // Dot product gives cos(angle) between normal and light
    // Clamp to 0 to avoid negative values (surface facing away from light)
    return max(dot(normal, light_dir), 0.0);
}

// Calculate specular lighting using Blinn-Phong model
//
// The specular component creates glossy highlights where light reflects toward
// the camera. Blinn-Phong uses a "half-vector" between the light and view
// directions, which is more efficient than calculating the reflection vector.
//
// Formula: specular = (N·H)^shininess
//   where H = normalize(L + V)
//
// The half-vector H points in the direction where the surface would need to
// face to perfectly reflect light from L toward V. When the normal aligns
// with H, we get maximum specular reflection.
//
// Arguments:
//   normal: Normalized surface normal
//   light_dir: Normalized direction from fragment to light source
//   view_dir: Normalized direction from fragment to camera
//   shininess: Specular power controlling highlight sharpness
//
// Returns:
//   Specular intensity [0, 1] raised to shininess power
float calculate_specular(vec3 normal, vec3 light_dir, vec3 view_dir, float shininess) {
    // Half-vector: direction halfway between light and view
    // This is where the normal would need to point for perfect reflection
    vec3 halfway_dir = normalize(light_dir + view_dir);
    
    // Compute (N·H)^shininess
    // The power function creates the sharp falloff characteristic of highlights
    // Higher shininess = smaller, sharper highlights
    return pow(max(dot(normal, halfway_dir), 0.0), shininess);
}

// Calculate attenuation for point lights based on distance
//
// Attenuation reduces light intensity with distance, simulating how real light
// spreads out and weakens. We use a physically-inspired inverse-square falloff
// combined with a smooth cutoff at the light's maximum range.
//
// Formula: attenuation = (1 / (1 + d^2)) * max(1 - d/range, 0)
//
// The (1 + d^2) term in the denominator approximates inverse-square falloff
// while avoiding division by zero at distance = 0. The range factor creates
// a smooth cutoff so lights don't affect distant objects.
//
// Arguments:
//   distance: Distance from fragment to light source
//   range: Maximum effective range of the light
//
// Returns:
//   Attenuation factor [0, 1] where 0 = no effect, 1 = full intensity
float calculate_attenuation(float distance, float range) {
    // Inverse square falloff (with +1 to avoid division by zero)
    // This simulates how light intensity decreases with distance squared
    float attenuation = 1.0 / (1.0 + distance * distance);
    
    // Smooth cutoff at range: linearly fade from 1 to 0 as distance approaches range
    // This ensures lights have no effect beyond their specified range
    float range_factor = max(1.0 - (distance / range), 0.0);
    
    return attenuation * range_factor;
}

// ============================================================================
// Main Fragment Shader
// ============================================================================
void main() {
    // ========================================================================
    // Step 1: Sample Texture and Compute Base Albedo
    // ========================================================================
    
    // Sample the texture at the interpolated UV coordinates
    // This gives us the base color (albedo) before lighting is applied
    vec4 tex_color = texture(albedo_texture, v_uv);
    
    // Combine vertex color with texture color and material base color
    // This allows triple modulation: vertex color * texture color * material tint
    // Material base_color.rgb provides per-material tinting
    // Material base_color.a provides opacity control
    vec3 albedo = v_color * tex_color.rgb * material.base_color.rgb;
    float alpha = tex_color.a * material.base_color.a;
    
    // ========================================================================
    // Step 2: Prepare Lighting Inputs
    // ========================================================================
    
    // Normalize the interpolated normal (interpolation can change length)
    // Normals must be unit length for lighting calculations to be correct
    vec3 normal = normalize(v_normal);
    
    // Calculate view direction (from fragment toward camera)
    // Used for specular calculations (highlights depend on view angle)
    vec3 view_dir = normalize(CAMERA_POS - v_world_pos);
    
    // Convert roughness [0,1] to shininess [MAX, MIN]
    // Roughness 0.0 (smooth) → high shininess (tight highlights)
    // Roughness 1.0 (rough) → low shininess (wide highlights)
    // We use an exponential mapping for more perceptually-linear control
    float shininess = mix(MAX_SHININESS, MIN_SHININESS, material.roughness);
    
    // Compute diffuse factor based on metallic property
    // Metals have little to no diffuse reflection (light doesn't penetrate surface)
    // Dielectrics have strong diffuse reflection (light scatters inside material)
    // Formula: diffuse_factor = 1.0 - metallic
    //   metallic = 0.0 → diffuse_factor = 1.0 (full diffuse, typical materials)
    //   metallic = 1.0 → diffuse_factor = 0.0 (no diffuse, pure metal)
    float diffuse_factor = 1.0 - material.metallic;
    
    // Compute specular color based on metallic property
    // Dielectrics: specular is white/neutral (reflects light color only)
    // Metals: specular is tinted by surface color (colored reflections)
    // Formula: lerp between white and albedo based on metallic
    //   metallic = 0.0 → specular_color = vec3(1.0) (white highlights)
    //   metallic = 1.0 → specular_color = albedo (colored highlights)
    vec3 specular_color = mix(vec3(1.0), albedo, material.metallic);
    
    // ========================================================================
    // Step 3: Initialize with Ambient Lighting
    // ========================================================================
    
    // Start with ambient lighting as the base
    // This ensures objects are never completely black, even in shadow
    // Ambient represents indirect/scattered light from the environment
    vec3 lighting_result = lighting.ambient_color.rgb;
    
    // ========================================================================
    // Step 4: Process Directional Lights
    // ========================================================================
    
    // Directional lights (like the sun) have direction but no position
    // They affect all surfaces equally regardless of distance
    
    // Loop through all active directional lights
    // The count is set by the CPU and may be less than the array size
    for (uint i = 0; i < lighting.directional_light_count; i++) {
        DirectionalLight light = lighting.directional_lights[i];
        
        // Light direction stored in buffer points FROM the source
        // For lighting calculations, we need direction TO the source
        // (negation converts "light travel direction" to "fragment to light")
        vec3 light_dir = -light.direction.xyz;
        
        // === Diffuse Component ===
        // How much light hits this surface based on angle between normal and light?
        // Surfaces facing the light (N·L = 1) get full intensity
        // Surfaces perpendicular (N·L = 0) get no direct light
        // Modulated by diffuse_factor: metals (high metallic) have reduced diffuse
        float diffuse = calculate_diffuse(normal, light_dir) * diffuse_factor;
        
        // === Specular Component ===
        // Is this surface positioned to reflect light toward the camera?
        // Uses half-vector for efficiency (Blinn-Phong vs. Phong)
        // Specular color varies based on metallic: white for dielectrics, colored for metals
        float specular = calculate_specular(normal, light_dir, view_dir, shininess);
        
        // === Combine Components ===
        // Multiply by light color and intensity, then accumulate
        // Diffuse uses albedo color (light scattered through surface)
        // Specular uses specular_color (light reflected from surface)
        // Specular is scaled by 0.5 to prevent over-brightening
        vec3 diffuse_contrib = light.color.rgb * light.intensity * diffuse;
        vec3 specular_contrib = light.color.rgb * light.intensity * specular * specular_color * 0.5;
        lighting_result += diffuse_contrib + specular_contrib;
    }
    
    // ========================================================================
    // Step 5: Process Point Lights
    // ========================================================================
    
    // Point lights have position and radiate in all directions
    // They attenuate (fade) with distance from the source
    
    // Loop through all active point lights
    for (uint i = 0; i < lighting.point_light_count; i++) {
        PointLight light = lighting.point_lights[i];
        
        // === Calculate Light Direction and Distance ===
        // Vector from fragment to light source
        vec3 light_vec = light.position.xyz - v_world_pos;
        
        // Distance from fragment to light (for attenuation)
        float distance = length(light_vec);
        
        // Normalize to get direction (reuse length calculation)
        vec3 light_dir = light_vec / distance;
        
        // === Calculate Attenuation ===
        // How much does the light intensity decrease at this distance?
        // Uses inverse-square falloff with smooth range cutoff
        float attenuation = calculate_attenuation(distance, light.range);
        
        // === Diffuse Component ===
        // Same calculation as directional, but will be attenuated
        // Modulated by diffuse_factor: metals have reduced diffuse
        float diffuse = calculate_diffuse(normal, light_dir) * diffuse_factor;
        
        // === Specular Component ===
        // Same calculation as directional, but will be attenuated
        // Specular color varies based on metallic
        float specular = calculate_specular(normal, light_dir, view_dir, shininess);
        
        // === Combine Components with Attenuation ===
        // Point light contributions are multiplied by attenuation
        // Diffuse and specular computed separately with proper color handling
        // Specular scaled by 0.3 (less than directional) for more subtle highlights
        vec3 diffuse_contrib = light.color.rgb * light.intensity * diffuse * attenuation;
        vec3 specular_contrib = light.color.rgb * light.intensity * specular * specular_color * 0.3 * attenuation;
        lighting_result += diffuse_contrib + specular_contrib;
    }
    
    // ========================================================================
    // Step 6: Apply Lighting to Albedo and Add Emissive
    // ========================================================================
    
    // Multiply the accumulated lighting by the surface's base color
    // This gives the lit color: lighting * albedo
    // Lighting already includes diffuse contributions (which use albedo color)
    // and specular contributions (which use specular_color)
    vec3 lit_color = lighting_result * albedo;
    
    // Add emissive contribution
    // Emissive makes the surface glow independent of external lighting
    // It's multiplied by base_color to provide colored emission
    // This is added AFTER lighting calculations, so it's unaffected by lights
    // Examples: LEDs, screens, hot metals, neon signs, magic effects
    vec3 emissive = material.base_color.rgb * material.emissive_strength;
    
    // Combine lit color with emissive
    // Emissive is additive - increases brightness regardless of lighting
    vec3 final_color = lit_color + emissive;
    
    // ========================================================================
    // Step 7: Output Final Color
    // ========================================================================
    
    // Output final color with combined alpha
    // Alpha comes from texture * material base_color alpha
    f_color = vec4(final_color, alpha);
}
