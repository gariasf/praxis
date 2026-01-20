# Render Pipeline: Multi-Engine Comparison

**Complexity**: Intermediate  
**Curriculum Module**: [Module 2 - Rendering Architecture Patterns](../modules/02-rendering-architecture-patterns.md)

## Problem Statement

Game engines must efficiently render 3D scenes to the screen. The fundamental challenges are:

- How do we abstract graphics APIs (Vulkan, DirectX, Metal, OpenGL)?
- How do we organize rendering work (forward, deferred, command buffers)?
- How do we batch draw calls and minimize GPU state changes?
- How do we handle materials, shaders, and render passes?
- How do we support multiple platforms with different capabilities?

## Design Philosophy Comparison

| Engine | Graphics API | Rendering Paradigm | Abstraction Level |
|--------|--------------|-------------------|-------------------|
| **Unity** | Multi-API (DX11/12, Vulkan, Metal, OpenGL) | Scriptable Render Pipeline (SRP) | High-level, material-centric |
| **Unreal** | Multi-API (DX11/12, Vulkan, Metal) | Deferred rendering (default) | Mid-level, shader graph + C++ |
| **Godot** | Multi-API (Vulkan, OpenGL) | Clustered forward/deferred | High-level, visual shader editor |
| **Praxis** | Vulkan (vulkano) | Flexible (forward/deferred) | Low-level, explicit control |

## Implementation Examples

### Basic Rendering Setup

#### Unity (C#)

```csharp
// Material-centric rendering (Built-in Render Pipeline)
public class BasicRenderer : MonoBehaviour
{
    public Material material;
    public Mesh mesh;

    void Update()
    {
        // Unity handles all rendering internally
        // Just set material properties
        material.SetColor("_Color", Color.red);
        material.SetFloat("_Metallic", 0.5f);
        
        // Draw call (internally batched)
        Graphics.DrawMesh(mesh, transform.localToWorldMatrix, material, 0);
    }
}

// Custom Render Pipeline (Unity 2019+)
public class CustomRenderPipeline : RenderPipeline
{
    protected override void Render(ScriptableRenderContext context, Camera[] cameras)
    {
        foreach (Camera camera in cameras)
        {
            // Setup camera rendering
            context.SetupCameraProperties(camera);
            
            // Culling
            camera.TryGetCullingParameters(out var cullingParams);
            var cullingResults = context.Cull(ref cullingParams);
            
            // Draw opaque objects
            var drawSettings = CreateDrawingSettings(
                new ShaderTagId("UnlitForward"), 
                SortingCriteria.CommonOpaque
            );
            var filterSettings = new FilteringSettings(RenderQueueRange.opaque);
            
            context.DrawRenderers(cullingResults, ref drawSettings, ref filterSettings);
            
            // Submit
            context.Submit();
        }
    }
}
```

#### Unreal (C++)

```cpp
// Custom Scene Proxy for rendering
class FMyComponentSceneProxy : public FPrimitiveSceneProxy
{
public:
    FMyComponentSceneProxy(const UMyRenderComponent* Component)
        : FPrimitiveSceneProxy(Component)
        , MaterialRelevance(Component->GetMaterialRelevance(GetScene().GetFeatureLevel()))
    {
        // Cache mesh data
        VertexBuffer = Component->VertexBuffer;
        IndexBuffer = Component->IndexBuffer;
    }

    // Called by rendering thread
    virtual void GetDynamicMeshElements(
        const TArray<const FSceneView*>& Views,
        const FSceneViewFamily& ViewFamily,
        uint32 VisibilityMap,
        FMeshElementCollector& Collector
    ) const override
    {
        for (int32 ViewIndex = 0; ViewIndex < Views.Num(); ViewIndex++)
        {
            if (VisibilityMap & (1 << ViewIndex))
            {
                const FSceneView* View = Views[ViewIndex];
                
                // Create mesh batch
                FMeshBatch& Mesh = Collector.AllocateMesh();
                Mesh.VertexFactory = &VertexFactory;
                Mesh.MaterialRenderProxy = Material->GetRenderProxy();
                Mesh.ReverseCulling = IsLocalToWorldDeterminantNegative();
                Mesh.Type = PT_TriangleList;
                Mesh.DepthPriorityGroup = SDPG_World;
                
                // Setup batch element
                FMeshBatchElement& BatchElement = Mesh.Elements[0];
                BatchElement.IndexBuffer = &IndexBuffer;
                BatchElement.FirstIndex = 0;
                BatchElement.NumPrimitives = NumTriangles;
                BatchElement.MinVertexIndex = 0;
                BatchElement.MaxVertexIndex = NumVertices - 1;
                
                Collector.AddMesh(ViewIndex, Mesh);
            }
        }
    }

    virtual FPrimitiveViewRelevance GetViewRelevance(const FSceneView* View) const override
    {
        FPrimitiveViewRelevance Result;
        Result.bDrawRelevance = IsShown(View);
        Result.bShadowRelevance = IsShadowCast(View);
        Result.bDynamicRelevance = true;
        Result.bStaticRelevance = false;
        Result.bRenderInMainPass = ShouldRenderInMainPass();
        MaterialRelevance.SetPrimitiveViewRelevance(Result);
        return Result;
    }

private:
    FMaterialRelevance MaterialRelevance;
    FVertexBuffer VertexBuffer;
    FIndexBuffer IndexBuffer;
};

// Deferred rendering pass (engine-level)
void FDeferredShadingSceneRenderer::Render(FRHICommandListImmediate& RHICmdList)
{
    // GBuffer pass
    RenderGBuffer(RHICmdList);
    
    // Lighting pass
    RenderLights(RHICmdList);
    
    // Post-processing
    RenderPostProcessing(RHICmdList);
}
```

