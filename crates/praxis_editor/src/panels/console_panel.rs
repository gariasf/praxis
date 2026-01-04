//! Console panel for displaying logs and executing commands.

use super::EditorPanel;
use egui::{Color32, ScrollArea, Ui};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

const MAX_LOG_MESSAGES: usize = 1000;

/// Log message level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<&Level> for LogLevel {
    fn from(level: &Level) -> Self {
        match *level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

impl LogLevel {
    fn color(self) -> Color32 {
        match self {
            LogLevel::Trace => Color32::from_rgb(128, 128, 128),
            LogLevel::Debug => Color32::from_rgb(160, 160, 200),
            LogLevel::Info => Color32::from_rgb(200, 200, 200),
            LogLevel::Warn => Color32::from_rgb(255, 200, 0),
            LogLevel::Error => Color32::from_rgb(255, 80, 80),
        }
    }

    fn icon(self) -> &'static str {
        match self {
            LogLevel::Trace => "🔍",
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Warn => "⚠️",
            LogLevel::Error => "❌",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

/// A single log message entry
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub timestamp: String,
}

/// Thread-safe log buffer that can be shared between tracing layer and console panel
#[derive(Clone)]
pub struct LogBuffer {
    inner: Arc<Mutex<VecDeque<LogMessage>>>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_MESSAGES))),
        }
    }

    pub fn push(&self, message: LogMessage) {
        if let Ok(mut buffer) = self.inner.lock() {
            if buffer.len() >= MAX_LOG_MESSAGES {
                buffer.pop_front();
            }
            buffer.push_back(message);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut buffer) = self.inner.lock() {
            buffer.clear();
        }
    }

    pub fn get_messages(&self) -> Vec<LogMessage> {
        if let Ok(buffer) = self.inner.lock() {
            buffer.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        if let Ok(buffer) = self.inner.lock() {
            buffer.len()
        } else {
            0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracing layer that captures logs and sends them to the console panel
pub struct ConsoleLayer {
    buffer: LogBuffer,
}

impl ConsoleLayer {
    pub fn new(buffer: LogBuffer) -> Self {
        Self { buffer }
    }
}

impl<S> Layer<S> for ConsoleLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = LogLevel::from(metadata.level());
        let target = metadata.target().to_string();

        let mut message = String::new();
        let mut visitor = MessageVisitor {
            message: &mut message,
        };
        event.record(&mut visitor);

        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();

        self.buffer.push(LogMessage {
            level,
            target,
            message,
            timestamp,
        });
    }
}

struct MessageVisitor<'a> {
    message: &'a mut String,
}

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.message = format!("{value:?}");
            if self.message.starts_with('"') && self.message.ends_with('"') {
                *self.message = self.message[1..self.message.len() - 1].to_string();
            }
        }
    }
}

/// Panel for displaying console output and entering commands.
pub struct ConsolePanel {
    title: String,
    log_buffer: LogBuffer,
    command_input: String,
    search_filter: String,
    show_trace: bool,
    show_debug: bool,
    show_info: bool,
    show_warn: bool,
    show_error: bool,
    auto_scroll: bool,
}

