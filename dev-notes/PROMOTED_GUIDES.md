# Promoted Implementation Guides

This document tracks implementation documents that have been promoted from `dev-notes/archived/` to polished technical guides in the main documentation.

## Promotion Summary (2024)

The following 8 high-value archived implementation documents have been promoted to polished technical guides with design rationale, usage examples, and comprehensive documentation.

### Guides (`docs/guides/`)

1. **Deferred Rendering** 
   - Source: `dev-notes/archived/DEFERRED_RENDERING_IMPLEMENTATION.md`
   - Destination: `docs/guides/deferred-rendering.md`
   - Topics: G-buffer architecture, two-pass rendering, lighting calculations, performance characteristics
   - Key additions: Design rationale, usage examples, hybrid rendering strategies, optimization tips

2. **Environment Probes**
   - Source: `dev-notes/archived/ENVIRONMENT_PROBE_IMPLEMENTATION.md`
   - Destination: `docs/guides/environment-probes.md`
   - Topics: Image-based lighting, cubemap capture, IBL precomputation, update modes
   - Key additions: Probe placement guidelines, shader integration, performance analysis, troubleshooting

3. **Skeletal Animation**
   - Source: `dev-notes/archived/SKELETAL_ANIMATION_IMPLEMENTATION.md`
   - Destination: `docs/guides/animation/skeletal-animation.md`
   - Topics: Bone hierarchy, keyframe interpolation, animation blending, GPU skinning
   - Key additions: Complete API reference, advanced techniques, performance optimization, practical examples

### Editor Guides (`docs/editor/`)

4. **Selection System**
   - Source: `dev-notes/archived/SELECTION_SYSTEM_IMPLEMENTATION.md`
   - Destination: `docs/editor/selection-system.md`
   - Topics: Multi-entity selection, raycast picking, marquee selection, selection events
   - Key additions: Design rationale for dual-component system, advanced usage patterns, performance considerations

5. **Asset Browser**
   - Source: `dev-notes/archived/ASSET_BROWSER_IMPLEMENTATION.md`
   - Destination: `docs/editor/asset-browser.md`
   - Topics: Filesystem navigation, thumbnail generation, drag-and-drop, hot-reload
   - Key additions: Architecture explanation, custom asset types, batch operations, troubleshooting guide

6. **Editor Camera**
   - Source: `dev-notes/archived/EDITOR_CAMERA_IMPLEMENTATION.md`
   - Destination: `docs/editor/editor-camera.md`
   - Topics: Orbit camera controls, smooth interpolation, focus-on-selection
   - Key additions: Design philosophy, camera presets, state serialization, comparison to game cameras

7. **Menu Bar**
   - Source: `dev-notes/archived/MENU_BAR_IMPLEMENTATION.md`
   - Destination: `docs/editor/menu-bar.md`
   - Topics: Menu structure, keyboard shortcuts, undo/redo integration, dirty state tracking
   - Key additions: Action-based architecture explanation, context-aware shortcuts, custom action handling

8. **Hierarchy Panel**
   - Source: `dev-notes/archived/HIERARCHY_PANEL_IMPLEMENTATION.md`
   - Destination: `docs/editor/hierarchy-panel.md`
   - Topics: Entity tree visualization, drag-and-drop reparenting, live updates
   - Key additions: Circular hierarchy prevention, expansion state management, virtual scrolling, filtering

## Documentation Updates

The following index files have been updated to reference the new guides:

- `docs/README.md` - Added all 8 new guides to appropriate sections
- `docs/guides/README.md` - Added rendering and animation guides
- `docs/editor/README.md` - Reorganized with new editor guides

## Key Improvements

Each promoted guide includes:

1. **Design Rationale**: Explains architectural decisions and trade-offs
2. **Complete API Reference**: Comprehensive coverage of all public APIs
3. **Usage Examples**: Practical, runnable code examples
4. **Advanced Techniques**: Beyond basic usage patterns
5. **Performance Considerations**: Memory usage, optimization tips, profiling
6. **Troubleshooting**: Common problems and solutions
7. **Integration Examples**: How to use with other systems
8. **See Also**: Cross-references to related documentation

## Style Improvements

- Converted implementation notes to user-facing documentation
- Added clear section hierarchies with descriptive headings
- Included tables for quick reference
- Provided code examples with syntax highlighting
- Added design philosophy and rationale sections
- Included troubleshooting sections for common issues
- Cross-linked related documentation

## Original Implementation Documents

The original implementation documents remain in `dev-notes/archived/` for historical reference and provide additional low-level implementation details that may be useful for engine developers.

## Future Promotion Candidates

Additional archived implementation documents that could be promoted in the future:

- `dev-notes/archived/ANIMATION_BLENDING_IMPLEMENTATION.md` (already have blending.md, may merge details)
- `dev-notes/archived/CONSOLE_PANEL_IMPLEMENTATION.md` (could expand editor/panels.md)
- `dev-notes/archived/PROJECT_SETTINGS_IMPLEMENTATION.md` (new guide needed)
- `docs/internals/*_IMPLEMENTATION.md` (internal implementation details, may not need promotion)

---

**Note**: This promotion exercise demonstrates best practices for documentation:
- Implementation notes → User-facing guides
- Technical details → Practical examples
- Code-centric → User-centric
- Isolated → Cross-referenced