#### Godot (GDScript)

```gdscript
# High-level rendering (automatic)
extends MeshInstance3D

func _ready():
    # Material setup
    var material = StandardMaterial3D.new()
    material.albedo_color = Color.RED
    material.metallic = 0.5
    material.roughness = 0.3
    set_surface_override_material(0, material)
    
    # Godot handles all rendering automatically
    # Rendering settings via project settings or RenderingServer

# Low-level rendering (advanced)
var rd: RenderingDevice = RenderingServer.get_rendering_device()

func custom_render_pass():
    # Create shader
    var shader_source = """
    #version 450
    layout(location = 0) in vec3 position;
    layout(location = 0) out vec4 fragColor;
    
    void main() {
        gl_Position = vec4(position, 1.0);
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
    }
    """
    
    var shader_spirv = rd.shader_compile_spirv_from_source(shader_source)
    var shader = rd.shader_create_from_spirv(shader_spirv)
    
    # Create pipeline
    var pipeline = rd.render_pipeline_create(
        shader,
        rd.screen_get_framebuffer_format(),
        RenderingDevice.INVALID_ID,
        RenderingDevice.RENDER_PRIMITIVE_TRIANGLES
    )
```

#### Praxis (Rust)

```rust
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};

// Low-level Vulkan rendering
pub struct Renderer {
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
    render_pass: Arc<RenderPass>,
}

impl Renderer {
    pub fn render_frame(&self, framebuffer: Arc<Framebuffer>, meshes: &[Mesh]) {
        // Create command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        
        // Begin render pass
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.0, 0.0, 0.0, 1.0].into()),
                        Some(1.0.into()),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo::default(),
            )
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .unwrap();
        
        // Draw each mesh
        for mesh in meshes {
            // Bind vertex buffer
            builder.bind_vertex_buffers(0, mesh.vertex_buffer.clone()).unwrap();
            
            // Bind index buffer
            builder.bind_index_buffer(mesh.index_buffer.clone()).unwrap();
            
            // Bind descriptor sets (uniforms, textures)
            builder.bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                mesh.descriptor_set.clone(),
            ).unwrap();
            
            // Draw indexed
            builder.draw_indexed(mesh.index_count, 1, 0, 0, 0).unwrap();
        }
        
        // End render pass
        builder.end_render_pass(SubpassEndInfo::default()).unwrap();
        
        // Build and submit
        let command_buffer = builder.build().unwrap();
        let future = sync::now(self.device.clone())
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), swapchain_info)
            .then_signal_fence_and_flush();
    }
}
```

### Shader Definition

#### Unity (ShaderLab)

