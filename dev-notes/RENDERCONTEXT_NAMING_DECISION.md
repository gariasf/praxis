# RenderContext Naming Decision

**Date**: December 2024  
**Status**: RESOLVED - No renaming required  
**Type**: `RenderContext` (keep as-is)

## Executive Summary

After thorough analysis, **RenderContext should NOT be renamed to RenderManager**. The "Context" suffix accurately reflects its role as a top-level API coordinator that manages Vulkan lifecycle, coordinates multiple subsystems, and serves as the primary entry point to the graphics system.

## Background

The naming standardization effort established three primary type suffix conventions:
- **Manager**: Resource caching, asset loading, lifetime management
- **Renderer**: GPU rendering, draw calls, pipeline management
- **System**: ECS behavior, component processing

`RenderContext` was flagged for evaluation because it doesn't fit neatly into these categories, leading to the question: should it be renamed to `RenderManager`?

## Analysis

### What RenderContext Actually Does

Analysis of `crates/praxis_graphics/src/lib.rs` reveals that `RenderContext` has four distinct responsibilities:

1. **Vulkan Lifecycle Management**
   - Creates and manages Vulkan instance, device, and queues
   - Manages swapchain creation, recreation, and presentation
   - Handles frame synchronization and GPU resource lifetimes

2. **Rendering Orchestration**
   - Issues GPU commands via `render()` method
   - Records command buffers and submits to GPU
   - Manages render passes and framebuffers

3. **Resource Management Coordination**
   - Provides access to `MeshAssetManager`
   - Provides access to `TextureManager`
   - Provides access to `MaterialManager`
   - Provides access to `MaterialInstanceManager`
   - Contains `DescriptorSetPool` for descriptor set caching

4. **Subsystem Integration**
   - Integrates optional line renderer
   - Integrates optional GPU culling
   - Integrates optional bindless rendering
   - Manages render statistics collection

### Why "Context" is Correct

The "Context" suffix is appropriate because `RenderContext`:

1. **Is More Than a Manager**: While it contains managers, it actively performs rendering, handles synchronization, and manages API lifecycle—responsibilities beyond resource management.

2. **Is More Than a Renderer**: While it performs rendering, it also manages the entire Vulkan context, recreates swapchains, handles window minimization, and provides access to subsystems—responsibilities beyond just drawing.

3. **Follows Established Patterns**: Graphics APIs consistently use "Context" for types that encapsulate API state:
   - OpenGL: `GLContext`
   - Vulkan: `VkContext` (common pattern)
   - Direct3D: `D3DContext` / `DeviceContext`

4. **Provides Unified API Surface**: Acts as the single entry point to the entire graphics subsystem, delegating to specialized managers and renderers internally.

5. **Manages API Lifecycle**: Unlike managers which manage resources, or renderers which issue draw calls, `RenderContext` manages the lifecycle of the Vulkan API itself.

### Why "RenderManager" Would Be Incorrect

Renaming to `RenderManager` would:

1. **Misrepresent Functionality**: Implies pure resource management, ignoring rendering and synchronization responsibilities
2. **Break Established Patterns**: Deviates from industry-standard naming in graphics programming
3. **Reduce Semantic Clarity**: "Manager" is less descriptive than "Context" for a type that manages API lifecycle
4. **Create Massive Disruption**: Used in 50+ example files across the codebase

## Impact Analysis

### API Usage

`RenderContext` is used extensively:
- 51 files contain references
- All examples depend on it
- Core public API of praxis_graphics crate
- Documented extensively in CLAUDE.md

### Breaking Change Assessment

Renaming would require:
- Updating 50+ example files
- Updating all documentation
- Creating deprecated type alias
- Migration guide for users
- High risk of confusion during transition

### Benefits vs. Costs

**Benefits of Renaming**: None—"Context" is more accurate than "Manager"  
**Costs of Renaming**: Extremely high disruption with no semantic improvement

## Decision

**KEEP `RenderContext` as-is.**

### Rationale

1. **Semantic Accuracy**: "Context" correctly describes a top-level API coordinator
2. **Industry Convention**: Aligns with established graphics programming patterns
3. **Architectural Role**: Reflects hybrid responsibilities that don't fit Manager/Renderer/System
4. **API Stability**: Avoids unnecessary breaking changes to core API
5. **Legitimate Exception**: Represents a valid architectural pattern for API abstraction layers

## Documentation Updates

The following files have been updated to reflect this decision:

1. **dev-notes/NAMING_STANDARDIZATION.md**
   - Moved RenderContext from "Medium Priority" to "Reviewed and Resolved"
   - Added detailed rationale for keeping the name
   - Updated Decision Log

2. **CLAUDE.md**
   - Added "Context" to the type suffix table
   - Documented when Context suffix is appropriate
   - Added anti-pattern guidance against overusing Context
   - Updated note to reference RenderContext evaluation

3. **dev-notes/RENDERCONTEXT_NAMING_DECISION.md** (this file)
   - Created comprehensive decision record

## Guidelines for Future Context Types

The "Context" suffix should be reserved for types that:

1. Manage entire API lifecycle (not just resources)
2. Coordinate multiple subsystems (managers, renderers, etc.)
3. Serve as the primary entry point to a subsystem
4. Handle synchronization and state management across components

**This is a rare pattern.** Most new types should use Manager/Renderer/System. Only create Context types when building top-level API abstractions.

## Conclusion

`RenderContext` is correctly named and represents a legitimate exception to the Manager/Renderer/System naming conventions. The "Context" suffix accurately reflects its role as a top-level graphics API coordinator that manages Vulkan lifecycle, coordinates subsystems, and provides a unified rendering interface.

No action is required. This decision establishes "Context" as an acceptable fourth category for top-level API coordinators, while maintaining Manager/Renderer/System as the primary conventions for most types.
