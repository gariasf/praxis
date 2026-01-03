# Praxis Engine Strategic Analysis & Roadmap

*Analysis Date: January 2026*

---

## Executive Summary

Praxis is a remarkably well-architected Rust 3D game engine with ~25,000 lines of high-quality code across 11 crates. The codebase demonstrates exceptional documentation standards, clean ECS-first architecture, and solid foundational systems. However, compared to modern cutting-edge engines, several critical subsystems are missing.

**Current State: Foundation Complete, Production Features Needed**

---

## Part 1: Current State Assessment

### 1.1 Architecture Strengths

| Aspect | Rating | Notes |
|--------|--------|-------|
| Code Quality | A+ | Strict clippy (pedantic + nursery), comprehensive docs |
| Documentation | A+ | 600+ line shader comments, beginner guides, architecture docs |
| ECS Design | A | Clean bevy_ecs integration, consistent patterns |
| Rendering Core | B+ | Working Vulkan pipeline, PBR materials, dynamic lighting |
| Physics | A | Full Rapier3D integration, fixed timestep, collision events |
| Testing | C- | Minimal test coverage (~150 lines) |
| Asset Pipeline | C | OBJ only, no GLTF, no hot-reload |

### 1.2 Feature Completeness Matrix

```
IMPLEMENTED                    MISSING
═══════════════════════════    ═══════════════════════════════════
✅ Forward rendering           ❌ Deferred rendering
✅ Blinn-Phong lighting        ❌ PBR IBL (environment probes)
✅ Dynamic point/dir lights    ❌ Shadow mapping
✅ PBR material properties     ❌ Normal/parallax mapping
✅ Texture sampling (PNG/JPG)  ❌ Texture compression (KTX2, BC)
✅ Transform hierarchy         ❌ Skeletal animation
✅ Rapier3D physics            ❌ Audio system
✅ Collision detection         ❌ Particle system
✅ OBJ model loading           ❌ GLTF/GLB support
✅ Basic egui debug UI         ❌ Visual scene editor
✅ Keyboard/mouse input        ❌ Advanced post-processing
✅ Gamepad support (gilrs)     ❌ Scripting (Lua/Rhai)
```

### 1.3 Technical Debt Identified

**Low-Risk Items (cosmetic):**
- 1 TODO in physics (`shape_cast` not implemented)
- Some `unwrap()` calls in test code (acceptable)
- gui_demo.rs is a placeholder (5 lines)

**Medium-Risk Items:**
- No normal matrix calculation for non-uniform scaling in vertex shader
- `expect()` on tracing directives in observability.rs
- No depth buffer in render pass (Z-fighting possible)

**High-Risk Items:**
- No test coverage for critical rendering paths
- No benchmarks for performance regression detection
- Dynamic uniform buffer hardcoded to 1024 objects

---

## Part 2: Modern Engine Comparison

### 2.1 Where Cutting-Edge Engines Are in 2026

| Feature | Bevy | Godot 4 | Unity | Unreal | Praxis |
|---------|------|---------|-------|--------|--------|
| Render Graph | ✅ | ✅ | ✅ | ✅ | ❌ |
| GPU-Driven Rendering | ✅ | Partial | ✅ | ✅ | ❌ |
| Clustered Lighting | ✅ | ✅ | ✅ | ✅ | ❌ |
| Shadow Maps | ✅ | ✅ | ✅ | ✅ | ❌ |
| Mesh Shaders | Partial | ❌ | ✅ | ✅ | ❌ |
| Raytracing | Experimental | Partial | ✅ | ✅ | ❌ |
| GLTF Support | ✅ | ✅ | ✅ | ✅ | ❌ |
| Animation | ✅ | ✅ | ✅ | ✅ | ❌ |
| Audio | ✅ | ✅ | ✅ | ✅ | ❌ |
| Visual Editor | ✅ | ✅ | ✅ | ✅ | ❌ |
| Hot Reload | ✅ | ✅ | ✅ | ✅ | ❌ |

### 2.2 Rust Engine Landscape Analysis

**Bevy (Primary Competitor):**
- Massive community, plugin ecosystem
- Full ECS, render graph, audio, UI
- WebGPU/wgpu based (more portable)
- Risk: Feature parity difficult

**Fyrox (Formerly rg3d):**
- Full-featured 3D engine
- Visual scene editor
- Animation system
- Risk: Less active community

