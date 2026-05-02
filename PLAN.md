# Phase 6: PBR Materials & load-bearing data shapes

## Context

Phase 5 done. Three helmets render via `bevy_ecs` query →
`prepare_renderables` → one persistent instance buffer → indexed draws.
Per-entity uniform buffers are gone; per-material bind groups remain.
Each `Mesh` still owns its own `wgpu::Buffer` for vertices and indices,
and each material still has its own texture bind group.

Phase 6 has two threads, equal weight, named in `ROADMAP.md`:

**Thread A — Visible feature work (PBR rendering).** Replace Blinn-Phong
with Cook-Torrance. Multi-texture surfaces (albedo, normal,
metallic-roughness, AO, emissive). HDR linear render target with ACES
tonemap. Image-based ambient lighting via the split-sum approximation.

**Thread B — Architectural payload (data shapes).** Material-as-ID: one
structured GPU buffer of `MaterialData`, indexed by `MaterialId`,
replaces per-material bind groups. Mega-buffer mesh storage: one shared
vertex buffer plus one shared index buffer for all meshes; meshes are
`(vertex_offset, index_offset, index_count)` sub-allocations. Texture
arrays (precursor to bindless): one `texture_2d_array` per channel;
materials reference layers by index.

Thread B is the load-bearing story. Phase 7 (scene graph + persistent
scene buffer), Phase 9 (frustum culling + frame-data snapshot), Phase 10
(bindless), Phase 11 (GPU-driven), Phase 12 (shadows), Phase 13
(streaming), and the RT phases all assume mega-buffer mesh storage and
material-as-ID. Per-mesh `wgpu::Buffer` and per-material bind group are
dead-end shapes — either rewritten now, or rewritten under five later
phases at once.

ROADMAP frames PBR as the *excuse* to introduce these shapes: PBR is
the visible deliverable that motivates the architectural rewrite.
Phase 6's result on screen — physically-grounded shading, surface
detail, IBL-lit helmets — is a side effect of getting the data flow
right.

## Decision: data shapes first, then PBR

Three orderings considered.

**A — PBR first, refactor at end.** Build Cook-Torrance + textures with
the current per-material bind groups and per-mesh buffers, then rewrite
to the final shapes. *Rejected.* Massive throwaway. Violates `CLAUDE.md`
"no throwaway code".

**B — Data shapes first, PBR layered on top.** Migrate mesh storage to
mega-buffers and material storage to material-as-ID with Blinn-Phong
still rendering. Then layer Cook-Torrance, then textures, then HDR,
then IBL. Every step preserves a working build. **Picked.**

**C — Hybrid: minimum-viable PBR shape from Step 1.** Same end state as
B but interleaves visible and architectural work. Acceptable but harder
to reason about — each step touches both the shading model and the data
flow.

Order **B** wins because every checkpoint is "looks identical" or
"looks visibly better". No step regresses, no step throws away.

## Architecture decisions

| # | Decision | Choice | Why |
|---|---|---|---|
| 1 | Texture array layout | One `texture_2d_array` per channel — albedo, normal, MR, AO, emissive | Simpler than unified atlas; bindless (Phase 10) merges later if useful |
| 2 | HDR target format | `Rgba16Float` | Half-precision sufficient for tone-mapped output; half the bandwidth of `Rgba32Float` |
| 3 | Tone mapping | ACES filmic curve | Industry standard; matches Filament/UE/Unity defaults |
| 4 | IBL scope | Full split-sum (irradiance cubemap + prefiltered specular + BRDF LUT) | Stop-gap ambient is throwaway; split-sum is canonical |
| 5 | IBL prefilter timing | Compute shaders at startup | Offline-baked is faster but adds tooling; runtime compute works for one HDR |
| 6 | Tangent source | Load from glTF when present; defer mikktspace until a model lacks tangents | DamagedHelmet ships tangents; computing them is a yak shave we don't need yet |
| 7 | Material storage | SSBO — `var<storage, read>` | Unbounded vs uniform's 16 KB cap; matches the GPU-driven shape coming later |
| 8 | Texture size constraint | All layers in an array must match dimensions; standardise on 2048×2048, scale-pad on load | Bindless (Phase 10) lifts this; for Phase 6 it's a real constraint |
| 9 | Vertex layout addition | Add `tangent: vec4<f32>` (xyz = tangent, w = bitangent sign per glTF spec) | Required for normal maps; standard glTF shape |
| 10 | Render pipeline split | Two passes — geometry → HDR target, tonemap → swap chain | Required for HDR; no shortcut |
| 11 | Mega-buffer growth | CPU re-upload to a larger buffer on overflow | Simple, slow at scale; revisit when streaming bites |
| 12 | Material handle | Plain `MaterialHandle(u32)`, no generations | Phase 6 never deletes from `MaterialPool`; revisit at Phase 13 |

