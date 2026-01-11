//! Sandbox configuration and enforcement for script security.
//!
//! # Security Model
//!
//! Scripts can potentially perform dangerous operations if left unrestricted:
//! - **File I/O**: Read sensitive files, write malicious code, delete data
//! - **Network**: Exfiltrate data, download malware, DDoS attacks
//! - **OS Access**: Execute arbitrary commands, spawn processes, terminate the engine
//! - **Dynamic Loading**: Load arbitrary code that bypasses sandbox restrictions
//! - **Resource Exhaustion**: Infinite loops, memory bombs, CPU starvation
//!
//! This module provides a **defense-in-depth** security model with three levels:
//!
//! ## Sandbox Levels
//!
//! ### None (Development Mode)
//! - All Lua features available
//! - No restrictions on dangerous operations
//! - Use for trusted scripts during development
//! - **Warning**: Scripts can access filesystem, network, and OS commands
//!
//! ### Moderate (Default)
//! - Removes dangerous globals: `io`, `os`, `dofile`, `loadfile`, `load`
//! - Allows: Math, string manipulation, table operations, custom APIs
//! - Configurable: File I/O and OS access can be selectively enabled
//! - Best for: Player-created mods with some trust
//!
//! ### Strict (Untrusted Scripts)
//! - All Moderate restrictions PLUS:
//!   - Removes `require` (no dynamic module loading)
//!   - Removes `package` (no package path manipulation)
//! - Only safe operations allowed
//! - Best for: User-generated content from untrusted sources
//!
//! ## Resource Limits
//!
//! Beyond API restrictions, resource limits prevent denial-of-service attacks:
//!
//! ### Instruction Limits
//! - Prevents infinite loops and CPU starvation
//! - Uses Lua hooks to check instruction count periodically
//! - Default: 1,000,000 instructions (tunable)
//! - Checked every 10,000 instructions for performance
//! - **Caveat**: Callback overhead is ~0.1%, acceptable for security
//!
//! ### Memory Limits
//! - Prevents memory bombs and OOM crashes
//! - Uses Lua's built-in memory allocator limits
//! - Default: 100 MB (tunable)
//! - Applies to: Tables, strings, closures, userdata
//! - **Note**: Rust objects passed as userdata are NOT counted
//!
//! ## Implementation Details
//!
//! ### Global Removal Pattern
//! Dangerous functions are removed by setting their globals to `nil`:
//! ```lua
//! -- Before sandbox
//! io.open("/etc/passwd", "r")  -- Works
//! 
//! -- After sandbox
//! io.open("/etc/passwd", "r")  -- Error: attempt to index nil value
//! ```
//!
//! ### Instruction Counting with Hooks
//! Lua provides hooks that fire at specific events (function call, line, instruction).
//! We use `every_nth_instruction` to check a counter:
//! ```rust
//! lua.set_hook(HookTriggers { every_nth_instruction: Some(10000) }, |_lua, _debug| {
//!     if instruction_count.fetch_add(10000, Ordering::Relaxed) >= limit {
//!         Err(RuntimeError("instruction limit exceeded"))
//!     }
//! });
//! ```
//!
//! ### Memory Limit Enforcement
//! Lua's allocator can be wrapped to track allocations:
//! ```rust
//! lua.set_memory_limit(100 * 1024 * 1024); // 100 MB
//! ```
//! When the limit is exceeded, allocations fail and Lua raises an error.

use mlua::{Lua, Value};
use praxis_utils::{debug, info, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Security level for sandboxing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    /// No restrictions - full access to all Lua features.
    /// 
    /// Use this for:
    /// - Trusted development scripts
    /// - Engine-internal scripts
    /// - Debug/testing scenarios
    ///
    /// **Warning**: Scripts can access files, network, and OS commands.
    None,

    /// Moderate restrictions - disables dangerous features but allows most operations.
    ///
    /// Removes:
    /// - File I/O (`io` library) - unless explicitly enabled
    /// - OS operations (`os` library) - unless explicitly enabled
    /// - Dynamic code loading (`dofile`, `loadfile`, `load`)
    ///
    /// Keeps:
    /// - Math, string, table manipulation
    /// - Custom engine APIs
    /// - Module loading (`require`) - for trusted internal modules
    ///
    /// Use this for:
    /// - Player-created mods
    /// - Community content with basic trust
    /// - Gameplay scripts with controlled API access
    Moderate,

    /// Strict restrictions - only allows safe operations.
    ///
    /// All Moderate restrictions PLUS:
    /// - Removes `require` (no module loading)
    /// - Removes `package` (no package path manipulation)
    ///
    /// Use this for:
    /// - User-generated content from unknown sources
    /// - Competitive multiplayer (anti-cheat)
    /// - Embedded scripts in player-submitted maps
    Strict,
}