```hlsl
Shader "Custom/PBRShader"
{
    Properties
    {
        _Color ("Main Color", Color) = (1,1,1,1)
        _MainTex ("Albedo (RGB)", 2D) = "white" {}
        _Metallic ("Metallic", Range(0,1)) = 0.5
        _Smoothness ("Smoothness", Range(0,1)) = 0.5
    }
    
    SubShader
    {
        Tags { "RenderType"="Opaque" "Queue"="Geometry" }
        
        CGPROGRAM
        #pragma surface surf Standard fullforwardshadows
        #pragma target 3.0
        
        sampler2D _MainTex;
        fixed4 _Color;
        half _Metallic;
        half _Smoothness;
        
        struct Input {
            float2 uv_MainTex;
        };
        
        void surf (Input IN, inout SurfaceOutputStandard o)
        {
            fixed4 c = tex2D(_MainTex, IN.uv_MainTex) * _Color;
            o.Albedo = c.rgb;
            o.Metallic = _Metallic;
            o.Smoothness = _Smoothness;
            o.Alpha = c.a;
        }
        ENDCG
    }
}
```

#### Unreal (HLSL in Material Editor / Custom)

```cpp
// Material expression graph (Blueprint-like)
// Or custom HLSL node:

float3 CustomLighting(float3 Normal, float3 ViewDir, float Roughness)
{
    float3 H = normalize(ViewDir + LightDir);
    float NdotH = saturate(dot(Normal, H));
    float D = D_GGX(NdotH, Roughness);
    return D * LightColor;
}

// In material graph: CustomNode with inputs Normal, ViewDir, Roughness
```

#### Godot (Godot Shader Language)

```glsl
shader_type spatial;
render_mode blend_mix, depth_draw_opaque, cull_back;

uniform vec4 albedo : source_color = vec4(1.0);
uniform sampler2D texture_albedo : source_color;
uniform float metallic : hint_range(0.0, 1.0) = 0.5;
uniform float roughness : hint_range(0.0, 1.0) = 0.5;

void fragment() {
    vec4 albedo_tex = texture(texture_albedo, UV);
    ALBEDO = albedo.rgb * albedo_tex.rgb;
    METALLIC = metallic;
    ROUGHNESS = roughness;
}
```

#### Praxis (GLSL via vulkano-shaders macro)

```rust
mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 450
            
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 normal;
            layout(location = 2) in vec2 uv;
            
            layout(location = 0) out vec3 v_normal;
            layout(location = 1) out vec2 v_uv;
            
            layout(set = 0, binding = 0) uniform MVP {
                mat4 model;
                mat4 view;
                mat4 projection;
            } mvp;
            
            void main() {
                gl_Position = mvp.projection * mvp.view * mvp.model * vec4(position, 1.0);
                v_normal = mat3(mvp.model) * normal;
                v_uv = uv;
            }
        "
    }
}

mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 450
            
            layout(location = 0) in vec3 v_normal;
            layout(location = 1) in vec2 v_uv;
            
            layout(location = 0) out vec4 f_color;
            
            layout(set = 1, binding = 0) uniform Material {
                vec4 albedo;
                float metallic;
                float roughness;
            } material;
            
            layout(set = 1, binding = 1) uniform sampler2D albedo_texture;
            
            void main() {
                vec4 albedo_tex = texture(albedo_texture, v_uv);
                vec3 albedo = material.albedo.rgb * albedo_tex.rgb;
                
                // Simple lighting
                vec3 light_dir = normalize(vec3(1.0, 1.0, 1.0));
                float diffuse = max(dot(normalize(v_normal), light_dir), 0.0);
                
                f_color = vec4(albedo * diffuse, 1.0);
            }
        "
    }
}
```

## Rendering Pipeline Patterns

### Unity (Forward vs. Deferred)

```csharp
// Forward Rendering (Universal Render Pipeline)
public class ForwardRendererData : ScriptableRendererData
{
    // Opaque pass
    // Transparent pass
    // Post-processing
}

// Deferred Rendering (HDRP)
public class HDRenderPipeline
{
    // GBuffer pass: Albedo, Normal, Metallic-Roughness, Emission
    // Lighting pass: Accumulate lighting from all lights
    // Post-processing: Tone mapping, bloom, etc.
}
```

**Unity Pipeline**:
1. Culling
2. Shadow map rendering
3. Opaque geometry (forward or GBuffer)
4. Sky/background
5. Transparent geometry (forward)
6. Post-processing

### Unreal (Deferred)

