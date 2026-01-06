//! Scripting context that manages the Lua VM and script execution.

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
            max_execution_time_ms: 16,
        }
    }
}

/// Main scripting context that manages the Lua VM and script execution.
pub struct ScriptingContext {
    lua: Arc<Lua>,
    config: ScriptingConfig,
    loaded_scripts: HashMap<String, PathBuf>,
    hot_reload_watcher: Option<Arc<RwLock<HotReloadWatcher>>>,
    performance_monitor: Option<Arc<ScriptPerformanceMonitor>>,
}

impl ScriptingContext {
    /// Creates a new scripting context with the given configuration.
    pub fn new(config: ScriptingConfig) -> Result<Self> {
        info!("Creating scripting context");

        let lua = Lua::new();

        let performance_monitor = if config.enable_performance_monitoring {
            Some(Arc::new(ScriptPerformanceMonitor::new(
                config.max_execution_time_ms,
            )))
        } else {
            None
        };

        let mut context = Self {
            #[allow(clippy::arc_with_non_send_sync)]
            lua: Arc::new(lua),
            config,
            loaded_scripts: HashMap::new(),
            hot_reload_watcher: None,
            performance_monitor,
        };

        context.setup_environment()?;

        Ok(context)
    }

    fn setup_environment(&mut self) -> Result<()> {
        debug!("Setting up Lua environment");

        bindings::register_math_api(&self.lua)?;
        bindings::register_engine_api(&self.lua)?;
        bindings::register_console_commands(&self.lua)?;

        crate::sandbox::apply_sandbox(&self.lua, &self.config.sandbox)?;

        Ok(())
    }

    /// Loads a Lua script from a file.
    pub fn load_script(&mut self, name: &str, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Loading script '{}' from {:?}", name, path);

        let source = std::fs::read_to_string(path)
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to read script file: {}", e))?;

        self.lua
            .load(&source)
            .set_name(name)
            .exec()
            .map_err(|e| praxis_utils::eyre::eyre!("Failed to execute script: {}", e))?;

        self.loaded_scripts
            .insert(name.to_string(), path.to_path_buf());

        Ok(())
    }

    /// Loads Lua code from a string.
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

        let globals = self.lua.globals();
        let function: mlua::Function = globals.get(function_name).map_err(|e| {
            praxis_utils::eyre::eyre!("Function '{}' not found: {}", function_name, e)
        })?;

        let result = function.call(args).map_err(|e| {
            praxis_utils::eyre::eyre!("Error calling function '{}': {}", function_name, e)
        })?;

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
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Enables hot-reload for scripts in the given directory.
    pub fn enable_hot_reload(&mut self, watch_path: impl AsRef<Path>) -> Result<()> {
        info!("Enabling hot-reload for {:?}", watch_path.as_ref());

        let watcher = HotReloadWatcher::new(watch_path)?;
        self.hot_reload_watcher = Some(Arc::new(RwLock::new(watcher)));

        Ok(())
    }

    /// Processes any pending hot-reload events.
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
    /// This allows scripts to access the world via the `world` global table.
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
        crate::bindings::ecs_api::set_world_context(&self.lua, world)?;
        let result = f(&self.lua);
        crate::bindings::ecs_api::clear_world_context(&self.lua)?;
        result
    }

    /// Evaluates Lua code interactively (REPL mode).
    ///
    /// This method is designed for console/REPL usage and provides:
    /// - Automatic return value printing
    /// - Expression evaluation (if statement fails, tries as expression)
    /// - Multi-value return support
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

        // Try to evaluate as a statement first
        let result = self.lua.load(code).eval::<mlua::MultiValue>();

        let output = match result {
            Ok(values) => {
                // Format the return values
                if values.is_empty() {
                    String::new()
                } else {
                    let formatted: Vec<String> = values
                        .iter()
                        .map(|v| format_lua_value(v))
                        .collect();
                    formatted.join(", ")
                }
            }
            Err(err) => {
                // If it failed as a statement, try as an expression by wrapping with "return"
                let expr_code = format!("return {code}");
                match self.lua.load(&expr_code).eval::<mlua::MultiValue>() {
                    Ok(values) => {
                        if values.is_empty() {
                            String::new()
                        } else {
                            let formatted: Vec<String> = values
                                .iter()
                                .map(|v| format_lua_value(v))
                                .collect();
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
    /// through the `world` global table.
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
    pub fn eval_interactive_with_world(&self, code: &str, world: &mut praxis_ecs::World) -> Result<String> {
        self.with_world(world, |_| self.eval_interactive(code))
    }
}

/// Formats a Lua value for display in the REPL.
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
