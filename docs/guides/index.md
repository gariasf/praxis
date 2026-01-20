# Guides

Task-oriented tutorials for implementing specific features. Each guide walks through a complete implementation with code examples in multiple languages.

## Quick Navigation

### Rendering
- [Rendering Overview](rendering.md) - Forward and deferred pipelines
- [Forward Rendering](rendering/forward-rendering.md) - Basic rendering pipeline
- [Deferred Rendering](rendering/deferred-rendering.md) - Multi-pass G-buffer approach
- [HDR & Tone Mapping](rendering/hdr-tonemapping.md) - High dynamic range
- [Shadows](rendering/shadows.md) - Cascaded shadow maps
- [Post-Processing](rendering/post-processing.md) - Bloom, color grading, effects

### Animation
- [Animation Overview](animation.md) - Quick start guide
- [Skeletal Basics](animation/skeletal-basics.md) - Core architecture
- [Animation Blending](animation/blending.md) - Blend trees and cross-fading
- [Advanced Features](animation/advanced-features.md) - IK, retargeting, root motion

### Systems
- [Physics](physics.md) - Rigid body dynamics with Rapier3D
- [Audio](audio.md) - Spatial audio with Kira
- [Scripting](scripting.md) - Lua integration
- [Networking](systems/networking.md) - Multiplayer architecture

## All Guides

### Graphics & Rendering

| Guide | Difficulty | Topics |
|-------|------------|--------|
| [Rendering Overview](rendering.md) | Beginner | Pipeline basics |
| [Forward Rendering](rendering/forward-rendering.md) | Beginner | Single-pass rendering |
| [Deferred Rendering](rendering/deferred-rendering.md) | Intermediate | G-buffer, lighting |
| [HDR & Tone Mapping](rendering/hdr-tonemapping.md) | Intermediate | Dynamic range, exposure |
| [Shadows](rendering/shadows.md) | Intermediate | Shadow mapping, PCF |
| [Post-Processing](rendering/post-processing.md) | Intermediate | Effects pipeline |

### Animation & Rigging

| Guide | Difficulty | Topics |
|-------|------------|--------|
| [Animation Overview](animation.md) | Beginner | Getting started |
| [Skeletal Basics](animation/skeletal-basics.md) | Beginner | Bones, joints, skinning |
| [Blending](animation/blending.md) | Intermediate | State machines, blend trees |
| [Advanced Features](animation/advanced-features.md) | Advanced | IK, retargeting, procedural |

### Physics & Simulation

| Guide | Difficulty | Topics |
|-------|------------|--------|
| [Physics](physics.md) | Beginner | Rigid bodies, colliders |

### Audio

| Guide | Difficulty | Topics |
|-------|------------|--------|
| [Audio](audio.md) | Beginner | Playback, spatial audio |

### Scripting & Tools

| Guide | Difficulty | Topics |
|-------|------------|--------|
| [Scripting](scripting.md) | Intermediate | Lua integration, hot-reload |

### Networking

| Guide | Difficulty | Topics |
|-------|------------|--------|
| [Networking](systems/networking.md) | Advanced | Client-server, replication |

## How to Use Guides

### Step-by-Step
Each guide provides:
1. **Overview** - What you'll build
2. **Prerequisites** - What you need to know first
3. **Implementation** - Code walkthrough
4. **Testing** - How to verify it works
5. **Next Steps** - Where to go from here

### Multi-Language Support
All code examples are available in:

=== "Pseudocode"
    High-level algorithm descriptions

=== "Rust (Praxis)"
    Full Praxis implementation

=== "C++ (Unreal-style)"
    Object-oriented approach

=== "C# (Unity-style)"
    Component-based approach

### Difficulty Levels

<span class="difficulty-badge difficulty-beginner">Beginner</span> - No prior engine knowledge needed

<span class="difficulty-badge difficulty-intermediate">Intermediate</span> - Requires basic understanding

<span class="difficulty-badge difficulty-advanced">Advanced</span> - Complex topics for experienced developers

## Related Resources

- [Concepts](../concepts/) - Theoretical background
- [Code Examples](../course/CODE_EXAMPLES.md) - Side-by-side comparisons
- [Learning Paths](../learning-paths/) - Structured progressions
- [Reference](../reference/) - API documentation

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="rendering.html" class="md-button md-button--primary">Start with Rendering</a>
  <a href="animation.html" class="md-button">Explore Animation</a>
</div>
