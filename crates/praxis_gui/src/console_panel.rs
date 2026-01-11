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
//! # Architecture
//!
//! ## Command Registry Pattern
//!
//! The `CommandRegistry` implements a **command pattern** for extensible debug commands:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ CommandRegistry                                              │
//! │  ┌────────────┬─────────────────────────────────────┐       │
//! │  │ "help"     │ CommandInfo { handler, desc, usage }│       │
//! │  │ "echo"     │ CommandInfo { ... }                 │       │
//! │  │ "spawn"    │ CommandInfo { ... } (custom)        │       │
//! │  │ "teleport" │ CommandInfo { ... } (custom)        │       │
//! │  └────────────┴─────────────────────────────────────┘       │
//! │                                                              │
//! │  execute(name, args) -> Result<String, String>              │
//! │  autocomplete(partial) -> Vec<String>                        │
//! │  list_all_commands() -> Vec<(name, desc, usage)>            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! **Why Arc<RwLock<CommandRegistry>>?**
//! - Multiple systems may register commands during initialization
//! - Console reads from registry while other systems may still modify it
//! - `Arc`: Shared ownership across console and registration sites
//! - `RwLock`: Many readers (autocomplete, execute) + rare writers (register)
//!
//! ## Lua REPL Integration
//!
//! The console can optionally integrate with `praxis_scripting` to provide a
//! **REPL (Read-Eval-Print Loop)** for Lua:
//!
//! ```text
//! User Input: "return 2 + 2"
//!      ↓
//! ┌────────────────────────────────────────────────────┐
//! │ ConsolePanel::execute_lua()                         │
//! │   if world_ptr exists:                             │
//! │     context.eval_interactive_with_world(code, world)│
//! │   else:                                            │
//! │     context.eval_interactive(code)                 │
//! └────────────────────────────────────────────────────┘
//!      ↓
//! ┌────────────────────────────────────────────────────┐
//! │ ScriptingContext::eval_interactive()                │
//! │ - Wraps code in "return ..." if expression         │
//! │ - Executes in Lua VM                               │
//! │ - Converts result to string (via Display/Debug)    │
//! │ - Maintains REPL state (locals persist)            │
//! └────────────────────────────────────────────────────┘
//!      ↓
//! Output: "4"  (logged as Success)
//! ```
//!
//! **`eval_interactive()` vs regular `eval()`:**
//! - Automatically wraps expressions: `2+2` → `return 2+2`
//! - Pretty-prints results (tables, objects, nil)
//! - Maintains persistent `_REPL_ENV` table for locals
//! - Handles multi-line statements (TODO: future enhancement)
//!
//! **ECS Access via `eval_interactive_with_world()`:**
//! When `set_world()` is called each frame, Lua code gains access to:
//! - `console.list_entities()`: Query all entities
//! - `console.get_component(entity_id, "Transform")`: Read components
//! - `console.set_component(entity_id, "Transform", data)`: Modify components
//! - `world.spawn_entity()`: Create new entities
//!
//! ## Log Filtering Implementation
//!
//! Filtering happens in `render_history()` using a **two-pass approach**:
//!
//! ```text
//! All Log Entries (VecDeque)
//!      ↓
//! ┌────────────────────────────────────────────────────┐
//! │ Pass 1: Filter by LogLevel                          │
//! │   if filter_level.is_some() && entry.level != filter│
//! │     skip entry                                      │
//! └────────────────────────────────────────────────────┘
//!      ↓
//! ┌────────────────────────────────────────────────────┐
//! │ Pass 2: Filter by text search                       │
//! │   if !filter_text.is_empty()                       │
//! │     && !entry.message.contains(filter_text)        │
//! │       skip entry                                   │
//! └────────────────────────────────────────────────────┘
//!      ↓
//! Render visible entries
//! ```
//!
//! **Why case-insensitive search?**
//! - User types "error" but logs say "ERROR" or "Error"
//! - Convert both to lowercase before comparison
//! - Performance: Only done for visible entries (~100s), not all logs (1000s)
//!
//! ## Command History Navigation
//!
//! Classic terminal-style history with **temporary buffer preservation**:
//!
//! ```text
//! State Machine:
//!
//! Initial State: history_index = None, temp_input = None
//!   Input buffer: "hello wo"  (user typing)
//!
//! User presses ↑:
//!   temp_input = Some("hello wo")  ← Save current input
//!   history_index = Some(len - 1)   ← Point to most recent
//!   input_buffer = command_history[len-1]
//!
//! User presses ↑ again:
//!   history_index = Some(len - 2)   ← Move back
//!   input_buffer = command_history[len-2]
//!
//! User presses ↓:
//!   history_index = Some(len - 1)   ← Move forward
//!   input_buffer = command_history[len-1]
//!
//! User presses ↓ at end:
//!   history_index = None            ← Exit history mode
//!   input_buffer = temp_input.take() ← Restore "hello wo"
//! ```
//!
//! **Why VecDeque for history?**
//! - Efficient `push_back()` for new commands
//! - Efficient `pop_front()` when exceeding MAX_COMMAND_HISTORY
//! - Random access via `get(index)` for navigation
//!
//! ## Immediate-Mode UI Pattern
//!
//! The console uses egui's immediate-mode pattern in `render()`:
//!
//! ```rust,ignore
//! pub fn render(&mut self, ctx: &egui::Context) {
//!     // Each frame, declaratively describe the UI:
//!     egui::Window::new("Console")
//!         .show(ctx, |ui| {
//!             // No widget tree to maintain
//!             // State lives in self (input_buffer, history, etc.)
//!             // UI code naturally reflects app state
//!             self.render_toolbar(ui);
//!             self.render_history(ui);
//!             self.render_input(ui);
//!         });
//! }
//! ```
//!
//! **Key insight**: `render()` is not "rendering" in the GPU sense. It's
//! **describing** what should appear. egui:
//! 1. Allocates vertices/indices for shapes
//! 2. Handles input (button clicks, text entry)
//! 3. Returns responses (was button clicked?)
//! 4. Manages layout automatically
//!
//! The actual GPU rendering happens later in `EguiIntegration`.
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

