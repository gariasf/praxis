# Console Panel Implementation

This document describes the implementation of the Console Panel for the Praxis editor, including log filtering, search functionality, and integration with the tracing system.

## Overview

The Console Panel is a comprehensive logging and debugging interface that captures all engine logs in real-time and provides powerful filtering and search capabilities. It integrates seamlessly with the `praxis_utils::tracing` system to capture logs from all engine subsystems.

## Features

### 1. Real-time Log Capture
- **Tracing Integration**: Custom `ConsoleLayer` implementation captures all tracing events
- **Thread-safe Buffer**: `LogBuffer` uses `Arc<Mutex<VecDeque>>` for safe concurrent access
- **Automatic Limit**: Maintains a maximum of 1000 messages to prevent memory overflow
- **Color-coded Levels**: Visual distinction between trace, debug, info, warn, and error

### 2. Log Filtering by Level
- **Five Log Levels**: Trace, Debug, Info, Warning, Error
- **Toggle Buttons**: Click to show/hide each level independently
- **Smart Defaults**: Info, Warn, and Error enabled by default
- **Visual Feedback**: Selected filters highlighted in the UI

### 3. Search Functionality
- **Real-time Search**: Filter logs by message content or target module
- **Case-insensitive**: Works regardless of case
- **Clear Button**: Quick way to reset search filter
- **Combined Filtering**: Search works together with level filters

### 4. Auto-scroll Toggle
- **Automatic Scrolling**: Follows new messages as they arrive
- **Manual Override**: Disable to review older messages
- **Sticky Bottom**: Uses egui's `stick_to_bottom` feature
- **Toggle Button**: Easy on/off control

### 5. Additional Features
- **Timestamps**: All messages include precise timestamps (HH:MM:SS.mmm)
- **Target Module**: Shows which subsystem generated each log
- **Clear Button**: Remove all messages with one click
- **Message Counter**: Displays total message count in toolbar
- **Command Input**: Text field for future command execution (extensible)

## Architecture

### Core Components

#### `LogMessage`
```rust
pub struct LogMessage {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub timestamp: String,
}
```

Represents a single log entry with all metadata needed for display and filtering.

#### `LogBuffer`
```rust
pub struct LogBuffer {
    inner: Arc<Mutex<VecDeque<LogMessage>>>,
}
```

Thread-safe buffer that stores log messages. Uses `Arc` for cloning and sharing between the tracing layer and console panel.

#### `ConsoleLayer`
```rust
pub struct ConsoleLayer {
    buffer: LogBuffer,
}

impl<S> Layer<S> for ConsoleLayer where S: tracing::Subscriber
```

Custom tracing layer that captures log events and pushes them to the buffer. Implements the `Layer` trait to integrate with `tracing-subscriber`.

#### `ConsolePanel`
```rust
pub struct ConsolePanel {
    log_buffer: LogBuffer,
    search_filter: String,
    show_trace: bool,
    show_debug: bool,
    show_info: bool,
    show_warn: bool,
    show_error: bool,
    auto_scroll: bool,
    // ...
}
```

Main panel implementation with filtering state and UI rendering.

### Integration Flow

```
┌─────────────────┐
│  Engine Code    │
│  (info!(...))   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Tracing System  │
│  (subscriber)   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ ConsoleLayer    │
│ (captures logs) │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  LogBuffer      │
│ (thread-safe)   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ ConsolePanel    │
│  (displays UI)  │
└─────────────────┘
```

## Usage

### Basic Setup

```rust
use praxis_editor::{init_with_console, LogBuffer, ConsolePanel};

// Create a shared log buffer
let log_buffer = LogBuffer::new();

// Initialize tracing with console capture
init_with_console(log_buffer.clone())?;

// Create console panel with the same buffer
let console_panel = ConsolePanel::with_buffer(log_buffer);

// Now all logs will be captured and displayed in the panel
info!("This message appears in the console!");
```

### With EditorState

```rust
use praxis_editor::{init_with_console, EditorState, LogBuffer};

// Create log buffer
let log_buffer = LogBuffer::new();

// Initialize with console integration
init_with_console(log_buffer.clone())?;

// Create editor with the log buffer
let editor = EditorState::with_log_buffer(log_buffer);

// All engine logs now appear in the editor console
```

### Standalone Console