**Praxis Unique Position:**
- Educational focus (exceptional docs)
- Vulkano-based (closer to metal)
- Leaner, more hackable
- Opportunity: Specialized niches

---

## Part 3: Cleanup Recommendations

### 3.1 Immediate Cleanups (Priority 1)

```rust
// 1. Add depth buffer to render pass (prevents Z-fighting)
// Location: praxis_graphics/src/lib.rs:1142-1158
attachments: {
    color: { format: format, samples: 1, load_op: Clear, store_op: Store },
    depth: { format: Format::D32_SFLOAT, samples: 1, load_op: Clear, store_op: DontCare }  // ADD THIS
}

// 2. Implement shape_cast in PhysicsWorld
// Location: praxis_physics/src/resources.rs:691
// TODO: Implement shape casting

// 3. Replace gui_demo.rs placeholder with real demo
// Location: examples/gui_demo.rs (5 lines → ~100 lines)
```

### 3.2 Code Quality Improvements (Priority 2)

1. **Add comprehensive tests:**
   - Unit tests for each crate (~80% coverage target)
   - Integration tests for cross-crate functionality
   - Render comparison tests using pixel hashing

2. **Add benchmarks:**
   - `cargo bench` suite for performance regression
   - Track: mesh upload, render loop, physics step

3. **Improve error types:**
   - Replace generic `eyre::Report` with domain-specific errors
   - Better error context in graphics operations

### 3.3 Architecture Improvements (Priority 3)

1. **Render Graph System:**
   - Abstract render pass dependencies
   - Enable easier post-processing pipeline
   - Reference: Bevy's render graph design

2. **Asset Handle System:**
   - Type-safe handles instead of string IDs
   - Reference counting for automatic cleanup
   - Async loading with progress tracking

3. **Dynamic Object Limit:**
   - Make 1024 object limit configurable
   - Auto-grow buffer when needed
   - Add warning when approaching limit

---

## Part 4: Strategic Roadmap

### Phase 1: Foundation Hardening (1-2 months)

**Goal:** Solidify existing systems before adding features

| Task | Priority | Effort | Impact |
|------|----------|--------|--------|
| Add depth buffer | Critical | Low | Fixes Z-fighting |
| Add test suite (50%+ coverage) | High | Medium | Reliability |
| Add benchmarks | High | Low | Performance tracking |
| Implement `shape_cast` | Medium | Low | Physics completeness |
| Real GUI demo | Low | Low | Documentation |

### Phase 2: Essential Rendering Features (2-3 months)

**Goal:** Basic visual feature parity with indie engines

| Task | Priority | Effort | Impact |
|------|----------|--------|--------|
| Shadow mapping (directional) | Critical | High | Visual quality |
| Normal mapping | High | Medium | Material quality |
| GLTF loader | High | Medium | Content pipeline |
| Post-processing framework | High | Medium | Enables effects |
| Bloom effect | Medium | Low | Visual polish |
| Skybox/Cubemap | Medium | Low | Environment |

**Shadow Mapping Implementation Strategy:**
```
1. Create depth-only render pass for shadow map
2. Render scene from light's perspective
3. Sample shadow map in fragment shader
4. Use PCF (Percentage Closer Filtering) for soft edges
5. Support: 1 directional light initially, then cascade
```

### Phase 3: Animation & Audio (2-3 months)

**Goal:** Support animated content and audio

| Task | Priority | Effort | Impact |
|------|----------|--------|--------|
| Skeletal animation system | High | High | Character animation |
| Animation blending | High | Medium | Smooth transitions |
| Audio system (kira/rodio) | High | Medium | Game feel |
| 3D positional audio | Medium | Low | Immersion |

**Animation System Architecture:**
```
Components:
  - Skeleton: bone hierarchy
  - AnimationClip: keyframe data
  - AnimationPlayer: playback state
  - SkinMesh: vertex weights

Systems:
  - animation_update_system: advance time, interpolate
  - skeletal_transform_system: compute bone matrices
  - skin_mesh_system: apply bone transforms to vertices
```

### Phase 4: Advanced Rendering (3-4 months)

**Goal:** Modern rendering techniques

