//! Console panel for displaying logs and executing commands.

use super::EditorPanel;
use egui::Ui;

/// Panel for displaying console output and entering commands.
pub struct ConsolePanel {
    title: String,
    log_messages: Vec<String>,
    command_input: String,
}

impl ConsolePanel {
    /// Creates a new console panel.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Console".to_string(),
            log_messages: Vec::new(),
            command_input: String::new(),
        }
    }

    /// Adds a log message to the console.
    pub fn add_log(&mut self, message: String) {
        self.log_messages.push(message);
    }

    /// Clears all log messages.
    pub fn clear(&mut self) {
        self.log_messages.clear();
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

    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("Console");
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in &self.log_messages {
                    ui.label(message);
                }

                if self.log_messages.is_empty() {
                    ui.label("No messages");
                }
            });

        ui.separator();

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

            if ui.button("Clear").clicked() {
                self.clear();
            }
        });
    }
}
