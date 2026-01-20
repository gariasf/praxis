# Exercise 01: Fixed Timestep Game Loop

**Difficulty**: 🟢 Beginner | **Estimated Time**: 2-3h | **Subsystem**: Core

## Overview

Implement a game loop that maintains a consistent 60 FPS update rate with a fixed physics timestep, independent of rendering frame rate. This is fundamental to deterministic game simulation and is used in nearly every game engine.

## Learning Objectives

- Understand the difference between variable and fixed timestep updates
- Learn how to decouple physics simulation from rendering
- Implement frame rate independent timing
- Handle the "spiral of death" when the simulation falls behind

## Requirements

### Functional Requirements

1. **Fixed Physics Timestep**
   - Physics updates at exactly 60 Hz (16.666ms per step)
   - Accumulate frame time and consume in fixed increments
   - Handle remainder time correctly for interpolation

2. **Variable Rendering**
   - Render as fast as possible (or capped at vsync)
   - Calculate interpolation factor for smooth rendering
   - Display current FPS

3. **Time Management**
   - Track delta time between frames
   - Track accumulator for fixed steps
   - Track total elapsed time

### Non-Functional Requirements

- **Performance**: Physics step must complete in < 16ms to maintain 60 Hz
- **Stability**: Must not skip physics updates under normal load
- **Accuracy**: Fixed timestep must be within ±0.1ms of target

## API Design

```rust
pub struct GameLoop {
    fixed_timestep: f64,  // 1/60 = 0.016666...
    accumulator: f64,
    last_frame_time: Instant,
}

impl GameLoop {
    pub fn new(target_fps: u32) -> Self;
    
    /// Call once per frame, returns how many physics steps to run
    pub fn tick(&mut self) -> TickResult;
}

pub struct TickResult {
    pub physics_steps: u32,
    pub alpha: f32,  // Interpolation factor [0, 1]
    pub delta_time: f64,
}
```

## Validation Criteria

### Correctness
- [ ] Physics updates at exactly 60 Hz when running at any frame rate
- [ ] Handles both faster (120 FPS) and slower (30 FPS) rendering correctly
- [ ] Interpolation alpha is in range [0.0, 1.0]
- [ ] No time is lost or duplicated (accumulator logic is correct)

### Performance
- [ ] Overhead < 0.1ms per frame
- [ ] Can maintain 60 Hz physics with up to 10,000 simple entities

### Code Quality
- [ ] Clean separation of concerns (timing, physics, rendering)
- [ ] Well-documented timing logic
- [ ] No unsafe code required

## Expected Behavior

### Scenario 1: Running at 60 FPS
- Frame time ≈ 16.6ms
- Exactly 1 physics step per frame
- Alpha ≈ 0.0 (accumulator starts near empty)

### Scenario 2: Running at 120 FPS
- Frame time ≈ 8.3ms
- 0-1 physics steps per frame (alternating)
- Alpha varies smoothly from 0.0 to 1.0

### Scenario 3: Running at 30 FPS
- Frame time ≈ 33.3ms
- Exactly 2 physics steps per frame
- Alpha ≈ 0.0

### Scenario 4: Temporary slowdown (spike to 100ms)
- Runs up to 6 physics steps to catch up
- System recovers once rendering speeds up

## Test Cases

```rust
#[test]
fn test_single_step_at_60fps() {
    let mut game_loop = GameLoop::new(60);
    std::thread::sleep(Duration::from_millis(16));
    
    let result = game_loop.tick();
    assert_eq!(result.physics_steps, 1);
    assert!(result.alpha >= 0.0 && result.alpha <= 1.0);
}

#[test]
fn test_multiple_steps_on_slow_frame() {
    let mut game_loop = GameLoop::new(60);
    std::thread::sleep(Duration::from_millis(50));
    
    let result = game_loop.tick();
    assert_eq!(result.physics_steps, 3);
}

#[test]
fn test_no_steps_on_fast_frame() {
    let mut game_loop = GameLoop::new(60);
    
    // First frame to initialize
    game_loop.tick();
    
    // Very fast frame
    std::thread::sleep(Duration::from_millis(5));
    let result = game_loop.tick();
    assert_eq!(result.physics_steps, 0);
}

#[test]
fn test_accumulator_carries_forward() {
    let mut game_loop = GameLoop::new(60);
    
    // Two frames at 8ms each should result in 1 step
    std::thread::sleep(Duration::from_millis(8));
    let result1 = game_loop.tick();
    
    std::thread::sleep(Duration::from_millis(8));
    let result2 = game_loop.tick();
    
    assert_eq!(result1.physics_steps + result2.physics_steps, 1);
}
```

