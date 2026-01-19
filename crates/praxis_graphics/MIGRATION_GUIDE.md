# Migration Guide - Graphics Module Reorganization

## TL;DR - No Action Required! 🎉

**All existing code continues to work without any changes.** The reorganization maintains full backwards compatibility through re-exports.

## What Changed?

Five utility modules have been consolidated into a new `utilities` module:
- `optimization_config` → `utilities::optimization_config`
- `render_stats` → `utilities::render_stats`
- `velocity_buffer` → `utilities::velocity_buffer`
- `light_linking` → `utilities::light_linking`
- `light_probe` → `utilities::light_probe`

## Import Paths

### Current Code (Still Works) ✅

```rust
// These imports continue to work unchanged
use praxis_graphics::RenderStats;
use praxis_graphics::RenderStatsHistory;
use praxis_graphics::RenderingOptimizationConfig;
use praxis_graphics::VelocityBuffer;
use praxis_graphics::VelocityBufferRenderer;
use praxis_graphics::LightLinkingManager;
use praxis_graphics::LightLinkingMask;
use praxis_graphics::LightChannel;
use praxis_graphics::LightProbe;
use praxis_graphics::LightProbeManager;
use praxis_graphics::LightProbeGrid;
```

### New Recommended Paths (Also Works) ✅

```rust
// New paths provide clearer organization
use praxis_graphics::utilities::{
    RenderStats,
    RenderStatsHistory,
    RenderingOptimizationConfig,
    VelocityBuffer,
    VelocityBufferRenderer,
    LightLinkingManager,
    LightProbeManager,
};

// Or import the submodules
use praxis_graphics::utilities::render_stats;
use praxis_graphics::utilities::optimization_config;
use praxis_graphics::utilities::velocity_buffer;
use praxis_graphics::utilities::light_linking;
use praxis_graphics::utilities::light_probe;
```

## Module Paths

### Old Module Paths (Deprecated but Still Work)

```rust
use praxis_graphics::optimization_config::RenderingOptimizationConfig;
use praxis_graphics::render_stats::{RenderStats, RenderStatsHistory};
use praxis_graphics::velocity_buffer::VelocityBuffer;
use praxis_graphics::light_linking::LightLinkingManager;
use praxis_graphics::light_probe::LightProbeManager;
```

### New Module Paths (Recommended)

```rust
use praxis_graphics::utilities::optimization_config::RenderingOptimizationConfig;
use praxis_graphics::utilities::render_stats::{RenderStats, RenderStatsHistory};
use praxis_graphics::utilities::velocity_buffer::VelocityBuffer;
use praxis_graphics::utilities::light_linking::LightLinkingManager;
use praxis_graphics::utilities::light_probe::LightProbeManager;
```

## Usage Examples

### Render Statistics

```rust
use praxis_graphics::{RenderContext, RenderStats};

fn check_stats(ctx: &RenderContext) {
    let stats: &RenderStats = ctx.render_stats();
    println!("Visible objects: {}", stats.visible_objects);
}
```

### Optimization Configuration

```rust
use praxis_graphics::RenderingOptimizationConfig;

fn configure_optimizations() {
    let mut config = RenderingOptimizationConfig::default();
    config.set_gpu_culling(true);
    config.set_multi_draw_indirect(true);
}
```

### Velocity Buffers

```rust
use praxis_graphics::{VelocityBuffer, VelocityBufferRenderer};

fn setup_motion_blur(device: Arc<Device>, allocator: Arc<StandardMemoryAllocator>) 
    -> Result<VelocityBufferRenderer> 
{
    VelocityBufferRenderer::new(device, allocator)
}
```

### Light Linking

```rust
use praxis_graphics::{LightLinkingManager, LightChannel};

fn setup_light_channels() {
    let mut manager = LightLinkingManager::new();
    manager.set_object_mask("hero", 0b0001).unwrap();
    manager.set_light_channel("key_light", 0).unwrap();
}
```

### Light Probes

```rust
use praxis_graphics::{LightProbeManager, LightProbeGrid};
use praxis_math::Vec3;

fn setup_probes(device: Arc<Device>, allocator: Arc<StandardMemoryAllocator>) 
    -> Result<LightProbeManager> 
{
    let mut manager = LightProbeManager::new(device, allocator)?;
    let grid = LightProbeGrid::new(
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(10.0, 5.0, 10.0),
        [5, 3, 5],
    );
    manager.add_grid(grid);
    Ok(manager)
}
```

## When to Update Your Code

### Immediate Action: **None Required**
Your existing code will continue to work without any changes.

### Optional Update: **At Your Convenience**
Consider updating to new paths when:
- Refactoring existing code
- Writing new features
- Improving code clarity
- Following latest best practices

### Recommended Update: **Eventually**
Update imports to use `utilities` module for:
- Better code organization
- Clearer intent (utility vs core rendering)
- Future-proofing your codebase

## Search and Replace

If you want to update all imports at once, use these patterns:

### VS Code / Regex
```regex
Find:    use praxis_graphics::(optimization_config|render_stats|velocity_buffer|light_linking|light_probe)
Replace: use praxis_graphics::utilities::$1
```

### sed (Unix/Linux/Mac)
```bash
sed -i 's/use praxis_graphics::\(optimization_config\|render_stats\|velocity_buffer\|light_linking\|light_probe\)/use praxis_graphics::utilities::\1/g' **/*.rs
```

### PowerShell (Windows)
```powershell
Get-ChildItem -Recurse -Filter *.rs | ForEach-Object {
    (Get-Content $_.FullName) -replace 
        'use praxis_graphics::(optimization_config|render_stats|velocity_buffer|light_linking|light_probe)', 
        'use praxis_graphics::utilities::$1' |
    Set-Content $_.FullName
}
```

## Frequently Asked Questions

### Q: Do I need to update my code immediately?
**A:** No! All existing code continues to work without changes.

### Q: Will the old import paths be removed?
**A:** Not in the foreseeable future. The re-export stubs provide long-term backwards compatibility.

### Q: What if I find a broken import?
**A:** Report it as a bug - all imports should continue working. The reorganization was designed to be 100% backwards compatible.

### Q: Should I use the old or new paths?
**A:** Either works, but new paths (`utilities::*`) are recommended for clarity and future-proofing.

### Q: How do I know which path to use for new code?
**A:** Use `use praxis_graphics::utilities::*` for utility functions. The module re-exports all types for convenience.

### Q: Can I mix old and new import styles?
**A:** Yes, but consistency is recommended for code clarity.

## Summary

✅ **No breaking changes**
✅ **All existing code works**  
✅ **Both old and new paths supported**
✅ **Update at your convenience**
✅ **Improved organization**

The reorganization was designed with backwards compatibility as the #1 priority. Take your time updating to new paths - there's no rush!
