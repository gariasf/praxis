//! Network profiler for monitoring bandwidth and latency.

use crate::MessageType;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// Bandwidth metrics.
#[derive(Debug, Clone)]
pub struct BandwidthMetrics {
    /// Total bytes sent
    pub bytes_sent: u64,

    /// Total bytes received
    pub bytes_received: u64,

    /// Bytes sent per second
    pub send_rate: f32,

    /// Bytes received per second
    pub receive_rate: f32,

    /// Peak send rate
    pub peak_send_rate: f32,

    /// Peak receive rate
    pub peak_receive_rate: f32,

    /// Bytes sent per message type
    pub bytes_by_type: HashMap<MessageType, u64>,
}

impl Default for BandwidthMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl BandwidthMetrics {
    /// Creates new bandwidth metrics.
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            send_rate: 0.0,
            receive_rate: 0.0,
            peak_send_rate: 0.0,
            peak_receive_rate: 0.0,
            bytes_by_type: HashMap::new(),
        }
    }
}

/// Latency metrics.
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    /// Current round-trip time in milliseconds
    pub rtt_ms: f32,

    /// Minimum RTT
    pub min_rtt_ms: f32,

    /// Maximum RTT
    pub max_rtt_ms: f32,

    /// Average RTT
    pub avg_rtt_ms: f32,

    /// Jitter (variance in latency)
    pub jitter_ms: f32,

    /// Packet loss percentage
    pub packet_loss: f32,
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyMetrics {
    /// Creates new latency metrics.
    pub fn new() -> Self {
        Self {
            rtt_ms: 0.0,
            min_rtt_ms: f32::MAX,
            max_rtt_ms: 0.0,
            avg_rtt_ms: 0.0,
            jitter_ms: 0.0,
            packet_loss: 0.0,
        }
    }
}

/// Combined profiler statistics.
#[derive(Debug, Clone)]
pub struct ProfilerStats {
    /// Bandwidth metrics
    pub bandwidth: BandwidthMetrics,

    /// Latency metrics
    pub latency: LatencyMetrics,

    /// Number of active connections
    pub active_connections: usize,

    /// Uptime in seconds
    pub uptime_seconds: f32,
}

impl Default for ProfilerStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfilerStats {
    /// Creates new profiler stats.
    pub fn new() -> Self {
        Self {
            bandwidth: BandwidthMetrics::new(),
            latency: LatencyMetrics::new(),
            active_connections: 0,
            uptime_seconds: 0.0,
        }
    }
}

/// Sample for rate calculation.
#[derive(Debug, Clone, Copy)]
struct RateSample {
    /// Timestamp
    timestamp: f32,

    /// Bytes in this sample
    bytes: u64,
}

/// Network profiler.
pub struct NetworkProfiler {
    /// Bandwidth metrics
    bandwidth: Arc<RwLock<BandwidthMetrics>>,

    /// Latency metrics
    latency: Arc<RwLock<LatencyMetrics>>,

    /// Send rate samples (for calculating bytes per second)
    send_samples: Arc<RwLock<VecDeque<RateSample>>>,

    /// Receive rate samples
    receive_samples: Arc<RwLock<VecDeque<RateSample>>>,

    /// Latency samples (for jitter calculation)
    latency_samples: Arc<RwLock<VecDeque<f32>>>,

    /// Current time
    current_time: Arc<RwLock<f32>>,

    /// Sample window in seconds
    sample_window: f32,

    /// Maximum samples to keep
    max_samples: usize,
}

impl NetworkProfiler {
    /// Creates a new network profiler.
    pub fn new() -> Self {
        Self {
            bandwidth: Arc::new(RwLock::new(BandwidthMetrics::new())),
            latency: Arc::new(RwLock::new(LatencyMetrics::new())),
            send_samples: Arc::new(RwLock::new(VecDeque::new())),
            receive_samples: Arc::new(RwLock::new(VecDeque::new())),
            latency_samples: Arc::new(RwLock::new(VecDeque::new())),
            current_time: Arc::new(RwLock::new(0.0)),
            sample_window: 1.0,
            max_samples: 100,
        }
    }

