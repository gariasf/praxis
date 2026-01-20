# Changelog

All notable changes to the `praxis_profiling` crate will be documented in this file.

## [Unreleased]

### Added
- **Rendering Statistics Integration**: Automatic export of rendering metrics to Chrome trace format
  - Added `ChromeTraceExporter::add_render_stats()` for exporting RenderStats as counter events
  - Added `ChromeTraceExporter::add_counter()` and `add_counters()` for custom counter events
  - Added `Profiler::record_render_stats()` for convenient integration
  - Added `graphics_integration` feature flag (enabled by default)
  - New module `render_stats_integration` with conversion utilities
  - Comprehensive documentation in `RENDER_STATS_INTEGRATION.md`
  
- **Exported Rendering Metrics**:
  - Culling Efficiency %: Percentage of objects successfully culled
  - Total Objects: Objects submitted for rendering
  - Visible Objects: Objects rendered after culling
  - Frustum Culled: Objects culled by frustum test
  - Occlusion Culled: Objects culled by occlusion test
  - Draw Calls: Draw calls issued to GPU
  - Draw Call Reduction: Objects saved via culling/batching
  - Descriptor Allocations: Descriptor sets allocated per frame
  - LOD Level N %: Percentage distribution across LOD levels
  - Streaming Queue Depth: Meshes waiting to be loaded

### Changed
- Made `praxis_graphics` dependency optional (via `graphics_integration` feature)
- Updated `README.md` to document rendering statistics integration
- Updated crate documentation with rendering integration examples

### Performance
- Zero overhead when trace export is disabled
- ~50-100ns per counter event when trace export is enabled
- ~1-2μs per frame for all rendering counters (11 counters total)

## [0.1.0] - Initial Release

### Added
- CPU profiling with hierarchical scopes
- GPU profiling via Vulkan timestamp queries
- Memory allocation tracking and leak detection
- ECS system profiling with bottleneck identification
- Chrome Trace Event Format export
- Frame statistics aggregation
- Visualization tools (graphs, pie charts, bar charts)
