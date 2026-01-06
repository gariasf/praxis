# praxis_scripting Audit Report

**Audit Date:** January 2026
**Lines of Code:** ~1,634
**Test Coverage:** 32 tests (excellent coverage)

## Executive Summary

`praxis_scripting` provides a comprehensive Lua scripting integration using [mlua](https://github.com/khvzak/mlua). The implementation includes sandboxing with three security levels, hot-reload via file watching, performance monitoring with warnings, and ECS integration. The code is **well-designed and feature-complete**. The main limitation is that the ECS systems (`script_start_system` and `script_update_system`) are **disabled** due to architectural challenges with World access.

**Overall Assessment: GOOD (8/10)**

---

## Features Inventory

### Feature 1: Scripting Context

**Location:** `src/context.rs`
**Purpose:** Lua VM management and script execution

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] No TODO/FIXME markers
- [x] Good test coverage (4 tests)

#### Code Analysis

```rust
pub struct ScriptingContext {
    lua: Arc<Lua>,
    config: ScriptingConfig,
    loaded_scripts: HashMap<String, PathBuf>,
    hot_reload_watcher: Option<Arc<RwLock<HotReloadWatcher>>>,
    performance_monitor: Option<Arc<ScriptPerformanceMonitor>>,
}
```

**Key Features:**
- Lua VM lifecycle management
- Script loading from file or string
- Global variable get/set
- Function calling with arguments
- Performance monitoring integration
- Hot-reload support
- World context for ECS access

#### Design Assessment
- **Pattern Used:** VM wrapper with resource management
- **Industry Alignment:** **Matches** - Standard script engine pattern
- **Modern Approach:** **Yes** - Using mlua (modern Lua bindings)

#### Positive Findings
- **Clean API** - load_script, call_function, get/set_global
- **Path tracking** - Remembers loaded script paths for reload
- **with_world()** - Scoped ECS access pattern

---

### Feature 2: Sandbox System

**Location:** `src/sandbox.rs`
**Purpose:** Script security isolation

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] Three security levels
- [x] Test coverage (3 tests)

#### Code Analysis

```rust
pub enum SandboxLevel {
    /// No restrictions - full access to all Lua features
    None,
    /// Moderate restrictions - disables dangerous features
    Moderate,
    /// Strict restrictions - only allows safe operations
    Strict,
}

pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allow_file_io: bool,
    pub allow_network: bool,
    pub allow_os_access: bool,
}
```

**Removals by Level:**

| Feature | Moderate | Strict |
|---------|----------|--------|
| io module | Yes | Yes |
| os module | Conditional | Yes |
| dofile/loadfile/load | Yes | Yes |
| require/package | No | Yes |

#### Design Assessment
- **Pattern Used:** Whitelist/blacklist sandbox
- **Industry Alignment:** **Matches** - Standard Lua sandbox approach
- **Modern Approach:** **Yes**

#### Issues Found

1. **No Memory/Instruction Limits** (Severity: MEDIUM)
   - **Location:** `src/sandbox.rs`
   - **Problem:** memory_limit in config is not enforced
   - **Impact:** Scripts can exhaust memory or run forever
   - **Proposed Fix:** Use mlua's hook system for instruction limits:
     ```rust
     lua.set_hook(HookTriggers::every_nth_instruction(10000), |lua, _debug| {
         // Check instruction count, raise error if exceeded
         Err(LuaError::RuntimeError("Instruction limit exceeded".into()))
     })?;
     ```
   - **References:** mlua documentation on hooks

#### Positive Findings
- **Three levels** - Granular security control
- **Configurable I/O** - allow_file_io, allow_network
- **Safe defaults** - Moderate level by default

---

### Feature 3: Hot Reload

**Location:** `src/hot_reload.rs`
**Purpose:** Automatic script reloading on file changes

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] Uses notify crate
- [x] Test coverage (3 tests)

#### Code Analysis