/// Configuration for script sandboxing.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Overall sandbox level (None/Moderate/Strict)
    pub level: SandboxLevel,

    /// Allow file I/O operations (reading/writing files).
    /// 
    /// Only applies to Moderate level. Strict always disables this.
    /// Grants access to: `io.open`, `io.read`, `io.write`, etc.
    pub allow_file_io: bool,

    /// Allow network operations (not currently implemented by Lua standard library,
    /// but could apply to custom network APIs).
    ///
    /// Reserved for future use.
    pub allow_network: bool,

    /// Allow OS operations (execute commands, get environment variables, etc.).
    ///
    /// Only applies to Moderate level. Strict always disables this.
    /// Grants access to: `os.execute`, `os.getenv`, `os.exit`, etc.
    pub allow_os_access: bool,

    /// Maximum number of instructions before script is interrupted (0 = unlimited).
    ///
    /// Prevents infinite loops and CPU exhaustion. The counter is checked every
    /// 10,000 instructions for performance (0.1% overhead).
    ///
    /// Recommended values:
    /// - Interactive console: 100,000 (fast feedback)
    /// - Per-frame scripts: 1,000,000 (default)
    /// - Background tasks: 10,000,000 (long-running)
    pub instruction_limit: usize,

    /// Memory limit in bytes (0 = unlimited).
    ///
    /// Prevents memory bombs and OOM crashes. Applies to all Lua allocations:
    /// tables, strings, closures, etc. Does NOT include Rust userdata size.
    ///
    /// Recommended values:
    /// - Mobile devices: 10 MB
    /// - Desktop: 100 MB (default)
    /// - Server: 500 MB
    pub memory_limit: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 1_000_000,    // 1 million instructions (about 16ms on modern CPU)
            memory_limit: 100 * 1024 * 1024, // 100 MB
        }
    }
}

/// Applies sandbox restrictions to the Lua environment.
///
/// This is the main entry point for sandboxing. Call this after creating the Lua VM
/// but before executing any untrusted scripts.
///
/// # Order of Operations
/// 1. Apply resource limits (instruction count, memory)
/// 2. Apply API restrictions based on sandbox level
///
/// Resource limits apply to ALL levels (even None) to prevent accidental DOS.
pub fn apply_sandbox(lua: &Lua, config: &SandboxConfig) -> Result<()> {
    // Apply instruction and memory limits first (applies to all sandbox levels)
    apply_resource_limits(lua, config)?;

    match config.level {
        SandboxLevel::None => {
            info!("Sandbox disabled - scripts have full access");
            Ok(())
        }
        SandboxLevel::Moderate => apply_moderate_sandbox(lua, config),
        SandboxLevel::Strict => apply_strict_sandbox(lua, config),
    }
}

/// Applies moderate sandbox restrictions.
///
/// Removes dangerous operations while keeping most functionality intact.
/// Suitable for player mods with basic trust.
fn apply_moderate_sandbox(lua: &Lua, config: &SandboxConfig) -> Result<()> {
    debug!("Applying moderate sandbox");
    let globals = lua.globals();

    if !config.allow_file_io {
        remove_io_operations(&globals)?;
    }

    if !config.allow_os_access {
        remove_os_operations(&globals)?;
    }

    // Always remove dangerous dynamic loading functions
    remove_dangerous_operations(&globals)?;

    Ok(())
}

/// Applies strict sandbox restrictions.
///
/// Only safe, controlled operations are allowed. Suitable for untrusted
/// user-generated content.
fn apply_strict_sandbox(lua: &Lua, _config: &SandboxConfig) -> Result<()> {
    debug!("Applying strict sandbox");
    let globals = lua.globals();

    // Remove all potentially dangerous operations
    remove_io_operations(&globals)?;
    remove_os_operations(&globals)?;
    remove_dangerous_operations(&globals)?;
    remove_module_loading(&globals)?;

    Ok(())
}

/// Removes file I/O operations by setting the `io` library to nil.
///
/// This prevents scripts from:
/// - Reading files: `io.open("file.txt", "r")`
/// - Writing files: `io.open("file.txt", "w")`
/// - Listing directories: Not in standard `io`, but custom extensions
fn remove_io_operations(globals: &mlua::Table) -> Result<()> {
    globals
        .set("io", Value::Nil)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove io: {}", e))?;

    debug!("Removed io operations");
    Ok(())
}

