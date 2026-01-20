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
pub enum ChromeTraceEventType {
    /// Duration begin event
    B,
    /// Duration end event
    E,
    /// Complete event (begin + end)
    X,
    /// Instant event
    #[serde(rename = "i")]
    Instant,
    /// Counter event
    C,
    /// Metadata event
    #[serde(rename = "M")]
    Metadata,
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
    /// Scope for instant events ("g" = global, "p" = process, "t" = thread)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<String>,
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
            s: None,
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
            ph: ChromeTraceEventType::Instant,
            ts: timestamp.as_micros() as u64,
            dur: None,
            pid,
            tid,
            s: Some("t".to_string()), // Thread scope for instant events
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
        args.insert(name.clone(), serde_json::json!(value));

        Self {
            name,
            cat: Some(category),
            ph: ChromeTraceEventType::C,
            ts: timestamp.as_micros() as u64,
            dur: None,
            pid,
            tid,
            s: None,
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

    /// Adds a generic counter to the trace with a custom category.
    pub fn add_counter(&mut self, name: String, category: String, timestamp: Instant, value: f64) {
        let timestamp = timestamp.duration_since(self.start_time);

        let event = ChromeTraceEvent::counter(name, category, timestamp, value, self.pid, 0);

        self.trace.add_event(event);
    }

    /// Adds multiple counters to the trace at the same timestamp.
    ///
    /// This is useful for recording related metrics together (e.g., all rendering stats).
    pub fn add_counters(&mut self, counters: Vec<(String, String, f64)>, timestamp: Instant) {
        for (name, category, value) in counters {
            self.add_counter(name, category, timestamp, value);
        }
    }

    /// Adds rendering statistics as counter events.
    ///
    /// This method exports comprehensive rendering metrics to the trace timeline:
    /// - **Culling Efficiency**: Percentage of objects successfully culled
    /// - **Draw Call Reduction**: How many draw calls were saved by culling
    /// - **Visible Objects**: Number of objects actually rendered
    /// - **Frustum Culled**: Number of objects culled by frustum test
    /// - **Occlusion Culled**: Number of objects culled by occlusion test
    /// - **LOD Distribution**: Percentage of objects at each LOD level
    /// - **Streaming Queue**: Number of meshes waiting to be loaded
    ///
    /// All metrics are recorded as counter events in the "Rendering" category,
    /// allowing visualization of rendering performance over time in chrome://tracing.
    ///
    /// # Arguments
    ///
    /// * `stats` - RenderStats snapshot from praxis_graphics
    /// * `timestamp` - Timestamp for these metrics (typically frame start time)
    #[cfg(feature = "graphics_integration")]
    pub fn add_render_stats(&mut self, stats: &praxis_graphics::RenderStats, timestamp: Instant) {
        let ts = timestamp.duration_since(self.start_time);

        // Core culling metrics
        self.trace.add_event(ChromeTraceEvent::counter(
            "Culling Efficiency %".to_string(),
            "Rendering".to_string(),
            ts,
            stats.culling_efficiency() as f64,
            self.pid,
            0,
        ));

        // Object counts
        self.trace.add_event(ChromeTraceEvent::counter(
            "Total Objects".to_string(),
            "Rendering".to_string(),
            ts,
            stats.total_objects as f64,
            self.pid,
            0,
        ));

        self.trace.add_event(ChromeTraceEvent::counter(
            "Visible Objects".to_string(),
            "Rendering".to_string(),
            ts,
            stats.visible_objects as f64,
            self.pid,
            0,
        ));

        self.trace.add_event(ChromeTraceEvent::counter(
            "Frustum Culled".to_string(),
            "Rendering".to_string(),
            ts,
            stats.frustum_culled as f64,
            self.pid,
            0,
        ));

        self.trace.add_event(ChromeTraceEvent::counter(
            "Occlusion Culled".to_string(),
            "Rendering".to_string(),
            ts,
            stats.occlusion_culled as f64,
            self.pid,
            0,
        ));

        // Draw call metrics
        self.trace.add_event(ChromeTraceEvent::counter(
            "Draw Calls".to_string(),
            "Rendering".to_string(),
            ts,
            stats.draw_calls as f64,
            self.pid,
            0,
        ));

        let draw_call_reduction = stats.total_objects.saturating_sub(stats.draw_calls);
        self.trace.add_event(ChromeTraceEvent::counter(
            "Draw Call Reduction".to_string(),
            "Rendering".to_string(),
            ts,
            draw_call_reduction as f64,
            self.pid,
            0,
        ));

        // Descriptor allocations
        self.trace.add_event(ChromeTraceEvent::counter(
            "Descriptor Allocations".to_string(),
            "Rendering".to_string(),
            ts,
            stats.descriptor_allocations as f64,
            self.pid,
            0,
        ));

        // LOD distribution (percentage per level)
        let lod_distribution = stats.lod_distribution_percentages();
        for (level, percentage) in lod_distribution {
            self.trace.add_event(ChromeTraceEvent::counter(
                format!("LOD Level {} %", level),
                "Rendering/LOD".to_string(),
                ts,
                percentage as f64,
                self.pid,
                0,
            ));
        }

        // Streaming metrics
        self.trace.add_event(ChromeTraceEvent::counter(
            "Streaming Queue Depth".to_string(),
            "Rendering".to_string(),
            ts,
            stats.streaming_queue_depth as f64,
            self.pid,
            0,
        ));
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
        let mut args = serde_json::Map::new();
        args.insert(key.clone(), value);

        let event = ChromeTraceEvent {
            name: key,
            cat: None,
            ph: ChromeTraceEventType::Metadata,
            ts: 0,
            dur: None,
            pid: self.pid,
            tid: 0,
            s: None,
            args: Some(serde_json::Value::Object(args)),
        };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_event_format() {
        let event = ChromeTraceEvent::duration(
            "test_scope".to_string(),
            "Test".to_string(),
            Duration::from_micros(1000),
            Duration::from_micros(500),
            123,
            456,
        );

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["name"], "test_scope");
        assert_eq!(json["cat"], "Test");
        assert_eq!(json["ph"], "X");
        assert_eq!(json["ts"], 1000);
        assert_eq!(json["dur"], 500);
        assert_eq!(json["pid"], 123);
        assert_eq!(json["tid"], 456);
        assert!(json["s"].is_null());
    }

    #[test]
    fn test_instant_event_format() {
        let event = ChromeTraceEvent::instant(
            "frame_marker".to_string(),
            "Frame".to_string(),
            Duration::from_micros(2000),
            123,
            456,
        );

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["name"], "frame_marker");
        assert_eq!(json["cat"], "Frame");
        assert_eq!(json["ph"], "i");
        assert_eq!(json["ts"], 2000);
        assert!(json["dur"].is_null());
        assert_eq!(json["pid"], 123);
        assert_eq!(json["tid"], 456);
        assert_eq!(json["s"], "t");
    }

