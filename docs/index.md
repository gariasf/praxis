# Praxis Game Engine Course

<div style="text-align: center; margin: 2rem 0;">
  <h2>Learn Game Engine Architecture with Multi-Language Examples</h2>
  <p style="font-size: 1.2rem; color: var(--md-default-fg-color--light);">
    Master fundamental concepts through side-by-side comparisons in Rust, C++, C#, and pseudocode
  </p>
</div>

---

## 🎯 What You'll Learn

<div class="feature-grid">
  <div class="feature-card">
    <h3>🏗️ Engine Architecture</h3>
    <p>Understand how modern game engines are structured, from ECS to rendering pipelines.</p>
  </div>
  
  <div class="feature-card">
    <h3>💻 Multi-Language Patterns</h3>
    <p>See the same algorithms implemented in Rust, C++, and C# to understand universal concepts.</p>
  </div>
  
  <div class="feature-card">
    <h3>🎨 Graphics Programming</h3>
    <p>Learn forward/deferred rendering, shadows, HDR, and modern GPU techniques.</p>
  </div>
  
  <div class="feature-card">
    <h3>⚡ Performance Optimization</h3>
    <p>Master culling, LOD, spatial partitioning, and data-oriented design.</p>
  </div>
  
  <div class="feature-card">
    <h3>🎮 Core Systems</h3>
    <p>Implement physics, animation, audio, scripting, and networking from scratch.</p>
  </div>
  
  <div class="feature-card">
    <h3>🛠️ Editor Tools</h3>
    <p>Build professional editor features: gizmos, undo/redo, selection, and more.</p>
  </div>
</div>

## 🚀 Quick Start

<div class="feature-grid">
  <div class="feature-card">
    <h3>1. Choose Your Path</h3>
    <p>Start with the <a href="getting-started/">Getting Started</a> guide or jump to the <a href="course/">Course Curriculum</a>.</p>
  </div>
  
  <div class="feature-card">
    <h3>2. Select Your Language</h3>
    <p>All code examples support <strong>Rust</strong>, <strong>C++</strong>, <strong>C#</strong>, and <strong>Pseudocode</strong> - pick your preference!</p>
  </div>
  
  <div class="feature-card">
    <h3>3. Learn By Doing</h3>
    <p>Work through <a href="course/exercises/">hands-on exercises</a> and <a href="course/projects/">complete projects</a>.</p>
  </div>
</div>

!!! tip "Your Language Preference is Saved"
    Choose your preferred language once using the tabs, and all examples across the site will sync automatically!

## 🌟 Features

### Interactive Code Tabs
Switch between languages with a single click. Compare implementations side-by-side.

=== "Pseudocode"

    ```
    FUNCTION greet(name):
        PRINT "Hello, " + name
    ```

=== "Rust (Praxis)"

    ```rust
    fn greet(name: &str) {
        println!("Hello, {}", name);
    }
    ```

=== "C++ (Unreal)"

    ```cpp
    void Greet(const FString& Name) {
        UE_LOG(LogTemp, Log, TEXT("Hello, %s"), *Name);
    }
    ```

=== "C# (Unity)"

    ```csharp
    void Greet(string name) {
        Debug.Log($"Hello, {name}");
    }
    ```

### Comprehensive Curriculum

| Section | Description |
|---------|-------------|
| [Getting Started](getting-started/) | Installation, setup, and first steps |
| [Course](course/) | Structured curriculum with multi-language examples |
| [Guides](guides/) | Task-oriented tutorials for specific features |
| [Concepts](concepts/) | Deep dives into theoretical foundations |
| [Reference](reference/) | API documentation and specifications |
| [Learning Paths](learning-paths/) | Structured progressions from beginner to advanced |

## 📚 Course Sections

### [Core Curriculum](course/)
Complete game engine architecture course with:

- 📖 [Curriculum](course/CURRICULUM.md) - Course outline and learning objectives
- 💻 [Code Examples](course/CODE_EXAMPLES.md) - Multi-language implementations
- 🎯 [Universal Patterns](course/patterns/) - Engine-agnostic design patterns
- 📝 [Exercises](course/exercises/) - Hands-on practice
- 🚀 [Projects](course/projects/) - Complete implementations

### [Guides](guides/)
Step-by-step tutorials for implementing features:

- **Rendering**: Forward/deferred pipelines, shadows, HDR, post-processing
- **Animation**: Skeletal animation, blending, IK, retargeting
- **Physics**: Rigid bodies, collisions, character controllers
- **Systems**: Audio, scripting, networking, terrain

### [Concepts](concepts/)
Theoretical foundations and design principles:

- ECS Architecture
- Vulkan Rendering
- Transform Hierarchies
- PBR Materials
- Physics Simulation