/// Maximum number of log entries to keep in the history buffer.
/// When exceeded, oldest entries are removed (FIFO via VecDeque::pop_front).
/// Trade-off: More entries = more memory but better debugging context.
const MAX_HISTORY_SIZE: usize = 1000;

/// Maximum number of command history entries (for Up/Down arrow navigation).
/// Separate from log history to avoid confusion between "what I typed" vs "what happened".
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

    /// Returns the color for this log level.
    /// Color-coding improves visual scanning - errors immediately stand out as red.
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

/// Type alias for a command handler function.
///
/// **Design choices:**
/// - `Box<dyn Fn>`: Commands can be closures capturing environment (e.g., Arc<World>)
/// - `&[&str]`: Arguments as string slices - cheap to pass, command parses if needed
/// - `Result<String, String>`: Ok = output to print, Err = error message
/// - `Send + Sync`: Commands may be registered from any thread, executed on main thread
pub type CommandHandler = Box<dyn Fn(&[&str]) -> Result<String, String> + Send + Sync>;

/// Registry for custom debug commands.
///
/// Uses a HashMap for O(1) command lookup. Alternative designs considered:
/// - Vec<(name, CommandInfo)>: Would be O(n) lookup, unacceptable for autocomplete
/// - BTreeMap: Sorted iteration for free, but slower lookup than HashMap
/// - Trie: Optimal for prefix search, but overkill for ~10-50 commands
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

    /// Returns autocomplete suggestions for a partial command.
    ///
    /// **Algorithm: Prefix matching with case-insensitive comparison**
    /// ```text
    /// User types: "sp"
    /// Commands: ["spawn", "speed", "help", "echo"]
    /// Matches: ["spawn", "speed"]  (both start with "sp")
    /// ```
    ///
    /// **Performance:** O(n) where n = number of commands (typically < 100).
    /// For larger command sets, consider a Trie or prefix tree.
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

        // Sort alphabetically for consistent UX
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

/// Console panel with command history, Lua REPL, and custom commands.
///
/// **Field organization:**
/// ```text
/// ┌─ UI State ──────────────────────────────────────┐
/// │ visible, focus_input                            │
/// │ input_buffer (what user is typing)              │
/// ├─ Log System ────────────────────────────────────┤
/// │ history (VecDeque<LogEntry>)                    │
/// │ filter_text, filter_level, auto_scroll          │
/// ├─ Command History (Up/Down arrows) ─────────────┤
/// │ command_history (VecDeque<String>)              │
/// │ history_index (current position in history)     │
/// │ temp_input (saves typing when entering history) │
/// ├─ Autocomplete (Tab key) ───────────────────────┤
/// │ autocomplete_suggestions (matching commands)    │
/// │ autocomplete_index (selected suggestion)        │
/// ├─ Command Execution ─────────────────────────────┤
/// │ command_registry (Arc<RwLock<...>>)             │
/// │ lua_context (optional Lua REPL)                 │
/// │ world_ptr (optional ECS access from Lua)        │
/// └─────────────────────────────────────────────────┘
/// ```
pub struct ConsolePanel {
    /// Whether the console is visible (toggleable with ~ key typically)
    pub visible: bool,

    /// Current text in the input field (user is typing this)
    input_buffer: String,

    /// Ring buffer of log entries (messages, warnings, errors).
    /// VecDeque allows efficient push_back/pop_front for FIFO behavior.
    history: VecDeque<LogEntry>,