/// Removes OS operations by setting the `os` library to nil.
///
/// This prevents scripts from:
/// - Executing commands: `os.execute("rm -rf /")`
/// - Exiting the program: `os.exit()`
/// - Getting environment: `os.getenv("SECRET_KEY")`
/// - Time manipulation: `os.time()` is safe but we remove whole lib for simplicity
fn remove_os_operations(globals: &mlua::Table) -> Result<()> {
    // Simply remove the os table entirely for security
    globals
        .set("os", Value::Nil)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove os: {}", e))?;

    debug!("Removed os operations");
    Ok(())
}

/// Removes dangerous dynamic code loading functions.
///
/// These functions allow scripts to bypass sandbox restrictions by:
/// - `dofile("malicious.lua")` - Execute arbitrary file
/// - `loadfile("evil.lua")` - Load and return arbitrary code
/// - `load("os.execute('rm -rf /')")` - Parse and execute strings
///
/// Even if `io` is disabled, these can load code from environment or other sources.
fn remove_dangerous_operations(globals: &mlua::Table) -> Result<()> {
    let dangerous = ["dofile", "loadfile", "load"];

    for func_name in &dangerous {
        globals
            .set(*func_name, Value::Nil)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove {}: {}", func_name, e))?;
    }

    debug!("Removed dangerous operations");
    Ok(())
}

/// Removes module loading capabilities (strict mode only).
///
/// Prevents scripts from:
/// - `require("socket")` - Load network library
/// - `require("ffi")` - Load FFI (if LuaJIT)
/// - Manipulating load paths via `package.path`, `package.cpath`
///
/// This is strict-mode only because internal scripts may need `require`
/// for legitimate module imports.
fn remove_module_loading(globals: &mlua::Table) -> Result<()> {
    globals
        .set("require", Value::Nil)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove require: {}", e))?;

    globals
        .set("package", Value::Nil)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove package: {}", e))?;

    debug!("Removed module loading");
    Ok(())
}

/// Applies resource limits (instruction count and memory) to prevent infinite loops and memory exhaustion.
///
/// # Instruction Limit Implementation
///
/// Lua's hook mechanism fires a callback at specified events. We use
/// `every_nth_instruction` to check the counter periodically:
///
/// - **Why 10,000 interval?** Balance between accuracy and performance.
///   - Too low (e.g., 100): High overhead, 10% slowdown
///   - Too high (e.g., 100,000): Scripts can run far past limit before catching
///   - 10,000: Good balance, ~0.1% overhead
///
/// - **Atomic counter**: Uses `AtomicUsize` for thread-safe increments
///   - Even though Lua is single-threaded, the hook closure may be called
///     from different contexts
///
/// - **Arc for closure capture**: The counter must be shared between the
///   setup code and the hook closure. Arc ensures it lives long enough.
///
/// # Memory Limit Implementation
///
/// Lua's built-in allocator can be limited. When exceeded, allocations fail
/// and Lua raises an error automatically. No manual checking needed.
///
/// # Performance Impact
///
/// - Instruction hooks: ~0.1% overhead (10k instruction interval)
/// - Memory tracking: Negligible (built into allocator)
fn apply_resource_limits(lua: &Lua, config: &SandboxConfig) -> Result<()> {
    // Apply instruction limit
    if config.instruction_limit > 0 {
        let instruction_limit = config.instruction_limit;
        let instruction_count = Arc::new(AtomicUsize::new(0));
        let count_clone = instruction_count.clone();

        lua.set_hook(
            mlua::HookTriggers {
                every_nth_instruction: Some(10000), // Check every 10k instructions for performance
                ..Default::default()
            },
            move |_lua, _debug| {
                // Atomically increment counter and check limit
                let count = count_clone.fetch_add(10000, Ordering::Relaxed) + 10000;
                if count >= instruction_limit {
                    Err(mlua::Error::RuntimeError(format!(
                        "Script exceeded instruction limit of {instruction_limit}"
                    )))
                } else {
                    Ok(())
                }
            },
        );

        info!("Set instruction limit to {instruction_limit}");
    }

    // Apply memory limit
    if config.memory_limit > 0 {
        let _ = lua.set_memory_limit(config.memory_limit);
        let memory_limit = config.memory_limit;
        info!("Set memory limit to {memory_limit} bytes");
    }

    Ok(())
}