### [Learning Paths](learning-paths/)
Structured progressions for mastering subsystems:

- [Rendering Path](learning-paths/rendering.md)
- [Animation Path](learning-paths/animation.md)
- [Physics Path](learning-paths/physics.md)
- [Performance Path](learning-paths/performance.md)

## 🎓 Who Is This For?

### Game Engine Developers
Building your own engine? Learn from production-quality patterns and see how different engines solve the same problems.

### Engine Users
Understand what's happening under the hood. Become a better Unity, Unreal, or Godot developer by understanding engine internals.

### Students
Learn game engine architecture systematically with clear explanations and working examples.

### Language Learners
See how the same concepts translate between Rust, C++, and C# - perfect for learning a new language in the context of game development.

## 🔑 Key Concepts Covered

- **ECS vs OOP** - Understand architectural trade-offs
- **Transform Propagation** - Scene graphs and hierarchies
- **Frustum Culling** - Spatial optimization techniques
- **Fixed Timestep Physics** - Deterministic simulation
- **Rendering Pipelines** - Forward, deferred, and hybrid approaches
- **Memory Management** - Ownership, GC, and manual allocation
- **Parallelization** - Multi-threaded game loops

## 💡 Learning Philosophy

This course teaches **universal principles**, not engine-specific APIs. You'll learn:

1. **The Why** - Understanding the problem being solved
2. **The How** - Multiple implementation approaches
3. **The Trade-offs** - When to use each approach
4. **The Practice** - Hands-on exercises and projects

## 🎯 Learning Paths by Role

### Graphics Programmer
1. [Vulkan Rendering Concepts](concepts/vulkan-rendering.md)
2. [Rendering Architecture Patterns](course/patterns/rendering-architecture-patterns.md)
3. [Rendering Learning Path](learning-paths/rendering.md)

### Gameplay Programmer
1. [ECS Architecture](concepts/ecs-architecture.md)
2. [Game Loop Patterns](course/patterns/game-loop-patterns.md)
3. [Physics Guide](guides/physics.md)

### Tools Programmer
1. [Editor Overview](editor/editor-overview.md)
2. [Undo/Redo System](editor/undo-redo.md)
3. [Editor Learning Path](learning-paths/editor.md)

### Engine Architect
1. [Component Storage Strategies](course/patterns/component-storage-strategies.md)
2. [Memory Management Approaches](course/patterns/memory-management-approaches.md)
3. [Performance Learning Path](learning-paths/performance.md)

## 🛠️ About Praxis

Praxis is an educational 3D game engine written in Rust. It demonstrates modern engine architecture using:

- **Vulkan** for rendering (via vulkano)
- **bevy_ecs** for Entity-Component-System
- **rapier3d** for physics
- **Modern Rust** patterns and best practices

While the course uses Praxis as a reference implementation, all concepts are taught in a language-agnostic way with examples in multiple languages.

## 📖 How to Use This Site

### Navigation
- **Top tabs** - Switch between major sections
- **Left sidebar** - Detailed navigation within sections
- **Right sidebar** - Table of contents for current page
- **Search** - Find content quickly (press `/` to focus)

### Code Examples
- **Click tabs** to switch languages
- **Copy button** appears on hover
- **Line numbers** for reference
- **Syntax highlighting** for readability

### Keyboard Shortcuts
- ++alt+1++ - Switch to Pseudocode
- ++alt+2++ - Switch to Rust
- ++alt+3++ - Switch to C++
- ++alt+4++ - Switch to C#
- ++slash++ - Focus search

## 🚀 Get Started

<div style="text-align: center; margin: 3rem 0;">
  <a href="getting-started/" class="md-button md-button--primary" style="margin: 0.5rem;">Installation & Setup</a>
  <a href="course/" class="md-button md-button--primary" style="margin: 0.5rem;">Start Course</a>
  <a href="course/CODE_EXAMPLES.html" class="md-button" style="margin: 0.5rem;">Browse Examples</a>
  <a href="beginners-guide.html" class="md-button" style="margin: 0.5rem;">Beginners Guide</a>
</div>

---

<div style="text-align: center; color: var(--md-default-fg-color--light); margin: 2rem 0;">
  <p>
    Built with <a href="https://www.mkdocs.org/">MkDocs</a> and 
    <a href="https://squidfunk.github.io/mkdocs-material/">Material for MkDocs</a>
  </p>
  <p>
    <a href="https://github.com/yourusername/praxis">View on GitHub</a> • 
    <a href="https://github.com/yourusername/praxis/issues">Report Issue</a> • 
    <a href="https://github.com/yourusername/praxis/blob/main/LICENSE">License</a>
  </p>
</div>
