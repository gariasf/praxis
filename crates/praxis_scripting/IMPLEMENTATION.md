# Scripting System Implementation

This document describes the implementation of the Praxis scripting system.

## Overview

The scripting system provides a comprehensive Lua integration layer that exposes the engine's ECS World and APIs to runtime scripts, enabling rapid iteration through hot-reload, security through sandboxing, and performance insights through monitoring.

## Architecture

### Core Components

#### 1. ScriptingContext (`context.rs`)

The main entry point for the scripting system. Manages the Lua VM lifecycle and provides the API for loading and executing scripts.

**Key Features:**
- Lua 5.4 VM initialization via `mlua`
- Script loading from files or strings
- Function calling with type-safe arguments
- Global variable management
- ECS World context provisioning
- Hot-reload processing

**Usage:**
```rust
let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;
context.load_script("game", "scripts/game.lua")?;
context.call_function("game", "update", 0.016)?;
```

#### 2. Sandboxing (`sandbox.rs`)

Provides configurable security restrictions to prevent malicious or accidental misuse of the Lua environment.

**Sandbox Levels:**
- **None**: No restrictions (full Lua access)
- **Moderate**: Disables dangerous operations (`dofile`, `loadfile`, `load`), restricts `os` and `io` modules
- **Strict**: Maximum restrictions, removes module loading capabilities

**Implementation:**
- Removes or restricts Lua global functions
- Filters OS module to safe functions only
- Prevents file I/O when configured
- Blocks dynamic code loading in strict mode

#### 3. Performance Monitoring (`performance.rs`)

Tracks script execution statistics to identify performance bottlenecks.

**Metrics Tracked:**
- Total execution count
- Total, average, min, max execution time
- Warning count (exceeding threshold)

**Features:**
- Automatic timing of script function calls
- Per-script and per-function statistics
- Slowest scripts identification
- Warning generation for expensive operations

#### 4. Hot-Reload (`hot_reload.rs`)

Watches script files for changes and automatically reloads them.

**Implementation:**
- Uses `notify` crate for filesystem watching
- Recursive directory monitoring
- Filters for `.lua` file extensions
- Event-based reload triggering

**Events:**
- `Modified`: Script file changed
- `Removed`: Script file deleted

#### 5. Bindings (`bindings/`)

Exposes engine APIs to Lua scripts.

##### Math API (`bindings/math_api.rs`)

- **Vec3**: 3D vectors with operations (add, sub, mul, dot, cross, length, normalize)
- **Quat**: Quaternions for rotations (from_rotation_x/y/z, multiply, normalize)
- **Constants**: `math.pi`, `math.tau`

##### ECS API (`bindings/ecs_api.rs`)

- **Entity Operations**: spawn, despawn, get_entity_by_name
- **Component Access**: get/set Transform, Name components
- **World Context**: Thread-local world pointer for safe access

##### Engine API (`bindings/engine_api.rs`)

- **Logging**: log_info, log_debug, log_warn, log_error

#### 6. Script Component (`script_component.rs`)

ECS component for attaching scripts to entities.

**Features:**
- Script name and path storage
- Initialization tracking
- Persistent user data (JSON values)
- Lifecycle management (on_start, on_update, on_destroy)

#### 7. ECS Systems (`systems.rs`)

Integration with the ECS scheduler for automatic script execution.

**Systems:**
- `script_initialization_system`: Loads scripts for new entities
- `script_start_system`: Calls `on_start` for initialized scripts
- `script_update_system`: Calls `on_update` every frame
- `script_hot_reload_system`: Processes file change events

**Resource:**
- `ScriptingResource`: Wraps `ScriptingContext` as an ECS resource

## Implementation Details

### Memory Management

- Lua VM owned by `ScriptingContext`
- Thread-local world pointer for safe ECS access
- Automatic cleanup on context drop

### Thread Safety

- Scripts execute on the main thread only
- World context uses thread-local storage
- Parking lot RwLock for concurrent statistics access

### Error Handling

- All public APIs return `Result<T>` using `praxis_utils::Result`
- Lua errors wrapped in Rust error types
- Detailed error messages with context

### Performance Considerations

- Minimal overhead for disabled features (monitoring, hot-reload)
- LuaJIT compatibility (future optimization path)
- Efficient component access patterns
- Statistics stored in lock-free structures when possible

## Configuration

### ScriptingConfig

```rust
pub struct ScriptingConfig {
    pub sandbox: SandboxConfig,
    pub enable_performance_monitoring: bool,
    pub max_execution_time_ms: u64,
    pub memory_limit: usize,
}
```

### SandboxConfig

```rust
pub struct SandboxConfig {
    pub level: SandboxLevel,
    pub allow_file_io: bool,
    pub allow_network: bool,
    pub allow_os_access: bool,
}
```

## Testing

### Unit Tests

