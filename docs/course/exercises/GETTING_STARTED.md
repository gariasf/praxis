# Getting Started with Praxis Engine Exercises

Welcome to the Praxis game engine exercise framework! This guide will help you make the most of the 60+ specification-based exercises designed to teach game engine development.

## What Are These Exercises?

Each exercise is a **specification-based learning challenge** where you'll implement a specific game engine feature from scratch. Unlike tutorials that walk you through code line-by-line, these exercises:

- **Give you requirements** but let you design the solution
- **Provide validation criteria** so you know when you're done
- **Include performance targets** for real-world applicability
- **Offer reference implementations** in multiple languages (study AFTER your attempt)
- **Link to related resources** for deeper understanding

## Philosophy

### Learn by Building
The best way to understand game engines is to build one. These exercises guide you through implementing every major subsystem, from basic game loops to advanced networking.

### Specifications Over Tutorials
Instead of copying code, you'll:
1. Read the specification (what to build)
2. Design your approach
3. Implement it your way
4. Validate against criteria
5. Compare with references

This builds deeper understanding and problem-solving skills.

### Educational Focus
Every exercise exists to teach specific concepts applicable across game engines, not Praxis-specific implementation details. You'll learn transferable skills.

## How to Use These Exercises

### 1. Choose Your Path

#### For Beginners (New to game engine development)
Start with **Path 1: Engine Fundamentals**:
- Exercise 01: Fixed Timestep Game Loop
- Exercise 02: Frame Time Profiler
- Exercise 08: Configuration Management
- Exercise 21: Component Registration
- Exercise 23: Entity Queries

**Est. Time**: 16-20 hours | **Difficulty**: 🟢

#### For Graphics Programmers
Jump into **Path 2: Graphics Programming**:
- Prerequisites: Complete Path 1 or have equivalent experience
- Start with Exercise 11 (Triangle Renderer)
- Progress through lighting, shadows, and advanced rendering

**Est. Time**: 30-40 hours | **Difficulty**: 🟡🔴

#### For Systems Programmers
Focus on **Path 3: Systems Programming**:
- Covers resource management, threading, hot-reloading
- Exercises 03-07, 22, 24

**Est. Time**: 28-38 hours | **Difficulty**: 🟡🔴

See [CATALOG.md](./CATALOG.md) for all learning paths.

### 2. Understand the Exercise

Each exercise follows a standard format:

```markdown
# Exercise XX: Title

**Difficulty**: 🟢/🟡/🔴 | **Time**: X-Yh | **Subsystem**: Category

## Overview
What you'll build and why it matters

## Learning Objectives
- Specific skills you'll learn

## Requirements
Functional and non-functional requirements

## Validation Criteria
How to verify your implementation works

## Test Cases
Concrete tests to run

## Performance Targets
Benchmarks to hit

## Hints & Guidance
Help without spoiling the solution

## Reference Implementation
Working code in Rust/C++/Python (reveal after your attempt)
```

### 3. Implement Your Solution

#### Step 1: Read the Specification
- Understand the requirements completely
- Note validation criteria
- Check performance targets

#### Step 2: Plan Your Approach
- Sketch out data structures
- Consider algorithms
- Think about edge cases
- **Don't look at references yet!**

#### Step 3: Implement
Create a new Rust project:
```bash
cargo new exercise_01_game_loop
cd exercise_01_game_loop
```

Write your implementation, referring to:
- Requirement specification
- API design suggestions
- Hints section
- Praxis documentation for context

#### Step 4: Validate
Run against validation criteria:
- Correctness tests
- Performance benchmarks
- Manual verification (for graphics)

```bash
cargo test
cargo bench
```

#### Step 5: Review References
**Only after attempting yourself**, study reference implementations:
- Compare approaches
- Learn alternative techniques
- Understand optimizations

### 4. Reflect and Extend

After completing an exercise:
- What worked well in your design?
- What would you do differently?
- Try optional extensions
- Move to next exercise

## Setting Up Your Environment

### Required Tools

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### For Graphics Exercises (11-20)

