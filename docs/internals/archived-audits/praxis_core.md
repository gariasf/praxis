# praxis_core Audit Report

**Audit Date:** January 2026
**Lines of Code:** ~13
**Test Coverage:** 0 tests (coordinator crate)

## Executive Summary

`praxis_core` is a minimal facade crate that orchestrates subsystem initialization. It follows a clean coordinator pattern, calling `init()` on subsystems in dependency order and delegating to `praxis_window::run()` for the event loop. The simplicity is appropriate for its role.

**Overall Assessment: GOOD (8/10)**

---

## Features Inventory

### Feature 1: Engine Lifecycle Orchestration (`run()`)

**Location:** `src/lib.rs:4-12`
**Purpose:** Initialize subsystems and start the engine

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [ ] Test coverage (no tests, acceptable for coordinator)

#### Code Analysis

```rust
pub fn run() -> praxis_utils::Result<()> {
    praxis_utils::init()?;
    praxis_ecs::init()?;
    praxis_input::init()?;
    praxis_audio::init()?;
    praxis_window::run()?;

    Ok(())
}
```

**Initialization Order:**
1. `praxis_utils` - Tracing/logging (must be first for diagnostics)
2. `praxis_ecs` - ECS world setup
3. `praxis_input` - Input system
4. `praxis_audio` - Audio manager
5. `praxis_window::run()` - Event loop (blocking, runs engine)

#### Design Assessment
- **Pattern Used:** Facade/Coordinator pattern
- **Industry Alignment:** **Matches** - Standard engine bootstrap pattern
- **Modern Approach:** **Yes** - Sequential init with error propagation

#### Issues Found

1. **Missing Subsystem Initializations** (Severity: MEDIUM)
   - **Location:** `src/lib.rs:4-12`
   - **Problem:** Several crates are dependencies but not initialized:
     - `praxis_graphics` - Listed in Cargo.toml but `init()` not called
     - `praxis_math` - Has `init()` function but not called
   - **Impact:** These may initialize lazily, but explicit init is more predictable
   - **Proposed Fix:**
     ```rust
     pub fn run() -> praxis_utils::Result<()> {
         praxis_utils::init()?;  // First: logging
         praxis_math::init()?;   // Second: math (SIMD detection)
         praxis_ecs::init()?;
         praxis_input::init()?;
         praxis_audio::init()?;
         // praxis_graphics is initialized by praxis_window with RenderContext
         praxis_window::run()?;

         Ok(())
     }
     ```
   - **References:** Engine architecture patterns

2. **No Shutdown Hooks** (Severity: LOW)
   - **Location:** `src/lib.rs`
   - **Problem:** No cleanup/shutdown functions for subsystems
   - **Impact:** Resources rely on `Drop` implementations, which is fine but less explicit
   - **Proposed Fix:** Consider adding shutdown hooks if cleanup needs coordination:
     ```rust
     // After window loop exits
     praxis_audio::shutdown()?;
     // etc.
     ```

3. **Hardcoded Initialization Order** (Severity: LOW)
   - **Location:** `src/lib.rs:4-12`
   - **Problem:** Order is fixed; no configuration for optional subsystems
   - **Impact:** Can't skip audio init on headless servers, etc.
   - **Proposed Fix:** Consider builder pattern for configurable engine:
     ```rust
     pub struct EngineBuilder {
         enable_audio: bool,
         enable_graphics: bool,
         // ...
     }

     impl EngineBuilder {
         pub fn new() -> Self { /* ... */ }
         pub fn without_audio(mut self) -> Self { self.enable_audio = false; self }
         pub fn run(self) -> Result<()> { /* ... */ }
     }
     ```
   - **References:** Bevy's `App` builder pattern

#### Positive Findings
- Correct initialization order (logging first for diagnostics)
- Proper error propagation with `?`
- Simple, understandable code
- praxis_graphics is initialized via praxis_window (correct architecture)

---

## Research Context

### Industry Standards Consulted
- Bevy App initialization
- Godot engine startup sequence
- Game engine lifecycle patterns

### Modern Best Practices (2024-2025)

| Practice | Status | Notes |
|----------|--------|-------|
| Logging first | **Matches** | praxis_utils first |
| Error propagation | **Matches** | Uses `?` operator |
| Configurable subsystems | **Missing** | Hardcoded order |
| Shutdown coordination | **Missing** | Uses Drop only |

### Deprecated Approaches Avoided
- Not using global mutable state for coordination
- Not using panics for recoverable errors

---

## Recommendations Summary

### Critical (Must Fix)
*None*

### High Priority
*None*

### Medium Priority
1. Add missing `praxis_math::init()` call for explicit initialization

### Low Priority / Nice to Have
1. Consider builder pattern for configurable engine initialization
2. Add explicit shutdown hooks for coordinated cleanup
3. Document initialization order dependencies

### Positive Highlights
- **Extremely simple** - Easy to understand and maintain
- **Correct order** - Logging first, then subsystems
- **Clean error handling** - Propagates errors correctly
- **Appropriate scope** - Doesn't try to do too much

---

## Final Rating

| Category | Score | Notes |
|----------|-------|-------|
| Implementation Completeness | 7/10 | Missing some init calls |
| Logic Correctness | 9/10 | Order is correct |
| Design Quality | 8/10 | Simple but inflexible |
| Modernness | 7/10 | Could use builder pattern |
| Performance | 10/10 | N/A - just coordination |
| **Overall** | **8/10** | Good |

---

*Report generated: January 2026*