/// Resets the instruction counter for the Lua VM.
///
/// # Why Reset is Needed
///
/// The instruction counter persists across script calls. Without resetting:
/// 1. Script A runs and uses 500k instructions
/// 2. Script B runs and uses 600k instructions
/// 3. Total: 1.1M instructions - exceeds limit of 1M
/// 4. Script B incorrectly fails even though it's under limit
///
/// By resetting before each script execution, each script gets a fresh budget.
///
/// # Implementation
///
/// We cannot simply reset a counter - we must remove the old hook (with its
/// old counter) and install a new hook with a fresh counter. Lua hooks are
/// closures that capture their environment.
///
/// # Usage Pattern
///
/// ```rust,no_run
/// # use praxis_scripting::*;
/// # let lua = mlua::Lua::new();
/// # let config = SandboxConfig::default();
/// // Before each script execution
/// reset_instruction_counter(&lua, &config)?;
/// lua.load("expensive_script()").exec()?;
/// 
/// // Counter is reset for next script
/// reset_instruction_counter(&lua, &config)?;
/// lua.load("another_script()").exec()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn reset_instruction_counter(lua: &Lua, config: &SandboxConfig) -> Result<()> {
    if config.instruction_limit > 0 {
        // Remove the old hook and set a new one with a fresh counter
        lua.remove_hook();

        let instruction_limit = config.instruction_limit;
        let instruction_count = Arc::new(AtomicUsize::new(0));
        let count_clone = instruction_count.clone();

        lua.set_hook(
            mlua::HookTriggers {
                every_nth_instruction: Some(10000),
                ..Default::default()
            },
            move |_lua, _debug| {
                let count = count_clone.fetch_add(10000, Ordering::Relaxed) + 10000;
                if count >= instruction_limit {
                    Err(mlua::Error::RuntimeError(format!(
                        "Script exceeded instruction limit of {instruction_limit}"
                    )))
                } else {
                    Ok(())
                }
            },
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moderate_sandbox() {
        let lua = Lua::new();
        let config = SandboxConfig {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 0,
            memory_limit: 0,
        };

        apply_sandbox(&lua, &config).unwrap();

        let globals = lua.globals();
        assert!(globals.get::<_, Value>("io").unwrap().is_nil());
    }

    #[test]
    fn test_strict_sandbox() {
        let lua = Lua::new();
        let config = SandboxConfig {
            level: SandboxLevel::Strict,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 0,
            memory_limit: 0,
        };

        apply_sandbox(&lua, &config).unwrap();

        let globals = lua.globals();
        assert!(globals.get::<_, Value>("io").unwrap().is_nil());
        assert!(globals.get::<_, Value>("require").unwrap().is_nil());
    }

    #[test]
    fn test_no_sandbox() {
        let lua = Lua::new();
        let config = SandboxConfig {
            level: SandboxLevel::None,
            allow_file_io: true,
            allow_network: true,
            allow_os_access: true,
            instruction_limit: 0,
            memory_limit: 0,
        };

        apply_sandbox(&lua, &config).unwrap();

        let globals = lua.globals();
        assert!(!globals.get::<_, Value>("print").unwrap().is_nil());
    }

    #[test]
    fn test_instruction_limit() {
        let lua = Lua::new();
        let config = SandboxConfig {
            level: SandboxLevel::None,
            allow_file_io: true,
            allow_network: true,
            allow_os_access: true,
            instruction_limit: 50000, // Very low limit
            memory_limit: 0,
        };

        apply_sandbox(&lua, &config).unwrap();

        // This should fail due to instruction limit
        let result = lua
            .load(
                r#"
            local sum = 0
            for i = 1, 1000000 do
                sum = sum + i
            end
            return sum
        "#,
            )
            .exec();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("instruction limit"));
    }

    #[test]
    fn test_memory_limit() {
        let lua = Lua::new();
        let config = SandboxConfig {
            level: SandboxLevel::None,
            allow_file_io: true,
            allow_network: true,
            allow_os_access: true,
            instruction_limit: 0,
            memory_limit: 1024 * 1024, // 1 MB limit
        };

        apply_sandbox(&lua, &config).unwrap();

        // Try to allocate a large table that should exceed the limit
        let result = lua
            .load(
                r#"
            local t = {}
            for i = 1, 1000000 do
                t[i] = string.rep("a", 1000)
            end
        "#,
            )
            .exec();

        assert!(result.is_err());
    }

    #[test]
    fn test_reset_instruction_counter() {
        let lua = Lua::new();
        let config = SandboxConfig {
            level: SandboxLevel::None,
            allow_file_io: true,
            allow_network: true,
            allow_os_access: true,
            instruction_limit: 100000,
            memory_limit: 0,
        };

        apply_sandbox(&lua, &config).unwrap();

        // Run a script that uses some instructions
        let result = lua
            .load(
                r#"
            local sum = 0
            for i = 1, 1000 do
                sum = sum + i
            end
        "#,
            )
            .exec();
        assert!(result.is_ok());

        // Reset the counter
        reset_instruction_counter(&lua, &config).unwrap();

        // Run another script - should work fine with reset counter
        let result = lua
            .load(
                r#"
            local sum = 0
            for i = 1, 1000 do
                sum = sum + i
            end
        "#,
            )
            .exec();
        assert!(result.is_ok());
    }
}