    /// Ring buffer of previously executed commands (for Up/Down arrow recall).
    /// Separate from log history: this is "what I typed", not "what happened".
    command_history: VecDeque<String>,

    /// Current position when navigating command history via Up/Down arrows.
    /// None = not in history mode, Some(idx) = viewing command_history[idx].
    history_index: Option<usize>,

    /// Saves the user's incomplete input when they press Up to enter history mode.
    /// Restored when they press Down past the most recent command.
    temp_input: Option<String>,

    /// If true, scroll to bottom when new log entries arrive (follow mode).
    /// If false, user scrolled up to read older logs, don't disturb them.
    auto_scroll: bool,

    /// Text filter for log entries (case-insensitive substring match)
    filter_text: String,

    /// Log level filter (None = show all, Some(level) = show only that level)
    filter_level: Option<LogLevel>,

    /// Registry of custom debug commands.
    /// Arc<RwLock> allows multiple systems to register commands at startup.
    command_registry: Arc<RwLock<CommandRegistry>>,

    /// Optional Lua scripting context for REPL functionality.
    /// If set, unrecognized commands are treated as Lua code.
    #[cfg(feature = "scripting")]
    lua_context: Option<Arc<RwLock<ScriptingContext>>>,

    /// Optional ECS World pointer for Lua console commands.
    /// Updated each frame via set_world() to enable ECS queries from Lua.
    /// Safety: Must be kept valid; set_world() should be called every frame.
    #[cfg(feature = "scripting")]
    world_ptr: Option<*mut praxis_ecs::World>,

    /// Flag to request focus on the input field next frame.
    /// (egui requires one frame delay to focus newly created widgets)
    focus_input: bool,

    /// List of command suggestions for the current input (prefix matching)
    autocomplete_suggestions: Vec<String>,

    /// Index of the currently selected autocomplete suggestion (Tab to cycle)
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

    /// Renders the log history with filtering and auto-scroll.
    ///
    /// **egui ScrollArea details:**
    /// - `auto_shrink([false, false])`: Don't shrink below min size (UX: stable layout)
    /// - `stick_to_bottom(true)`: Auto-scroll when new logs arrive
    /// - `stick_to_bottom(false)`: User scrolled up, stay there (preserve reading position)
    fn render_history(&mut self, ui: &mut egui::Ui) {
        let scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(ui.available_height() - 60.0)
            .stick_to_bottom(self.auto_scroll);

        scroll_area.show(ui, |ui| {
            // Tighter spacing for log readability (default is too loose)
            ui.spacing_mut().item_spacing.y = 2.0;

            // Pre-lowercase once, not per-entry (micro-optimization)
            let filter_text_lower = self.filter_text.to_lowercase();

            // **Two-pass filtering**: level first (cheaper check), then text search
            for entry in &self.history {
                // Pass 1: Filter by log level (exact match, very fast)
                if let Some(level) = self.filter_level {
                    if entry.level != level {
                        continue;
                    }
                }

                // Pass 2: Filter by text search (substring match, case-insensitive)
                if !filter_text_lower.is_empty()
                    && !entry.message.to_lowercase().contains(&filter_text_lower)
                {
                    continue;
                }

                // Format with level prefix for context
                let prefix = match entry.level {
                    LogLevel::Info => "[INFO] ",
                    LogLevel::Warning => "[WARN] ",
                    LogLevel::Error => "[ERROR] ",
                    LogLevel::Success => "[OK] ",
                    LogLevel::Debug => "[DEBUG] ",
                };

                // Monospace font for alignment and "code feel"
                let text = egui::RichText::new(format!("{}{}", prefix, entry.message))
                    .color(entry.color())
                    .monospace();

                ui.label(text);
            }
        });
    }