## Gotchas

### `vec3<f32>` in WGSL uniform/storage structs

`MaterialData` will tempt `vec3<f32>` for colors. WGSL pads `vec3<f32>`
to 16 bytes silently — the Rust-side `[f32; 3]` is 12 bytes and the
layouts mismatch with no error. Use `vec4<f32>` or explicit pad fields.
Same trap `CLAUDE.md` flags. The recommended `MaterialData` layout in
Step 2 is all `vec4`s for this reason.

### sRGB confusion

The single most common bug class in PBR. Per-texture format must match
content:

| Texture | Storage format | Why |
|---|---|---|
| Albedo / base color | sRGB | Authored in sRGB; sampler auto-linearises on read |
| Emissive | sRGB | Color values, treated as sRGB-encoded |
| Normal | linear (`Rgba8Unorm`) | Vector data, not color |
| Metallic-roughness | linear | Scalar parameters |
| AO | linear | Scalar parameter |

HDR target is linear (`Rgba16Float`); swap chain target is sRGB
(`Bgra8UnormSrgb` or similar) so the tonemap pass writes linear and the
hardware encodes gamma on output. Document each texture's format on
load.

### Texture array dimension constraint

Uploading a 1024×1024 image into a 2048×2048 array layer truncates or
mis-aligns silently. Validate at load time: scale-pad to 2048×2048 (or
whatever the array dimension is) before `queue.write_texture`. Mip
count must also match across layers.

### Cook-Torrance division-by-zero

`(D · F · G) / (4 · NdotV · NdotL)` blows up at grazing angles where
`NdotV` or `NdotL` reach zero. Add `+ 0.001` to the denominator (or
`max(NdotV * NdotL, 0.001)`). LearnOpenGL's reference shader does the
same.

### HDR target + depth attachment

Both are recreated on resize. Both must match sample count, dimensions,
and the surface size. The depth view in the bind group used by any
sampling pass (none in Phase 6, but coming in shadows) must also be
recreated — bind groups snapshot views, not buffers.

### IBL prefilter requires compute