    /// Records bytes sent.
    pub fn record_sent(&self, bytes: usize) {
        let current_time = *self.current_time.read();

        let mut bandwidth = self.bandwidth.write();
        bandwidth.bytes_sent += bytes as u64;

        let mut samples = self.send_samples.write();
        samples.push_back(RateSample {
            timestamp: current_time,
            bytes: bytes as u64,
        });

        // Remove old samples
        let cutoff_time = current_time - self.sample_window;
        while let Some(sample) = samples.front() {
            if sample.timestamp < cutoff_time {
                samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Records bytes received.
    pub fn record_received(&self, bytes: usize) {
        let current_time = *self.current_time.read();

        let mut bandwidth = self.bandwidth.write();
        bandwidth.bytes_received += bytes as u64;

        let mut samples = self.receive_samples.write();
        samples.push_back(RateSample {
            timestamp: current_time,
            bytes: bytes as u64,
        });

        // Remove old samples
        let cutoff_time = current_time - self.sample_window;
        while let Some(sample) = samples.front() {
            if sample.timestamp < cutoff_time {
                samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Records bytes sent for a specific message type.
    pub fn record_sent_by_type(&self, message_type: MessageType, bytes: usize) {
        self.record_sent(bytes);

        let mut bandwidth = self.bandwidth.write();
        *bandwidth.bytes_by_type.entry(message_type).or_insert(0) += bytes as u64;
    }

    /// Records latency measurement.
    pub fn record_latency(&self, rtt_ms: f32) {
        let mut latency = self.latency.write();

        latency.rtt_ms = rtt_ms;
        latency.min_rtt_ms = latency.min_rtt_ms.min(rtt_ms);
        latency.max_rtt_ms = latency.max_rtt_ms.max(rtt_ms);

        // Update latency samples for jitter calculation
        let mut samples = self.latency_samples.write();
        samples.push_back(rtt_ms);

        if samples.len() > self.max_samples {
            samples.pop_front();
        }

        // Calculate average
        if !samples.is_empty() {
            let sum: f32 = samples.iter().sum();
            latency.avg_rtt_ms = sum / samples.len() as f32;

            // Calculate jitter (standard deviation)
            let variance: f32 = samples
                .iter()
                .map(|&sample| {
                    let diff = sample - latency.avg_rtt_ms;
                    diff * diff
                })
                .sum::<f32>()
                / samples.len() as f32;

            latency.jitter_ms = variance.sqrt();
        }
    }

    /// Updates the profiler (should be called every frame).
    pub fn update(&self, delta_time: f32) {
        let mut current_time = self.current_time.write();
        *current_time += delta_time;

        // Calculate send rate
        let send_samples = self.send_samples.read();
        let total_send_bytes: u64 = send_samples.iter().map(|s| s.bytes).sum();
        let send_rate = total_send_bytes as f32 / self.sample_window;

        // Calculate receive rate
        let receive_samples = self.receive_samples.read();
        let total_receive_bytes: u64 = receive_samples.iter().map(|s| s.bytes).sum();
        let receive_rate = total_receive_bytes as f32 / self.sample_window;

        // Update bandwidth metrics
        let mut bandwidth = self.bandwidth.write();
        bandwidth.send_rate = send_rate;
        bandwidth.receive_rate = receive_rate;
        bandwidth.peak_send_rate = bandwidth.peak_send_rate.max(send_rate);
        bandwidth.peak_receive_rate = bandwidth.peak_receive_rate.max(receive_rate);
    }

    /// Gets current profiler statistics.
    pub fn get_stats(&self) -> ProfilerStats {
        ProfilerStats {
            bandwidth: self.bandwidth.read().clone(),
            latency: self.latency.read().clone(),
            active_connections: 0,
            uptime_seconds: *self.current_time.read(),
        }
    }

    /// Resets all statistics.
    pub fn reset(&self) {
        *self.bandwidth.write() = BandwidthMetrics::new();
        *self.latency.write() = LatencyMetrics::new();
        self.send_samples.write().clear();
        self.receive_samples.write().clear();
        self.latency_samples.write().clear();
        *self.current_time.write() = 0.0;
    }

    /// Gets bandwidth metrics.
    pub fn bandwidth(&self) -> BandwidthMetrics {
        self.bandwidth.read().clone()
    }

    /// Gets latency metrics.
    pub fn latency(&self) -> LatencyMetrics {
        self.latency.read().clone()
    }
}

impl Default for NetworkProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_creation() {
        let profiler = NetworkProfiler::new();
        let stats = profiler.get_stats();
        assert_eq!(stats.bandwidth.bytes_sent, 0);
        assert_eq!(stats.bandwidth.bytes_received, 0);
    }

    #[test]
    fn test_record_sent() {
        let profiler = NetworkProfiler::new();
        profiler.record_sent(100);

        let stats = profiler.get_stats();
        assert_eq!(stats.bandwidth.bytes_sent, 100);
    }

    #[test]
    fn test_record_received() {
        let profiler = NetworkProfiler::new();
        profiler.record_received(200);

        let stats = profiler.get_stats();
        assert_eq!(stats.bandwidth.bytes_received, 200);
    }

    #[test]
    fn test_record_latency() {
        let profiler = NetworkProfiler::new();

        profiler.record_latency(50.0);
        profiler.record_latency(60.0);
        profiler.record_latency(55.0);

        let stats = profiler.get_stats();
        assert_eq!(stats.latency.rtt_ms, 55.0);
        assert_eq!(stats.latency.min_rtt_ms, 50.0);
        assert_eq!(stats.latency.max_rtt_ms, 60.0);
        assert!((stats.latency.avg_rtt_ms - 55.0).abs() < 0.1);
    }

    #[test]
    fn test_update_rates() {
        let profiler = NetworkProfiler::new();

        profiler.record_sent(1000);
        profiler.update(0.5);

        let stats = profiler.get_stats();
        assert!(stats.bandwidth.send_rate > 0.0);
    }

    #[test]
    fn test_reset() {
        let profiler = NetworkProfiler::new();

        profiler.record_sent(100);
        profiler.record_received(200);
        profiler.record_latency(50.0);

        profiler.reset();

        let stats = profiler.get_stats();
        assert_eq!(stats.bandwidth.bytes_sent, 0);
        assert_eq!(stats.bandwidth.bytes_received, 0);
        assert_eq!(stats.uptime_seconds, 0.0);
    }

    #[test]
    fn test_bandwidth_by_type() {
        let profiler = NetworkProfiler::new();

        profiler.record_sent_by_type(MessageType::Replication, 100);
        profiler.record_sent_by_type(MessageType::Command, 50);

        let bandwidth = profiler.bandwidth();
        assert_eq!(
            *bandwidth
                .bytes_by_type
                .get(&MessageType::Replication)
                .unwrap(),
            100
        );
        assert_eq!(
            *bandwidth.bytes_by_type.get(&MessageType::Command).unwrap(),
            50
        );
    }
}
