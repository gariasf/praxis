//! Scripting system for the Praxis engine using Lua.
//!
//! This crate provides a comprehensive scripting layer that allows runtime game logic
//! to be written in Lua, with full access to the ECS World and engine APIs.
//!
//! # Features
//!
//! - **ECS Integration**: Access and modify entities, components, and resources from Lua
//! - **Hot-Reload**: Automatically reload scripts when files change for rapid iteration
//! - **Sandboxing**: Restrict access to dangerous operations for security
//! - **Performance Monitoring**: Track script execution time to detect expensive operations
//! - **Type-Safe Bindings**: Expose Rust APIs to Lua with type safety
//! - **REPL Support**: Interactive console with automatic expression evaluation
//! - **Console Commands**: Built-in commands for ECS introspection and runtime modification
//!
//! # Lua-Rust FFI Architecture
//!
//! This crate uses the `mlua` library to provide a safe Rust-Lua FFI layer. Key patterns:
//!
//! ## 1. UserData for Complex Types
//! Rust types are exposed to Lua through the [`UserData`](mlua::UserData) trait, which provides:
//! - **Type safety**: Rust types retain their identity and can be type-checked
//! - **Memory safety**: References are validated, preventing use-after-free
//! - **Method exposure**: Rust methods can be called from Lua with automatic marshalling
//!
//! Example: `LuaEntity` wraps `Entity` and exposes it to Lua scripts.
//!
//! ## 2. Function Closures
//! Lua functions are created from Rust closures using `lua.create_function()`:
//! - Closures can capture Rust context (via `move` semantics)
//! - Arguments are automatically converted from Lua types to Rust types
//! - Return values are automatically converted back to Lua
//! - Errors are propagated as Lua runtime errors
//!
//! ## 3. Thread-Local World Access
//! The ECS World cannot be safely shared across threads or stored in Lua's GC'd memory.
//! Instead, we use a **thread-local raw pointer** pattern:
//! - `WORLD_CONTEXT: RefCell<Option<*mut World>>` stores the current world pointer
//! - `set_world_context()` establishes the world for the current script execution
//! - `with_world()` provides safe access to the world within Lua functions
//! - `clear_world_context()` removes the world reference after execution
//!
//! This ensures:
//! - **Exclusive access**: Only one script can access the World at a time
//! - **Memory safety**: The pointer is cleared after use, preventing dangling references
//! - **No GC issues**: The World is not owned by Lua's garbage collector
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_scripting::{ScriptingContext, ScriptingConfig};
//!
//! let config = ScriptingConfig::default();
//! let mut context = ScriptingContext::new(config).unwrap();
//!
//! context.load_script("game_logic", "scripts/game.lua").unwrap();
//! context.call_function::<_, ()>("game_logic", "update", 0.016).unwrap();
//! ```
//!
//! # Interactive REPL
//!
//! ```rust,no_run
//! use praxis_scripting::ScriptingContext;
//! # let context = ScriptingContext::new(Default::default()).unwrap();
//!
//! // Evaluate expressions interactively
//! let result = context.eval_interactive("2 + 2").unwrap();
//! assert_eq!(result, "4");
//!
//! // With ECS World access
//! # let mut world = praxis_ecs::World::new();
//! let result = context.eval_interactive_with_world(
//!     "console.list_entities()",
//!     &mut world
//! ).unwrap();
//! ```
//!
//! # Security Considerations
//!
//! Scripts can be sandboxed to prevent malicious or buggy code from:
//! - Accessing the filesystem (file I/O operations)
//! - Making network requests
//! - Executing OS commands
//! - Loading arbitrary code dynamically
//! - Consuming excessive CPU (instruction limits)
//! - Consuming excessive memory (memory limits)
//!
//! See [`SandboxConfig`] and [`SandboxLevel`] for configuration options.

mod bindings;
mod context;
mod hot_reload;
mod performance;
mod sandbox;
mod script_component;
mod systems;

pub use bindings::console_commands;
pub use context::{ScriptingConfig, ScriptingContext};
pub use hot_reload::{HotReloadWatcher, ScriptEvent};
pub use performance::{ScriptPerformanceMonitor, ScriptStats};
pub use sandbox::{reset_instruction_counter, SandboxConfig, SandboxLevel};
pub use script_component::{ScriptComponent, ScriptInstance};
pub use systems::{
    script_hot_reload_system, script_initialization_system, script_start_system,
    script_update_system, ScriptInitialized, ScriptStarted, ScriptingResource,
};

// Re-export mlua types for use in examples and user code
pub use mlua;

use praxis_utils::{info, Result};

/// Initializes the scripting system.
pub fn init() -> Result<()> {
    info!("Initializing scripting system");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let result = init();
        assert!(result.is_ok());
    }
}
