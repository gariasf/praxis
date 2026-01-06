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
