# Exercise 02: Frame Time Profiler

**Difficulty**: 🟢 Beginner | **Estimated Time**: 1-2h | **Subsystem**: Core

## Overview

Build a lightweight profiler that tracks frame times and calculates FPS statistics. Essential for identifying performance issues and monitoring game performance.

## Learning Objectives

- Understand frame time vs FPS metrics
- Learn rolling average and percentile calculations
- Implement efficient ring buffer for time samples
- Detect performance spikes and stuttering

## Requirements

### Functional Requirements

1. **Frame Time Tracking**
   - Record frame time for each frame
   - Store last N frames (e.g., 100) in a ring buffer
   - Calculate current FPS from most recent frame

2. **Statistics Calculation**
   - Average FPS over window
   - Minimum/maximum frame times
   - 95th and 99th percentile frame times
   - Standard deviation

3. **Display**
   - Current FPS
   - Average FPS
   - Frame time graph (optional)

### Non-Functional Requirements

- **Performance**: Profiler overhead < 0.05ms per frame
- **Memory**: Fixed memory allocation (no dynamic allocation per frame)
- **Accuracy**: Time measurement precision to 0.1ms

## API Design

```rust
pub struct FrameProfiler {
    samples: RingBuffer<Duration>,
    window_size: usize,
}

impl FrameProfiler {
    pub fn new(window_size: usize) -> Self;
    pub fn record_frame(&mut self, frame_time: Duration);
    pub fn current_fps(&self) -> f32;
    pub fn average_fps(&self) -> f32;
    pub fn min_frame_time(&self) -> Duration;
    pub fn max_frame_time(&self) -> Duration;
    pub fn percentile(&self, p: f32) -> Duration;
    pub fn get_stats(&self) -> FrameStats;
}

pub struct FrameStats {
    pub current_fps: f32,
    pub average_fps: f32,
    pub min_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
}
```

## Validation Criteria

### Correctness
- [ ] Accurately calculates FPS from frame time
- [ ] Ring buffer correctly wraps around
- [ ] Percentile calculations are accurate
- [ ] Handles edge cases (empty buffer, single sample)

### Performance
- [ ] < 0.05ms overhead per frame
- [ ] No heap allocations after initialization
- [ ] Statistics calculation < 0.1ms

### Code Quality
- [ ] Efficient ring buffer implementation
- [ ] Clear documentation
- [ ] Unit tests for statistics

## Test Cases