```cpp
void FDeferredShadingSceneRenderer::Render(FRHICommandListImmediate& RHICmdList)
{
    // GBuffer Pass
    {
        // RT0: Albedo (RGB), Shading Model (A)
        // RT1: Normal (RGB), AO (A)
        // RT2: Metallic, Specular, Roughness
        // RT3: Emissive + other data
        RenderBasePass(RHICmdList);
    }
    
    // Lighting Pass
    {
        // Deferred lighting using GBuffer
        RenderLights(RHICmdList);
    }
    
    // Translucency (Forward)
    RenderTranslucency(RHICmdList);
    
    // Post-processing
    PostProcessing(RHICmdList);
}
```

**Unreal Pipeline**:
1. Pre-pass (depth/normals)
2. GBuffer generation
3. Deferred lighting
4. Forward translucency
5. Post-processing chain

### Godot (Clustered Forward+)

```gdscript
# Godot 4.x uses Clustered Forward+ by default
# Configured via project settings

# Rendering pipeline (internal):
# 1. Depth pre-pass (optional)
# 2. Shadow map generation
# 3. Opaque geometry (forward with light clustering)
# 4. Sky
# 5. Transparent geometry
# 6. Post-processing (glow, tonemapping, etc.)
```

### Praxis (Flexible)

```rust
// Example: Custom deferred pipeline
pub struct DeferredRenderer {
    gbuffer_pass: Arc<RenderPass>,
    lighting_pass: Arc<RenderPass>,
    // ...
}

impl DeferredRenderer {
    pub fn render(&mut self, scene: &Scene, camera: &Camera) {
        // 1. GBuffer Pass
        self.render_gbuffer(scene, camera);
        
        // 2. Lighting Pass
        self.render_lighting(scene, camera);
        
        // 3. Forward Transparent
        self.render_transparent(scene, camera);
        
        // 4. Post-processing
        self.render_postprocessing();
    }
    
    fn render_gbuffer(&self, scene: &Scene, camera: &Camera) {
        // Render to multiple render targets:
        // RT0: Albedo (RGB) + Roughness (A)
        // RT1: Normal (RGB) + Metallic (A)
        // RT2: Emissive (RGB)
        // Depth: Depth buffer
    }
}
```

## Trade-Off Analysis

### Unity

**Pros**:
- Cross-platform abstraction (works everywhere)
- SRP allows customization without C++ engine changes
- Excellent shader compilation caching
- Material inspector provides immediate feedback
- Built-in URP (forward) and HDRP (deferred) pipelines

**Cons**:
- Abstraction overhead reduces peak performance
- Shader compilation can be slow (many variants)
- Less control over low-level optimizations
- Multi-threading rendering is engine-controlled

**Best For**: Cross-platform games, rapid prototyping, teams without graphics engineers

### Unreal

**Pros**:
- AAA-quality deferred renderer out-of-box
- Advanced features (Lumen, Nanite, Virtual Shadow Maps)
- Material editor is powerful and designer-friendly
- C++ access for custom render passes
- Excellent profiling tools (GPU Visualizer)

**Cons**:
- Deferred rendering limits MSAA (must use TAA)
- Heavy runtime for small/mobile games
- Complex codebase for customization
- Shader recompilation on material changes

**Best For**: AAA graphics, large teams, first/third-person games, cinematic quality

### Godot

**Pros**:
- Lightweight and fast iteration
- Visual shader editor for artists
- Modern Clustered Forward+ in 4.x
- Open-source (can modify renderer)
- Good mobile performance

**Cons**:
- Less mature than Unity/Unreal
- Fewer advanced features (no Nanite equivalent)
- Smaller ecosystem of rendering resources
- OpenGL backend aging out

**Best For**: Indie games, 2D/stylized 3D, open-source projects, learning

### Praxis

**Pros**:
- Direct Vulkan control (maximum performance)
- Zero-cost abstractions in Rust
- Educational transparency (see every detail)
- Modern GPU features (compute, ray tracing)
- Memory safety from Rust

**Cons**:
- Verbose Vulkan boilerplate
- Manual synchronization complexity
- No visual shader editor (code-only)
- Vulkan-only (no Metal/DX12 backends yet)
- Requires graphics programming knowledge

**Best For**: Learning graphics, custom engines, high-performance needs, Rust enthusiasts

## Performance Comparison

### Draw Call Batching

| Engine | Batching Strategy | Effectiveness |
|--------|------------------|---------------|
| Unity | Dynamic batching, SRP batcher, GPU instancing | Good (SRP batcher excellent) |
| Unreal | GPU instancing, mesh merging | Excellent (automatic ISM) |
| Godot | MultiMesh, GPU instancing | Good (manual setup) |
| Praxis | Manual batching, indirect drawing | Excellent (full control) |

