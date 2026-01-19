# Naming Standardization Tracking

This document tracks types that need renaming to follow the established conventions documented in `CLAUDE.md`.

## Naming Convention Summary

- **Manager**: Resource caching, asset loading, lifetime management
- **Renderer**: GPU rendering, draw calls, pipeline management  
- **System**: ECS behavior, component processing

## Types Requiring Standardization

### Completed Standardizations

#### ParticleRenderer (✅ Complete)
- **Location**: `crates/praxis_graphics/src/particles.rs`
- **Status**: Renamed from previous inconsistent naming
- **Current**: `pub struct ParticleRenderer`
- **Result**: Correctly follows conventions for GPU rendering types

### Medium Priority (Internal Inconsistencies)

#### RenderContext
- **Location**: `crates/praxis_graphics/src/lib.rs` (main rendering type)
- **Current**: `RenderContext`
- **Issue**: Manages Vulkan resources but uses generic `Context` suffix
- **Consider**: `RenderManager` or keep as-is (established convention)
- **Breaking**: Yes (core API)
- **Note**: May be too widely used to justify change; evaluate cost/benefit

### Lower Priority (Consider Future Refactoring)

#### Types to Review
These types should be audited to confirm they follow conventions:

1. **TextureManager** (assumed exists) - Verify it manages caching/loading
2. **SpatialManager** (`crates/praxis_spatial/src/spatial_manager.rs:50`) - Verify it's not doing rendering
3. **SceneManager** (`crates/praxis_scene/src/manager.rs:36`) - Confirm resource management role
4. **AudioManager** (`crates/praxis_audio/src/manager.rs:34`) - Confirm resource management role

## Migration Strategy

### Phase 1: Documentation (✅ Complete)
- [x] Document conventions in `CLAUDE.md`
- [x] Create this tracking issue

### Phase 2: Type Aliases (Recommended First Step)
For breaking changes, introduce type aliases to maintain compatibility:

```rust
// Example pattern for future renames:
// Old name kept as deprecated alias
#[deprecated(since = "0.x.0", note = "Use `NewName` instead")]
pub type OldName = NewName;

pub struct NewName {
    // ...
}
```

### Phase 3: Update Internal References
- Update all internal crate references to new names
- Keep aliases for public API compatibility
- Update examples gradually

### Phase 4: Deprecation Period
- Mark old names with deprecation warnings
- Document migration in changelog
- Provide automated migration guide (search/replace patterns)

### Phase 5: Remove Aliases
- After 2-3 releases, remove type aliases
- Update all examples and tests

## Guidelines for New Code

All new types **must** follow the naming conventions:
- If it manages resources/caching → `Manager`
- If it issues GPU commands/rendering → `Renderer`  
- If it processes ECS components → `System`

When in doubt, ask: "What is this type's primary responsibility?"

## Decision Log

### Initial Conventions Established
- Defined Manager/Renderer/System suffixes
- Created tracking document

### ParticleRenderer Standardization Complete
- Renamed to `ParticleRenderer` to follow rendering type conventions
- Updated all documentation references
- Type correctly reflects its GPU rendering responsibility

### Future Decisions
Document any decisions to keep existing names or approve exceptions here.

## References

- **CLAUDE.md**: Main conventions documentation
- **Rust API Guidelines**: [Naming conventions](https://rust-lang.github.io/api-guidelines/naming.html)
- **Migration Examples**: See archived implementation tracking docs for patterns
