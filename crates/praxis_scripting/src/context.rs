//! Scripting context that manages the Lua VM and script execution.
//!
//! # Core Architecture
//!
//! ## Lua VM Management
//! The [`ScriptingContext`] owns a Lua VM (via `Arc<Lua>`) and manages its lifecycle.
//! Each context has its own isolated Lua environment with:
//! - Global environment (variables, functions)
//! - Registered APIs (math, engine, console, ECS)
//! - Sandbox restrictions (if configured)
//! - Performance monitoring hooks
//!
//! ## ECS World Access Pattern
//!
//! The ECS World requires **exclusive mutable access** (`&mut World`) for component
//! manipulation. This conflicts with Lua's ownership model where the VM must own
//! all data or use Lua-managed references. Our solution uses a **thread-local
//! pointer pattern**:
//!
//! 1. **Temporary injection**: The World is NOT stored in the Lua VM or context
//! 2. **Scoped access**: `with_world()` temporarily sets a thread-local pointer
//! 3. **Function closures**: Lua functions created with `lua.create_function()` can
//!    access the World via `with_world_raw()` which dereferences the pointer
//! 4. **Cleanup**: The pointer is cleared after script execution completes
//!
//! ### Why Not Store World in Lua?
//! - Lua's GC would not understand Rust's borrow checker rules
//! - Multiple Lua values could hold World references, violating exclusivity
//! - Lua userdata cannot hold mutable references safely across yields/resumes
//!
//! ### Safety Invariants
//! - The World pointer is only valid during `with_world()` execution
//! - Scripts must not store World references in global variables
//! - All World access must go through the API, not direct pointer access
//!
//! ## Hot-Reload Implementation
//!
//! Hot-reload watches script files and automatically reloads them on changes:
//!
//! 1. **File watching**: Uses `notify` crate to monitor filesystem events
//! 2. **Event polling**: `process_hot_reload()` checks for file modifications
//! 3. **Script reloading**: Modified scripts are re-executed in the same Lua VM
//! 4. **State preservation**: Global variables persist unless overwritten
//!
//! ### Hot-Reload Caveats
//! - Functions defined in reloaded scripts replace old versions
//! - Closures capturing old values may still reference stale data
//! - Active coroutines/threads may behave unpredictably after reload
//! - Best practice: Keep scripts stateless or reinitialize after reload