### Shader Compilation

| Engine | Compile Time | Runtime Overhead | Caching |
|--------|--------------|------------------|---------|
| Unity | Medium (variants) | Low (cached) | Persistent |
| Unreal | High (material graph) | Low (PSO cache) | Per-project |
| Godot | Fast (simple shaders) | Low | Per-project |
| Praxis | Fast (SPIR-V) | Very low | Manual |

### Memory Bandwidth

| Engine | Deferred GBuffer Size (1080p) | Notes |
|--------|------------------------------|-------|
| Unity HDRP | ~40-60 MB | Optimized GBuffer layout |
| Unreal | ~50-70 MB | Rich GBuffer (multiple RTs) |
| Godot | ~30-50 MB | Simpler forward+ |
| Praxis | Variable | Configurable (full control) |

## Key Takeaways

### Universal Principles

1. **Deferred vs. Forward Trade-Off**:
   - Deferred: Many lights cheap, bandwidth heavy, no MSAA
   - Forward: MSAA possible, less bandwidth, expensive lights
   - Forward+/Clustered: Best of both (Godot 4, modern engines)

2. **Abstraction Has Cost**:
   - Unity/Godot: Higher-level = easier but slower peak performance
   - Unreal: Mid-level = good balance
   - Praxis/Vulkan: Low-level = maximum control but complex

3. **Shader Compilation is Critical**:
   - Precompile all variants (avoid runtime hitches)
   - Cache SPIR-V/DXIL/Metal bytecode
   - Persistent shader cache between sessions

4. **Batching is Essential**:
   - Minimize state changes (material, pipeline)
   - Use GPU instancing for repeated meshes
   - Indirect drawing for GPU-driven rendering

5. **Platform Matters**:
   - Mobile: Forward rendering, simplified shaders
   - PC/Console: Deferred, advanced features
   - Cross-platform: Abstract differences carefully

### Design Patterns to Steal

- **Command Buffer Recording**: Separate recording from execution (Vulkan, DX12)
- **Frame-in-Flight**: Render frame N while GPU processes N-1 (prevents stalls)
- **Descriptor Sets**: Group related resources (textures, uniforms) for efficient binding
- **Render Graph**: Declare dependencies between passes, auto-optimize (Unity SRP, Unreal RDG)
- **Material Instancing**: Share shaders, vary parameters per-instance

### Common Pitfalls

- **Too Many Render Targets**: GBuffer bloat wastes bandwidth
- **Unbatched Draw Calls**: Thousands of individual draws = CPU bottleneck
- **Shader Recompilation**: Avoid dynamic shader generation
- **Synchronization Overhead**: Unnecessary GPU waits (read-backs, barriers)
- **Ignoring GPU Profiling**: Assumptions about bottlenecks are often wrong

## Further Reading

### Unity
- [Scriptable Render Pipeline](https://docs.unity3d.com/Manual/ScriptableRenderPipeline.html)
- [Universal Render Pipeline](https://docs.unity3d.com/Packages/com.unity.render-pipelines.universal@latest)
- [High Definition Render Pipeline](https://docs.unity3d.com/Packages/com.unity.render-pipelines.high-definition@latest)

### Unreal
- [Rendering Programming](https://docs.unrealengine.com/5.0/en-US/rendering-programming-in-unreal-engine/)
- [Material Editor](https://docs.unrealengine.com/5.0/en-US/unreal-engine-material-editor-user-guide/)
- [Render Dependency Graph](https://docs.unrealengine.com/5.0/en-US/render-dependency-graph-in-unreal-engine/)

### Godot
- [Rendering Architecture](https://docs.godotengine.org/en/stable/tutorials/rendering/index.html)
- [Shading Language](https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/index.html)
- [Clustered Rendering](https://godotengine.org/article/clustered-forward-rendering-in-godot-4-0)

### Praxis
- [Praxis Graphics](../../guides/rendering.md)
- [Vulkano Tutorial](https://vulkano.rs/guide/introduction)

### General
- [Real-Time Rendering Resources](http://www.realtimerendering.com/)
- [Learn OpenGL](https://learnopengl.com/) (concepts apply broadly)
- [Vulkan Tutorial](https://vulkan-tutorial.com/)