impl ConsolePanel {
    /// Creates a new console panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Console".to_string(),
            log_buffer: LogBuffer::new(),
            command_input: String::new(),
            search_filter: String::new(),
            show_trace: false,
            show_debug: false,
            show_info: true,
            show_warn: true,
            show_error: true,
            auto_scroll: true,
        }
    }

    /// Creates a new console panel with a shared log buffer
    #[must_use]
    pub fn with_buffer(buffer: LogBuffer) -> Self {
        Self {
            title: "Console".to_string(),
            log_buffer: buffer,
            command_input: String::new(),
            search_filter: String::new(),
            show_trace: false,
            show_debug: false,
            show_info: true,
            show_warn: true,
            show_error: true,
            auto_scroll: true,
        }
    }

    /// Gets the log buffer for this console
    pub fn log_buffer(&self) -> &LogBuffer {
        &self.log_buffer
    }

    /// Adds a log message to the console.
    pub fn add_log(&mut self, message: String) {
        self.log_buffer.push(LogMessage {
            level: LogLevel::Info,
            target: "console".to_string(),
            message,
            timestamp: chrono::Local::now().format("%H:%M:%S%.3f").to_string(),
        });
    }

    /// Clears all log messages.
    pub fn clear(&mut self) {
        self.log_buffer.clear();
    }

    /// Checks if a log message matches current filters
    fn matches_filters(&self, msg: &LogMessage) -> bool {
        let level_match = match msg.level {
            LogLevel::Trace => self.show_trace,
            LogLevel::Debug => self.show_debug,
            LogLevel::Info => self.show_info,
            LogLevel::Warn => self.show_warn,
            LogLevel::Error => self.show_error,
        };

        if !level_match {
            return false;
        }

        if self.search_filter.is_empty() {
            return true;
        }

        let search_lower = self.search_filter.to_lowercase();
        msg.message.to_lowercase().contains(&search_lower)
            || msg.target.to_lowercase().contains(&search_lower)
    }

    /// Renders the toolbar with filter controls
    fn render_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");

            if ui
                .selectable_label(self.show_trace, format!("{} Trace", LogLevel::Trace.icon()))
                .on_hover_text("Show trace messages")
                .clicked()
            {
                self.show_trace = !self.show_trace;
            }

            if ui
                .selectable_label(self.show_debug, format!("{} Debug", LogLevel::Debug.icon()))
                .on_hover_text("Show debug messages")
                .clicked()
            {
                self.show_debug = !self.show_debug;
            }

            if ui
                .selectable_label(self.show_info, format!("{} Info", LogLevel::Info.icon()))
                .on_hover_text("Show info messages")
                .clicked()
            {
                self.show_info = !self.show_info;
            }

            if ui
                .selectable_label(self.show_warn, format!("{} Warn", LogLevel::Warn.icon()))
                .on_hover_text("Show warning messages")
                .clicked()
            {
                self.show_warn = !self.show_warn;
            }

            if ui
                .selectable_label(self.show_error, format!("{} Error", LogLevel::Error.icon()))
                .on_hover_text("Show error messages")
                .clicked()
            {
                self.show_error = !self.show_error;
            }

            ui.separator();

            if ui
                .button("Clear")
                .on_hover_text("Clear all messages")
                .clicked()
            {
                self.clear();
            }

            if ui
                .selectable_label(self.auto_scroll, "📜 Auto-scroll")
                .on_hover_text("Automatically scroll to bottom")
                .clicked()
            {
                self.auto_scroll = !self.auto_scroll;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let total = self.log_buffer.len();
                ui.label(format!("{total} messages"));
            });
        });
    }

    /// Renders the search bar
    fn render_search(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            let response = ui.text_edit_singleline(&mut self.search_filter);
            response.on_hover_text("Search in messages");

            if ui.button("✖").on_hover_text("Clear search").clicked() {
                self.search_filter.clear();
            }
        });
    }

    /// Renders the log messages
    fn render_logs(&self, ui: &mut Ui) {
        let messages = self.log_buffer.get_messages();
        let filtered_messages: Vec<&LogMessage> = messages
            .iter()
            .filter(|msg| self.matches_filters(msg))
            .collect();

        let scroll_area = ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.auto_scroll);

        scroll_area.show(ui, |ui| {
            if filtered_messages.is_empty() {
                ui.label("No messages");
            } else {
                for msg in filtered_messages {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&msg.timestamp)
                                .color(Color32::from_rgb(128, 128, 128))
                                .monospace(),
                        );

                        ui.label(
                            egui::RichText::new(format!("[{}]", msg.level.label()))
                                .color(msg.level.color())
                                .monospace(),
                        );

                        ui.label(
                            egui::RichText::new(&msg.target)
                                .color(Color32::from_rgb(150, 150, 200))
                                .monospace(),
                        );

                        ui.label(egui::RichText::new(&msg.message).color(msg.level.color()));
                    });
                }
            }
        });
    }

    /// Renders the command input
    fn render_command_input(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(">");
            let response = ui.text_edit_singleline(&mut self.command_input);

            if response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !self.command_input.is_empty()
            {
                self.add_log(format!("> {}", self.command_input));
                self.command_input.clear();
            }
        });
    }
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for ConsolePanel {
    fn title(&self) -> &str {
        &self.title
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        ui.vertical(|ui| {
            self.render_toolbar(ui);
            ui.separator();
            self.render_search(ui);
            ui.separator();

            let available_height = ui.available_height();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), available_height - 30.0),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    self.render_logs(ui);
                },
            );

            ui.separator();
            self.render_command_input(ui);
        });
    }
}