use crate::bindings;
use crate::hot_reload::HotReloadWatcher;
use crate::performance::ScriptPerformanceMonitor;
use crate::sandbox::SandboxConfig;
use mlua::Lua;
use parking_lot::RwLock;
use praxis_utils::{debug, error, info, warn, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for the scripting system.
#[derive(Debug, Clone)]
pub struct ScriptingConfig {
    /// Sandbox configuration for security
    pub sandbox: SandboxConfig,

    /// Whether to enable performance monitoring
    pub enable_performance_monitoring: bool,

    /// Maximum script execution time in milliseconds before warning
    pub max_execution_time_ms: u64,
}

impl Default for ScriptingConfig {
    fn default() -> Self {
        Self {
            sandbox: SandboxConfig::default(),
            enable_performance_monitoring: true,
            max_execution_time_ms: 16, // One frame at 60 FPS
        }
    }
}

/// Main scripting context that manages the Lua VM and script execution.
///
/// # Thread Safety
/// The Lua VM is NOT thread-safe (mlua::Lua is !Send + !Sync). Each context
/// must be used from a single thread. Use `Arc<Lua>` to share VM handles
/// across async tasks on the same thread.
pub struct ScriptingContext {
    /// The Lua VM instance. Arc is used for sharing with closures and performance monitor.
    /// Note: Lua is !Send + !Sync, so this can only be used on one thread.
    lua: Arc<Lua>,

    /// Configuration including sandbox settings
    config: ScriptingConfig,

    /// Map of script names to their file paths for hot-reload tracking
    loaded_scripts: HashMap<String, PathBuf>,

    /// Optional hot-reload watcher for automatic script reloading
    hot_reload_watcher: Option<Arc<RwLock<HotReloadWatcher>>>,

    /// Optional performance monitor for tracking execution time
    performance_monitor: Option<Arc<ScriptPerformanceMonitor>>,
}

impl ScriptingContext {
    /// Creates a new scripting context with the given configuration.
    ///
    /// This initializes the Lua VM and sets up:
    /// - Standard libraries (math, string, table, etc.)
    /// - Engine-specific APIs (math helpers, console commands)
    /// - Sandbox restrictions (if enabled)
    /// - Performance monitoring hooks (if enabled)
    pub fn new(config: ScriptingConfig) -> Result<Self> {
        info!("Creating scripting context");

        // Create a new Lua VM with standard libraries
        let lua = Lua::new();

        // Set up performance monitoring before any scripts run
        let performance_monitor = if config.enable_performance_monitoring {
            Some(Arc::new(ScriptPerformanceMonitor::new(
                config.max_execution_time_ms,
            )))
        } else {
            None
        };

        let mut context = Self {
            // SAFETY: Arc<Lua> is safe here despite Lua being !Send + !Sync because:
            // 1. Single-threaded access: The ScriptingContext itself is !Send + !Sync
            //    (inherited from the Lua field), ensuring it can never be moved to
            //    another thread or accessed concurrently.
            // 2. Arc is only used for shared ownership within the same thread:
            //    - Closures created via lua.create_function() capture Arc<Lua>
            //    - Performance monitor holds Arc<Lua> for instrumentation hooks
            //    - All clones stay within the same thread that created the context
            // 3. No cross-thread sharing: The Arc reference count is only incremented
            //    and decremented on the thread that owns the ScriptingContext.
            // 4. Lua VM guarantee: mlua enforces that Lua cannot be used across threads,
            //    so even if we tried to send Arc<Lua> elsewhere, mlua's API would prevent
            //    unsafe access via compile-time !Send bounds.
            #[allow(clippy::arc_with_non_send_sync)]
            lua: Arc::new(lua),
            config,
            loaded_scripts: HashMap::new(),
            hot_reload_watcher: None,
            performance_monitor,
        };

        // Register all engine APIs and apply security restrictions
        context.setup_environment()?;

        Ok(context)
    }

    /// Sets up the Lua environment with engine APIs and security restrictions.
    ///
    /// This is called during initialization and must be done before any scripts
    /// are loaded. The order matters:
    /// 1. Register APIs (adds functions to globals)
    /// 2. Apply sandbox (removes dangerous functions)
    fn setup_environment(&mut self) -> Result<()> {
        debug!("Setting up Lua environment");

        // Register custom math utilities (Vec3, etc.)
        bindings::register_math_api(&self.lua)?;

        // Register engine-specific APIs (logging, timing, etc.)
        bindings::register_engine_api(&self.lua)?;

        // Register console commands for REPL/debugging
        bindings::register_console_commands(&self.lua)?;

        // Apply sandbox restrictions last (removes dangerous globals)
        crate::sandbox::apply_sandbox(&self.lua, &self.config.sandbox)?;

        Ok(())
    }

    /// Loads a Lua script from a file.
    ///
    /// The script is executed immediately in the context's Lua VM. Any global
    /// variables or functions defined in the script become available for future
    /// calls to `call_function()`.
    ///
    /// # Arguments
    /// - `name`: Identifier for this script (used for hot-reload tracking)
    /// - `path`: Filesystem path to the .lua file
    pub fn load_script(&mut self, name: &str, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Loading script '{}' from {:?}", name, path);

        let source = std::fs::read_to_string(path)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read script file: {}", e))?;

        // Execute the script in the Lua VM with the given name (for error messages)
        self.lua
            .load(&source)
            .set_name(name)
            .exec()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute script: {}", e))?;

        // Track the script for hot-reload
        self.loaded_scripts
            .insert(name.to_string(), path.to_path_buf());

        Ok(())
    }

    /// Loads Lua code from a string.
    ///
    /// Useful for executing dynamically generated code or inline scripts
    /// without requiring a file on disk.
    pub fn load_string(&mut self, name: &str, source: &str) -> Result<()> {
        debug!("Loading script '{}' from string", name);

        self.lua
            .load(source)
            .set_name(name)
            .exec()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute script: {}", e))?;

        Ok(())
    }

    /// Reloads a previously loaded script.
    ///
    /// This re-reads the script file and re-executes it in the same Lua VM.
    /// Global variables from the old version persist unless overwritten.
    pub fn reload_script(&mut self, name: &str) -> Result<()> {
        let path = self
            .loaded_scripts
            .get(name)
            .ok_or_else(|| praxis_utils::eyre::eyre!("Script '{}' not found", name))?
            .clone();

        info!("Reloading script '{}'", name);
        self.load_script(name, path)
    }

    /// Calls a global Lua function.
    ///
    /// # Type Parameters
    /// - `A`: Argument types (automatically converted to Lua values)
    /// - `R`: Return type (automatically converted from Lua values)
    ///
    /// # Performance Tracking
    /// If performance monitoring is enabled, execution time is recorded
    /// and warnings are logged for slow functions.
    pub fn call_function<'a, A, R>(
        &'a self,
        script_name: &str,
        function_name: &str,
        args: A,
    ) -> Result<R>
    where
        A: mlua::IntoLuaMulti<'a>,
        R: mlua::FromLuaMulti<'a>,
    {
        let start = if self.performance_monitor.is_some() {
            Some(std::time::Instant::now())
        } else {
            None
        };

        // Get the function from global scope
        let globals = self.lua.globals();
        let function: mlua::Function = globals.get(function_name).map_err(|e| {
            praxis_utils::eyre::eyre!("Function '{}' not found: {}", function_name, e)
        })?;

        // Call the function with automatic argument/return type conversion
        let result = function.call(args).map_err(|e| {
            praxis_utils::eyre::eyre!("Error calling function '{}': {}", function_name, e)
        })?;

        // Record performance metrics
        if let (Some(start), Some(ref monitor)) = (start, &self.performance_monitor) {
            let elapsed = start.elapsed();
            monitor.record_execution(script_name, function_name, elapsed);
        }

        Ok(result)
    }

    /// Sets a global variable in the Lua environment.
    pub fn set_global<'a, V>(&'a self, name: &str, value: V) -> Result<()>
    where
        V: mlua::IntoLua<'a>,
    {
        let globals = self.lua.globals();
        globals
            .set(name, value)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to set global '{}': {}", name, e))?;
        Ok(())
    }

    /// Gets a global variable from the Lua environment.
    pub fn get_global<'a, V>(&'a self, name: &str) -> Result<V>
    where
        V: mlua::FromLua<'a>,
    {
        let globals = self.lua.globals();
        globals
            .get(name)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to get global '{}': {}", name, e))
    }

    /// Gets a reference to the Lua VM for direct access.
    ///
    /// Use this for advanced scenarios where you need to directly interact
    /// with the Lua VM API.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Enables hot-reload for scripts in the given directory.
    ///
    /// This starts a filesystem watcher that monitors the directory for changes.
    /// Call `process_hot_reload()` regularly (e.g., each frame) to apply changes.
    ///
    /// # Hot-Reload Workflow
    /// 1. Developer modifies a .lua file
    /// 2. Filesystem watcher detects the change
    /// 3. Event is queued in the watcher
    /// 4. `process_hot_reload()` processes the event
    /// 5. Script is reloaded via `reload_script()`
    /// 6. New code is active immediately
    pub fn enable_hot_reload(&mut self, watch_path: impl AsRef<Path>) -> Result<()> {
        info!("Enabling hot-reload for {:?}", watch_path.as_ref());

        let watcher = HotReloadWatcher::new(watch_path)?;
        self.hot_reload_watcher = Some(Arc::new(RwLock::new(watcher)));

        Ok(())
    }

    /// Processes any pending hot-reload events.
    ///
    /// Call this regularly (e.g., once per frame) to apply script changes.
    /// This is a non-blocking poll - if no events are pending, it returns immediately.
    pub fn process_hot_reload(&mut self) -> Result<()> {
        if let Some(ref watcher) = self.hot_reload_watcher {
            let events = {
                let mut w = watcher.write();
                w.poll_events()
            };

            for event in events {
                match event {
                    crate::hot_reload::ScriptEvent::Modified(path) => {
                        self.handle_script_modified(&path)?;
                    }
                    crate::hot_reload::ScriptEvent::Removed(path) => {
                        self.handle_script_removed(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_script_modified(&mut self, path: &Path) -> Result<()> {
        info!("Script modified: {:?}", path);

        // Find all scripts that match this path and reload them
        for (name, script_path) in &self.loaded_scripts.clone() {
            if script_path == path {
                if let Err(e) = self.reload_script(name) {
                    error!("Failed to reload script '{}': {}", name, e);
                } else {
                    info!("Successfully reloaded script '{}'", name);
                }
            }
        }

        Ok(())
    }

    fn handle_script_removed(&mut self, path: &Path) -> Result<()> {
        warn!("Script removed: {:?}", path);

        // Untrack removed scripts but keep their code in the Lua VM
        // (functions may still be referenced)
        let names: Vec<String> = self
            .loaded_scripts
            .iter()
            .filter(|(_, p)| *p == path)
            .map(|(n, _)| n.clone())
            .collect();

        for name in names {
            self.loaded_scripts.remove(&name);
            warn!("Unloaded script '{}'", name);
        }

        Ok(())
    }

    /// Gets the performance monitor if enabled.
    pub fn performance_monitor(&self) -> Option<&ScriptPerformanceMonitor> {
        self.performance_monitor.as_ref().map(|m| m.as_ref())
    }

    /// Executes a closure with the ECS World context set up for Lua scripts.
    ///
    /// # Exclusive World Access Pattern
    ///
    /// This method is the gateway for ECS integration. It:
    /// 1. Stores a raw pointer to the World in thread-local storage
    /// 2. Executes the provided closure (which may run Lua code)
    /// 3. Clears the pointer when done
    ///
    /// During step 2, Lua functions can access the World via `with_world_raw()`,
    /// which dereferences the thread-local pointer. This allows Lua code to
    /// query and modify ECS components as if it had direct World access.
    ///
    /// # Safety
    /// This uses unsafe code internally to dereference the World pointer.
    /// Safety is guaranteed by:
    /// - The pointer is set immediately before use and cleared after
    /// - Only one script can execute at a time (single-threaded Lua VM)
    /// - The World reference is valid for the duration of the closure
    /// - Scripts cannot store the World reference in Lua globals
    ///
    /// # Arguments
    ///
    /// * `world` - Mutable reference to the ECS World
    /// * `f` - Closure that takes the Lua VM and returns a Result
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use praxis_scripting::{ScriptingContext, ScriptingConfig};
    /// # use praxis_ecs::World;
    /// # let mut context = ScriptingContext::new(ScriptingConfig::default()).unwrap();
    /// # let mut world = World::new();
    /// context.with_world(&mut world, |lua| {
    ///     lua.globals().get::<_, mlua::Function>("on_update")?
    ///         .call::<_, ()>(0.016)?;
    ///     Ok(())
    /// }).unwrap();
    /// ```
    pub fn with_world<F, R>(&self, world: &mut praxis_ecs::World, f: F) -> Result<R>
    where
        F: FnOnce(&Lua) -> Result<R>,
    {
        // Set the world pointer in thread-local storage
        crate::bindings::ecs_api::set_world_context(&self.lua, world)?;

        // Execute the closure (may call Lua functions that access the world)
        let result = f(&self.lua);

        // Always clear the world pointer, even if an error occurred
        crate::bindings::ecs_api::clear_world_context(&self.lua)?;

        result
    }

    /// Evaluates Lua code interactively (REPL mode).
    ///
    /// # REPL Evaluation Pattern
    ///
    /// This method implements a common REPL pattern used by interactive Lua shells:
    ///
    /// 1. **Try as statement**: First, execute the code as-is
    ///    - Handles variable assignments: `x = 5`
    ///    - Handles function calls: `print("hello")`
    ///    - Returns empty string if no value is returned
    ///
    /// 2. **Try as expression**: If the statement fails, wrap with `return`
    ///    - Handles expressions: `2 + 2` becomes `return 2 + 2`
    ///    - Handles function calls that return values: `math.sqrt(16)`
    ///    - Returns the formatted result value
    ///
    /// 3. **Error handling**: If both fail, return the original error
    ///    - Syntax errors are reported from the statement attempt
    ///    - Runtime errors are propagated to the caller
    ///
    /// This two-phase approach allows the REPL to "do what you mean" for both
    /// statements (which don't return values) and expressions (which do).
    ///
    /// # Multi-Value Returns
    /// Lua functions can return multiple values. These are collected and formatted
    /// as a comma-separated string: `1, 2, 3`
    ///
    /// # Arguments
    ///
    /// * `code` - Lua code string to evaluate
    ///
    /// # Returns
    ///
    /// A formatted string with the evaluation result or error message.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use praxis_scripting::{ScriptingContext, ScriptingConfig};
    /// # let context = ScriptingContext::new(ScriptingConfig::default()).unwrap();
    /// let result = context.eval_interactive("2 + 2").unwrap();
    /// assert_eq!(result, "4");
    ///
    /// let result = context.eval_interactive("x = 5").unwrap();
    /// assert!(result.is_empty()); // Statements don't return values
    /// ```
    pub fn eval_interactive(&self, code: &str) -> Result<String> {
        let start = if self.performance_monitor.is_some() {
            Some(std::time::Instant::now())
        } else {
            None
        };

        // Phase 1: Try to evaluate as a statement first
        let result = self.lua.load(code).eval::<mlua::MultiValue>();

        let output = match result {
            Ok(values) => {
                // Format the return values
                if values.is_empty() {
                    String::new()
                } else {
                    let formatted: Vec<String> =
                        values.iter().map(|v| format_lua_value(v)).collect();
                    formatted.join(", ")
                }
            }
            Err(err) => {
                // Phase 2: If it failed as a statement, try as an expression by wrapping with "return"
                let expr_code = format!("return {code}");
                match self.lua.load(&expr_code).eval::<mlua::MultiValue>() {
                    Ok(values) => {
                        if values.is_empty() {
                            String::new()
                        } else {
                            let formatted: Vec<String> =
                                values.iter().map(|v| format_lua_value(v)).collect();
                            formatted.join(", ")
                        }
                    }
                    Err(_) => {
                        // Return the original error if both attempts failed
                        return Err(praxis_utils::eyre::eyre!("Lua error: {}", err));
                    }
                }
            }
        };

        if let (Some(start), Some(ref monitor)) = (start, &self.performance_monitor) {
            let elapsed = start.elapsed();
            monitor.record_execution("interactive", "eval", elapsed);
        }

        Ok(output)
    }

    /// Evaluates Lua code interactively with ECS World context.
    ///
    /// Similar to `eval_interactive`, but provides access to the ECS World
    /// through the `world` global table and console commands.
    ///
    /// This is the primary method for REPL/console usage in the editor or
    /// during gameplay debugging.
    ///
    /// # Arguments
    ///
    /// * `code` - Lua code string to evaluate
    /// * `world` - Mutable reference to the ECS World
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use praxis_scripting::{ScriptingContext, ScriptingConfig};
    /// # use praxis_ecs::World;
    /// # let context = ScriptingContext::new(ScriptingConfig::default()).unwrap();
    /// # let mut world = World::new();
    /// let result = context.eval_interactive_with_world("world.spawn()", &mut world).unwrap();
    /// ```
    pub fn eval_interactive_with_world(
        &self,
        code: &str,
        world: &mut praxis_ecs::World,
    ) -> Result<String> {
        self.with_world(world, |_| self.eval_interactive(code))
    }
}

/// Formats a Lua value for display in the REPL.
///
/// This provides human-readable representations of Lua values with
/// special handling for:
/// - Numbers: Clean formatting (no unnecessary decimals)
/// - Strings: Quoted for clarity
/// - Complex types: Type name only (table, function, etc.)
fn format_lua_value(value: &mlua::Value) -> String {
    match value {
        mlua::Value::Nil => "nil".to_string(),
        mlua::Value::Boolean(b) => b.to_string(),
        mlua::Value::Integer(i) => i.to_string(),
        mlua::Value::Number(n) => {
            // Format numbers nicely
            if n.fract() == 0.0 && n.abs() < 1e10 {
                format!("{n:.0}")
            } else {
                format!("{n}")
            }
        }
        mlua::Value::String(s) => {
            // Show strings with quotes for clarity
            format!("\"{}\"", s.to_str().unwrap_or("<invalid utf8>"))
        }
        mlua::Value::Table(_) => "table".to_string(),
        mlua::Value::Function(_) => "function".to_string(),
        mlua::Value::Thread(_) => "thread".to_string(),
        mlua::Value::UserData(_) => "userdata".to_string(),
        mlua::Value::LightUserData(_) => "lightuserdata".to_string(),
        mlua::Value::Error(e) => format!("error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_context() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config);
        assert!(context.is_ok());
    }

    #[test]
    fn test_load_string() {
        let config = ScriptingConfig::default();
        let mut context = ScriptingContext::new(config).unwrap();

        let result = context.load_string("test", "x = 42");
        assert!(result.is_ok());

        let value: i32 = context.get_global("x").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_call_function() {
        let config = ScriptingConfig::default();
        let mut context = ScriptingContext::new(config).unwrap();

        context
            .load_string("test", "function add(a, b) return a + b end")
            .unwrap();

        let result: i32 = context.call_function("test", "add", (5, 3)).unwrap();
        assert_eq!(result, 8);
    }

    #[test]
    fn test_set_and_get_global() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();

        context.set_global("test_value", 123).unwrap();
        let value: i32 = context.get_global("test_value").unwrap();
        assert_eq!(value, 123);
    }

    #[test]
    fn test_eval_interactive_expression() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();

        // Test simple expression
        let result = context.eval_interactive("2 + 2").unwrap();
        assert_eq!(result, "4");

        // Test with decimal
        let result = context.eval_interactive("math.sqrt(16)").unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_eval_interactive_statement() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();

        // Test assignment statement (no return value)
        let result = context.eval_interactive("x = 42").unwrap();
        assert_eq!(result, "");

        // Verify the value was set
        let value: i32 = context.get_global("x").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_eval_interactive_with_world() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();
        let mut world = praxis_ecs::World::new();

        // Spawn some entities
        world.spawn((
            praxis_ecs::Name::new("TestEntity"),
            praxis_ecs::Transform::default(),
            praxis_ecs::GlobalTransform::default(),
        ));

        // Test console command with world context
        let result = context
            .eval_interactive_with_world("console.entity_count()", &mut world)
            .unwrap();
        assert_eq!(result, "1");
    }

    #[test]
    fn test_eval_interactive_error() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();

        // Test invalid syntax
        let result = context.eval_interactive("invalid lua syntax )))");
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_interactive_multi_value() {
        let config = ScriptingConfig::default();
        let context = ScriptingContext::new(config).unwrap();

        // Test multiple return values
        let result = context.eval_interactive("return 1, 2, 3").unwrap();
        assert_eq!(result, "1, 2, 3");
    }
}
