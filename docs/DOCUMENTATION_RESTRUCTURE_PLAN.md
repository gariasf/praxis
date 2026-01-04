# Documentation Restructure Plan

This document outlines the comprehensive restructuring of Praxis documentation.

## Status: FULLY COMPLETED (2026-01-04)

### All Tasks Completed
- [x] Created new directory structure (getting-started/, guides/, concepts/, reference/, editor/)
- [x] Created section README indexes for all sections
- [x] Consolidated rendering documentation (forward + deferred)
- [x] Consolidated HDR and tone mapping guide
- [x] Consolidated shadow mapping guide
- [x] Consolidated post-processing guide
- [x] Consolidated editor documentation (selection, undo-redo, gizmos, camera, panels)
- [x] Created main docs/README.md navigation index
- [x] Created getting-started guides (installation, project-structure)
- [x] Created dev-notes/ARCHIVE_NOTES.md documenting superseded files
- [x] Slimmed CLAUDE.md from 918 lines to 174 lines (81% reduction)
- [x] Created concept files (ecs-architecture, vulkan-rendering, transform-hierarchy, pbr-materials, spatial-audio)
- [x] Created reference files (crates, components, shaders, configuration)
- [x] Archived 17 implementation tracking files to dev-notes/archived/
- [x] Removed 6 superseded duplicate files
- [x] Updated examples/README.md with categorized table format

## Goals

1. **Eliminate duplication** - Consolidate scattered docs into single authoritative sources
2. **Clear hierarchy** - Organize by purpose (guides, concepts, reference, editor)
3. **Discoverable** - Easy navigation from entry points to detailed content
4. **Educational focus** - Maintain learning-oriented content prominently
5. **Maintainable** - Fewer files, clear ownership, consistent conventions

## New Structure

```
docs/
├── getting-started/
│   ├── README.md              # Section index
│   ├── installation.md        # Requirements, setup
│   └── project-structure.md   # Workspace layout
│
├── guides/                    # Task-oriented "how to" guides
│   ├── README.md              # Section index
│   ├── rendering.md           # Forward + deferred rendering
│   ├── hdr-and-tonemapping.md # HDR pipeline (consolidated)
│   ├── shadows.md             # Shadow mapping (consolidated)
│   ├── animation.md           # Skeletal + blending
│   ├── physics.md             # Physics integration
│   ├── audio.md               # Spatial audio
│   ├── input.md               # Input handling
│   ├── assets.md              # GLTF/OBJ loading
│   └── post-processing.md     # Bloom, effects
│
├── concepts/                  # Educational explanations
│   ├── README.md              # Section index
│   ├── ecs-architecture.md    # ECS patterns
│   ├── vulkan-rendering.md    # Graphics pipeline
│   ├── transform-hierarchy.md # Scene graph
│   ├── pbr-materials.md       # PBR theory
│   └── spatial-audio.md       # 3D audio concepts
│
├── reference/                 # API and configuration
│   ├── README.md              # Section index
│   ├── crates.md              # All crates overview
│   ├── components.md          # ECS components
│   ├── shaders.md             # Shader reference
│   └── configuration.md       # Constants and config
│
├── editor/                    # Editor documentation
│   ├── README.md              # Editor overview
│   ├── panels.md              # All panels
│   ├── selection.md           # Selection system
│   ├── undo-redo.md           # Undo/redo system
│   ├── gizmos.md              # Transform gizmos
│   └── commands.md            # Command system
│
├── architecture.md            # High-level design (existing, kept)
└── beginners-guide.md         # Comprehensive intro (renamed)
```

## Consolidation Map

| Original Files | Consolidated To |
|----------------|-----------------|
| HDR_IMPLEMENTATION_*.md, crates/.../HDR_*.md, docs/hdr_* | docs/guides/hdr-and-tonemapping.md |
| docs/shadow_mapping.md, docs/shadow_system.md | docs/guides/shadows.md |
| UNDO_REDO*.md (4 files) | docs/editor/undo-redo.md |
| SELECTION_SYSTEM*.md (2 files) | docs/editor/selection.md |
| docs/deferred_rendering.md, DEFERRED_RENDERING_IMPLEMENTATION.md | docs/guides/rendering.md |

## Files to Archive

Move to `dev-notes/archived/` (historical reference):
- IMPLEMENTATION_SUMMARY.md
- IMPLEMENTATION_COMPLETE.md
- All *_IMPLEMENTATION.md files
- CODEBASE_REVIEW_PLAN.md
- TEST_SUMMARY.md, TEST_IMPLEMENTATION_SUMMARY.md
- STRATEGIC_ANALYSIS_2026.md

## Naming Conventions

- Guides/concepts/reference: `lowercase-kebab.md`
- Top-level project files: `UPPERCASE.md` (README, CONTRIBUTING, CHANGELOG)
- Section indexes: `README.md`

## CLAUDE.md Approach

Reduce from 650+ lines to ~200 lines:
- Keep essential commands
- Link to docs/ for detailed information
- Remove duplicated architecture content
- Focus on AI-specific guidance

## Execution Order

1. Create new directory structure
2. Create section README indexes
3. Consolidate duplicate content into unified guides
4. Archive implementation tracking files
5. Update cross-references
6. Slim CLAUDE.md
7. Update examples/README.md