```rust
use praxis_editor::{ConsolePanel, EditorPanel};

// Create console without log capture
let mut console = ConsolePanel::new();

// Manually add messages
console.add_log("Custom message".to_string());

// Render in UI
console.ui(&mut ui);
```

## Implementation Details

### Log Level Colors

Each log level has a distinct color for easy visual identification:

- **Trace**: Gray (128, 128, 128) - 🔍
- **Debug**: Light Blue (160, 160, 200) - 🐛
- **Info**: White (200, 200, 200) - ℹ️
- **Warn**: Yellow/Orange (255, 200, 0) - ⚠️
- **Error**: Red (255, 80, 80) - ❌

### Filtering Logic

Messages are filtered through a two-stage process:

1. **Level Filter**: Check if the message's level is enabled
2. **Search Filter**: Check if search text appears in message or target

```rust
fn matches_filters(&self, msg: &LogMessage) -> bool {
    // Level filter
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
    
    // Search filter
    if self.search_filter.is_empty() {
        return true;
    }
    
    let search_lower = self.search_filter.to_lowercase();
    msg.message.to_lowercase().contains(&search_lower)
        || msg.target.to_lowercase().contains(&search_lower)
}
```

### Buffer Management

The buffer automatically maintains a maximum size:

```rust
pub fn push(&self, message: LogMessage) {
    if let Ok(mut buffer) = self.inner.lock() {
        if buffer.len() >= MAX_LOG_MESSAGES {
            buffer.pop_front(); // Remove oldest message
        }
        buffer.push_back(message);
    }
}
```

### UI Layout

The console panel is organized into distinct sections:

```
┌─────────────────────────────────────────┐
│ Toolbar (Filters, Clear, Auto-scroll)  │
├─────────────────────────────────────────┤
│ Search Bar                              │
├─────────────────────────────────────────┤
│                                         │
│  Log Messages (Scrollable)              │
│                                         │
│  [12:34:56.789] [INFO] praxis_core: ... │
│  [12:34:56.790] [WARN] praxis_ecs: ...  │
│  [12:34:56.791] [ERROR] praxis_gpu: ... │
│                                         │
├─────────────────────────────────────────┤
│ Command Input (> _)                     │
└─────────────────────────────────────────┘
```

## Dependencies

The console panel requires the following dependencies:

### praxis_editor/Cargo.toml
```toml
chrono = "0.4"  # For timestamp formatting
tracing = "0.1"  # For log capture
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

### praxis_utils
The `init_tracing_with_layer` function was added to support custom layers:

```rust
pub fn init_tracing_with_layer<L>(custom_layer: Option<L>) -> Result<()>
where
    L: tracing_subscriber::Layer<tracing_subscriber::Registry> + Send + Sync + 'static,
```

## Testing

Run the console demo to see all features in action:

```bash
cargo run --example console_demo
```

The demo:
- Generates sample logs at different levels
- Shows filtering in action
- Demonstrates search functionality
- Tests auto-scroll behavior

## Future Enhancements

Potential improvements for future versions:

1. **Command Execution**: Execute console commands to control the engine
2. **Log Export**: Save logs to file for debugging
3. **Regex Search**: More powerful search with regular expressions
4. **Log History**: Persistent log history across sessions
5. **Custom Filters**: Save and load custom filter configurations
6. **Performance Metrics**: Show performance data in console
7. **Stack Traces**: Display stack traces for error messages
8. **Log Grouping**: Collapse repeated messages
9. **Color Customization**: User-configurable colors for log levels
10. **Notification System**: Pop-up notifications for errors/warnings

## Comparison with Other Editors

The Praxis console panel provides features comparable to:

- **Unity Console**: Log filtering, search, clear, stack traces
- **Unreal Output Log**: Level filtering, search, auto-scroll
- **Godot Output**: Color coding, filtering, message counts
- **Visual Studio Output**: Search, clear, auto-scroll

## Performance Considerations

- **Buffer Limit**: 1000 messages maximum prevents unbounded growth
- **Lazy Rendering**: Only visible messages are rendered (scroll culling)
- **Efficient Filtering**: Filters applied once per frame, not per message
- **Minimal Allocations**: Reuses string buffers where possible
- **Lock Contention**: Mutex held only briefly during buffer operations

## Conclusion

The Console Panel implementation provides a professional-grade logging interface that integrates seamlessly with the Praxis engine. It captures all tracing events, provides powerful filtering and search, and maintains good performance even with high log volumes.