    /// Renders the command input field with autocomplete and keyboard shortcuts.
    ///
    /// **Immediate-mode pattern in action:**
    /// Every frame, we check `input_response.changed()` and update autocomplete.
    /// No need to "attach event listeners" - just check state each frame.
    fn render_input(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Show autocomplete popup above input if available
            if !self.autocomplete_suggestions.is_empty() {
                self.render_autocomplete(ui);
            }

            ui.horizontal(|ui| {
                // Terminal-style prompt
                ui.label(">");

                let input_response = ui.add(
                    egui::TextEdit::singleline(&mut self.input_buffer)
                        .desired_width(ui.available_width())
                        .hint_text("Enter command or Lua code..."),
                );

                // Delayed focus: set flag when console opens, apply next frame
                // (egui needs one frame to create the widget before focusing it)
                if self.focus_input {
                    input_response.request_focus();
                    self.focus_input = false;
                }

                // Update autocomplete as user types (immediate feedback)
                if input_response.changed() {
                    self.update_autocomplete();
                }

                // Handle keyboard shortcuts (Up/Down for history, Tab for autocomplete)
                if input_response.has_focus() {
                    self.handle_input_shortcuts(ui);
                }

                // Execute on Enter key
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

    /// Navigate backwards through command history (Up arrow key).
    ///
    /// **State transitions:**
    /// 1. First Up press: Save current input, jump to most recent command
    /// 2. Subsequent Up: Walk backwards through history
    /// 3. At oldest command: Stay there (don't wrap around)
    fn navigate_history_up(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        // First time entering history mode?
        if self.history_index.is_none() {
            // Save what user was typing (may be incomplete command)
            self.temp_input = Some(self.input_buffer.clone());
            // Start at most recent command (end of VecDeque)
            self.history_index = Some(self.command_history.len() - 1);
        } else if let Some(idx) = self.history_index {
            // Already in history mode, move backwards
            if idx > 0 {
                self.history_index = Some(idx - 1);
            }
            // If idx == 0, we're at oldest command, do nothing
        }

        // Update input buffer to show selected command
        if let Some(idx) = self.history_index {
            if let Some(cmd) = self.command_history.get(idx) {
                self.input_buffer = cmd.clone();
            }
        }
    }

    /// Navigate forwards through command history (Down arrow key).
    ///
    /// **State transitions:**
    /// 1. Down press: Move forward in history
    /// 2. At newest command + Down: Exit history mode, restore original input
    fn navigate_history_down(&mut self) {
        if let Some(idx) = self.history_index {
            // Can we move forward?
            if idx < self.command_history.len() - 1 {
                self.history_index = Some(idx + 1);
                if let Some(cmd) = self.command_history.get(idx + 1) {
                    self.input_buffer = cmd.clone();
                }
            } else {
                // At newest command, exit history mode
                self.history_index = None;
                // Restore what user was originally typing
                if let Some(temp) = self.temp_input.take() {
                    self.input_buffer = temp;
                }
            }
        }
        // If not in history mode, Down does nothing
    }

    /// Executes the current input buffer as either a command or Lua code.
    ///
    /// **Execution priority:**
    /// 1. Built-in commands (clear, help) - handled inline for speed
    /// 2. Registered commands from CommandRegistry
    /// 3. Lua REPL (if scripting feature enabled)
    /// 4. Error: unknown command
    fn execute_input(&mut self) {
        let input = self.input_buffer.trim().to_string();
        if input.is_empty() {
            return;
        }

        // Echo command to log for context
        self.log(format!("> {input}"), LogLevel::Info);

        // Add to command history for Up/Down arrow recall
        self.command_history.push_back(input.clone());
        if self.command_history.len() > MAX_COMMAND_HISTORY {
            self.command_history.pop_front();
        }

        // Reset navigation state (user submitted command, exit history mode)
        self.history_index = None;
        self.temp_input = None;
        self.autocomplete_suggestions.clear();
        self.autocomplete_index = None;

        // Parse command and arguments: "spawn box 5" → ["spawn", "box", "5"]
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            self.input_buffer.clear();
            return;
        }

        let command = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        // Fast path for built-in commands (no registry lookup needed)
        if command == "clear" {
            self.clear();
            self.input_buffer.clear();
            return;
        }

        // Help command: list all commands or show specific command info
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

        // Try to execute as registered command
        // Scope to release RwLock before potentially long-running Lua code
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
            // Command found in registry
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
            // Command not found - try Lua REPL as fallback
            #[cfg(feature = "scripting")]
            if let Some(lua_context) = self.lua_context.clone() {
                self.execute_lua(&input, &lua_context);
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

    /// Executes Lua code via the scripting context's REPL functionality.
    ///
    /// **Two execution modes:**
    ///
    /// 1. **With ECS World access** (`eval_interactive_with_world`):
    ///    ```lua
    ///    -- User can query/modify entities from console
    ///    entities = console.list_entities()
    ///    transform = console.get_component(entities[1], "Transform")
    ///    console.set_component(entities[1], "Transform", {x=0, y=5, z=0})
    ///    ```
    ///
    /// 2. **Without World** (`eval_interactive`):
    ///    ```lua
    ///    -- Basic Lua REPL for math, prototyping, testing
    ///    return 2 + 2  -- Outputs: 4
    ///    x = 10        -- Variable persists in REPL environment
    ///    return x * 2  -- Outputs: 20
    ///    ```
    ///
    /// **Why `eval_interactive()` instead of `eval()`?**
    /// - Auto-wraps expressions: `2+2` → `return 2+2`
    /// - Pretty-prints results (tables, userdata, nil)
    /// - Maintains `_REPL_ENV` for persistent variables
    /// - Returns String output suitable for console display
    ///
    /// **Safety note:**
    /// The world pointer comes from `set_world()` which must be called each frame.
    /// This ensures the pointer is always valid during command execution.
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