```rust
pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    receiver: Arc<Mutex<Receiver<NotifyResult<Event>>>>,
    events: Vec<ScriptEvent>,
}

pub enum ScriptEvent {
    Modified(PathBuf),
    Removed(PathBuf),
}
```

**Key Features:**
- File system watching via notify crate
- Recursive directory watching
- Filters for .lua files only
- Debounced event processing

#### Design Assessment
- **Pattern Used:** File watcher with event queue
- **Industry Alignment:** **Matches** - Standard hot-reload pattern
- **Modern Approach:** **Yes** - Using notify crate

#### Positive Findings
- **Automatic filtering** - Only .lua files trigger events
- **Clean event API** - Modified/Removed events
- **Polling model** - Non-blocking event retrieval

---

### Feature 4: Performance Monitoring

**Location:** `src/performance.rs`
**Purpose:** Script execution time tracking

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] Comprehensive statistics
- [x] Test coverage (5 tests)

#### Code Analysis

```rust
pub struct ScriptStats {
    pub script_name: String,
    pub function_name: Option<String>,
    pub execution_count: u64,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub warning_count: u64,
}

pub struct ScriptPerformanceMonitor {
    stats: RwLock<HashMap<String, ScriptStats>>,
    warning_threshold_ms: u64,
}
```

**Key Features:**
- Per-function execution tracking
- Automatic warning on slow scripts
- Min/max/average time statistics
- Sorted queries (slowest scripts)

#### Design Assessment
- **Pattern Used:** Statistical profiler
- **Industry Alignment:** **Matches** - Standard script profiling
- **Modern Approach:** **Yes**

#### Positive Findings
- **Configurable threshold** - 16ms default (60fps budget)
- **Warning logging** - Automatic slow script warnings
- **Reset capability** - Clear stats for fresh measurement

---

### Feature 5: ECS Bindings

**Location:** `src/bindings/ecs_api.rs`
**Purpose:** Expose ECS World to Lua scripts

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [ ] Limited component support
- [x] Test coverage (1 test)

#### Code Analysis

```rust
thread_local! {
    static WORLD_CONTEXT: RefCell<Option<*mut World>> = const { RefCell::new(None) };
}

// World table functions:
// - spawn() -> Entity
// - despawn(entity)
// - get_entity_by_name(name) -> Entity?
// - add_component_transform(entity, x, y, z)
// - add_component_name(entity, name)
// - get_component_transform(entity) -> Transform
// - set_component_transform(entity, transform)
// - get_component_name(entity) -> string
```

#### Design Assessment
- **Pattern Used:** Thread-local context with raw pointer
- **Industry Alignment:** **Partial** - Works but unsafe
- **Modern Approach:** **Partial** - Raw pointer is risky

#### Issues Found

1. **Limited Component Support** (Severity: MEDIUM)
   - **Location:** `src/bindings/ecs_api.rs:97-167`
   - **Problem:** Only Transform and Name components exposed
   - **Impact:** Scripts can't access physics, audio, etc.
   - **Proposed Fix:** Add generic component access or more bindings:
     ```rust
     // Add more component bindings
     table.set("get_component_velocity", ...)?;
     table.set("get_component_audio_source", ...)?;

     // Or use reflection-like API
     table.set("get_component", |entity, component_type| ...)?;
     ```

2. **Raw Pointer World Access** (Severity: MEDIUM)
   - **Location:** `src/bindings/ecs_api.rs:40-55`
   - **Problem:** Uses raw pointer stored in thread_local
   - **Impact:** Potential for use-after-free if not careful
   - **Note:** Safe in practice due to scoped with_world() usage
   - **Proposed Fix:** Consider using a RefCell or safer abstraction

#### Positive Findings
- **LuaEntity UserData** - Proper entity wrapper
- **LuaTransform UserData** - With translate method
- **Scoped access** - with_world() pattern prevents leaks

---

### Feature 6: Math API

**Location:** `src/bindings/math_api.rs`
**Purpose:** Math types for Lua scripts

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] Test coverage (2 tests)

#### Code Analysis

