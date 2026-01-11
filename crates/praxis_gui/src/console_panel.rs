//! In-game console panel with command history, Lua REPL, and debug commands.
//!
//! The console panel provides a powerful in-game debugging interface with:
//! - Command history navigation (Up/Down arrows)
//! - Lua REPL for executing Lua code
//! - Custom debug command registration
//! - Autocomplete for commands (Tab key)
//! - Log filtering by level and text search
//! - Auto-scroll to latest messages
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_gui::ConsolePanel;
//! use praxis_scripting::{ScriptingContext, ScriptingConfig};
//! use std::sync::Arc;
//! use parking_lot::RwLock;
//!
//! // Create the console
//! let mut console = ConsolePanel::new();
//!
//! // Optional: Set up Lua REPL integration
//! let scripting_context = Arc::new(RwLock::new(
//!     ScriptingContext::new(ScriptingConfig::default()).unwrap()
//! ));
//! console.set_lua_context(scripting_context);
//!
//! // Register a custom command
//! {
//!     let registry = console.command_registry();
//!     let mut registry = registry.write();
//!     registry.register(
//!         "greet",
//!         "Greet someone",
//!         "greet <name>",
//!         |args| {
//!             if args.is_empty() {
//!                 return Err("Usage: greet <name>".to_string());
//!             }
//!             Ok(format!("Hello, {}!", args[0]))
//!         }
//!     );
//! }
//!
//! // Log messages
//! console.log_info("Console initialized");
//! console.log_warning("This is a warning");
//! console.log_error("This is an error");
//!
//! // Render in your game loop
//! // console.render(&egui_ctx);
//! ```

use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

#[cfg(feature = "scripting")]
use mlua;
#[cfg(feature = "scripting")]
use praxis_scripting::ScriptingContext;

/// Maximum number of history entries to keep
const MAX_HISTORY_SIZE: usize = 1000;
/// Maximum number of command history entries
const MAX_COMMAND_HISTORY: usize = 100;

/// Log level for console messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Success messages (e.g., command completed successfully)
    Success,
    /// Debug messages
    Debug,
}

/// A single log entry in the console
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// The log message
    pub message: String,
    /// Log level
    pub level: LogLevel,
    /// Timestamp when the entry was created
    pub timestamp: std::time::Instant,
}

