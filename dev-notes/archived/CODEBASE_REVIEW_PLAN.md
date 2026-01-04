# Praxis Codebase Review and Improvement Plan

## Executive Summary

This document outlines a comprehensive plan for two major tasks:
1. **Documentation Reorganization** - Unifying and professionalizing the documentation structure
2. **Test Quality Review** - Ensuring all tests run, pass, and test valid/useful code paths

---

## Current State Analysis

### Code Quality Status

| Check | Status | Details |
|-------|--------|---------|
| `cargo fmt` | ❌ FAIL | Formatting issues in `loader.rs` and test files |
| `cargo clippy` | ✅ PASS | No warnings or errors |
| `cargo test` | ❌ FAIL | Compilation errors in examples and integration tests |

### Compilation Issues Identified

**Root Cause:** API mismatch between `praxis_ecs::World` wrapper and `bevy_ecs::World`

| File | Issue |
|------|-------|
| `examples/input_integration.rs` | Direct `bevy_ecs::world::World` import instead of using praxis_ecs |
| `examples/common.rs` | Direct `bevy_ecs::system::Resource` import |
| `examples/environment_probe_demo.rs` | Unresolved `praxis::prelude::*` import |
| `examples/skeletal_animation_demo.rs` | Type mismatch: `World` vs `BevyWorld`, `Query` vs `QueryState` |
| `examples/transform_propagation_demo.rs` | Missing methods on World wrapper |
| `examples/material_demo.rs` | `resource_mut` vs `get_resource_mut`, Query type issues |
| `tests/resource_cleanup_test.rs` | Unresolved `praxis_ecs::Resource` derive |
| `tests/asset_integration_test.rs` | Various World/Query API mismatches |
| `tests/integration_test.rs` | World wrapper API mismatches |

### Documentation Inventory

**Total: 56 markdown files across the project**

#### Root Level (11 files - needs consolidation)
- `CLAUDE.md` - Claude Code instructions (keep as-is)
- `README.md` - Main project readme
- `TEST_SUMMARY.md` - Test implementation summary
- `TEST_IMPLEMENTATION_SUMMARY.md` - Duplicate/overlapping with TEST_SUMMARY
- `IMPLEMENTATION_SUMMARY.md` - General implementation summary
- `SKELETAL_ANIMATION_IMPLEMENTATION.md` - Animation implementation details
- `ANIMATION_BLENDING_IMPLEMENTATION.md` - Blending implementation details
- `DEFERRED_RENDERING_IMPLEMENTATION.md` - Deferred rendering details
- `ENVIRONMENT_PROBE_IMPLEMENTATION.md` - Environment probes details
- `HDR_IMPLEMENTATION_SUMMARY.md` - HDR summary
- `HDR_IMPLEMENTATION_CHECKLIST.md` - HDR checklist

#### docs/ Directory (23 files)
Well-organized but some overlap with root-level files and crate READMEs.

#### Crate-Level Documentation (15 files)
Individual README.md and specialized docs in each crate.

---

## Part 1: Code Quality Fixes

### Phase 1.1: Fix Formatting Issues

**Priority: High | Estimated Effort: Low**

Run `cargo fmt --all` to automatically fix formatting issues in:
- `crates/praxis_assets/src/loader.rs`
- `tests/audio_comprehensive_test.rs`
- Various other test files

### Phase 1.2: Fix Example Compilation Errors

**Priority: Critical | Estimated Effort: Medium-High**

The examples have compilation errors due to inconsistent use of the `praxis_ecs::World` wrapper vs direct `bevy_ecs` types.

**Strategy Options:**

**Option A: Extend praxis_ecs::World wrapper** (Recommended)
- Add missing methods to the `World` wrapper: `resource()`, `resource_mut()`, `query()`, etc.
- This maintains the abstraction and keeps examples cleaner

**Option B: Update examples to use inner() access pattern**
- Modify examples to call `world.inner()` or `world.inner_mut()` to access bevy_ecs::World
- Faster fix but exposes internal implementation

