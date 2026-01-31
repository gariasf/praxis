# Shader Pipeline and Material System

This document describes the shader compilation, reflection, pipeline state management, and material system in the Praxis graphics engine.

## Overview

The Praxis graphics engine provides a comprehensive shader pipeline and material system with the following features:

- **Shader Compilation**: GLSL to SPIR-V compilation at build time via `vulkano-shaders`
- **Shader Reflection**: Automatic extraction of shader metadata (descriptors, inputs, outputs)
- **Pipeline State Objects**: Cacheable, reusable graphics pipeline configurations
- **Material System**: PBR-based materials with texture support
- **Descriptor Management**: Automated descriptor set creation and binding
- **Forward Rendering**: PBR forward renderer with Cook-Torrance BRDF

## Architecture

### Shader Compilation

Shaders are written in GLSL and compiled to SPIR-V at build time using the `vulkano-shaders` macro:

```rust
// In src/shaders.rs
pub mod forward_pbr_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/forward_pbr.vert"
    }
}

pub mod forward_pbr_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/forward_pbr.frag"
    }
}
```

**Benefits:**
- Compile-time shader validation
- Zero runtime overhead for shader loading
- Type-safe shader interfaces
- Automatic reflection data generation

### Shader Reflection

The `shader_reflection` module provides introspection of shader metadata:

```rust
use praxis_graphics::shader_reflection::{ShaderReflection, ShaderStage, PipelineReflection};

// Create reflection for a shader
let reflection = ShaderReflection::from_entry_point(ShaderStage::Vertex, &vertex_entry);

// Query descriptor bindings
let bindings = reflection.get_bindings_for_set(0);
assert!(reflection.uses_descriptor_set(0));

// Build pipeline reflection
let mut pipeline_reflection = PipelineReflection::new();
pipeline_reflection.add_stage(vs_reflection);
pipeline_reflection.add_stage(fs_reflection);

// Get all descriptor sets used by pipeline
let descriptor_sets = pipeline_reflection.get_all_descriptor_sets();
```

**Use Cases:**
- Automatic descriptor set layout generation
- Pipeline validation and debugging
- Shader compatibility checking
- Resource binding verification

### Pipeline State Objects

The `pipeline_state` module provides a fluent API for configuring graphics pipelines:

```rust
use praxis_graphics::pipeline_state::{PipelineStateConfig, DepthTestConfig, BlendMode};
use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;

// Create pipeline configuration
let config = PipelineStateConfig::new()
    .with_topology(PrimitiveTopology::TriangleList)
    .with_cull_mode(CullMode::Back)
    .with_depth_test(DepthTestConfig::default())
    .with_blend_mode(BlendMode::Alpha);

// Use pipeline cache to avoid recreating identical pipelines
let mut cache = PipelineCache::new(device.clone());
let pipeline = cache.get_or_create(
    &config,
    shaders,
    vertex_input,
    render_pass,
    extent,
)?;
```

**Pipeline State Features:**
- Depth testing configuration
- Blend mode selection (None, Alpha, Additive, Premultiplied)
- Rasterization state (cull mode, front face, polygon mode)
- Dynamic state management
- Pipeline caching for performance

**Predefined Configurations:**
- `PipelineStateConfig::default()`: Standard opaque rendering
- `DepthTestConfig::disabled()`: No depth testing
- `DepthTestConfig::test_only()`: Depth test without writing
- `BlendMode::Alpha`: Standard alpha blending
- `BlendMode::Additive`: Additive blending for effects

### Material System

The material system provides PBR (Physically-Based Rendering) materials with texture support:

```rust
use praxis_graphics::material::{Material, MaterialProperties};

// Create material with PBR properties
let material = Material::new("metal")
    .with_albedo_texture(albedo_texture)
    .with_normal_texture(normal_texture)
    .with_metallic_roughness_texture(mr_texture)
    .with_properties(
        MaterialProperties::new()
            .with_base_color([1.0, 0.8, 0.5, 1.0])
            .with_metallic(1.0)
            .with_roughness(0.2)
            .with_emissive_strength(0.0)
    );
```

**Material Properties (std140 layout):**
```rust
#[repr(C)]
pub struct MaterialProperties {
    pub base_color: [f32; 4],    // RGBA tint
    pub metallic: f32,            // [0,1] metallic factor
    pub roughness: f32,           // [0,1] roughness factor
    pub emissive_strength: f32,   // Emissive intensity
    _padding: f32,                // Alignment padding
}
```

**Supported Textures:**
- **Albedo**: Base color (RGB + optional alpha)
- **Normal Map**: Tangent-space normals for surface detail
- **Metallic-Roughness**: Combined texture (B=metallic, G=roughness)
- **Ambient Occlusion**: Baked ambient shadows
- **Emissive**: Self-illumination
- **Height Map**: Displacement for parallax mapping

### Descriptor Set Management

The `descriptor_binding` module provides type-safe descriptor set creation:

```rust
use praxis_graphics::descriptor_binding::{DescriptorSetLayoutBuilder, DescriptorSetWriter};
use vulkano::shader::ShaderStages;

// Build descriptor set layout
let layout = DescriptorSetLayoutBuilder::new()
    .add_uniform_buffer(0, ShaderStages::VERTEX | ShaderStages::FRAGMENT)
    .add_dynamic_uniform_buffer(1, ShaderStages::VERTEX)
    .add_combined_image_sampler(2, ShaderStages::FRAGMENT)
    .build(device)?;

// Write descriptor set bindings
let writes = DescriptorSetWriter::new()
    .write_buffer(0, view_proj_buffer)
    .write_buffer_with_range(1, dynamic_buffer_info)
    .write_image_view_sampler(2, texture_view, sampler)
    .build();

let descriptor_set = DescriptorSet::new(allocator, layout, writes, [])?;
```

**Standard Layouts:**
```rust
use praxis_graphics::descriptor_binding::StandardDescriptorLayouts;

// Set 0: Per-frame data (view/projection, model, textures, lighting)
let per_frame_layout = StandardDescriptorLayouts::per_frame_layout(device)?;

// Set 1: Per-material data (material properties)
let per_material_layout = StandardDescriptorLayouts::per_material_layout(device)?;

// Set 2: Bindless resources (texture arrays, material buffers)
let bindless_layout = StandardDescriptorLayouts::bindless_layout(device)?;
```

## Forward PBR Rendering

The forward PBR renderer implements physically-based lighting using the Cook-Torrance BRDF.

### Vertex Shader (`forward_pbr.vert`)

**Responsibilities:**
- Transform vertices from model space to clip space (MVP)
- Transform normals and tangents to world space
- Compute bitangent for TBN matrix
- Pass interpolated data to fragment shader

**Descriptor Sets:**
- Set 0, Binding 0: View/Projection uniform buffer
- Set 0, Binding 1: Model matrix (dynamic uniform)

**Outputs:**
- World-space position (for lighting calculations)
- World-space normal (for lighting)
- Tangent and bitangent (for normal mapping)
- UV coordinates (for texture sampling)
- Vertex color (for tinting)

### Fragment Shader (`forward_pbr.frag`)

**Responsibilities:**
- Sample material textures (albedo, normal)
- Perform normal mapping using TBN matrix
- Calculate PBR lighting using Cook-Torrance BRDF
- Apply tone mapping and gamma correction

**Descriptor Sets:**
- Set 0, Binding 0: View/Projection (for camera position)
- Set 0, Binding 2: Albedo texture
- Set 0, Binding 3: Lighting data
- Set 0, Binding 9: Normal map
- Set 1, Binding 0: Material properties

**PBR Functions:**

1. **Normal Distribution Function (GGX/Trowbridge-Reitz)**:
   ```glsl
   float distribution_ggx(vec3 N, vec3 H, float roughness)
   ```
   Determines microfacet distribution - how many microfacets are aligned with the half-vector.

2. **Geometry Function (Schlick-GGX)**:
   ```glsl
   float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness)
   ```
   Accounts for geometry obstruction and shadowing of microfacets.

3. **Fresnel Equation (Schlick Approximation)**:
   ```glsl
   vec3 fresnel_schlick(float cos_theta, vec3 F0)
   ```
   Determines how much light is reflected vs. refracted at the surface.

**Cook-Torrance BRDF:**
```glsl
// Specular component
float NDF = distribution_ggx(normal, H, roughness);
float G = geometry_smith(normal, V, L, roughness);
vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);
vec3 specular = (NDF * G * F) / (4.0 * NdotV * NdotL);

// Diffuse component (Lambert)
vec3 kD = (1.0 - F) * (1.0 - metallic);
vec3 diffuse = kD * albedo / PI;

// Final radiance
Lo += (diffuse + specular) * radiance * NdotL;
```

**Post-Processing:**
- Reinhard tone mapping: `color = color / (color + 1.0)`
- Gamma correction: `color = pow(color, 1.0 / 2.2)`

## Descriptor Set Layout Convention

The Praxis engine uses a standardized descriptor set layout across all shaders:

### Set 0: Per-Frame / Per-Draw Resources
- **Binding 0**: View/Projection uniform buffer (std140)
  - `mat4 view`
  - `mat4 proj`
  - `vec3 camera_position`
- **Binding 1**: Model matrix (dynamic uniform, std140)
  - `mat4 model`
- **Binding 2**: Albedo texture sampler
- **Binding 3**: Lighting uniform buffer (std140)
  - Directional lights array
  - Point lights array
  - Ambient color
  - Light counts
- **Binding 4**: Shadow uniforms (optional)
- **Binding 5-8**: Shadow map samplers (optional)
- **Binding 9**: Normal map texture sampler
- **Binding 10**: Bone matrices for skeletal animation (optional)

### Set 1: Per-Material Resources
- **Binding 0**: Material properties uniform buffer (std140)
  - `vec4 base_color`
  - `float metallic`
  - `float roughness`
  - `float emissive_strength`