impl LogEntry {
    /// Creates a new log entry
    pub fn new(message: impl Into<String>, level: LogLevel) -> Self {
        Self {
            message: message.into(),
            level,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Returns the color for this log level
    fn color(&self) -> egui::Color32 {
        match self.level {
            LogLevel::Info => egui::Color32::LIGHT_GRAY,
            LogLevel::Warning => egui::Color32::YELLOW,
            LogLevel::Error => egui::Color32::RED,
            LogLevel::Success => egui::Color32::GREEN,
            LogLevel::Debug => egui::Color32::LIGHT_BLUE,
        }
    }
}

/// Type alias for a command handler function
pub type CommandHandler = Box<dyn Fn(&[&str]) -> Result<String, String> + Send + Sync>;

/// Registry for custom debug commands
pub struct CommandRegistry {
    commands: std::collections::HashMap<String, CommandInfo>,
}

struct CommandInfo {
    handler: CommandHandler,
    description: String,
    usage: String,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRegistry {
    /// Creates a new command registry with built-in commands
    pub fn new() -> Self {
        let mut registry = Self {
            commands: std::collections::HashMap::new(),
        };

        registry.register_builtin_commands();
        registry
    }

    /// Registers a custom command
    pub fn register<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        usage: impl Into<String>,
        handler: F,
    ) where
        F: Fn(&[&str]) -> Result<String, String> + Send + Sync + 'static,
    {
        let name = name.into();
        self.commands.insert(
            name.clone(),
            CommandInfo {
                handler: Box::new(handler),
                description: description.into(),
                usage: usage.into(),
            },
        );
    }

    /// Executes a command
    pub fn execute(&self, command: &str, args: &[&str]) -> Result<String, String> {
        if let Some(cmd_info) = self.commands.get(command) {
            (cmd_info.handler)(args)
        } else {
            Err(format!("Unknown command: {command}"))
        }
    }

    /// Gets all registered command names
    pub fn command_names(&self) -> Vec<&str> {
        self.commands.keys().map(|s| s.as_str()).collect()
    }

    /// Returns the number of registered commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Gets command info for a specific command
    pub fn get_command_info(&self, command: &str) -> Option<(&str, &str)> {
        self.commands
            .get(command)
            .map(|info| (info.description.as_str(), info.usage.as_str()))
    }

    /// Returns autocomplete suggestions for a partial command
    pub fn autocomplete(&self, partial: &str) -> Vec<String> {
        if partial.is_empty() {
            return Vec::new();
        }

        let partial_lower = partial.to_lowercase();
        let mut matches: Vec<String> = self
            .commands
            .keys()
            .filter(|name| name.to_lowercase().starts_with(&partial_lower))
            .cloned()
            .collect();

        matches.sort();
        matches
    }

    /// Gets all commands with their info for help display
    pub fn list_all_commands(&self) -> Vec<(String, String, String)> {
        let mut commands: Vec<_> = self
            .commands
            .iter()
            .map(|(name, info)| (name.clone(), info.description.clone(), info.usage.clone()))
            .collect();
        commands.sort_by(|a, b| a.0.cmp(&b.0));
        commands
    }

    fn register_builtin_commands(&mut self) {
        self.register(
            "help",
            "Display help information about available commands",
            "help [command]",
            |args| {
                if args.is_empty() {
                    Ok("Available commands: help, clear, echo\nType 'help <command>' for more information or just 'help' to list all commands.".to_string())
                } else {
                    match args[0] {
                        "help" => Ok("Display this help message\nUsage: help [command]".to_string()),
                        "clear" => Ok("Clear the console history\nUsage: clear".to_string()),
                        "echo" => Ok("Echo text to the console\nUsage: echo <text>".to_string()),
                        _ => Err(format!("Unknown command: {}", args[0])),
                    }
                }
            },
        );

        self.register("echo", "Echo text to the console", "echo <text>", |args| {
            if args.is_empty() {
                Err("Usage: echo <text>".to_string())
            } else {
                Ok(args.join(" "))
            }
        });
    }
}

/// Console panel with command history, Lua REPL, and custom commands
pub struct ConsolePanel {
    /// Whether the console is visible
    pub visible: bool,
    /// Command input buffer
    input_buffer: String,
    /// History of log entries
    history: VecDeque<LogEntry>,
    /// Command history for recall
    command_history: VecDeque<String>,
    /// Current position in command history
    history_index: Option<usize>,
    /// Temporary buffer when navigating history
    temp_input: Option<String>,
    /// Whether to auto-scroll to bottom
    auto_scroll: bool,
    /// Filter text for log entries
    filter_text: String,
    /// Selected log level filter
    filter_level: Option<LogLevel>,
    /// Command registry
    command_registry: Arc<RwLock<CommandRegistry>>,
    /// Lua scripting context (optional)
    #[cfg(feature = "scripting")]
    lua_context: Option<Arc<RwLock<ScriptingContext>>>,
    /// ECS World pointer for Lua commands (optional)
    #[cfg(feature = "scripting")]
    world_ptr: Option<*mut praxis_ecs::World>,
    /// Whether to focus input on next frame
    focus_input: bool,
    /// Autocomplete suggestions
    autocomplete_suggestions: Vec<String>,
    /// Selected autocomplete index
    autocomplete_index: Option<usize>,
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolePanel {
    /// Creates a new console panel
    pub fn new() -> Self {
        Self {
            visible: false,
            input_buffer: String::new(),
            history: VecDeque::new(),
            command_history: VecDeque::new(),
            history_index: None,
            temp_input: None,
            auto_scroll: true,
            filter_text: String::new(),
            filter_level: None,
            command_registry: Arc::new(RwLock::new(CommandRegistry::new())),
            #[cfg(feature = "scripting")]
            lua_context: None,
            #[cfg(feature = "scripting")]
            world_ptr: None,
            focus_input: false,
            autocomplete_suggestions: Vec::new(),
            autocomplete_index: None,
        }
    }

    /// Sets the Lua scripting context for REPL functionality
    #[cfg(feature = "scripting")]
    pub fn set_lua_context(&mut self, context: Arc<RwLock<ScriptingContext>>) {
        self.lua_context = Some(context);
    }

    /// Sets the ECS World for Lua console commands.
    ///
    /// This should be called each frame with the current world to enable
    /// console commands like `console.list_entities()` to work.
    ///
    /// # Safety
    ///
    /// The world pointer must remain valid for the lifetime of console command execution.
    /// Callers should ensure this is called each frame with a valid world reference.
    #[cfg(feature = "scripting")]
    pub fn set_world(&mut self, world: &mut praxis_ecs::World) {
        self.world_ptr = Some(world as *mut praxis_ecs::World);
    }

    /// Gets a reference to the command registry
    pub fn command_registry(&self) -> Arc<RwLock<CommandRegistry>> {
        Arc::clone(&self.command_registry)
    }

    /// Logs a message to the console
    pub fn log(&mut self, message: impl Into<String>, level: LogLevel) {
        let entry = LogEntry::new(message, level);
        self.history.push_back(entry);

        if self.history.len() > MAX_HISTORY_SIZE {
            self.history.pop_front();
        }

        self.auto_scroll = true;
    }

    /// Logs an info message
    pub fn log_info(&mut self, message: impl Into<String>) {
        self.log(message, LogLevel::Info);
    }

    /// Logs a warning message
    pub fn log_warning(&mut self, message: impl Into<String>) {
        self.log(message, LogLevel::Warning);
    }

    /// Logs an error message
    pub fn log_error(&mut self, message: impl Into<String>) {
        self.log(message, LogLevel::Error);
    }

    /// Logs a success message
    pub fn log_success(&mut self, message: impl Into<String>) {
        self.log(message, LogLevel::Success);
    }

    /// Logs a debug message
    pub fn log_debug(&mut self, message: impl Into<String>) {
        self.log(message, LogLevel::Debug);
    }

    /// Clears the console history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Toggles console visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.focus_input = true;
        }
    }

    /// Shows the console
    pub fn show(&mut self) {
        self.visible = true;
        self.focus_input = true;
    }

    /// Hides the console
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Returns the number of log entries in the history
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Returns the number of commands in the command history
    pub fn command_history_count(&self) -> usize {
        self.command_history.len()
    }

    /// Renders the console panel
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new("Console")
            .default_pos(egui::pos2(10.0, 400.0))
            .default_size(egui::vec2(800.0, 400.0))
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                self.render_toolbar(ui);
                ui.separator();
                self.render_history(ui);
                ui.separator();
                self.render_input(ui);
            });
    }

    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear();
            }

            ui.separator();

            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);

            if ui.button("✖").clicked() {
                self.filter_text.clear();
            }

            ui.separator();

            ui.label("Level:");
            egui::ComboBox::from_id_salt("log_level_filter")
                .selected_text(match self.filter_level {
                    Some(LogLevel::Info) => "Info",
                    Some(LogLevel::Warning) => "Warning",
                    Some(LogLevel::Error) => "Error",
                    Some(LogLevel::Success) => "Success",
                    Some(LogLevel::Debug) => "Debug",
                    None => "All",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_level, None, "All");
                    ui.selectable_value(&mut self.filter_level, Some(LogLevel::Info), "Info");
                    ui.selectable_value(&mut self.filter_level, Some(LogLevel::Warning), "Warning");
                    ui.selectable_value(&mut self.filter_level, Some(LogLevel::Error), "Error");
                    ui.selectable_value(&mut self.filter_level, Some(LogLevel::Success), "Success");
                    ui.selectable_value(&mut self.filter_level, Some(LogLevel::Debug), "Debug");
                });

            ui.separator();

            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
        });
    }

    fn render_history(&mut self, ui: &mut egui::Ui) {
        let scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(ui.available_height() - 60.0)
            .stick_to_bottom(self.auto_scroll);

        scroll_area.show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 2.0;

            let filter_text_lower = self.filter_text.to_lowercase();

            for entry in &self.history {
                if let Some(level) = self.filter_level {
                    if entry.level != level {
                        continue;
                    }
                }

                if !filter_text_lower.is_empty()
                    && !entry.message.to_lowercase().contains(&filter_text_lower)
                {
                    continue;
                }

                let prefix = match entry.level {
                    LogLevel::Info => "[INFO] ",
                    LogLevel::Warning => "[WARN] ",
                    LogLevel::Error => "[ERROR] ",
                    LogLevel::Success => "[OK] ",
                    LogLevel::Debug => "[DEBUG] ",
                };

                let text = egui::RichText::new(format!("{}{}", prefix, entry.message))
                    .color(entry.color())
                    .monospace();

                ui.label(text);
            }
        });
    }

    fn render_input(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if !self.autocomplete_suggestions.is_empty() {
                self.render_autocomplete(ui);
            }

            ui.horizontal(|ui| {
                ui.label(">");

                let input_response = ui.add(
                    egui::TextEdit::singleline(&mut self.input_buffer)
                        .desired_width(ui.available_width())
                        .hint_text("Enter command or Lua code..."),
                );

                if self.focus_input {
                    input_response.request_focus();
                    self.focus_input = false;
                }

                if input_response.changed() {
                    self.update_autocomplete();
                }

                if input_response.has_focus() {
                    self.handle_input_shortcuts(ui);
                }

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) && input_response.has_focus() {
                    self.execute_input();
                }
            });
        });
    }

    fn render_autocomplete(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                let suggestions = self.autocomplete_suggestions.clone();
                for (idx, suggestion) in suggestions.iter().enumerate() {
                    let is_selected = self.autocomplete_index == Some(idx);
                    let response = ui.selectable_label(is_selected, suggestion);

                    if response.clicked() {
                        self.input_buffer = suggestion.clone();
                        self.autocomplete_suggestions.clear();
                        self.autocomplete_index = None;
                    }

                    if is_selected && response.hovered() {
                        self.input_buffer = suggestion.clone();
                    }
                }
            });
        });
    }

    fn handle_input_shortcuts(&mut self, ui: &egui::Ui) {
        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.navigate_history_up();
        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.navigate_history_down();
        }

        if !self.autocomplete_suggestions.is_empty() {
            if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
                self.cycle_autocomplete();
            } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.autocomplete_suggestions.clear();
                self.autocomplete_index = None;
            }
        }
    }

    fn update_autocomplete(&mut self) {
        let input_trimmed = self.input_buffer.trim();

        if input_trimmed.is_empty() || input_trimmed.contains(' ') {
            self.autocomplete_suggestions.clear();
            self.autocomplete_index = None;
            return;
        }

        let registry = self.command_registry.read();
        self.autocomplete_suggestions = registry.autocomplete(input_trimmed);
        self.autocomplete_index = if self.autocomplete_suggestions.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    fn cycle_autocomplete(&mut self) {
        if let Some(current_idx) = self.autocomplete_index {
            let next_idx = (current_idx + 1) % self.autocomplete_suggestions.len();
            self.autocomplete_index = Some(next_idx);
            if let Some(suggestion) = self.autocomplete_suggestions.get(next_idx) {
                self.input_buffer = suggestion.clone();
            }
        }
    }

    fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        if self.history_index.is_none() {
            self.temp_input = Some(self.input_buffer.clone());
            self.history_index = Some(self.command_history.len() - 1);
        } else if let Some(idx) = self.history_index {
            if idx > 0 {
                self.history_index = Some(idx - 1);
            }
        }

        if let Some(idx) = self.history_index {
            if let Some(cmd) = self.command_history.get(idx) {
                self.input_buffer = cmd.clone();
            }
        }
    }

    fn navigate_history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.command_history.len() - 1 {
                self.history_index = Some(idx + 1);
                if let Some(cmd) = self.command_history.get(idx + 1) {
                    self.input_buffer = cmd.clone();
                }
            } else {
                self.history_index = None;
                if let Some(temp) = self.temp_input.take() {
                    self.input_buffer = temp;
                }
            }
        }
    }

    fn execute_input(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            return;
        }

        self.log(format!("> {input}"), LogLevel::Info);

        self.command_history.push_back(input.clone());
        if self.command_history.len() > MAX_COMMAND_HISTORY {
            self.command_history.pop_front();
        }

        self.history_index = None;
        self.temp_input = None;
        self.autocomplete_suggestions.clear();
        self.autocomplete_index = None;

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            self.input_buffer.clear();
            return;
        }

        let command = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        if command == "clear" {
            self.clear();
            self.input_buffer.clear();
            return;
        }

        if command == "help" {
            if args.is_empty() {
                let all_commands = {
                    let registry = self.command_registry.read();
                    registry.list_all_commands()
                };
                let mut help_text = format!("Available commands ({}):\n", all_commands.len());
                for (name, description, usage) in all_commands {
                    help_text.push_str(&format!("  {name}: {description}\n    Usage: {usage}\n"));
                }
                help_text.push_str("\nYou can also execute Lua code directly.");
                self.log(help_text, LogLevel::Info);
                self.input_buffer.clear();
                return;
            } else {
                let command_info = {
                    let registry = self.command_registry.read();
                    registry
                        .get_command_info(&args[0])
                        .map(|(d, u)| (d.to_string(), u.to_string()))
                };
                if let Some((description, usage)) = command_info {
                    self.log(format!("{description}\nUsage: {usage}"), LogLevel::Info);
                    self.input_buffer.clear();
                    return;
                } else {
                    self.log(format!("Unknown command: {}", args[0]), LogLevel::Error);
                    self.input_buffer.clear();
                    return;
                }
            }
        }

        let result = {
            let registry = self.command_registry.read();
            if registry.command_names().contains(&command) {
                let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                Some(registry.execute(command, &args_refs))
            } else {
                None
            }
        };

        if let Some(result) = result {
            match result {
                Ok(output) => {
                    if !output.is_empty() {
                        self.log(output, LogLevel::Success);
                    }
                }
                Err(error) => {
                    self.log(error, LogLevel::Error);
                }
            }
        } else {
            #[cfg(feature = "scripting")]
            if let Some(ref lua_context) = self.lua_context {
                self.execute_lua(&input, lua_context);
            } else {
                self.log(
                    format!("Unknown command: {command}. Use 'help' for available commands."),
                    LogLevel::Error,
                );
            }

            #[cfg(not(feature = "scripting"))]
            self.log(
                format!("Unknown command: {command}. Use 'help' for available commands."),
                LogLevel::Error,
            );
        }

        self.input_buffer.clear();
    }

    #[cfg(feature = "scripting")]
    fn execute_lua(&mut self, code: &str, lua_context: &Arc<RwLock<ScriptingContext>>) {
        let context = lua_context.read();

        // If we have a world pointer, use eval_interactive_with_world for full ECS access
        if let Some(world_ptr) = self.world_ptr {
            #[allow(unsafe_code)]
            let world = unsafe { &mut *world_ptr };

            match context.eval_interactive_with_world(code, world) {
                Ok(output) => {
                    if !output.is_empty() {
                        self.log(output, LogLevel::Success);
                    }
                }
                Err(error) => {
                    self.log(format!("{}", error), LogLevel::Error);
                }
            }
        } else {
            // Fall back to basic eval without world access
            match context.eval_interactive(code) {
                Ok(output) => {
                    if !output.is_empty() {
                        self.log(output, LogLevel::Success);
                    }
                }
                Err(error) => {
                    self.log(format!("{}", error), LogLevel::Error);
                }
            }
        }
    }
}
