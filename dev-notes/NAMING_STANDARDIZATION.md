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

### Reviewed and Resolved

#### RenderContext (✅ Evaluated - Keep as-is)
- **Location**: `crates/praxis_graphics/src/lib.rs` (main rendering type)
- **Current**: `RenderContext`
- **Decision**: **KEEP** - Does not require renaming
- **Rationale**: 
  - **Hybrid Responsibility**: RenderContext is a unique type that combines multiple responsibilities:
    1. **Vulkan Context Management**: Manages instance, device, queues, swapchain lifecycle
    2. **Rendering Orchestration**: Issues GPU commands and manages render passes
    3. **Resource Management**: Provides access to mesh, texture, material managers
    4. **Frame Coordination**: Handles frame synchronization and presentation
  - **Established Pattern**: The "Context" suffix correctly reflects its role as a central coordination point that encapsulates the entire rendering subsystem, similar to established patterns in graphics APIs (OpenGL Context, Direct3D Device Context)
  - **Not Purely Manager**: While it contains managers (MeshAssetManager, TextureManager, MaterialManager), it does more than resource management
  - **Not Purely Renderer**: While it performs rendering via the `render()` method, it also manages the Vulkan lifecycle, swapchain recreation, and synchronization
  - **API Stability**: Core type used extensively across all examples (50+ files), renaming would be extremely disruptive
  - **Semantic Clarity**: "Context" accurately describes a centralized state container that provides access to all graphics subsystem functionality
- **Conclusion**: RenderContext represents a legitimate architectural pattern for a top-level graphics context that doesn't fit neatly into Manager/Renderer/System categories. No action required.

#### praxis_editor Crate (✅ Audited - All Compliant)
- **Location**: `crates/praxis_editor/` (entire crate)
- **Audit Date**: December 2024
- **Decision**: **ALL TYPES COMPLIANT** - No changes required
- **Details**: See `PRAXIS_EDITOR_AUDIT.md` for comprehensive analysis
- **Summary**:
  - **5 System types**: SelectionSystem, UndoRedoSystem, GizmoSystem, PlayModeSystem, DragDropSystem
  - **All correctly named**: Each System processes ECS components or manages editor behavior
  - **Non-system types correct**: EntityOperations, EditorState, etc. appropriately avoid "System" suffix
  - **No duplicate functionality**: Clear separation of concerns across all subsystems
  - **Exemplary design**: Demonstrates proper use of naming conventions and design patterns
- **Conclusion**: praxis_editor serves as a model for correct naming convention usage. No refactoring needed.

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
- If it's a top-level API coordinator → `Context` (rare exception)

When in doubt, ask: "What is this type's primary responsibility?"

### Context Types (Rare Exception)

The `Context` suffix is reserved for top-level types that:
1. Manage entire API lifecycle (not just resources)
2. Coordinate multiple subsystems (managers, renderers, etc.)
3. Serve as the primary entry point to a subsystem
4. Handle synchronization and state management across components

**Example**: `RenderContext` manages Vulkan lifecycle, coordinates multiple managers/renderers, handles swapchain recreation, and provides unified rendering API.

**Note**: This is a rare pattern. Most new types should use Manager/Renderer/System. Only create Context types when building top-level API abstractions.

## Decision Log

### Initial Conventions Established
- Defined Manager/Renderer/System suffixes
- Created tracking document

### ParticleRenderer Standardization Complete
- Renamed to `ParticleRenderer` to follow rendering type conventions
- Updated all documentation references
- Type correctly reflects its GPU rendering responsibility

### RenderContext Evaluated (December 2024)
- **Decision**: Keep as-is - no renaming required
- **Analysis**: Evaluated whether `RenderContext` should be renamed to `RenderManager` per naming conventions
- **Result**: `RenderContext` is a legitimate exception to the Manager/Renderer/System pattern
- **Key Insight**: Top-level context types that encapsulate entire subsystems and provide unified API surface don't fit neatly into the three categories. The "Context" suffix is appropriate for:
  - Types that manage API lifecycle (Vulkan instance, device, queues)
  - Types that coordinate multiple subsystems (managers, renderers, synchronization)
  - Types that serve as the primary entry point to a graphics API
- **Precedent**: Established pattern in graphics programming (GLContext, VkContext, D3DContext)
- **Impact**: This decision clarifies that "Context" is an acceptable suffix for top-level API coordinators

### praxis_editor Crate Audit (December 2024)
- **Decision**: All types compliant - no changes required
- **Scope**: Complete audit of all exported types in praxis_editor crate
- **Result**: 100% compliance with naming conventions
- **Key Findings**:
  - 5 System types all correctly named (SelectionSystem, UndoRedoSystem, GizmoSystem, PlayModeSystem, DragDropSystem)
  - Non-system types appropriately avoid "System" suffix (EntityOperations, EditorState, etc.)
  - No duplicate functionality found across editor subsystems
  - Clear separation of concerns with minimal, well-defined integration points
  - Exemplary use of design patterns (Command, Facade, Observer, Composite, State Machine)
- **Impact**: Establishes praxis_editor as a reference implementation for proper naming convention usage
- **Documentation**: Created comprehensive audit document at `PRAXIS_EDITOR_AUDIT.md`

### Future Decisions
Document any decisions to keep existing names or approve exceptions here.

## References

- **CLAUDE.md**: Main conventions documentation
- **Rust API Guidelines**: [Naming conventions](https://rust-lang.github.io/api-guidelines/naming.html)
- **Migration Examples**: See archived implementation tracking docs for patterns