    #[test]
    fn test_counter_event_format() {
        let event = ChromeTraceEvent::counter(
            "Memory".to_string(),
            "Memory".to_string(),
            Duration::from_micros(3000),
            1024.0,
            123,
            0,
        );

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["name"], "Memory");
        assert_eq!(json["cat"], "Memory");
        assert_eq!(json["ph"], "C");
        assert_eq!(json["ts"], 3000);
        assert!(json["dur"].is_null());
        assert_eq!(json["pid"], 123);
        assert_eq!(json["tid"], 0);
        assert_eq!(json["args"]["Memory"], 1024.0);
    }

    #[test]
    fn test_metadata_event_format() {
        let mut exporter = ChromeTraceExporter::new();
        exporter.add_metadata("process_name".to_string(), serde_json::json!("TestProcess"));

        let trace = exporter.trace();
        assert_eq!(trace.trace_events.len(), 1);

        let json = serde_json::to_value(&trace.trace_events[0]).unwrap();
        assert_eq!(json["name"], "process_name");
        assert_eq!(json["ph"], "M");
        assert_eq!(json["args"]["process_name"], "TestProcess");
    }

    #[test]
    fn test_chrome_trace_serialization() {
        let mut trace = ChromeTrace::new();

        // Add a duration event
        trace.add_event(ChromeTraceEvent::duration(
            "test".to_string(),
            "Test".to_string(),
            Duration::from_micros(100),
            Duration::from_micros(50),
            1,
            2,
        ));

        // Add an instant event
        trace.add_event(ChromeTraceEvent::instant(
            "marker".to_string(),
            "Marker".to_string(),
            Duration::from_micros(200),
            1,
            2,
        ));

        // Serialize to JSON
        let json = serde_json::to_string(&trace).unwrap();
        assert!(json.contains("\"displayTimeUnit\":\"ms\""));
        assert!(json.contains("\"traceEvents\""));
        assert!(json.contains("\"ph\":\"X\""));
        assert!(json.contains("\"ph\":\"i\""));
        assert!(json.contains("\"s\":\"t\""));
    }

    #[test]
    #[cfg(feature = "graphics_integration")]
    fn test_add_render_stats() {
        use praxis_graphics::RenderStats;
        use std::time::Instant;

        let mut exporter = ChromeTraceExporter::new();
        let start_time = Instant::now();

        let stats = RenderStats {
            frame_number: 1,
            total_objects: 1000,
            visible_objects: 250,
            frustum_culled: 650,
            occlusion_culled: 100,
            draw_calls: 120,
            descriptor_allocations: 15,
            active_lod_levels: vec![(0, 50), (1, 150), (2, 50)],
            streaming_queue_depth: 5,
        };

        exporter.add_render_stats(&stats, start_time);

        let trace = exporter.trace();

        // Should have base counters (8) + LOD levels (3) = 11 events
        assert_eq!(trace.trace_events.len(), 11);

        // Verify culling efficiency counter exists
        let culling_eff = trace
            .trace_events
            .iter()
            .find(|e| e.name == "Culling Efficiency %");
        assert!(culling_eff.is_some());
        let culling_eff = culling_eff.unwrap();
        assert_eq!(culling_eff.cat, Some("Rendering".to_string()));

        // Verify counter value is correct (75% culling efficiency)
        if let Some(args) = &culling_eff.args {
            if let Some(value) = args.get("Culling Efficiency %") {
                assert_eq!(value.as_f64().unwrap(), 75.0);
            }
        }

        // Verify LOD counter exists
        let lod_0 = trace
            .trace_events
            .iter()
            .find(|e| e.name == "LOD Level 0 %");
        assert!(lod_0.is_some());
        assert_eq!(lod_0.unwrap().cat, Some("Rendering/LOD".to_string()));

        // Verify draw call reduction
        let draw_call_reduction = trace
            .trace_events
            .iter()
            .find(|e| e.name == "Draw Call Reduction");
        assert!(draw_call_reduction.is_some());
        if let Some(args) = &draw_call_reduction.unwrap().args {
            if let Some(value) = args.get("Draw Call Reduction") {
                // total_objects (1000) - draw_calls (120) = 880
                assert_eq!(value.as_f64().unwrap(), 880.0);
            }
        }
    }

    #[test]
    fn test_add_counter() {
        use std::time::Instant;

        let mut exporter = ChromeTraceExporter::new();
        let timestamp = Instant::now();

        exporter.add_counter(
            "Test Counter".to_string(),
            "Test Category".to_string(),
            timestamp,
            42.5,
        );

        let trace = exporter.trace();
        assert_eq!(trace.trace_events.len(), 1);

        let event = &trace.trace_events[0];
        assert_eq!(event.name, "Test Counter");
        assert_eq!(event.cat, Some("Test Category".to_string()));

        if let Some(args) = &event.args {
            if let Some(value) = args.get("Test Counter") {
                assert_eq!(value.as_f64().unwrap(), 42.5);
            }
        }
    }

    #[test]
    fn test_add_counters() {
        use std::time::Instant;

        let mut exporter = ChromeTraceExporter::new();
        let timestamp = Instant::now();

        exporter.add_counters(
            vec![
                ("Counter 1".to_string(), "Category".to_string(), 1.0),
                ("Counter 2".to_string(), "Category".to_string(), 2.0),
                ("Counter 3".to_string(), "Category".to_string(), 3.0),
            ],
            timestamp,
        );

        let trace = exporter.trace();
        assert_eq!(trace.trace_events.len(), 3);

        for (i, event) in trace.trace_events.iter().enumerate() {
            assert_eq!(event.name, format!("Counter {}", i + 1));
            assert_eq!(event.cat, Some("Category".to_string()));
        }
    }
}
