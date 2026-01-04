//! Sandbox configuration and enforcement for script security.

use mlua::{Lua, Value};
use praxis_utils::{debug, info, Result};

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
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            level: SandboxLevel::Moderate,
            allow_file_io: false,
            allow_network: false,
            allow_os_access: false,
        }
    }
}

/// Applies sandbox restrictions to the Lua environment.
pub fn apply_sandbox(lua: &Lua, config: &SandboxConfig) -> Result<()> {
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
        };
        
        apply_sandbox(&lua, &config).unwrap();
        
        let globals = lua.globals();
        assert!(!globals.get::<_, Value>("print").unwrap().is_nil());
    }
}