**Exposed Functions:**
- `math.Vec3(x, y, z)` - Create vector
- `math.Quat(x, y, z, w)` - Create quaternion
- `math.pi`, `math.tau` - Constants
- `math.sqrt`, `math.sin`, `math.cos`, `math.tan`, `math.abs`

#### Issues Found

1. **Vec3/Quat as Tables** (Severity: LOW)
   - **Location:** `src/bindings/math_api.rs:12-33`
   - **Problem:** Returns plain tables, not UserData with methods
   - **Impact:** No operator overloading, no methods
   - **Proposed Fix:** Create proper UserData types:
     ```rust
     struct LuaVec3(Vec3);

     impl UserData for LuaVec3 {
         fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
             methods.add_meta_method(MetaMethod::Add, |_, this, other: Self| {
                 Ok(LuaVec3(this.0 + other.0))
             });
             methods.add_method("length", |_, this, ()| Ok(this.0.length()));
         }
     }
     ```

#### Positive Findings
- **Core math functions** - All basic trig and utilities
- **Constants** - pi, tau available

---

### Feature 7: Engine API

**Location:** `src/bindings/engine_api.rs`
**Purpose:** Engine utility functions for scripts

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] Test coverage (1 test)

#### Code Analysis

**Exposed Functions:**
- `engine.log_info(msg)`
- `engine.log_debug(msg)`
- `engine.log_warn(msg)`
- `engine.log_error(msg)`

#### Design Assessment
- **Pattern Used:** Engine utility namespace
- **Industry Alignment:** **Matches** - Standard logging API
- **Modern Approach:** **Yes**

#### Issues Found

1. **Limited Engine API** (Severity: LOW)
   - **Location:** `src/bindings/engine_api.rs`
   - **Problem:** Only logging functions exposed
   - **Impact:** Scripts can't access time, input, scene management
   - **Proposed Fix:** Add more engine utilities:
     ```rust
     engine_table.set("get_delta_time", ...)?;
     engine_table.set("get_time", ...)?;
     engine_table.set("quit", ...)?;
     ```

#### Positive Findings
- **Clean logging** - All log levels available
- **Script prefix** - Messages prefixed with [Script]

---

### Feature 8: Script Component

**Location:** `src/script_component.rs`
**Purpose:** Attach scripts to ECS entities

#### Implementation Status
- [x] Real implementation (not stub)
- [x] Logic correctness verified
- [x] User data storage
- [x] Test coverage (2 tests)

#### Code Analysis

```rust
#[derive(Component, Clone)]
pub struct ScriptComponent {
    pub name: String,
    pub script_path: PathBuf,
    pub initialized: bool,
    pub user_data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}
```

**Lifecycle Methods:**
- `on_start()` - Called once when spawned
- `on_update(delta_time)` - Called every frame
- `on_destroy()` - Called when destroyed

#### Design Assessment
- **Pattern Used:** Component-based scripting
- **Industry Alignment:** **Matches** - Similar to Unity MonoBehaviour
- **Modern Approach:** **Yes**

#### Positive Findings
- **Persistent data** - user_data HashMap survives reloads
- **JSON values** - Flexible data storage
- **Clean lifecycle** - Unity-like on_start/on_update

---

### Feature 9: ECS Systems

**Location:** `src/systems.rs`
**Purpose:** Script execution via ECS systems

#### Implementation Status
- [ ] PARTIALLY DISABLED - Major systems not working
- [x] ScriptingResource implemented
- [ ] script_start_system disabled
- [ ] script_update_system disabled

#### Code Analysis

```rust
#[derive(Resource)]
pub struct ScriptingResource {
    context: ScriptingContext,
}

// SAFETY: ScriptingContext is only accessed from the main thread
unsafe impl Send for ScriptingResource {}
unsafe impl Sync for ScriptingResource {}
```

**Systems:**
- `script_initialization_system` - Works, loads scripts
- `script_start_system` - **DISABLED** - Can't access World
- `script_update_system` - **DISABLED** - Can't access World
- `script_hot_reload_system` - Works, processes file changes

#### Issues Found

