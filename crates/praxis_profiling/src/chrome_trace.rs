//! Chrome tracing format export for performance analysis.
//!
//! This module exports profiling data in the Chrome Trace Event Format,
//! which can be viewed in chrome://tracing or tools like Perfetto.

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

/// Chrome trace event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ChromeTraceEventType {
    /// Duration begin event
    B,
    /// Duration end event
    E,
    /// Complete event (begin + end)
    X,
    /// Instant event
    I,
    /// Counter event
    C,
    /// Metadata event
    M,
}

/// A single event in Chrome trace format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeTraceEvent {
    /// Event name
    pub name: String,
    /// Event category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cat: Option<String>,
    /// Event type
    pub ph: ChromeTraceEventType,
    /// Timestamp in microseconds
    pub ts: u64,
    /// Duration in microseconds (for complete events)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dur: Option<u64>,
    /// Process ID
    pub pid: u32,
    /// Thread ID
    pub tid: u64,
    /// Additional arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Value>,
}

impl ChromeTraceEvent {
    /// Creates a new complete duration event.
    pub fn duration(
        name: String,
        category: String,
        start_time: Duration,
        duration: Duration,
        pid: u32,
        tid: u64,
    ) -> Self {
        Self {
            name,
            cat: Some(category),
            ph: ChromeTraceEventType::X,
            ts: start_time.as_micros() as u64,
            dur: Some(duration.as_micros() as u64),
            pid,
            tid,
            args: None,
        }
    }

    /// Creates a new instant event.
    pub fn instant(
        name: String,
        category: String,
        timestamp: Duration,
        pid: u32,
        tid: u64,
    ) -> Self {
        Self {
            name,
            cat: Some(category),
            ph: ChromeTraceEventType::I,
            ts: timestamp.as_micros() as u64,
            dur: None,
            pid,
            tid,
            args: None,
        }
    }

    /// Creates a counter event.
    pub fn counter(
        name: String,
        category: String,
        timestamp: Duration,
        value: f64,
        pid: u32,
        tid: u64,
    ) -> Self {
        let mut args = serde_json::Map::new();
        args.insert("value".to_string(), serde_json::json!(value));

        Self {
            name,
            cat: Some(category),
            ph: ChromeTraceEventType::C,
            ts: timestamp.as_micros() as u64,
            dur: None,
            pid,
            tid,
            args: Some(serde_json::Value::Object(args)),
        }
    }

    /// Adds an argument to this event.
    pub fn with_arg(mut self, key: String, value: serde_json::Value) -> Self {
        let args = self
            .args
            .get_or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

        if let Some(map) = args.as_object_mut() {
            map.insert(key, value);
        }

        self
    }
}

/// Chrome trace document containing all events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChromeTrace {
    /// Display time unit
    #[serde(rename = "displayTimeUnit")]
    pub display_time_unit: String,
    /// All trace events
    #[serde(rename = "traceEvents")]
    pub trace_events: Vec<ChromeTraceEvent>,
}

impl ChromeTrace {
    /// Creates a new empty Chrome trace.
    pub fn new() -> Self {
        Self {
            display_time_unit: "ms".to_string(),
            trace_events: Vec::new(),
        }
    }

    /// Adds an event to the trace.
    pub fn add_event(&mut self, event: ChromeTraceEvent) {
        self.trace_events.push(event);
    }

    /// Adds multiple events to the trace.
    pub fn add_events(&mut self, events: Vec<ChromeTraceEvent>) {
        self.trace_events.extend(events);
    }

    /// Saves the trace to a JSON file.
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

impl Default for ChromeTrace {
    fn default() -> Self {
        Self::new()
    }
}

/// Exports profiling data to Chrome trace format.
pub struct ChromeTraceExporter {
    /// The trace being built
    trace: ChromeTrace,
    /// Process ID for events
    pid: u32,
    /// Start time reference
    start_time: Instant,
}

impl ChromeTraceExporter {
    /// Creates a new Chrome trace exporter.
    pub fn new() -> Self {
        Self {
            trace: ChromeTrace::new(),
            pid: std::process::id(),
            start_time: Instant::now(),
        }
    }

    /// Adds a CPU scope timing to the trace.
    pub fn add_cpu_scope(
        &mut self,
        name: String,
        category: String,
        start_time: Instant,
        duration: Duration,
        thread_id: std::thread::ThreadId,
    ) {
        let timestamp = start_time.duration_since(self.start_time);
        let tid = thread_id_to_u64(thread_id);

        let event = ChromeTraceEvent::duration(name, category, timestamp, duration, self.pid, tid);

        self.trace.add_event(event);
    }

    /// Adds a GPU timing to the trace.
    pub fn add_gpu_timing(&mut self, name: String, start_ns: u64, duration_ns: u64) {
        let timestamp = Duration::from_nanos(start_ns);
        let duration = Duration::from_nanos(duration_ns);

        let event = ChromeTraceEvent::duration(
            name,
            "GPU".to_string(),
            timestamp,
            duration,
            self.pid,
            0, // Use thread ID 0 for GPU events
        );

        self.trace.add_event(event);
    }

    /// Adds a memory counter to the trace.
    pub fn add_memory_counter(&mut self, name: String, timestamp: Instant, bytes: usize) {
        let timestamp = timestamp.duration_since(self.start_time);

        let event = ChromeTraceEvent::counter(
            name,
            "Memory".to_string(),
            timestamp,
            bytes as f64,
            self.pid,
            0,
        );

        self.trace.add_event(event);
    }

    /// Adds a frame marker to the trace.
    pub fn add_frame_marker(&mut self, frame_number: u64, timestamp: Instant) {
        let timestamp = timestamp.duration_since(self.start_time);

        let event = ChromeTraceEvent::instant(
            format!("Frame {frame_number}"),
            "Frame".to_string(),
            timestamp,
            self.pid,
            0,
        );

        self.trace.add_event(event);
    }

    /// Adds metadata to the trace.
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        let mut event = ChromeTraceEvent {
            name: key.clone(),
            cat: None,
            ph: ChromeTraceEventType::M,
            ts: 0,
            dur: None,
            pid: self.pid,
            tid: 0,
            args: None,
        };

        event = event.with_arg(key, value);
        self.trace.add_event(event);
    }

    /// Saves the trace to a file.
    pub fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        self.trace.save_to_file(path)
    }

    /// Returns the accumulated trace.
    pub fn trace(&self) -> &ChromeTrace {
        &self.trace
    }

    /// Consumes the exporter and returns the trace.
    pub fn into_trace(self) -> ChromeTrace {
        self.trace
    }
}

impl Default for ChromeTraceExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts a `ThreadId` to a `u64` for Chrome trace format.
fn thread_id_to_u64(thread_id: std::thread::ThreadId) -> u64 {
    // This is a bit of a hack, but ThreadId doesn't expose its inner value
    // We use the debug format which includes the numeric ID
    let debug_str = format!("{thread_id:?}");
    let id_str = debug_str
        .trim_start_matches("ThreadId(")
        .trim_end_matches(')');
    id_str.parse().unwrap_or(0)
}