Prefilter passes run in a compute shader. Most desktops support
compute; mobile and web back-ends may not. Check `Features::COMPUTE_SHADER`
at adapter request time and fail fast if absent (Phase 6 doesn't need a
no-compute fallback — that's a Phase-13+ portability concern).

### Cubemap face winding

wgpu cubemap layer order: +X, -X, +Y, -Y, +Z, -Z. Equirectangular →
cubemap conversion must respect this. Easiest sanity check: render the
cubemap as a skybox and look around — flipped axes are visually obvious.

### Mega-buffer growth

Grow strategy in Phase 6: when `MeshPool::insert` would exceed
`vertex_capacity`, allocate a new buffer at 2× size, re-upload the
existing data from CPU, swap. The CPU path is slow but simple. The fast
path uses `CommandEncoder::copy_buffer_to_buffer`; it requires the
existing buffer to have `COPY_SRC` usage and is a small change later.

### Bind-group rebuild on buffer recreation

Same pattern as Phase 5's instance buffer overflow. The material-buffer
bind group references the materials buffer; when the buffer is
recreated for growth, the bind group must be recreated too. Phase 5
already understands this — re-apply.

### Material handle stability

`MaterialHandle` is a plain `u32` index into `MaterialPool::materials`.
Stable as long as we never delete. Phase 6 doesn't delete. Generational
handles arrive when Phase 13 (streaming) needs them.

### glTF material → engine material translation

A glTF `Material` carries factor values + texture references with their
own UV coords + samplers. Translate at load:

- `pbrMetallicRoughness.baseColorFactor` → `MaterialData::base_color`
- `pbrMetallicRoughness.metallicFactor` / `.roughnessFactor` →
  `MaterialData::metallic_roughness.x` / `.y`
- `emissiveFactor` → `MaterialData::emissive.xyz`
- Texture references → upload pixels into the per-channel arrays, store
  layer index in `MaterialData::texture_indices`

If a texture is absent, point at a default layer (1×1 white for
albedo/MR/AO, flat normal for normals, black for emissive). Default
layers live at fixed indices written into the arrays at startup.

---

## Mental models

Cross-cutting concepts. Read once before starting Step 1. The Phase 6
reading contract: AI provides inline teaching at each substep that
introduces new theory (Steps 3, 4, 5, 6). The condensed primers below
are scaffolding; the deep dives happen at the substep, not now.

### PBR / Cook-Torrance

Physically-based rendering models surfaces as a microfacet
distribution. The reflected radiance from a point is the integral of
incoming light over the upper hemisphere, weighted by a bidirectional
reflectance distribution function (BRDF). Cook-Torrance factorises the
specular BRDF as `(D · F · G) / (4 · NdotV · NdotL)`:

- **D — Normal distribution function.** What fraction of microfacets
  point in the half-vector direction. GGX is the standard choice;
  rougher surfaces spread the lobe wider.
- **F — Fresnel.** What fraction of incoming light reflects vs
  transmits, as a function of viewing angle. Schlick approximation:
  `F0 + (1 - F0) · pow(1 - HdotV, 5)`. `F0` is the surface's reflectance
  at normal incidence — 0.04 for non-metals, base color for metals.
- **G — Geometry / shadowing-masking.** Microfacets shadow each other
  at grazing angles. Smith function, separable into masking +
  shadowing terms.

Diffuse term: `(1 - F) · (1 - metallic) · baseColor / π`. Metals have
no diffuse — they reflect entirely via the specular path, with `F0`
tinted by the base color.

The metallic-roughness workflow encodes physically meaningful
parameters in two scalar values per texel: `metallic` selects between
dielectric (plastic, wood, fabric) and conductor (metal); `roughness`
controls the microfacet distribution width.

References: LearnOpenGL PBR chapters; Filament design doc
(google.github.io/filament/Filament.md.html). Step 3 expands this with
intuition + diagrams + the actual WGSL.

### Linear / sRGB / HDR pipeline

Where lighting math happens vs where pixels are stored, end to end:

```
sRGB albedo texture
  → sampler auto-linearises on read (sRGB format)
  → math in linear space
  → write to Rgba16Float HDR target
  → tonemap fragment shader reads HDR, applies ACES
  → writes to sRGB swap chain target
  → hardware auto-encodes gamma on present
  → display
```

Three rules that prevent 90% of PBR color bugs:

1. **All shading math is linear.** No `pow(color, 2.2)` smuggled in;
   the sampler does it for sRGB textures. Don't double-correct.
2. **Color textures are sRGB; data textures are linear.** Albedo and
   emissive are sRGB. Normal, MR, AO are linear. Tag at load.
3. **HDR target is linear; swap chain is sRGB.** The tonemap pass
   bridges them. The geometry pass writes linear values that may
   exceed 1.0; the tonemap pass compresses them into [0, 1] for
   display.

### IBL split-sum approximation

Image-based ambient lighting integrates incoming radiance from an
environment cubemap over the hemisphere. The integral is too expensive
per pixel; Karis (Unreal Engine 4) split it into two pre-computable
terms:

```
ambientSpecular ≈ prefilteredEnv(R, roughness) · (F · brdf.x + brdf.y)
ambientDiffuse  = irradiance(N) · (baseColor / π) · (1 - F) · (1 - metallic)
```

Three artifacts pre-computed once at startup:

- **Irradiance cubemap** (32×32 typical). Each face texel stores the
  cosine-weighted hemisphere integral for that direction. Diffuse
  ambient samples it by surface normal.
- **Prefiltered specular cubemap** (mip chain, e.g. 5 levels from
  128×128 down to 8×8). Each mip stores the GGX-importance-sampled
  environment for a roughness value (mip 0 = mirror, max mip = fully
  rough). Specular ambient samples by reflection vector with mip
  selected from roughness.
- **BRDF LUT** (256×256, `Rg16Float`). A 2D table indexed by
  `(NdotV, roughness)` storing the precomputed Fresnel-weighted
  microfacet integral. Per-pixel cost is one texture sample.

The split is approximate but visually convincing and standard across
modern engines. Step 6 derives the math and walks through the
prefilter compute shaders.

### Mega-buffer mesh storage

One shared `wgpu::Buffer` for vertex data, one for index data, holding
every mesh's vertices/indices end-to-end. Meshes become offsets into
those buffers. The renderer binds the vertex/index buffers once and
issues `draw_indexed(indices_range, base_vertex, instance_range)` per
mesh, where `indices_range` and `base_vertex` come from the mesh's
sub-allocation record.

Why this shape:

- **Indirect draws.** Phase 11 needs the GPU to issue draws by reading
  a buffer of `(index_count, instance_count, base_index, base_vertex,
  base_instance)` records. That requires every mesh's geometry to be
  in one buffer it can address by offset.
- **BLAS construction.** Phase 17 ray-tracing wants per-mesh BLAS built
  over a contiguous range of a mega-buffer. Buffer-per-mesh forces an
  extra copy.
- **Bind churn.** With one shared vertex buffer, `set_vertex_buffer`
  is called once per frame. Buffer-per-mesh calls it per draw.

Migration is local to `MeshPool` and `Primitive`. Renderer-side: drop
`set_vertex_buffer` / `set_index_buffer` from the per-mesh loop, add
them once before the loop. Indexed draw becomes
`draw_indexed(prim.index_offset..prim.index_offset + prim.index_count,
prim.vertex_offset as i32, instance_range)`.

### Material-as-ID

Materials are entries in a structured GPU buffer indexed by
`MaterialId` (a `u32`). One bind group serves all materials: the
material buffer + the per-channel texture arrays + a sampler.
Per-instance data carries the material ID; the fragment shader reads
`materials[instance.material_id]` to get parameters and texture
indices, then samples each texture array by index.

Why this shape:

- **Bindless precursor.** Phase 10 swaps the per-channel texture
  arrays for one big bindless descriptor array. The data flow — "look
  up material by ID, then sample textures by index" — stays. The
  bindless transition becomes additive.
- **GPU-driven.** Phase 11 indirect draws want material parameters
  reachable from the GPU without CPU-issued bind-group changes.
  Material-as-ID delivers exactly that.
- **One bind-group rebind, not many.** Per-material bind groups force
  a `set_bind_group` call between every draw. Material-as-ID binds
  once per frame.

The `texture_indices` field in `MaterialData` is a `[u32; 4]` packing
albedo / normal / MR / AO indices. Emissive index lives in `extra.x`.
Default layers (white-1×1 albedo, flat normal, etc.) live at fixed
indices in each array so a missing texture has a valid fallback.

---

## Steps

Six steps. Each ends with a visible or verifiable checkpoint. The rule:
no step advances until its checkpoint holds.

Steps 1 and 2 are architectural; the rendered image must be
**identical** to Phase 5 output at their checkpoints. Steps 3–6 are
visible; each adds a measurable improvement.

### Step 1: Mega-buffer mesh storage

_One shared `wgpu::Buffer` for all vertex data, one for all index data.
`Primitive` stores `(vertex_offset, vertex_count, index_offset,
index_count)` — no per-primitive buffers. Visible behavior unchanged._

**Notes.** This step is invisible. The goal is the data flow, not the
picture. The picture must look identical to Phase 5 at the checkpoint.

Final shape:

```rust
pub struct Primitive {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
    // material reference comes in Step 2
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

pub struct MeshPool {
    meshes: Vec<Mesh>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    vertex_capacity: u64,
    index_capacity: u64,
    vertex_used: u64,
    index_used: u64,
}
```

`MeshPool::insert(vertices, indices, ...)` writes data at the current
offsets, advances `vertex_used` / `index_used`, and returns the `Mesh`
with `Primitive` records carrying the offsets. Growth path: when an
`insert` would overflow capacity, allocate a new buffer at 2× size,
re-upload existing data from CPU, swap the buffer handle.

Render-loop change: bind shared vertex + index buffers **once** before
iterating primitives. Per-primitive draw becomes:

```rust
pass.draw_indexed(
    prim.index_offset..prim.index_offset + prim.index_count,
    prim.vertex_offset as i32,
    instance_range,
);
```

Where to look:

- `src/assets/mesh.rs` (current `Mesh` / `Primitive` types) — add the
  offset/count fields, drop per-primitive `wgpu::Buffer` ownership.
- `src/assets/pools.rs` (or wherever `MeshPool` lives) — add the two
  shared buffers + capacity tracking + growth logic.
- `src/assets/loaders/gltf.rs` (or equivalent) — `load_model` writes
  vertex + index data into `MeshPool` instead of building per-primitive
  buffers.
- `src/render/...` — the render path drops `set_vertex_buffer` /
  `set_index_buffer` from the per-mesh loop and binds the pool's
  shared buffers once before iterating.
- Initial capacity is a tuning call. Pick `1 MiB` of vertex + `256 KiB`
  of index as a first guess; growth handles the rest. DamagedHelmet's
  largest primitive is well under this.

- [ ] Done

Checkpoint: 3 helmets render identically. `MeshPool` exposes one vertex
buffer and one index buffer. `cargo build && cargo run` shows the same
pixels as end-of-Phase-5.

### Step 2: Material-as-ID + structured material buffer + texture arrays

_One bind group for all materials. Materials are entries in an SSBO
indexed by `MaterialId`. Textures live in per-channel
`texture_2d_array`s. Visible behavior unchanged: still Blinn-Phong, but
data flows through the new shape._

**Notes.** This step is also invisible. The existing Blinn-Phong shader
keeps running but reads material params from the structured buffer
instead of bind-group-per-material uniforms.

Final shape (CPU side):

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialData {
    pub base_color: [f32; 4],
    pub metallic_roughness: [f32; 4],   // x=metallic, y=roughness, zw=pad
    pub emissive: [f32; 4],             // xyz=color, w=strength
    pub texture_indices: [u32; 4],      // x=albedo, y=normal, z=MR, w=AO
    pub extra: [u32; 4],                // x=emissive_idx, yzw=pad
}

pub struct MaterialPool {
    materials: Vec<MaterialData>,
    materials_buffer: wgpu::Buffer,           // SSBO
    albedo_array: wgpu::Texture,              // texture_2d_array
    normal_array: wgpu::Texture,
    metallic_roughness_array: wgpu::Texture,
    ao_array: wgpu::Texture,
    emissive_array: wgpu::Texture,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout, // material_buffer + 5 arrays + sampler
    bind_group: wgpu::BindGroup,
    capacity: u64,
}
```

WGSL side:

```wgsl
struct MaterialData {
    base_color: vec4<f32>,
    metallic_roughness: vec4<f32>,
    emissive: vec4<f32>,
    texture_indices: vec4<u32>,
    extra: vec4<u32>,
};

@group(2) @binding(0) var<storage, read> materials: array<MaterialData>;
@group(2) @binding(1) var albedo_array: texture_2d_array<f32>;
@group(2) @binding(2) var normal_array: texture_2d_array<f32>;
@group(2) @binding(3) var mr_array: texture_2d_array<f32>;
@group(2) @binding(4) var ao_array: texture_2d_array<f32>;
@group(2) @binding(5) var emissive_array: texture_2d_array<f32>;
@group(2) @binding(6) var pbr_sampler: sampler;
```

Per-instance data gains `material_id: u32`. Vertex shader passes it
through to the fragment shader; the fragment reads
`materials[instance.material_id]`.

ECS side: `MaterialRef(MaterialHandle)` component (Phase 5 declared the
marker; flesh out the handle here). `HelmetAssets` (or whatever the
spawn-time bundle is called) gains `material: MaterialHandle`. The
spawn system attaches `MaterialRef(assets.material)` on each entity.

Texture array setup: standardise on 2048×2048 RGBA. Default layers at
fixed indices (e.g. layer 0 = white 1×1 scaled to 2048; layer 1 = flat
normal `(0.5, 0.5, 1.0)` scaled to 2048) so missing textures have valid
fallbacks. glTF materials without an explicit texture point at the
default layer.

Where to look:

- `src/assets/material.rs` — `MaterialData`, `MaterialHandle`,
  `MaterialPool` definitions.
- `src/components.rs` — `MaterialRef` already declared in Phase 5;
  ensure it wraps `MaterialHandle`.
- `src/render/instance.rs` — `InstanceData` gains `material_id: u32`.
  Bump WGSL `InstanceData` layout to match (16-byte alignment).
- `src/shader.wgsl` — switch from per-material uniform sample to
  buffer-indexed lookup. Existing Blinn-Phong math stays; only the
  parameter source changes.
- `src/assets/loaders/gltf.rs` — translate glTF `Material` to
  `MaterialData`. Upload textures to the per-channel arrays at load.
  Track which layer each texture lives at; populate `texture_indices`
  accordingly.
- `src/state.rs` — drop the per-material `texture_bind_group` /
  `texture_bind_group_layout` if they exist; the `MaterialPool` bind
  group replaces them. Pipeline layout updated to reference the new
  bind group.

- [ ] Done

Checkpoint: 3 helmets render identically (still Blinn-Phong). One bind
group rebind per frame regardless of material count. Spawning a fourth
helmet sharing the same `MaterialHandle` works without allocating a
new bind group.

### Step 3: Cook-Torrance BRDF

_Replace Blinn-Phong with Cook-Torrance. Single shader, no new
bindings. First visible change in Phase 6._

**Theory block (inline teaching, ~30–60 min).** Microfacet model
intuition (rough surface = many tiny facets with normals near `N`).
Why factorise the BRDF into `D · F · G / (4 · NdotV · NdotL)`. ASCII
diagram of half-vector geometry. Schlick Fresnel derivation. GGX vs
other NDFs (why GGX won — long tails match real surfaces). Smith
geometry. Energy conservation: `kD = (1 - F) · (1 - metallic)`. Why
metals have no diffuse. The metallic/roughness workflow as a
2-parameter physical encoding.

WGSL functions to write:

```wgsl
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3(1.0) - F0) * pow(1.0 - cos_theta, 5.0);
}

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx_v = NdotV / (NdotV * (1.0 - k) + k);
    let ggx_l = NdotL / (NdotL * (1.0 - k) + k);
    return ggx_v * ggx_l;
}
```

Per-light contribution:

```wgsl
let H = normalize(V + L);
let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
let D = distribution_ggx(N, H, roughness);
let G = geometry_smith(N, V, L, roughness);
let specular = (D * F * G) / (4.0 * NdotV * NdotL + 0.001);
let kD = (vec3(1.0) - F) * (1.0 - metallic);
let diffuse = kD * baseColor / PI;
Lo += (diffuse + specular) * radiance * NdotL;
```

`F0` for non-metals: `vec3(0.04)`. For metals:
`mix(vec3(0.04), baseColor, metallic)`.

Apply across the existing 1 directional + 4 point lights from
`LightUniform`. Loop over all five.

Where to look:

- `src/shader.wgsl` — replace the Blinn-Phong fragment body with
  Cook-Torrance. Keep ambient at constant `0.03 * baseColor` for now;
  Step 6 replaces it with IBL.
- Material-derived params: `baseColor` from `mat.base_color.rgb`,
  `metallic` from `mat.metallic_roughness.x`, `roughness` from
  `mat.metallic_roughness.y`. Textures don't apply yet (Step 4); use
  the factor values directly.
- Add `PI = 3.14159265` as a WGSL `const`.

- [ ] Done

Checkpoint: 3 helmets shaded with Cook-Torrance. Highlights are
roughness-dependent (smooth = tight, rough = wide). Metallic surfaces
tinted by base color in specular. Compared to Phase 5 Blinn-Phong, the
shape of the highlights differs visibly. Helmets still recognisable;
overall brightness and colors comparable.

### Step 4: PBR texture maps + tangents

_Apply albedo, normal, metallic-roughness, AO, and emissive textures.
Vertex layout gains tangents._

**Theory block (inline teaching, ~30–60 min).** Tangent space: why
per-vertex tangent + bitangent + normal form an orthonormal frame that
lets normal maps store *relative* perturbations independent of mesh
orientation. ASCII diagram of TBN basis. `tangent.w` as the bitangent
sign (handedness fix for mirrored UVs). Computing TBN in the vertex
shader (transform tangent + normal by world matrix; reconstruct
bitangent from `cross(N, T) * tangent.w`). Normal-map sampling: read
RGB, expand from `[0, 1]` to `[-1, 1]` (`2.0 * sample - 1.0`), interpret
as tangent-space vector, transform to world space via TBN. Why normal
maps speed up surface detail — geometry resolution unchanged, shading
resolution increased.

Vertex layout:

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],   // xyz = tangent, w = bitangent sign
    pub uv: [f32; 2],
}
```