**Files requiring fixes:**
1. `examples/input_integration.rs` - Change imports
2. `examples/common.rs` - Change imports
3. `examples/environment_probe_demo.rs` - Fix import path
4. `examples/skeletal_animation_demo.rs` - Fix Query/World usage
5. `examples/transform_propagation_demo.rs` - Fix World methods
6. `examples/material_demo.rs` - Fix resource access and Query types
7. `tests/resource_cleanup_test.rs` - Fix Resource derive
8. `tests/asset_integration_test.rs` - Fix World API usage
9. `tests/integration_test.rs` - Fix World API usage

### Phase 1.3: Test Quality Review

**Priority: High | Estimated Effort: Medium**

After compilation fixes, review tests for:
1. **Coherence** - Do tests test what they claim to test?
2. **Coverage** - Are critical paths covered?
3. **Value** - Do tests catch real bugs or just implementation details?

**Test Files to Review:**
| File | Test Count | Area |
|------|------------|------|
| `crates/praxis_scene/src/animation_tests.rs` | ~25 | Animation interpolation |
| `crates/praxis_physics/src/tests.rs` | ~30 | Physics components/queries |
| `crates/praxis_audio/src/audio_tests.rs` | ~20 | Audio components |
| `crates/praxis_graphics/src/post_process/tests.rs` | ~33 | Post-processing |
| `crates/praxis_graphics/src/shadow.rs` | ~16 | Shadow mapping |
| `crates/praxis_graphics/src/mesh.rs` | ~12 | Tangent calculation |
| `tests/integration_test.rs` | - | Cross-crate integration |
| `tests/animation_comprehensive_test.rs` | - | Animation integration |
| `tests/asset_integration_test.rs` | - | Asset loading |
| `tests/audio_comprehensive_test.rs` | - | Audio integration |
| `tests/resource_cleanup_test.rs` | - | Resource lifecycle |

---

## Part 2: Documentation Reorganization

### Current Issues

1. **Fragmentation**: Implementation details scattered across root, docs/, and crates
2. **Duplication**: Multiple files covering similar topics (HDR has 5+ related files)
3. **Inconsistent Naming**: Mix of `SCREAMING_SNAKE_CASE` and `lowercase` names
4. **Outdated Content**: Some implementation docs may not reflect current state
5. **No Clear Hierarchy**: Root-level files compete with docs/ directory

### Proposed Structure

```
praxis/
├── README.md                    # Main project readme (keep)
├── CLAUDE.md                    # Claude Code instructions (keep)
├── CONTRIBUTING.md              # Contributing guidelines (new)
├── CHANGELOG.md                 # Version changelog (new if needed)
│
├── docs/
│   ├── README.md                # Docs index/navigation
│   │
│   ├── getting-started/
│   │   ├── installation.md
│   │   ├── quick-start.md
│   │   └── first-project.md
│   │
│   ├── architecture/
│   │   ├── overview.md          # (consolidate ARCHITECTURE.md)
│   │   ├── crate-structure.md
│   │   └── design-principles.md
│   │
│   ├── systems/
│   │   ├── rendering/
│   │   │   ├── overview.md
│   │   │   ├── forward-rendering.md
│   │   │   ├── deferred-rendering.md
│   │   │   ├── hdr-pipeline.md
│   │   │   ├── shadows.md
│   │   │   ├── post-processing.md
│   │   │   └── environment-probes.md
│   │   ├── animation/
│   │   │   ├── skeletal-animation.md
│   │   │   └── blending.md
│   │   ├── physics/
│   │   │   └── overview.md
│   │   ├── audio/
│   │   │   └── overview.md
│   │   ├── ecs/
│   │   │   └── overview.md
│   │   └── assets/
│   │       ├── gltf-loading.md
│   │       └── obj-loading.md
│   │
│   ├── development/
│   │   ├── testing.md
│   │   ├── benchmarking.md
│   │   └── ci-cd.md
│   │
│   └── reference/
│       ├── shader-reference.md
│       └── api-patterns.md
│
├── crates/
│   └── <each crate>/
│       └── README.md            # Crate-specific docs (keep)
│
└── examples/
    └── README.md                # Examples guide (keep)
```

