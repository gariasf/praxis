//! Sandbox configuration and enforcement for script security.

use mlua::{Lua, Value};
use praxis_utils::{debug, info, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Security level for sandboxing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    /// No restrictions - full access to all Lua features
    None,

    /// Moderate restrictions - disables dangerous features but allows most operations
    Moderate,

    /// Strict restrictions - only allows safe operations
    Strict,
}

/// Configuration for script sandboxing.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Overall sandbox level
    pub level: SandboxLevel,

    /// Allow file I/O operations
    pub allow_file_io: bool,

    /// Allow network operations
    pub allow_network: bool,

    /// Allow OS operations (execute, exit, etc.)
    pub allow_os_access: bool,

    /// Maximum number of instructions before script is interrupted (0 = unlimited)
    pub instruction_limit: usize,

    /// Memory limit in bytes (0 = unlimited)
    pub memory_limit: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
            instruction_limit: 1_000_000, // 1 million instructions
            memory_limit: 100 * 1024 * 1024, // 100 MB
        }
    }
}

/// Applies sandbox restrictions to the Lua environment.
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

fn apply_moderate_sandbox(lua: &Lua, config: &SandboxConfig) -> Result<()> {
    debug!("Applying moderate sandbox");
    let globals = lua.globals();

    if !config.allow_file_io {
        remove_io_operations(&globals)?;
    }

    if !config.allow_os_access {
        remove_os_operations(&globals)?;
    }

    remove_dangerous_operations(&globals)?;

    Ok(())
}

fn apply_strict_sandbox(lua: &Lua, _config: &SandboxConfig) -> Result<()> {
    debug!("Applying strict sandbox");
    let globals = lua.globals();

    remove_io_operations(&globals)?;
    remove_os_operations(&globals)?;
    remove_dangerous_operations(&globals)?;
    remove_module_loading(&globals)?;

    Ok(())
}

fn remove_io_operations(globals: &mlua::Table) -> Result<()> {
    globals
        .set("io", Value::Nil)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove io: {}", e))?;

    debug!("Removed io operations");
    Ok(())
}

fn remove_os_operations(globals: &mlua::Table) -> Result<()> {
    // Simply remove the os table entirely for security
    globals
        .set("os", Value::Nil)
        .map_err(|e| praxis_utils::eyre::eyre!("Failed to remove os: {}", e))?;

    debug!("Removed os operations");
    Ok(())
}

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
/// This should be called at the start of each script execution to ensure
/// the instruction count is reset properly.
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
        let result = lua.load(r#"
            local sum = 0
            for i = 1, 1000000 do
                sum = sum + i
            end
            return sum
        "#).exec();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("instruction limit"));
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
        let result = lua.load(r#"
            local t = {}
            for i = 1, 1000000 do
                t[i] = string.rep("a", 1000)
            end
        "#).exec();

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
        let result = lua.load(r#"
            local sum = 0
            for i = 1, 1000 do
                sum = sum + i
            end
        "#).exec();
        assert!(result.is_ok());

        // Reset the counter
        reset_instruction_counter(&lua, &config).unwrap();

        // Run another script - should work fine with reset counter
        let result = lua.load(r#"
            local sum = 0
            for i = 1, 1000 do
                sum = sum + i
            end
        "#).exec();
        assert!(result.is_ok());
    }
}
