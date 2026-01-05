# praxis_utils Audit Report

**Audit Date:** January 2026
**Lines of Code:** ~300
**Test Coverage:** 3 tests (timing: 2, observability: 1)

## Executive Summary

`praxis_utils` provides foundational utilities for logging/tracing and frame timing. The implementation is **solid and well-designed** with proper use of the Rust ecosystem's best libraries (tracing, color-eyre). The timing module provides a clean frame timer with global state access. Minor improvements could be made to global state handling and tracing performance.

**Overall Assessment: VERY GOOD (8.5/10)**

---

## Features Inventory

### Feature 1: Tracing/Logging System (`observability` module)

**Location:** `src/observability.rs`
**Purpose:** Initialize tracing and error reporting infrastructure

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [x] Adequate test coverage (1 test)

#### Code Analysis

**`init_tracing()`** (lines 37-58):
```rust
pub fn init_tracing() -> Result<()> {
    color_eyre::install()?;

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))?
        .add_directive("winit=info".parse().unwrap())
        .add_directive("vulkano=debug".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}
```

**Logic Flow:**
1. Installs color-eyre for pretty panic reports
2. Creates a formatting layer with thread IDs and span events
3. Sets up environment-based filtering with defaults
4. Suppresses verbose winit/vulkano logs
5. Initializes global subscriber

#### Design Assessment
- **Pattern Used:** Layered subscriber pattern (tracing-subscriber)
- **Industry Alignment:** **Matches** - Standard Rust approach
- **Modern Approach:** **Yes** - Using tracing 0.1 / tracing-subscriber 0.3 (current)

#### Issues Found