## Performance Targets

| Scenario | Target | Measurement Method |
|----------|--------|-------------------|
| Loop overhead | < 0.1ms | Time tick() with no work |
| 60 FPS stability | ±1 FPS | Run for 1000 frames |
| Catch-up performance | 6 steps in < 100ms | Simulate slowdown |

## Hints & Guidance

### Getting Started
1. Start with a simple accumulator pattern
2. Use `std::time::Instant` for timing
3. Calculate alpha as `accumulator / fixed_timestep`

### Common Pitfalls
- **Spiral of Death**: If physics takes longer than the timestep, you can get stuck in an infinite loop trying to catch up. Implement a max steps per frame (e.g., 10).
- **Lost Time**: Make sure to properly carry forward fractional timestep remainder in the accumulator.
- **First Frame**: Special case the first frame where you don't have a previous time.

### Key Concepts

**Fixed vs Variable Timestep**
- Fixed: Same Δt every physics update → deterministic, stable simulation
- Variable: Δt matches frame time → simple but unstable physics

**Accumulator Pattern**
```
accumulator += frame_time
while accumulator >= fixed_timestep:
    physics_update(fixed_timestep)
    accumulator -= fixed_timestep
alpha = accumulator / fixed_timestep  // for interpolation
```

**Interpolation Alpha**
Used to smoothly render between physics states:
```
rendered_position = previous_position * (1 - alpha) + current_position * alpha
```

## Extensions (Optional)

1. **Spiral of Death Protection**: Implement max steps per frame with time scaling when overloaded
2. **Frame Rate Statistics**: Track min/max/average FPS over time windows
3. **Configurable Timestep**: Allow changing target FPS at runtime
4. **Time Scaling**: Implement slow-motion/fast-forward by scaling timestep

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::time::{Duration, Instant};

pub struct GameLoop {
    fixed_timestep: f64,
    accumulator: f64,
    last_frame_time: Instant,
    max_steps_per_frame: u32,
}

impl GameLoop {
    pub fn new(target_fps: u32) -> Self {
        Self {
            fixed_timestep: 1.0 / target_fps as f64,
            accumulator: 0.0,
            last_frame_time: Instant::now(),
            max_steps_per_frame: 10,
        }
    }
    
    pub fn tick(&mut self) -> TickResult {
        let current_time = Instant::now();
        let frame_time = current_time
            .duration_since(self.last_frame_time)
            .as_secs_f64();
        self.last_frame_time = current_time;
        
        // Add frame time to accumulator
        self.accumulator += frame_time;
        
        // Consume accumulator in fixed steps
        let mut steps = 0;
        while self.accumulator >= self.fixed_timestep 
            && steps < self.max_steps_per_frame 
        {
            self.accumulator -= self.fixed_timestep;
            steps += 1;
        }
        
        // If we hit max steps, discard extra time to avoid spiral of death
        if steps >= self.max_steps_per_frame {
            self.accumulator = 0.0;
        }
        
        // Calculate interpolation factor
        let alpha = (self.accumulator / self.fixed_timestep) as f32;
        
        TickResult {
            physics_steps: steps,
            alpha: alpha.clamp(0.0, 1.0),
            delta_time: frame_time,
        }
    }
}

pub struct TickResult {
    pub physics_steps: u32,
    pub alpha: f32,
    pub delta_time: f64,
}

// Example usage
fn main() {
    let mut game_loop = GameLoop::new(60);
    let mut position = 0.0;
    let mut previous_position = 0.0;
    let velocity = 10.0; // units per second
    
    loop {
        let tick = game_loop.tick();
        
        // Physics updates (fixed timestep)
        for _ in 0..tick.physics_steps {
            previous_position = position;
            position += velocity * game_loop.fixed_timestep as f32;
        }
        
        // Rendering (variable timestep with interpolation)
        let rendered_position = previous_position * (1.0 - tick.alpha) 
                              + position * tick.alpha;
        
        println!("Rendered position: {:.2}", rendered_position);
        
        // Simulate some frame time
        std::thread::sleep(Duration::from_millis(10));
    }
}
```

</details>

### C++ (Alternative)

<details>
<summary>Click to reveal C++ implementation</summary>

```cpp
#include <chrono>
#include <iostream>
#include <algorithm>