1. **ECS Systems Disabled** (Severity: HIGH)
   - **Location:** `src/systems.rs:80-105`
   - **Problem:** script_start_system and script_update_system are disabled
   - **Impact:** Scripts can't use on_start/on_update lifecycle
   - **Root Cause:** Can't access World from within ECS system
   - **Proposed Fix:** Use exclusive system or command queue:
     ```rust
     // Option 1: Exclusive system
     fn script_update_system(world: &mut World) {
         let mut scripting = world.resource_mut::<ScriptingResource>();
         let delta = world.resource::<DeltaTime>().0;

         scripting.context_mut().with_world(world, |lua| {
             // Call scripts with world access
         });
     }

     // Option 2: Command queue pattern
     // Scripts queue commands, systems execute them later
     ```

2. **Unsafe Send+Sync** (Severity: MEDIUM)
   - **Location:** `src/systems.rs:21-22`
   - **Problem:** Manual unsafe Send+Sync implementation
   - **Impact:** Could cause UB if used incorrectly
   - **Note:** Safe in practice with single-threaded access
   - **Proposed Fix:** Document thread safety requirements clearly

#### Positive Findings
- **Marker components** - ScriptInitialized, ScriptStarted
- **Error handling** - Logs errors during initialization

---

## Research Context

### Industry Standards Consulted
- mlua documentation
- Unity MonoBehaviour lifecycle
- Godot GDScript design
- Lua 5.4 Reference Manual
- Game engine scripting patterns

### Modern Best Practices (2024-2025)

| Practice | Praxis Status | Notes |
|----------|---------------|-------|
| Sandboxing | **Matches** | Three security levels |
| Hot-reload | **Matches** | File watching with notify |
| Performance monitoring | **Matches** | Comprehensive stats |
| ECS integration | **Partial** | Systems disabled |
| Type-safe bindings | **Matches** | mlua UserData |
| Coroutine support | **Missing** | No async/yield |
| Memory limits | **Missing** | Not enforced |
| Instruction limits | **Missing** | No timeout |

### Deprecated Approaches Avoided
- Not using raw Lua C API (uses mlua)
- Not blocking on script execution (could add coroutines)
- Not exposing unsafe operations by default

---

## Recommendations Summary

### Critical (Must Fix)
*None*

### High Priority
1. **Fix script_start_system and script_update_system** - Currently disabled, scripts can't use lifecycle methods

### Medium Priority
1. Enforce memory limits (use mlua memory hooks)
2. Add instruction count limits for infinite loop protection
3. Expand ECS bindings (physics, audio, etc.)
4. Consider safer World access pattern (avoid raw pointer)

### Low Priority / Nice to Have
1. Add Vec3/Quat as proper UserData with operators
2. Expand engine API (time, input, scene)
3. Add coroutine/async script support
4. Add script debugging (breakpoints, step)
5. Add script error recovery (continue after error)
6. Add script-to-script communication

### Positive Highlights
- **Clean mlua integration** - Modern Rust-Lua bindings
- **Three-level sandbox** - Configurable security
- **Hot-reload** - Rapid iteration with file watching
- **Performance monitoring** - Automatic slow script detection
- **Good test coverage** - 32 tests
- **User data persistence** - Scripts can store state
- **Unity-like lifecycle** - Familiar on_start/on_update pattern

---

## Final Rating

| Category | Score | Notes |
|----------|-------|-------|
| Implementation Completeness | 7/10 | ECS systems disabled |
| Logic Correctness | 9/10 | All working features correct |
| Design Quality | 9/10 | Clean architecture |
| Modernness | 8/10 | Missing memory/instruction limits |
| Test Coverage | 9/10 | 32 tests, excellent coverage |
| **Overall** | **8/10** | Good |

**Note:** The scripting system has excellent infrastructure but the disabled ECS systems significantly limit its usefulness. Once `script_start_system` and `script_update_system` are working, this would be a 9/10 system. The sandbox, hot-reload, and performance monitoring are production-quality.

---

*Report generated: January 2026*