### Migration Plan

#### Phase 2.1: Audit and Categorize
- Map all existing docs to new structure
- Identify duplicates and conflicts
- Determine what should be consolidated vs. kept separate

#### Phase 2.2: Consolidate Root-Level Files
- Move implementation summaries into appropriate docs/ subdirectories
- Merge overlapping content (e.g., HDR_IMPLEMENTATION_* files)
- Delete redundant files after merging

#### Phase 2.3: Restructure docs/ Directory
- Create new subdirectory structure
- Move and rename files following naming conventions
- Update all internal links

#### Phase 2.4: Update Cross-References
- Update CLAUDE.md if docs paths change
- Update crate READMEs to point to central docs
- Update README.md navigation section

#### Phase 2.5: Validate and Clean
- Check all links work
- Remove any orphaned files
- Add docs/README.md as navigation hub

---

## Execution Plan

### Immediate Actions (Can parallelize)

1. **Agent: Format Fix**
   - Run `cargo fmt --all`
   - Commit formatting changes

2. **Agent: World API Analysis**
   - Analyze praxis_ecs::World wrapper
   - Determine best approach for API extension
   - Document required changes

3. **Agent: Documentation Audit**
   - List all .md files with summaries
   - Identify duplicates and overlaps
   - Propose merge/consolidation plan

### Sequential Actions

4. **Fix Example/Test Compilation**
   - Apply World API fixes
   - Fix import statements
   - Verify compilation succeeds

5. **Run and Review Tests**
   - Execute `cargo test --workspace`
   - Review test output for coherence
   - Document any test quality issues

6. **Execute Documentation Reorganization**
   - Create new directory structure
   - Migrate and merge content
   - Update cross-references

---

## Success Criteria

### Code Quality
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo clippy --all -- -D warnings` passes
- [ ] `cargo test --workspace` compiles and all tests pass
- [ ] All examples compile and run

### Documentation
- [ ] No duplicate documentation files
- [ ] Clear navigation from README.md
- [ ] Consistent naming conventions
- [ ] All cross-references work
- [ ] Each major system has one authoritative doc location

---

## Appendix: File-by-File Migration Map

### Root Files to Migrate

| Current Location | Action | New Location |
|-----------------|--------|--------------|
| `TEST_SUMMARY.md` | Merge | `docs/development/testing.md` |
| `TEST_IMPLEMENTATION_SUMMARY.md` | Delete (duplicate) | - |
| `IMPLEMENTATION_SUMMARY.md` | Archive/Delete | - |
| `SKELETAL_ANIMATION_IMPLEMENTATION.md` | Merge | `docs/systems/animation/skeletal-animation.md` |
| `ANIMATION_BLENDING_IMPLEMENTATION.md` | Merge | `docs/systems/animation/blending.md` |
| `DEFERRED_RENDERING_IMPLEMENTATION.md` | Merge | `docs/systems/rendering/deferred-rendering.md` |
| `ENVIRONMENT_PROBE_IMPLEMENTATION.md` | Merge | `docs/systems/rendering/environment-probes.md` |
| `HDR_IMPLEMENTATION_SUMMARY.md` | Merge | `docs/systems/rendering/hdr-pipeline.md` |
| `HDR_IMPLEMENTATION_CHECKLIST.md` | Delete (outdated) | - |

### docs/ Files to Reorganize

| Current | Action | New Location |
|---------|--------|--------------|
| `docs/ARCHITECTURE.md` | Merge | `docs/architecture/overview.md` |
| `docs/BEGINNERS_GUIDE.md` | Move | `docs/getting-started/quick-start.md` |
| `docs/TESTING.md` | Merge | `docs/development/testing.md` |
| `docs/RENDERING_EXPLAINED.md` | Move | `docs/systems/rendering/overview.md` |
| (remaining files follow similar pattern) |

---

*Document created: 2026-01-04*
*Status: Planning Phase*