Each module includes focused unit tests:
- `context.rs`: Script loading, function calls, globals
- `sandbox.rs`: Security enforcement
- `performance.rs`: Statistics tracking
- `hot_reload.rs`: File watching

### Integration Tests

Comprehensive tests in `tests/integration_test.rs`:
- ECS World access from scripts
- Component manipulation
- Entity spawning/despawning
- Math API operations
- Sandbox enforcement
- Performance monitoring

### Example Programs

- `examples/scripting_demo.rs`: Basic usage demonstration
- `examples/scripting_advanced_demo.rs`: ECS systems integration
- `examples/scripts/*.lua`: Sample Lua scripts

## API Surface

### Rust API

```rust
// Context creation
let context = ScriptingContext::new(config)?;

// Script loading
context.load_script(name, path)?;
context.load_string(name, source)?;

// Function calling
let result: T = context.call_function(script, function, args)?;

// World access
context.with_world(&mut world, |lua| { ... })?;

// Hot-reload
context.enable_hot_reload(path)?;
context.process_hot_reload()?;

// Performance
if let Some(monitor) = context.performance_monitor() {
    let stats = monitor.get_stats(script, function)?;
}
```

### Lua API

```lua
-- World operations
local entity = world.spawn()
world.despawn(entity)
local entity = world.get_entity_by_name("Player")

-- Component operations
world.add_component_transform(entity, x, y, z)
world.add_component_name(entity, name)
local transform = world.get_component_transform(entity)
world.set_component_transform(entity, transform)

-- Math operations
local v = math.Vec3(x, y, z)
local length = v:length()
local normalized = v:normalize()
local sum = v1 + v2

-- Logging
engine.log_info("Message")
engine.log_debug("Debug")
engine.log_warn("Warning")
engine.log_error("Error")

-- Lifecycle
function on_start() end
function on_update(delta_time) end
function on_destroy() end
```

## Dependencies

- **mlua**: Lua bindings for Rust (0.9)
- **notify**: Filesystem watching for hot-reload (6.1)
- **parking_lot**: Efficient synchronization primitives (0.12)
- **serde/serde_json**: Serialization for user data (1.0)
- **bevy_ecs**: ECS integration (0.14)
- **praxis_ecs**: Engine ECS abstractions
- **praxis_math**: Math types (glam wrapper)
- **praxis_utils**: Logging and error handling

## Future Enhancements

### Short Term
- [ ] More component bindings (Mesh, Material, RigidBody, etc.)
- [ ] Input system API
- [ ] Physics system API
- [ ] Audio system API

### Medium Term
- [ ] LuaJIT support for performance
- [ ] Script debugging integration
- [ ] Better error reporting with stack traces
- [ ] Script profiler integration

### Long Term
- [ ] Parallel script execution
- [ ] Visual scripting bridge
- [ ] Network API (with sandbox)
- [ ] Asset system API

## Security Considerations

### Sandboxing

The sandbox system prevents:
- File system access (when configured)
- Network operations (future)
- OS command execution
- Dynamic code loading (strict mode)
- Module system abuse

### Best Practices

1. **Always use sandboxing** for untrusted scripts
2. **Enable performance monitoring** to detect DoS attempts
3. **Set memory limits** to prevent memory exhaustion
4. **Validate script sources** before loading
5. **Use Moderate/Strict sandbox** by default

### Known Limitations

- Scripts run on main thread (CPU bound)
- No async/await support
- Limited to Lua 5.4 features
- Component access not fully type-safe at Lua level

## Performance Characteristics

### Benchmarks (Estimated)

- Script load: ~1-5ms for typical scripts
- Function call overhead: ~10-50μs
- Component access: ~100-500ns (cached) to ~10-50μs (queried)
- Hot-reload check: ~100μs per directory
- Performance monitoring: ~50-100ns overhead per call

### Optimization Tips

1. Cache entity references in `on_start()`
2. Use local variables in Lua
3. Batch component operations
4. Minimize ECS queries per frame
5. Profile with performance monitor

## Maintenance

### Adding New Bindings

1. Create new module in `bindings/`
2. Implement `UserData` for complex types
3. Register with `lua.globals()` or custom table
4. Add tests for new functionality
5. Document in README and guides

### Extending Sandbox

1. Add new restrictions in `sandbox.rs`
2. Update `SandboxConfig` with new options
3. Test restriction enforcement
4. Document security implications

### Improving Performance

1. Profile with criterion benchmarks
2. Optimize hot paths (component access, function calls)
3. Consider LuaJIT migration
4. Cache expensive computations
5. Use parallel execution where possible (future)

## Documentation

- **README.md**: Quick start and API reference
- **docs/guides/scripting.md**: Comprehensive guide
- **IMPLEMENTATION.md**: This document
- **examples/**: Runnable examples
- **Rustdoc**: API documentation (cargo doc)

## Conclusion

The scripting system provides a powerful, safe, and performant way to extend the Praxis engine with runtime logic. The modular architecture allows for incremental feature addition while maintaining stability and security.