Install Vulkan SDK:
- **Windows**: [LunarG Vulkan SDK](https://vulkan.lunarg.com/)
- **Linux**: `sudo apt install vulkan-tools libvulkan-dev`
- **macOS**: MoltenVK (via Vulkan SDK)

Verify GPU support:
```bash
vulkaninfo
```

### Recommended Editor

- **VS Code** with rust-analyzer extension
- **RustRover** (JetBrains)
- **Vim/Neovim** with LSP

## Project Structure

### Option 1: Separate Projects (Recommended)
Create one project per exercise:
```
learning/
  exercise_01_game_loop/
  exercise_02_profiler/
  exercise_03_resource_manager/
  ...
```

### Option 2: Workspace
Create a Cargo workspace:
```toml
# Cargo.toml
[workspace]
members = [
    "exercise_01",
    "exercise_02",
    ...
]
```

### Option 3: Praxis Fork
Fork Praxis and add exercises as examples:
```
praxis/
  examples/
    exercise_01_game_loop.rs
    exercise_02_profiler.rs
```

## Tips for Success

### 1. Start Simple
Don't over-engineer. Get basic functionality working first, then optimize.

### 2. Use Tests
Write tests as you go. They clarify requirements and catch bugs early.

### 3. Read Error Messages
Rust's compiler errors are helpful. Read them carefully.

### 4. Benchmark Early
For performance-critical exercises, benchmark from the start to avoid surprises.

### 5. Don't Peek!
Resist looking at reference implementations until you've tried yourself. The struggle builds understanding.

### 6. Take Breaks
These exercises are mentally demanding. Take breaks, come back fresh.

### 7. Document Your Learning
- Keep notes on what you learned
- Blog about your approach
- Share solutions (after completing the exercise)

## Common Pitfalls

### Overcomplicating
Especially on early exercises, keep it simple. Advanced patterns come later.

### Ignoring Performance Targets
Real game engines must hit performance targets. If your implementation is 10x slower than the target, something's wrong architecturally.

### Skipping Validation
Don't move on until you've verified correctness. Bugs compound.

### Not Using Type System
Let Rust's type system guide you. If something is hard to express in types, reconsider the design.

## Getting Help

### 1. Review Hints
Every exercise has a hints section with guidance.

### 2. Check Related Resources
Exercises link to relevant documentation, articles, and examples.

### 3. Study Praxis Code
The Praxis codebase implements many of these concepts. See how they're done in production.

```bash
# Clone Praxis
git clone https://github.com/your-org/praxis.git
cd praxis

# Study relevant crate
cd crates/praxis_core  # for core exercises
cd crates/praxis_graphics  # for graphics exercises
```

### 4. Reference Implementations
After attempting, study reference implementations for alternative approaches.

### 5. Community
- Ask questions in Praxis Discord/forum
- Share your implementations for feedback
- Learn from others' solutions

## Validation & Assessment

### Self-Assessment Checklist

For each exercise, verify:

- [ ] **Correctness**: All validation criteria met
- [ ] **Tests**: All test cases pass
- [ ] **Performance**: Meets or exceeds targets
- [ ] **Code Quality**: Clean, documented, no warnings
- [ ] **Understanding**: Can explain design decisions

### When to Move On

Move to the next exercise when:
1. All validation criteria pass
2. You understand WHY your solution works
3. You've studied reference implementations
4. You can articulate trade-offs in your design

Don't move on if:
- Tests are failing
- Performance is far below targets
- You're confused about core concepts

## Tracking Progress

### Create a Learning Log

```markdown
# Exercise 01: Fixed Timestep Game Loop

**Date Started**: 2024-01-15
**Date Completed**: 2024-01-16
**Time Spent**: 3 hours

## Approach
- Used accumulator pattern
- Implemented with std::time::Instant
- Added spiral of death protection

## Challenges
- Initially forgot to clamp alpha
- Struggled with test timing precision

## Learnings
- Importance of fixed timestep for determinism
- How accumulator pattern prevents time loss

## Next Steps
- Move to Exercise 02
- Consider adding variable timestep option
```

### Track Statistics

- Exercises completed: X / 60
- Total time invested: Y hours
- Difficulty breakdown: Z beginner, W intermediate, V advanced

## Additional Resources

### Books
- "Game Programming Patterns" by Robert Nystrom
- "Real-Time Rendering" by Akenine-Möller et al.
- "Game Engine Architecture" by Jason Gregory

### Online
- [Praxis Documentation](../../README.md)
- [Bevy Engine](https://bevyengine.org/) - Another Rust game engine
- [Learn OpenGL](https://learnopengl.com/) - Graphics fundamentals
- [Gaffer on Games](https://gafferongames.com/) - Networking, physics

### Video
- "Handmade Hero" by Casey Muratori
- GDC talks on engine architecture
- Conference presentations on specific subsystems

## Contributing

Found an issue or have an improvement?

1. Exercises are in `docs/course/exercises/`
2. Follow [EXERCISE_TEMPLATE.md](./EXERCISE_TEMPLATE.md)
3. Submit PR with:
   - Clear specification
   - Validation criteria
   - Reference implementations
   - Related resources

## Next Steps

Ready to start? Here's your first exercise:

👉 **[Exercise 01: Fixed Timestep Game Loop](./01-fixed-timestep-game-loop.md)**

This foundational exercise teaches game loop architecture that you'll use throughout engine development.

**Estimated Time**: 2-3 hours  
**Difficulty**: 🟢 Beginner  
**Prerequisites**: Basic Rust knowledge

Good luck, and enjoy building your game engine knowledge! 🎮🚀

---

*Remember: The goal isn't to rush through exercises, but to deeply understand game engine concepts. Take your time, experiment, and learn.*
