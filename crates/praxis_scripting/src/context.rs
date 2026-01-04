//! Scripting context that manages the Lua VM and script execution.

use crate::bindings;
use crate::hot_reload::HotReloadWatcher;
use crate::performance::ScriptPerformanceMonitor;
use crate::sandbox::{SandboxConfig, SandboxLevel};
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
    
    /// Memory limit for Lua VM in bytes (0 = unlimited)
    pub memory_limit: usize,
}

impl Default for ScriptingConfig {
    fn default() -> Self {
        Self {
            sandbox: SandboxConfig {
                level: SandboxLevel::Moderate,
                allow_file_io: false,
                allow_network: false,
                allow_os_access: false,
            },
            enable_performance_monitoring: true,
            max_execution_time_ms: 16,
            memory_limit: 100 * 1024 * 1024,
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
            Some(Arc::new(ScriptPerformanceMonitor::new(config.max_execution_time_ms)))
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
        
        self.loaded_scripts.insert(name.to_string(), path.to_path_buf());
        
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
        let path = self.loaded_scripts.get(name)
            .ok_or_else(|| praxis_utils::eyre::eyre!("Script '{}' not found", name))?
            .clone();
        
        info!("Reloading script '{}'", name);
        self.load_script(name, path)
    }
    
    /// Calls a global Lua function.
    pub fn call_function<'a, A, R>(&'a self, script_name: &str, function_name: &str, args: A) -> Result<R>
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
        let function: mlua::Function = globals
            .get(function_name)
            .map_err(|e| praxis_utils::eyre::eyre!("Function '{}' not found: {}", function_name, e))?;
        
        let result = function
            .call(args)
            .map_err(|e| praxis_utils::eyre::eyre!("Error calling function '{}': {}", function_name, e))?;
        
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
        
        let names: Vec<String> = self.loaded_scripts
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
        
        context.load_string("test", "function add(a, b) return a + b end").unwrap();
        
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
}