1. **Pretty Formatting May Impact Performance** (Severity: LOW)
   - **Location:** `src/observability.rs:45`
   - **Problem:** `.pretty()` formatting is slower than compact output
   - **Impact:** Minor performance overhead in debug builds, but acceptable for development
   - **Proposed Fix:** Consider adding a release build variant or env-based toggle:
     ```rust
     // Before
     .pretty();

     // After
     #[cfg(debug_assertions)]
     let fmt_layer = fmt_layer.pretty();
     ```
   - **References:** [tracing-subscriber docs](https://docs.rs/tracing-subscriber)

2. **`.unwrap()` on Directive Parsing** (Severity: LOW)
   - **Location:** `src/observability.rs:49-50`
   - **Problem:** Uses `.unwrap()` on string parsing, though the strings are hardcoded and valid
   - **Impact:** None in practice - these are compile-time constant strings that will always parse
   - **Proposed Fix:** Document why unwrap is safe, or use const parsing:
     ```rust
     // Add comment explaining safety
     // SAFETY: These are hardcoded valid directives that cannot fail to parse
     .add_directive("winit=info".parse().expect("valid directive"))
     ```
   - **References:** Rust API guidelines on `.unwrap()` usage

3. **Missing Non-blocking Option for Game Engine** (Severity: MEDIUM)
   - **Location:** `src/observability.rs`
   - **Problem:** The current setup blocks on stdout writes, which can cause frame hitches
   - **Impact:** In release builds with logging enabled, I/O can cause frame stuttering
   - **Proposed Fix:** Add tracing-appender for non-blocking writes:
     ```rust
     use tracing_appender::non_blocking;

     pub fn init_tracing_non_blocking() -> Result<WorkerGuard> {
         let (non_blocking, guard) = non_blocking(std::io::stdout());
         // ... use non_blocking writer
         Ok(guard)
     }
     ```
   - **References:** [tracing-appender docs](https://docs.rs/tracing-appender)

4. **Default Filter Level is Debug** (Severity: LOW)
   - **Location:** `src/observability.rs:48`
   - **Problem:** Falls back to `debug` level, which is verbose for release builds
   - **Impact:** Excessive logging if RUST_LOG not set
   - **Proposed Fix:**
     ```rust
     // Before
     .or_else(|_| EnvFilter::try_new("debug"))?

     // After
     .or_else(|_| {
         #[cfg(debug_assertions)]
         return EnvFilter::try_new("debug");
         #[cfg(not(debug_assertions))]
         return EnvFilter::try_new("info");
     })?
     ```

#### Positive Findings
- Proper layered subscriber architecture
- Environment variable configuration (`RUST_LOG`)
- Suppresses verbose dependency logs (winit, vulkano)
- Thread ID tracking useful for debugging
- Span events for performance tracing
- Custom layer support via `init_tracing_with_layer()`

---

### Feature 2: Frame Timer (`timing` module)

**Location:** `src/timing.rs`
**Purpose:** Track frame timing, delta time, FPS

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [x] Adequate test coverage (2 tests)

#### Code Analysis

**`FrameTimer` struct** (lines 118-149):
- Tracks frame timestamps with `Instant`
- Supports optional FPS limiting
- Calculates FPS over 1-second windows
- Updates global timing state

**Key Logic:**

```rust
// Delta clamping (line 190-193)
const MAX_DELTA: Duration = Duration::from_millis(100);
if self.delta_time > MAX_DELTA {
    self.delta_time = MAX_DELTA;
}
```

This correctly prevents physics/animation explosions after debug pauses or loading screens.

**Global Timing Access:**

```rust
static GLOBAL_TIMING: OnceLock<Mutex<GlobalTiming>> = OnceLock::new();
```

Uses `OnceLock` + `Mutex` for thread-safe global access.

#### Design Assessment
- **Pattern Used:** Global singleton with lock
- **Industry Alignment:** **Partially matches** - ECS-based engines typically use resources instead
- **Modern Approach:** **Partially** - `OnceLock` is modern (stable Rust 1.70+), but global mutex has overhead

#### Issues Found

1. **Mutex Lock on Every `delta_time()` Call** (Severity: MEDIUM)
   - **Location:** `src/timing.rs:51-57`
   - **Problem:** Every call to `delta_time()` acquires a mutex lock
   - **Impact:** Contention if called from multiple threads; unnecessary overhead for single-threaded access
   - **Proposed Fix:** Use `AtomicU64` for delta_secs (via `f32::to_bits()`):
     ```rust
     // Before
     static GLOBAL_TIMING: OnceLock<Mutex<GlobalTiming>> = OnceLock::new();

     // After (for frequently-read values)
     static DELTA_SECS: AtomicU32 = AtomicU32::new(0);

     pub fn delta_time() -> f32 {
         f32::from_bits(DELTA_SECS.load(Ordering::Relaxed))
     }
     ```
   - **References:** [Rust atomics](https://doc.rust-lang.org/std/sync/atomic/)

2. **FPS Calculated Every Second** (Severity: LOW)
   - **Location:** `src/timing.rs:212-217`
   - **Problem:** FPS only updates once per second, causing stale readings
   - **Impact:** UI showing FPS will appear "stuck" between updates
   - **Proposed Fix:** Use exponential moving average for smoother updates:
     ```rust
     // Rolling average
     let alpha = 0.1; // Smoothing factor
     self.current_fps = alpha * instantaneous_fps + (1.0 - alpha) * self.current_fps;
     ```

3. **`sleep_if_needed()` Uses `thread::sleep()`** (Severity: LOW)
   - **Location:** `src/timing.rs:243-253`
   - **Problem:** `thread::sleep()` is imprecise (OS scheduler granularity ~15ms on Windows)
   - **Impact:** FPS limiting may overshoot or undershoot target
   - **Proposed Fix:** Use spin-wait for final sub-millisecond precision:
     ```rust
     pub fn sleep_if_needed(&self) -> Duration {
         if let Some(target_duration) = self.target_frame_duration {
             let frame_time = self.current_frame.elapsed();
             if frame_time < target_duration {
                 let remaining = target_duration - frame_time;
                 // Sleep for most of the time
                 if remaining > Duration::from_millis(2) {
                     std::thread::sleep(remaining - Duration::from_millis(1));
                 }
                 // Spin-wait for precision
                 while self.current_frame.elapsed() < target_duration {}
             }
         }
         Duration::ZERO
     }
     ```
   - **References:** Game engine frame pacing techniques

4. **No Fixed Timestep Support** (Severity: MEDIUM)
   - **Location:** `src/timing.rs`
   - **Problem:** Only provides variable delta time, no fixed timestep accumulator
   - **Impact:** Physics simulations may be unstable without fixed timestep
   - **Proposed Fix:** Add fixed timestep helper:
     ```rust
     pub struct FixedTimestep {
         accumulator: Duration,
         fixed_delta: Duration,
     }

     impl FixedTimestep {
         pub fn new(hz: f64) -> Self { /* ... */ }
         pub fn update(&mut self, delta: Duration) -> impl Iterator<Item = ()> {
             // Yields once for each fixed step to process
         }
     }
     ```
   - **References:** ["Fix Your Timestep!"](https://gafferongames.com/post/fix_your_timestep/) by Glenn Fiedler

#### Positive Findings
- Excellent delta time clamping (100ms max)
- Clean API with builder pattern
- Global access pattern for convenience
- FPS tracking with readable stats output
- Proper test coverage for edge cases

---

### Feature 3: Re-exports and init()

**Location:** `src/lib.rs`
**Purpose:** Unified API for utilities

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers

#### Code Analysis

Re-exports `color_eyre` types and `tracing` macros for convenience:
```rust
pub use color_eyre::{eyre::{self, Error}, Report, Result};
pub use tracing::{debug, error, info, instrument, trace, warn};
```

#### Design Assessment
- **Pattern Used:** Facade pattern
- **Industry Alignment:** **Matches**
- **Modern Approach:** **Yes**

#### Issues Found
*None* - Clean and appropriate re-exports.

#### Positive Findings
- Single import point for common utilities
- Exposes `instrument` for easy function tracing

---

## Research Context

### Industry Standards Consulted
- [tracing crate](https://github.com/tokio-rs/tracing) (262M+ downloads)
- [tracing-subscriber guide](https://docs.rs/tracing-subscriber)
- [color-eyre](https://docs.rs/color-eyre) for error handling
- Bevy's `Time` resource implementation
- Game engine frame timing articles

### Modern Best Practices (2024-2025)

| Topic | Best Practice | Praxis Status |
|-------|---------------|---------------|
| Logging | tracing with layered subscribers | **Matches** |
| Error handling | color-eyre for dev, custom for prod | **Matches** |
| Frame timing | Fixed timestep for physics | **Missing** |
| Global state | ECS resource over global | **Partial** |
| Non-blocking I/O | tracing-appender | **Missing** |

### Deprecated Approaches Avoided
- Not using `log` crate directly (tracing is successor)
- Not using `env_logger` (tracing-subscriber is better)
- Not using `anyhow` in library code (color-eyre more featureful)

---

## Recommendations Summary

### Critical (Must Fix)
*None*

### High Priority
*None*

### Medium Priority
1. Add non-blocking tracing option for release builds
2. Add fixed timestep support for physics
3. Consider atomic access for frequently-read timing values

### Low Priority / Nice to Have
1. Conditional pretty formatting for debug builds only
2. Rolling average FPS for smoother display
3. Hybrid sleep + spin-wait for precise frame limiting
4. Document `.unwrap()` safety in observability module

### Positive Highlights
- **Excellent library choices** - tracing/color-eyre are best-in-class
- **Clean API** - Easy to use from other crates
- **Proper delta clamping** - Prevents game explosions after pauses
- **Environment configuration** - RUST_LOG support
- **Custom layer support** - Extensible for editor console
- **Well-tested** - Edge cases covered (delta clamping test)

---

## Final Rating

| Category | Score | Notes |
|----------|-------|-------|
| Implementation Completeness | 9/10 | Missing fixed timestep, non-blocking |
| Logic Correctness | 10/10 | All logic verified correct |
| Design Quality | 8/10 | Global mutex could be improved |
| Modernness | 8/10 | Current libs, missing some modern patterns |
| Performance | 7/10 | Mutex locks, blocking I/O |
| **Overall** | **8.5/10** | Very Good |

---

*Report generated: January 2026*
