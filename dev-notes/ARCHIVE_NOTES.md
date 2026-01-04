# Archive Notes

Documentation for the 2026-01-04 documentation restructure.

## Actions Taken

### Files Archived (moved to dev-notes/archived/)

17 implementation tracking files were archived:

1. `TEST_IMPLEMENTATION_SUMMARY.md`
2. `TEST_SUMMARY.md`
3. `SKELETAL_ANIMATION_IMPLEMENTATION.md`
4. `ANIMATION_BLENDING_IMPLEMENTATION.md`
5. `DEFERRED_RENDERING_IMPLEMENTATION.md`
6. `HDR_IMPLEMENTATION_CHECKLIST.md`
7. `HDR_IMPLEMENTATION_SUMMARY.md`
8. `ENVIRONMENT_PROBE_IMPLEMENTATION.md`
9. `IMPLEMENTATION_SUMMARY.md`
10. `IMPLEMENTATION_COMPLETE.md`
11. `CODEBASE_REVIEW_PLAN.md`
12. `ASSET_BROWSER_IMPLEMENTATION.md`
13. `CONSOLE_PANEL_IMPLEMENTATION.md`
14. `CONSOLE_PANEL_SUMMARY.md`
15. `EDITOR_CAMERA_IMPLEMENTATION.md`
16. `MENU_BAR_IMPLEMENTATION.md`
17. `PROJECT_SETTINGS_IMPLEMENTATION.md`
18. `SELECTION_SYSTEM_IMPLEMENTATION.md`
19. `HIERARCHY_PANEL_IMPLEMENTATION.md`

### Files Removed (superseded duplicates)

6 files were deleted:

1. `docs/shadow_system.md` - Duplicated shadow_mapping.md content
2. `crates/praxis_editor/UNDO_REDO.md` - Superseded by UNDO_REDO_SYSTEM.md
3. `crates/praxis_editor/QUICK_START_UNDO_REDO.md` - Content in UNDO_REDO_SYSTEM.md
4. `crates/praxis_editor/UNDO_REDO_FEATURE_SUMMARY.md` - Superseded
5. `crates/praxis_graphics/HDR_QUICK_START.md` - Consolidated into guide
6. `crates/praxis_graphics/POST_PROCESSING_QUICK_START.md` - Consolidated into guide

## Files Kept as References

Crate-level detailed docs remain as API references:

- `crates/praxis_graphics/HDR_RENDERING.md` - Full HDR API
- `crates/praxis_graphics/POST_PROCESSING.md` - Full post-processing API
- `crates/praxis_editor/SELECTION_SYSTEM.md` - Full selection API
- `crates/praxis_editor/GIZMOS.md` - Full gizmo API
- `crates/praxis_editor/UNDO_REDO_SYSTEM.md` - Full undo/redo API
- `crates/praxis_editor/EDITOR_CAMERA.md` - Full camera API
- `docs/deferred_rendering.md` - Detailed G-buffer documentation
- `docs/RENDERING_EXPLAINED.md` - Deep-dive rendering pipeline
- `docs/shadow_mapping.md` - Shadow mapping details

## New Documentation Structure

```
docs/
├── README.md                  # Main navigation index
├── BEGINNERS_GUIDE.md         # Core learning document
├── ARCHITECTURE.md            # Design overview
├── getting-started/
│   ├── README.md
│   ├── installation.md
│   └── project-structure.md
├── guides/
│   ├── README.md
│   ├── rendering.md           # Forward + deferred
│   ├── hdr-and-tonemapping.md
│   ├── shadows.md
│   └── post-processing.md
├── concepts/
│   ├── README.md
│   ├── ecs-architecture.md
│   ├── vulkan-rendering.md
│   ├── transform-hierarchy.md
│   ├── pbr-materials.md
│   └── spatial-audio.md
├── reference/
│   ├── README.md
│   ├── crates.md
│   ├── components.md
│   ├── shaders.md
│   └── configuration.md
├── editor/
│   ├── README.md
│   ├── selection.md
│   ├── undo-redo.md
│   ├── gizmos.md
│   ├── camera.md
│   └── panels.md
└── (original detailed docs remain)

dev-notes/
├── ARCHIVE_NOTES.md           # This file
└── archived/                  # Historical implementation files
```

## CLAUDE.md Changes

Reduced from 918 lines to 174 lines (81% reduction):
- Kept essential commands
- Added links to docs/ for detailed information
- Removed duplicated architecture content
- Focused on AI-specific guidance

## Rationale

1. **Summaries in docs/**: Quick reference, link to details
2. **Detailed docs in crates/**: API reference, implementation details
3. **Archived files**: Historical reference, not for daily use
4. **Slim CLAUDE.md**: AI guidance, not documentation repository