class GameLoop {
public:
    struct TickResult {
        uint32_t physics_steps;
        float alpha;
        double delta_time;
    };
    
    explicit GameLoop(uint32_t target_fps, uint32_t max_steps = 10)
        : fixed_timestep_(1.0 / target_fps)
        , accumulator_(0.0)
        , last_frame_time_(std::chrono::high_resolution_clock::now())
        , max_steps_per_frame_(max_steps)
    {}
    
    TickResult tick() {
        auto current_time = std::chrono::high_resolution_clock::now();
        double frame_time = std::chrono::duration<double>(
            current_time - last_frame_time_
        ).count();
        last_frame_time_ = current_time;
        
        accumulator_ += frame_time;
        
        uint32_t steps = 0;
        while (accumulator_ >= fixed_timestep_ && steps < max_steps_per_frame_) {
            accumulator_ -= fixed_timestep_;
            steps++;
        }
        
        if (steps >= max_steps_per_frame_) {
            accumulator_ = 0.0;
        }
        
        float alpha = std::clamp(
            static_cast<float>(accumulator_ / fixed_timestep_),
            0.0f,
            1.0f
        );
        
        return TickResult{steps, alpha, frame_time};
    }
    
    double fixed_timestep() const { return fixed_timestep_; }

private:
    double fixed_timestep_;
    double accumulator_;
    std::chrono::high_resolution_clock::time_point last_frame_time_;
    uint32_t max_steps_per_frame_;
};

int main() {
    GameLoop game_loop(60);
    float position = 0.0f;
    float previous_position = 0.0f;
    const float velocity = 10.0f;
    
    for (int frame = 0; frame < 100; ++frame) {
        auto tick = game_loop.tick();
        
        // Physics updates
        for (uint32_t i = 0; i < tick.physics_steps; ++i) {
            previous_position = position;
            position += velocity * static_cast<float>(game_loop.fixed_timestep());
        }
        
        // Rendering with interpolation
        float rendered_position = previous_position * (1.0f - tick.alpha)
                                + position * tick.alpha;
        
        std::cout << "Frame " << frame 
                  << ": position=" << rendered_position 
                  << ", steps=" << tick.physics_steps
                  << ", alpha=" << tick.alpha << std::endl;
        
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    
    return 0;
}
```

</details>

### Python (Simplified)

<details>
<summary>Click to reveal Python implementation</summary>

```python
import time

class GameLoop:
    def __init__(self, target_fps=60, max_steps=10):
        self.fixed_timestep = 1.0 / target_fps
        self.accumulator = 0.0
        self.last_frame_time = time.perf_counter()
        self.max_steps_per_frame = max_steps
    
    def tick(self):
        current_time = time.perf_counter()
        frame_time = current_time - self.last_frame_time
        self.last_frame_time = current_time
        
        self.accumulator += frame_time
        
        steps = 0
        while self.accumulator >= self.fixed_timestep and steps < self.max_steps_per_frame:
            self.accumulator -= self.fixed_timestep
            steps += 1
        
        if steps >= self.max_steps_per_frame:
            self.accumulator = 0.0
        
        alpha = max(0.0, min(1.0, self.accumulator / self.fixed_timestep))
        
        return {
            'physics_steps': steps,
            'alpha': alpha,
            'delta_time': frame_time
        }

# Example usage
def main():
    game_loop = GameLoop(60)
    position = 0.0
    previous_position = 0.0
    velocity = 10.0
    
    for frame in range(100):
        tick = game_loop.tick()
        
        # Physics updates
        for _ in range(tick['physics_steps']):
            previous_position = position
            position += velocity * game_loop.fixed_timestep
        
        # Rendering with interpolation
        rendered_position = (previous_position * (1.0 - tick['alpha']) +
                           position * tick['alpha'])
        
        print(f"Frame {frame}: pos={rendered_position:.2f}, "
              f"steps={tick['physics_steps']}, alpha={tick['alpha']:.2f}")
        
        time.sleep(0.01)

if __name__ == '__main__':
    main()
```

</details>

## Related Resources

- [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/) - Classic article by Glenn Fiedler
- [Game Programming Patterns: Game Loop](https://gameprogrammingpatterns.com/game-loop.html)
- [Praxis Core Documentation](../../reference/crates.md#praxis_core)

## Next Steps

After completing this exercise:
- Implement Exercise 02: Frame Time Profiler to visualize loop performance
- Add the physics integration from Exercise 34
- Study how Praxis implements this in `praxis_core`