### Set 2: Bindless Resources (Optional)
- **Binding 0**: Texture array (up to 4096 textures)
- **Binding 1**: Material data buffer

## Usage Examples

### Creating a Material

```rust
use praxis_graphics::material::{Material, MaterialProperties};

// Load textures
let albedo = texture_manager.load_texture("metal_albedo.png")?;
let normal = texture_manager.load_texture("metal_normal.png")?;
let mr = texture_manager.load_texture("metal_mr.png")?;

// Create material
let material = Material::new("polished_metal")
    .with_albedo_texture(albedo)
    .with_normal_texture(normal)
    .with_metallic_roughness_texture(mr)
    .with_properties(
        MaterialProperties::new()
            .with_base_color([1.0, 0.9, 0.8, 1.0])
            .with_metallic(1.0)
            .with_roughness(0.1)
    );

// Register with material manager
material_manager.register_material("polished_metal", material);
```

### Configuring a Pipeline

```rust
use praxis_graphics::pipeline_state::{PipelineStateConfig, BlendMode};

// Opaque objects
let opaque_config = PipelineStateConfig::new()
    .with_blend_mode(BlendMode::None)
    .with_depth_test(DepthTestConfig::default());

// Transparent objects
let transparent_config = PipelineStateConfig::new()
    .with_blend_mode(BlendMode::Alpha)
    .with_depth_test(DepthTestConfig::test_only());

// Additive effects (particles, lights)
let additive_config = PipelineStateConfig::new()
    .with_blend_mode(BlendMode::Additive)
    .with_depth_test(DepthTestConfig::test_only());
```

### Using the Render System

```rust
use praxis_graphics::{RenderContext, RenderCommands, DrawCommand};

// Create draw commands with materials
let draw_commands = vec![
    DrawCommand {
        mesh_id: "sphere".to_string(),
        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        texture_name: Some("metal_albedo".to_string()),
        material_properties: Some(
            MaterialProperties::new()
                .with_metallic(1.0)
                .with_roughness(0.2)
        ),
        material_instance_id: None,
        bone_matrices: None,
    },
];

// Render
let commands = RenderCommands {
    view,
    proj,
    draw_commands: &draw_commands,
    lighting: Some(&lighting_data),
};

render_context.render(&commands)?;
```

## Performance Considerations

### Pipeline Caching
- Pipelines are expensive to create (~100ms)
- Use `PipelineCache` to reuse identical pipelines
- Hash-based lookup ensures efficient retrieval
- Clear cache when render passes change

### Descriptor Set Pooling
- Reuse descriptor sets for identical materials
- Pool tracked by texture name + properties hash
- LRU eviction prevents memory bloat
- Dramatically reduces per-frame allocations

### Material Instancing
- Share textures across material variants
- Override only necessary properties per-object
- Reduces memory usage for large scenes
- See `material_instancing` module for details

### Uniform Buffer Updates
- Use dynamic uniform buffers for per-object data
- Update lighting uniforms only when changed
- Batch material updates where possible
- Leverage host-visible memory for frequent updates

## Extending the System

### Adding New Shaders

1. Write GLSL shaders in `src/shaders/`
2. Add shader module in `src/shaders.rs`:
   ```rust
   pub mod my_shader_vs {
       vulkano_shaders::shader! {
           ty: "vertex",
           path: "src/shaders/my_shader.vert"
       }
   }
   ```
3. Use in pipeline creation

### Custom Material Properties

1. Define property struct with `#[repr(C)]` and std140 layout
2. Implement `bytemuck::Pod` and `bytemuck::Zeroable`
3. Add to material descriptor set (Set 1, Binding 0)
4. Update shader to match struct layout

### Custom Pipeline States

1. Create `PipelineStateConfig` with desired settings
2. Add to `PipelineCache` for reuse
3. Bind pipeline in render commands

## Troubleshooting

### Shader Compilation Errors
- Check GLSL syntax in shader files
- Verify descriptor set bindings match
- Ensure uniform block layouts are std140-compliant
- Run `cargo clean` to force shader recompilation

### Descriptor Set Errors
- Verify all required bindings are present
- Check buffer sizes match struct sizes
- Ensure textures are uploaded before use
- Validate descriptor set layouts match shader expectations

### Material Not Rendering
- Confirm mesh is loaded and valid
- Check texture names are correct
- Verify material properties are uploaded
- Enable graphics validation layers for detailed errors

### Performance Issues
- Use pipeline caching
- Enable descriptor set pooling
- Batch draw calls with same material
- Profile with `RenderStats` to identify bottlenecks

## References

- [Vulkan Specification](https://www.khronos.org/vulkan/)
- [PBR Theory (Unreal Engine)](https://blog.selfshadow.com/publications/s2013-shading-course/)
- [LearnOpenGL PBR](https://learnopengl.com/PBR/Theory)
- [vulkano Documentation](https://docs.rs/vulkano/)
