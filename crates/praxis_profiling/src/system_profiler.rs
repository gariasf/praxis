//! ECS system profiling and bottleneck identification.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Statistics for a single system.
#[derive(Debug, Clone)]
pub struct SystemStats {
    /// System name
    pub name: String,
    /// Total execution time
    pub total_time: Duration,
    /// Number of times executed
    pub execution_count: u64,
    /// Average execution time
    pub avg_time: Duration,
    /// Minimum execution time
    pub min_time: Duration,
    /// Maximum execution time
    pub max_time: Duration,
    /// Last execution time
    pub last_time: Duration,
    /// Percentage of total frame time
    pub frame_percentage: f32,
}

/// Information about a bottleneck.
#[derive(Debug, Clone)]
pub struct BottleneckInfo {
    /// Name of the system or entity
    pub name: String,
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Average time spent
    pub avg_time: Duration,
    /// Percentage of total frame time
    pub percentage: f32,
    /// Severity (0.0 - 1.0)
    pub severity: f32,
    /// Recommendation for fixing
    pub recommendation: String,
}

/// Type of bottleneck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BottleneckType {
    /// System taking too long
    SlowSystem,
    /// Entity with expensive components
    ExpensiveEntity,
    /// Too many entities in query
    LargeQuery,
    /// Frequent system execution
    FrequentExecution,
}

impl BottleneckType {
    /// Returns a description of this bottleneck type.
    pub fn description(&self) -> &'static str {
        match self {
            Self::SlowSystem => "System execution time is high",
            Self::ExpensiveEntity => "Entity has expensive component operations",
            Self::LargeQuery => "System queries many entities",
            Self::FrequentExecution => "System executes very frequently",
        }
    }
}

/// Profiles ECS systems and identifies bottlenecks.
pub struct SystemProfiler {
    /// System statistics by name
    system_stats: Arc<Mutex<HashMap<String, SystemStats>>>,
    /// Active system measurements
    active_systems: Arc<Mutex<HashMap<String, Instant>>>,
    /// Total frame time for percentage calculations
    total_frame_time: Arc<Mutex<Duration>>,
    /// Bottleneck detection threshold (percentage of frame time)
    bottleneck_threshold: f32,
}

impl SystemProfiler {
    /// Creates a new system profiler.
    ///
    /// # Arguments
    ///
    /// * `bottleneck_threshold` - Percentage of frame time (0.0-1.0) to consider a bottleneck
    pub fn new(bottleneck_threshold: f32) -> Self {
        Self {
            system_stats: Arc::new(Mutex::new(HashMap::new())),
            active_systems: Arc::new(Mutex::new(HashMap::new())),
            total_frame_time: Arc::new(Mutex::new(Duration::ZERO)),
            bottleneck_threshold,
        }
    }

    /// Begins profiling a system.
    pub fn begin_system(&self, name: impl Into<String>) {
        let name = name.into();
        let mut active = self.active_systems.lock();
        active.insert(name, Instant::now());
    }

    /// Ends profiling a system.
    pub fn end_system(&self, name: &str) {
        let mut active = self.active_systems.lock();
        if let Some(start_time) = active.remove(name) {
            let duration = start_time.elapsed();
            drop(active);

            let mut stats = self.system_stats.lock();
            let stat = stats.entry(name.to_string()).or_insert_with(|| SystemStats {
                name: name.to_string(),
                total_time: Duration::ZERO,
                execution_count: 0,
                avg_time: Duration::ZERO,
                min_time: Duration::MAX,
                max_time: Duration::ZERO,
                last_time: Duration::ZERO,
                frame_percentage: 0.0,
            });

            stat.total_time += duration;
            stat.execution_count += 1;
            stat.last_time = duration;

            if duration < stat.min_time {
                stat.min_time = duration;
            }
            if duration > stat.max_time {
                stat.max_time = duration;
            }

            // Update rolling average
            let count = stat.execution_count as f64;
            stat.avg_time = Duration::from_secs_f64(
                stat.avg_time.as_secs_f64() * ((count - 1.0) / count)
                    + duration.as_secs_f64() / count,
            );
        }
    }

    /// Updates frame time for percentage calculations.
    pub fn set_frame_time(&self, frame_time: Duration) {
        *self.total_frame_time.lock() = frame_time;

        // Update percentages for all systems
        let mut stats = self.system_stats.lock();
        for stat in stats.values_mut() {
            if frame_time.as_secs_f32() > 0.0 {
                stat.frame_percentage =
                    (stat.avg_time.as_secs_f32() / frame_time.as_secs_f32()) * 100.0;
            }
        }
    }

    /// Returns statistics for all systems.
    pub fn system_statistics(&self) -> Vec<SystemStats> {
        let stats = self.system_stats.lock();
        let mut result: Vec<_> = stats.values().cloned().collect();
        result.sort_by(|a, b| b.avg_time.cmp(&a.avg_time));
        result
    }

    /// Returns statistics for a specific system.
    pub fn system_stat(&self, name: &str) -> Option<SystemStats> {
        self.system_stats.lock().get(name).cloned()
    }

    /// Identifies bottlenecks in the current frame.
    pub fn identify_bottlenecks(&self) -> Vec<BottleneckInfo> {
        let stats = self.system_stats.lock();
        let frame_time = *self.total_frame_time.lock();

        if frame_time.as_secs_f32() == 0.0 {
            return Vec::new();
        }

        let mut bottlenecks = Vec::new();

        for stat in stats.values() {
            let percentage = stat.frame_percentage / 100.0;

            if percentage >= self.bottleneck_threshold {
                let severity = (percentage - self.bottleneck_threshold)
                    / (1.0 - self.bottleneck_threshold);

                let recommendation = if percentage > 0.3 {
                    "Consider splitting into smaller systems or optimizing algorithm"
                } else if percentage > 0.2 {
                    "Profile internal operations to find hotspots"
                } else {
                    "Monitor for further increases"
                };

                bottlenecks.push(BottleneckInfo {
                    name: stat.name.clone(),
                    bottleneck_type: BottleneckType::SlowSystem,
                    avg_time: stat.avg_time,
                    percentage: stat.frame_percentage,
                    severity: severity.clamp(0.0, 1.0),
                    recommendation: recommendation.to_string(),
                });
            }
        }

        bottlenecks.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());
        bottlenecks
    }

    /// Resets all statistics.
    pub fn reset(&self) {
        self.system_stats.lock().clear();
        self.active_systems.lock().clear();
        *self.total_frame_time.lock() = Duration::ZERO;
    }

    /// Returns the top N slowest systems.
    pub fn top_slowest_systems(&self, n: usize) -> Vec<SystemStats> {
        let stats = self.system_statistics();
        stats.into_iter().take(n).collect()
    }
}

impl Default for SystemProfiler {
    fn default() -> Self {
        Self::new(0.15) // Default to 15% threshold
    }
}

/// RAII guard for profiling a system.
pub struct SystemProfileScope<'a> {
    profiler: &'a SystemProfiler,
    name: String,
}

impl<'a> SystemProfileScope<'a> {
    /// Creates a new system profile scope.
    #[allow(dead_code)]
    pub fn new(profiler: &'a SystemProfiler, name: impl Into<String>) -> Self {
        let name = name.into();
        profiler.begin_system(&name);
        Self { profiler, name }
    }
}

impl<'a> Drop for SystemProfileScope<'a> {
    fn drop(&mut self) {
        self.profiler.end_system(&self.name);
    }
}