| Task | Priority | Effort | Impact |
|------|----------|--------|--------|
| Deferred rendering option | Medium | High | Many lights |
| Clustered forward | Medium | High | Alternative to deferred |
| SSAO | Medium | Medium | Visual depth |
| HDR + Tonemapping | Medium | Medium | Dynamic range |
| Environment probes | Low | High | Realistic reflections |

### Phase 5: Editor & Tools (4-6 months)

**Goal:** Visual development workflow

| Task | Priority | Effort | Impact |
|------|----------|--------|--------|
| Scene editor (egui-based) | High | High | UX |
| Asset hot-reload | High | Medium | Iteration speed |
| Material editor | Medium | Medium | Artist workflow |
| Animation preview | Medium | Medium | Content workflow |
| Performance profiler | Medium | Medium | Optimization |

---

## Part 5: Technical Recommendations

### 5.1 Vulkano vs WGPU Consideration

**Current Choice: Vulkano 0.35**
- Pros: Close to Vulkan, good control, established
- Cons: Vulkan-only (no Metal, DX12, WebGPU)

**Alternative: wgpu**
- Pros: Cross-platform (Vulkan, Metal, DX12, WebGPU), Bevy uses it
- Cons: Slightly higher abstraction, different API

**Recommendation:** Stay with Vulkano for now. Migration to wgpu would require significant refactoring and the educational value of staying close to Vulkan is high. Consider wgpu for a future major version (2.0) if web/Apple platform support becomes critical.

### 5.2 Dependency Updates

| Current | Latest | Action |
|---------|--------|--------|
| bevy_ecs 0.14 | 0.15+ | Monitor, update when stable |
| vulkano 0.35 | 0.35.1 | Current |
| rapier3d 0.22 | 0.22 | Current |
| egui 0.29 | 0.30+ | Update available |
| winit 0.30.11 | 0.30.11 | Current |

### 5.3 Performance Optimization Opportunities

1. **Frustum Culling:** Don't submit invisible objects to GPU
2. **Occlusion Culling:** Hide objects behind other objects
3. **Instanced Rendering:** Batch identical meshes
4. **Indirect Drawing:** GPU-driven command buffer generation
5. **LOD System:** Distance-based mesh simplification

---

## Part 6: Competitive Strategy

### 6.1 Differentiation Opportunities

1. **Educational Excellence:** Maintain exceptional documentation
2. **Modular Design:** Enable easy subsystem replacement
3. **Specialized Domains:** Focus on specific game types (voxel, simulation)
4. **Rust Ecosystem Integration:** First-class Cargo/Crates.io support

### 6.2 Target Use Cases

**Primary:**
- Learning Rust game development
- Small indie 3D games
- Game jam projects

**Secondary:**
- Procedural/simulation games
- Research/academic projects
- Embedded/specialized applications

---

## Part 7: Action Items Summary

### Immediate (This Week)
- [ ] Add depth buffer to render pass
- [ ] Create test scaffolding for all crates
- [ ] Implement shape_cast in PhysicsWorld

### Short-term (This Month)
- [ ] Achieve 50% test coverage on core systems
- [ ] Add cargo bench suite
- [ ] Begin shadow mapping implementation
- [ ] Start GLTF loader research

### Medium-term (This Quarter)
- [ ] Complete shadow mapping (directional)
- [ ] Add normal mapping support
- [ ] Implement post-processing framework
- [ ] Basic animation system prototype

### Long-term (This Year)
- [ ] Full animation system
- [ ] Audio integration
- [ ] Scene editor MVP
- [ ] Performance optimization pass

---

## Appendix: Crate-by-Crate Status

| Crate | Lines | Status | Next Action |
|-------|-------|--------|-------------|
| praxis_core | ~200 | Complete | Add integration tests |
| praxis_utils | ~300 | Complete | Add observability tests |
| praxis_math | ~100 | Complete | Stable |
| praxis_window | ~280 | Complete | Add resize tests |
| praxis_graphics | ~6500 | Mature | Add depth buffer, shadows |
| praxis_ecs | ~3800 | Mature | Add transform tests |
| praxis_physics | ~2400 | Complete | Implement shape_cast |
| praxis_scene | ~800 | Complete | Add serialization tests |
| praxis_input | ~600 | Complete | Add action mapping tests |
| praxis_gui | ~400 | Complete | Expand debug features |
| praxis_assets | ~200 | Basic | Add GLTF support |

---

*This analysis reflects the state of the codebase as of January 2026 and should be updated quarterly.*