Loader (glTF) reads the `TANGENT` accessor when present; reject (or
warn + skip) primitives without tangents in Phase 6. Mikktspace
fallback is deferred — note in the plan, address only when a model
breaks it.

Vertex shader: transform `tangent.xyz` and `normal` by the model matrix
(rotation only — drop translation), pass `T_world` and `N_world` plus
the bitangent sign to the fragment shader; build TBN there.

Fragment shader:

```wgsl
let T = normalize(in.tangent_world);
let N_geom = normalize(in.normal_world);
let B = cross(N_geom, T) * in.tangent_w;
let TBN = mat3x3<f32>(T, B, N_geom);

let n_sample = textureSample(normal_array, pbr_sampler, in.uv,
                              i32(mat.texture_indices.y)).rgb;
let n_tangent = n_sample * 2.0 - vec3(1.0);
let N = normalize(TBN * n_tangent);

let albedo = textureSample(albedo_array, pbr_sampler, in.uv,
                           i32(mat.texture_indices.x)).rgb
             * mat.base_color.rgb;
let mr = textureSample(mr_array, pbr_sampler, in.uv,
                       i32(mat.texture_indices.z)).rg;
let metallic = mr.r * mat.metallic_roughness.x;
let roughness = mr.g * mat.metallic_roughness.y;
let ao = textureSample(ao_array, pbr_sampler, in.uv,
                       i32(mat.texture_indices.w)).r;
let emissive = textureSample(emissive_array, pbr_sampler, in.uv,
                             i32(mat.extra.x)).rgb * mat.emissive.rgb;
```