```rust
#[test]
fn test_fps_calculation() {
    let mut profiler = FrameProfiler::new(100);
    profiler.record_frame(Duration::from_millis(16)); // ~60 FPS
    
    let fps = profiler.current_fps();
    assert!((fps - 60.0).abs() < 2.0);
}

#[test]
fn test_ring_buffer_wraparound() {
    let mut profiler = FrameProfiler::new(3);
    profiler.record_frame(Duration::from_millis(10));
    profiler.record_frame(Duration::from_millis(20));
    profiler.record_frame(Duration::from_millis(30));
    profiler.record_frame(Duration::from_millis(40)); // Wraps
    
    let max = profiler.max_frame_time();
    assert_eq!(max, Duration::from_millis(40));
}

#[test]
fn test_percentile_calculation() {
    let mut profiler = FrameProfiler::new(100);
    for i in 1..=100 {
        profiler.record_frame(Duration::from_millis(i));
    }
    
    let p50 = profiler.percentile(0.50);
    assert_eq!(p50, Duration::from_millis(50));
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::time::Duration;

pub struct FrameProfiler {
    samples: Vec<Duration>,
    head: usize,
    count: usize,
    capacity: usize,
}

impl FrameProfiler {
    pub fn new(window_size: usize) -> Self {
        Self {
            samples: vec![Duration::ZERO; window_size],
            head: 0,
            count: 0,
            capacity: window_size,
        }
    }
    
    pub fn record_frame(&mut self, frame_time: Duration) {
        self.samples[self.head] = frame_time;
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }
    
    pub fn current_fps(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let last_idx = if self.head == 0 {
            self.capacity - 1
        } else {
            self.head - 1
        };
        let frame_time = self.samples[last_idx].as_secs_f32();
        if frame_time > 0.0 {
            1.0 / frame_time
        } else {
            0.0
        }
    }
    
    pub fn average_fps(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: Duration = self.samples.iter()
            .take(self.count)
            .sum();
        let avg_frame_time = sum.as_secs_f32() / self.count as f32;
        if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        }
    }
    
    pub fn min_frame_time(&self) -> Duration {
        self.samples.iter()
            .take(self.count)
            .min()
            .copied()
            .unwrap_or(Duration::ZERO)
    }
    
    pub fn max_frame_time(&self) -> Duration {
        self.samples.iter()
            .take(self.count)
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
    }
    
    pub fn percentile(&self, p: f32) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }
        
        let mut sorted: Vec<Duration> = self.samples.iter()
            .take(self.count)
            .copied()
            .collect();
        sorted.sort();
        
        let index = ((self.count as f32 * p) as usize).min(self.count - 1);
        sorted[index]
    }
    
    pub fn get_stats(&self) -> FrameStats {
        FrameStats {
            current_fps: self.current_fps(),
            average_fps: self.average_fps(),
            min_frame_time_ms: self.min_frame_time().as_secs_f32() * 1000.0,
            max_frame_time_ms: self.max_frame_time().as_secs_f32() * 1000.0,
            p95_ms: self.percentile(0.95).as_secs_f32() * 1000.0,
            p99_ms: self.percentile(0.99).as_secs_f32() * 1000.0,
        }
    }
}

pub struct FrameStats {
    pub current_fps: f32,
    pub average_fps: f32,
    pub min_frame_time_ms: f32,
    pub max_frame_time_ms: f32,
    pub p95_ms: f32,
    pub p99_ms: f32,
}

impl std::fmt::Display for FrameStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FPS: {:.1} (avg: {:.1}) | Frame: min={:.2}ms max={:.2}ms p95={:.2}ms p99={:.2}ms",
            self.current_fps,
            self.average_fps,
            self.min_frame_time_ms,
            self.max_frame_time_ms,
            self.p95_ms,
            self.p99_ms
        )
    }
}
```

</details>

### C++ (Alternative)

<details>
<summary>Click to reveal C++ implementation</summary>

```cpp
#include <chrono>
#include <vector>
#include <algorithm>
#include <numeric>

class FrameProfiler {
public:
    explicit FrameProfiler(size_t window_size)
        : samples_(window_size)
        , head_(0)
        , count_(0)
        , capacity_(window_size)
    {}
    
    void recordFrame(std::chrono::duration<double> frame_time) {
        samples_[head_] = frame_time;
        head_ = (head_ + 1) % capacity_;
        if (count_ < capacity_) {
            count_++;
        }
    }
    
    float currentFPS() const {
        if (count_ == 0) return 0.0f;
        
        size_t last_idx = (head_ == 0) ? capacity_ - 1 : head_ - 1;
        double frame_time = samples_[last_idx].count();
        return frame_time > 0.0 ? 1.0f / frame_time : 0.0f;
    }
    
    float averageFPS() const {
        if (count_ == 0) return 0.0f;
        
        double sum = std::accumulate(
            samples_.begin(),
            samples_.begin() + count_,
            0.0,
            [](double acc, const auto& d) { return acc + d.count(); }
        );
        
        double avg_frame_time = sum / count_;
        return avg_frame_time > 0.0 ? 1.0f / avg_frame_time : 0.0f;
    }
    
    std::chrono::duration<double> percentile(float p) const {
        if (count_ == 0) return std::chrono::duration<double>(0);
        
        std::vector<std::chrono::duration<double>> sorted(
            samples_.begin(),
            samples_.begin() + count_
        );
        std::sort(sorted.begin(), sorted.end());
        
        size_t index = std::min(
            static_cast<size_t>(count_ * p),
            count_ - 1
        );
        return sorted[index];
    }

private:
    std::vector<std::chrono::duration<double>> samples_;
    size_t head_;
    size_t count_;
    size_t capacity_;
};
```

</details>

## Related Resources

- [Praxis Profiling Documentation](../../profiling.md)
- [Performance Profiling Guide](../../performance_profiling_guide.md)

## Next Steps

- Integrate with Exercise 01's game loop
- Add visualization with Exercise 11 (rendering)
- Explore `praxis_profiling` crate implementation
