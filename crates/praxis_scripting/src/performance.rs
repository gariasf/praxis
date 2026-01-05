//! Performance monitoring for script execution.

use parking_lot::RwLock;
use praxis_utils::warn;
use std::collections::HashMap;
use std::time::Duration;

/// Statistics for a single script or function.
#[derive(Debug, Clone)]
pub struct ScriptStats {
    /// Name of the script
    pub script_name: String,

    /// Name of the function (if applicable)
    pub function_name: Option<String>,

    /// Total number of executions
    pub execution_count: u64,

    /// Total execution time
    pub total_time: Duration,

    /// Average execution time
    pub average_time: Duration,

    /// Minimum execution time
    pub min_time: Duration,

    /// Maximum execution time
    pub max_time: Duration,

    /// Number of times execution exceeded warning threshold
    pub warning_count: u64,
}

impl ScriptStats {
    fn new(script_name: String, function_name: Option<String>) -> Self {
        Self {
            script_name,
            function_name,
            execution_count: 0,
            total_time: Duration::ZERO,
            average_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            warning_count: 0,
        }
    }

    fn record(&mut self, duration: Duration, warning_threshold_ms: u64) {
        self.execution_count += 1;
        self.total_time += duration;
        self.average_time = self.total_time / self.execution_count as u32;

        if duration < self.min_time {
            self.min_time = duration;
        }

        if duration > self.max_time {
            self.max_time = duration;
        }

        if duration.as_millis() as u64 > warning_threshold_ms {
            self.warning_count += 1;
        }
    }
}

/// Monitors script performance and tracks execution statistics.
pub struct ScriptPerformanceMonitor {
    stats: RwLock<HashMap<String, ScriptStats>>,
    warning_threshold_ms: u64,
}

impl ScriptPerformanceMonitor {
    /// Creates a new performance monitor.
    pub fn new(warning_threshold_ms: u64) -> Self {
        Self {
            stats: RwLock::new(HashMap::new()),
            warning_threshold_ms,
        }
    }

    /// Records a script execution.
    pub fn record_execution(&self, script_name: &str, function_name: &str, duration: Duration) {
        let key = format!("{script_name}::{function_name}");

        let mut stats = self.stats.write();
        let script_stats = stats.entry(key.clone()).or_insert_with(|| {
            ScriptStats::new(script_name.to_string(), Some(function_name.to_string()))
        });

        script_stats.record(duration, self.warning_threshold_ms);

        if duration.as_millis() as u64 > self.warning_threshold_ms {
            warn!(
                "Script '{}::{}' took {:.2}ms (threshold: {}ms)",
                script_name,
                function_name,
                duration.as_secs_f64() * 1000.0,
                self.warning_threshold_ms
            );
        }
    }

    /// Gets statistics for a specific script function.
    pub fn get_stats(&self, script_name: &str, function_name: &str) -> Option<ScriptStats> {
        let key = format!("{script_name}::{function_name}");
        self.stats.read().get(&key).cloned()
    }

    /// Gets all statistics.
    pub fn get_all_stats(&self) -> Vec<ScriptStats> {
        self.stats.read().values().cloned().collect()
    }

    /// Gets statistics sorted by total execution time (descending).
    pub fn get_slowest_scripts(&self) -> Vec<ScriptStats> {
        let mut stats: Vec<ScriptStats> = self.get_all_stats();
        stats.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        stats
    }

    /// Gets statistics sorted by average execution time (descending).
    pub fn get_slowest_average(&self) -> Vec<ScriptStats> {
        let mut stats: Vec<ScriptStats> = self.get_all_stats();
        stats.sort_by(|a, b| b.average_time.cmp(&a.average_time));
        stats
    }

    /// Resets all statistics.
    pub fn reset(&self) {
        self.stats.write().clear();
    }

    /// Gets the total number of tracked scripts/functions.
    pub fn tracked_count(&self) -> usize {
        self.stats.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_execution() {
        let monitor = ScriptPerformanceMonitor::new(10);

        let duration = Duration::from_millis(5);
        monitor.record_execution("test_script", "update", duration);

        let stats = monitor.get_stats("test_script", "update").unwrap();
        assert_eq!(stats.execution_count, 1);
        assert_eq!(stats.average_time, duration);
    }

    #[test]
    fn test_multiple_executions() {
        let monitor = ScriptPerformanceMonitor::new(10);

        monitor.record_execution("test", "func", Duration::from_millis(5));
        monitor.record_execution("test", "func", Duration::from_millis(3));
        monitor.record_execution("test", "func", Duration::from_millis(7));

        let stats = monitor.get_stats("test", "func").unwrap();
        assert_eq!(stats.execution_count, 3);
        assert_eq!(stats.min_time, Duration::from_millis(3));
        assert_eq!(stats.max_time, Duration::from_millis(7));
    }

    #[test]
    fn test_warning_threshold() {
        let monitor = ScriptPerformanceMonitor::new(5);

        monitor.record_execution("test", "slow", Duration::from_millis(10));
        monitor.record_execution("test", "fast", Duration::from_millis(2));

        let slow_stats = monitor.get_stats("test", "slow").unwrap();
        let fast_stats = monitor.get_stats("test", "fast").unwrap();

        assert_eq!(slow_stats.warning_count, 1);
        assert_eq!(fast_stats.warning_count, 0);
    }

    #[test]
    fn test_get_slowest() {
        let monitor = ScriptPerformanceMonitor::new(100);

        monitor.record_execution("fast", "func", Duration::from_millis(1));
        monitor.record_execution("medium", "func", Duration::from_millis(5));
        monitor.record_execution("slow", "func", Duration::from_millis(10));

        let slowest = monitor.get_slowest_scripts();
        assert_eq!(slowest[0].script_name, "slow");
        assert_eq!(slowest[1].script_name, "medium");
        assert_eq!(slowest[2].script_name, "fast");
    }

    #[test]
    fn test_reset() {
        let monitor = ScriptPerformanceMonitor::new(10);

        monitor.record_execution("test", "func", Duration::from_millis(5));
        assert_eq!(monitor.tracked_count(), 1);

        monitor.reset();
        assert_eq!(monitor.tracked_count(), 0);
    }
}
