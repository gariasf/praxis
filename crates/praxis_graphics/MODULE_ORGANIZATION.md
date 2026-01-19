# Graphics Module Organization

This document describes the module structure of the `praxis_graphics` crate after the consolidation effort.

## Module Count Reduction

**Before**: 35+ modules at the root level
**After**: 30 modules at the root level (5 modules consolidated)

## Consolidated Modules

The following smaller, single-purpose modules have been merged into the `utilities` module:

### utilities Module

The `utilities` module consolidates supporting systems that provide functionality for the main rendering pipeline but are not core rendering features themselves. This includes:

1. **optimization_config** (~29KB)
   - Runtime toggles for rendering optimizations
   - GUI integration for A/B performance comparison
   - Keyboard shortcuts for quick toggling

2. **render_stats** (~27KB)
   - Per-frame statistics collection
   - Rolling history with statistical aggregation
   - CSV export for analysis

3. **velocity_buffer** (~12KB)
   - Motion vector generation for motion blur
   - Specialized render pass for velocity data

4. **light_linking** (~11KB)
   - Channel-based light-object interaction control
   - Bit masks for selective lighting
   - Dynamic light grouping

5. **light_probe** (~13KB)
   - Dynamic global illumination using spherical harmonics
   - Probe grids for spatial interpolation
   - Trilinear and tetrahedral blending

## Module Structure

```
src/
├── utilities/
│   ├── mod.rs (re-exports)
│   ├── optimization_config.rs
│   ├── render_stats.rs
│   ├── velocity_buffer.rs
│   ├── light_linking.rs
│   └── light_probe.rs
├── hdr/
│   ├── mod.rs
│   ├── exposure.rs
│   ├── render_target.rs
│   └── tone_mapper.rs
├── post_process/
│   ├── mod.rs
│   ├── bloom.rs
│   ├── chain.rs
│   ├── cinematic.rs
│   ├── full_screen_quad.rs
│   ├── pass.rs
│   ├── passes.rs
│   ├── render_target.rs
│   └── tests.rs
└── [other modules...]
```

## Benefits

1. **Clearer Organization**: Related utility functions are now grouped together
2. **Reduced Cognitive Load**: Fewer top-level modules to navigate
3. **Better Discoverability**: Utility functions have a clear home
4. **Maintained API Compatibility**: All types are re-exported from `utilities` module
5. **Logical Grouping**: Supporting systems separated from core rendering features

## API Compatibility

All previously public types remain accessible through re-exports:

```rust
// Old path (still works via re-export):
use praxis_graphics::RenderStats;
use praxis_graphics::RenderingOptimizationConfig;
use praxis_graphics::VelocityBuffer;
use praxis_graphics::LightLinkingManager;
use praxis_graphics::LightProbeManager;

// New path (also works):
use praxis_graphics::utilities::RenderStats;
use praxis_graphics::utilities::RenderingOptimizationConfig;
use praxis_graphics::utilities::VelocityBuffer;
use praxis_graphics::utilities::LightLinkingManager;
use praxis_graphics::utilities::LightProbeManager;
```

## Rationale

The consolidation was guided by these principles:

1. **Size**: Modules under ~30KB are candidates for merging
2. **Cohesion**: Merged modules share a common purpose (utilities/support)
3. **Independence**: Utilities don't form core rendering features
4. **Discoverability**: Grouping improves organization without hiding functionality

## Future Considerations

The `utilities` module may be further subdivided if it grows significantly. Potential future subdivisions could be:

- `utilities::profiling` - Performance monitoring and statistics
- `utilities::optimization` - Optimization configuration and toggles
- `utilities::lighting` - Advanced lighting utilities (linking, probes)

However, the current size (~92KB total) doesn't warrant further subdivision.