glTF metallic-roughness convention: green = roughness, blue = metallic.
The `.r` / `.g` sample above matches that convention when the texture
is uploaded with the channel order. Verify against the asset.

Where to look:

- `src/assets/mesh.rs` (or wherever `Vertex` lives) — add the `tangent`
  field. Update the vertex buffer layout descriptor.
- `src/assets/loaders/gltf.rs` — read the `TANGENT` accessor; emit
  warning if absent.
- `src/shader.wgsl` — vertex stage passes `tangent_world` +
  `bitangent_sign` to fragment. Fragment builds TBN, samples 5 texture
  arrays, modulates Cook-Torrance inputs.
- `MaterialPool` texture upload (Step 2) handles all 5 channels;
  ensure the loader uploads each one.

- [ ] Done

Checkpoint: helmet surface shows damage in normals and AO. Metallic
strips read distinctly from rough painted areas. Emissive areas (the
helmet's visor edges) glow. Lighting unchanged in concept; surface
detail dramatically improved.

### Step 5: HDR target + ACES tonemap + gamma

_Render geometry into a linear `Rgba16Float` HDR target. Tonemap pass
samples the HDR and writes to the sRGB swap chain._

**Theory block (inline teaching, ~30–60 min).** Why HDR: real lighting
exceeds `[0, 1]` (sun is ~10,000 in some normalisations; light bulbs
are hundreds). Clipping at 1.0 destroys highlight detail. Tonemap
curves compress linear HDR into displayable LDR while preserving
perceptual contrast. ACES filmic curve as a standard choice. Gamma
encoding (sRGB transfer function) as the final step — display expects
~2.2 power-law encoded values; hardware does this for sRGB swap chain
formats.

Two-pass pipeline:

1. **Geometry pass.** Render to `Rgba16Float` HDR texture + depth.
   Output values may exceed 1.0. Clear color stays in linear space.
2. **Tonemap pass.** Fullscreen triangle samples HDR target, applies
   ACES, writes to sRGB swap chain target. Hardware encodes gamma on
   output.

ACES curve (Krzysztof Narkowicz fit, the common "ACES filmic" used
across engines):

```wgsl
fn aces_filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}
```

New artifacts:

- `hdr_color: wgpu::Texture` (`Rgba16Float`, swap chain dimensions,
  recreated on resize).
- `hdr_view: wgpu::TextureView`.
- `tonemap_pipeline: wgpu::RenderPipeline` — fullscreen-triangle vertex
  + ACES fragment.
- `tonemap_bind_group_layout` — single sampled-texture binding for the
  HDR view.
- `tonemap_bind_group` — built from `hdr_view` + a sampler. Recreated
  on resize (bind groups snapshot views).

Geometry render pass color attachment switches to `hdr_view`. Tonemap
pass color attachment is the swap chain texture view.

Where to look:

- `src/render/hdr.rs` — new module owning `hdr_color` /
  `tonemap_pipeline` / `tonemap_bind_group`. Resize hook recreates
  `hdr_color` + the bind group.
- `src/state.rs` — wire the HDR module's resize callback into the
  existing `resize()`. Both depth and HDR textures recreated together.
- `src/render/passes.rs` (or the existing render entrypoint) — split
  into two passes: geometry-into-HDR, tonemap-into-swapchain.
- New `src/tonemap.wgsl` (or fold into `shader.wgsl` with separate
  entrypoints) — fullscreen-triangle vertex + ACES fragment.
- Verify the swap chain format is sRGB (`Bgra8UnormSrgb` or similar).
  If currently `Bgra8Unorm`, switch — the tonemap shader writes linear
  and the hardware encodes gamma.

- [ ] Done

Checkpoint: highlights no longer clip to white. A bright light close
to a surface produces bright but tone-rolled-off response, not pure
white. Overall image looks visually similar in mid-tones; differences
concentrate at the bright end.

### Step 6: IBL split-sum

_Image-based ambient lighting. Equirectangular HDR loaded once,
prefiltered into irradiance + specular cubemaps + BRDF LUT at startup.
Shading samples them per-pixel for ambient diffuse + ambient specular._

**Theory block (inline teaching, ~30–60 min).** Recap of the split-sum
approximation (Mental models section). Walk through each prefilter:

- **Equirect → cubemap.** Sample the equirectangular HDR using
  `(theta, phi)` derived from the direction vector for each face
  texel. Compute shader: 6 dispatches (one per face) or one dispatch
  with a 3D output.
- **Irradiance.** For each output texel direction `N`, integrate the
  cosine-weighted hemisphere by sampling the input cubemap N×N times
  with stratified angles. Output: small (32×32 per face) low-frequency
  cubemap.
- **Prefiltered specular.** For each mip, for each output texel
  direction `R`, integrate using GGX importance sampling with
  roughness derived from mip level. Output: full-mip cubemap (mip 0
  mirror, max mip fully rough).
- **BRDF LUT.** 2D table indexed by `(NdotV, roughness)`. Each texel
  computes the Fresnel-weighted microfacet integral via importance
  sampling. Output: `Rg16Float` 256×256.

Apply in the geometry fragment shader:

```wgsl
let R = reflect(-V, N);
let NdotV = max(dot(N, V), 0.0);

let F_ibl = fresnel_schlick_roughness(NdotV, F0, roughness);
let kD_ibl = (vec3(1.0) - F_ibl) * (1.0 - metallic);

let irradiance = textureSample(irradiance_cube, pbr_sampler, N).rgb;
let diffuse_ibl = irradiance * albedo;

let MAX_REFLECTION_LOD = 4.0;   // matches mip count of prefiltered cube
let prefilteredColor = textureSampleLevel(prefiltered_cube, pbr_sampler,
                                          R, roughness * MAX_REFLECTION_LOD).rgb;
let brdf = textureSample(brdf_lut, lut_sampler, vec2(NdotV, roughness)).rg;
let specular_ibl = prefilteredColor * (F_ibl * brdf.x + brdf.y);

let ambient = (kD_ibl * diffuse_ibl + specular_ibl) * ao;
```

`fresnel_schlick_roughness` is a roughness-aware variant — a roughness
term is added inside the Fresnel to prevent ambient specular from
blowing up at glancing angles on rough surfaces. Standard PBR
addendum.

New artifacts:

- `assets/skybox.hdr` — equirectangular HDR file. Use any free HDRI
  (polyhaven.com is a good source).
- HDR loader (`image` crate's `OpenExr` or `Hdr` decoder). Pick one;
  prefer Radiance HDR (`.hdr`) for compactness.
- `EnvironmentMap` resource:
  - `env_cube: wgpu::Texture` (cubemap, mip 0 only).
  - `irradiance_cube: wgpu::Texture` (32×32 per face).
  - `prefiltered_cube: wgpu::Texture` (128×128 base, 5 mips).
  - `brdf_lut: wgpu::Texture` (256×256, `Rg16Float`).
- Compute shaders (4 of them):
  - `equirect_to_cubemap.wgsl`
  - `prefilter_irradiance.wgsl`
  - `prefilter_specular.wgsl` (per-mip dispatch)
  - `brdf_lut.wgsl`
- Bind group: extends the material bind group or sits in its own group
  (group 3, say). Either works; per-group binding-count limits make
  the choice — check device limits.

Build at startup, in order:

1. Load equirect HDR.
2. Run `equirect_to_cubemap` compute → `env_cube`.
3. Run `prefilter_irradiance` → `irradiance_cube`.
4. Run `prefilter_specular` (per-mip) → `prefiltered_cube`.
5. Run `brdf_lut` → `brdf_lut`. Cache to disk if the cost matters.

The geometry fragment shader replaces the `0.03 * baseColor` ambient
constant from Step 3 with the IBL contribution above.

Optional: render the cubemap as a skybox behind the helmets. Gives an
environment context and is also a good debug — if the skybox looks
wrong (flipped axes, wrong colors) the IBL math is wrong too. Not
load-bearing for Phase 6 but easy to add (5-line fullscreen draw +
cubemap sample). Recommend including.

Where to look:

- `src/assets/environment.rs` — HDR loading + cube buffer setup.
- `src/render/ibl.rs` — prefilter compute pipeline orchestration.
- `src/shaders/ibl/*.wgsl` — the 4 compute shaders.
- `src/shader.wgsl` — fragment ambient term replaced with IBL formula.
- Material bind group layout extended (or new group) to include the 3
  cubemap views + the LUT view.

- [ ] Done

Checkpoint: helmets exhibit plausible reflections (visible environment
in metallic strips, reflection-direction-dependent). Diffuse ambient
picks up environment color, not a constant grey. Toggling the HDR
environment swaps the lighting visibly. Metallic vs dielectric
distinct.

---

## Final shape

Where the codebase ends after Phase 6.

| Subsystem | Phase 5 | Phase 6 |
|---|---|---|
| Mesh storage | One `wgpu::Buffer` per primitive | One shared vertex buffer + one shared index buffer in `MeshPool`; primitives store offsets |
| Material | Per-material `wgpu::BindGroup` | One `wgpu::BindGroup` for `MaterialPool` (SSBO + 5 texture arrays + sampler); materials are SSBO entries |
| Texture binding | One bind group per texture | Per-channel `texture_2d_array`s; materials index by layer |
| Shading | Blinn-Phong | Cook-Torrance + PBR maps |
| Vertex layout | position + normal + uv | position + normal + tangent (vec4) + uv |
| Render passes | One: geometry → swap chain | Two: geometry → HDR target, tonemap → swap chain |
| Color pipeline | sRGB swap chain only | sRGB textures → linear math → `Rgba16Float` HDR → ACES → sRGB swap chain |
| Ambient | Constant | IBL split-sum (irradiance + prefiltered specular + BRDF LUT) |
| Instance data | `model_matrix` only | `model_matrix` + `material_id` |

What stays from Phase 5: per-frame full-rewrite of the instance buffer
(no dirty flags yet — Phase 7), per-entity draw call (instancing
collapse — natural Phase-6 follow-up), Blinn-Phong-shaped 1 directional
+ 4 point lights structure (replaced by area lights + analytic shadows
in much later phases).

What's set up for Phase 7+: the persistent scene buffer (Phase 7) lays
on top of the instance buffer with dirty flags. Frame-data snapshot
(Phase 9) reads from `World` once per frame into a `FrameData` struct
the renderer consumes. Bindless (Phase 10) collapses the per-channel
texture arrays into one big descriptor array but keeps material-as-ID.
GPU-driven (Phase 11) adds an indirect-draw buffer alongside the
instance buffer. None of these require revisiting Phase 6 work.
