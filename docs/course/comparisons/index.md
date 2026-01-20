# Engine Comparisons

Detailed comparisons of how different engines solve the same problems. Learn from multiple approaches and make informed architectural decisions.

## Comparison Categories

### Architecture Comparisons

<div class="feature-grid">
  <div class="feature-card">
    <h3>🏗️ ECS vs OOP vs Components</h3>
    <p>Compare data-oriented, object-oriented, and component-based architectures.</p>
    <p><em>See: <a href="../examples/ecs-vs-oop.html">ECS vs OOP</a></em></p>
  </div>
  
  <div class="feature-card">
    <h3>🔄 Game Loop Implementations</h3>
    <p>How Praxis, Unreal, Unity, and Godot structure their main loops.</p>
    <p><em>Coming soon</em></p>
  </div>
  
  <div class="feature-card">
    <h3>💾 Memory Management</h3>
    <p>Manual allocation, reference counting, GC, and ownership models.</p>
    <p><em>Coming soon</em></p>
  </div>
</div>

### Rendering Comparisons

<div class="feature-grid">
  <div class="feature-card">
    <h3>🎨 Rendering Pipelines</h3>
    <p>Forward, deferred, and forward+ approaches across engines.</p>
    <p><em>Coming soon</em></p>
  </div>
  
  <div class="feature-card">
    <h3>🌟 Material Systems</h3>
    <p>How different engines handle shaders and materials.</p>
    <p><em>Coming soon</em></p>
  </div>
  
  <div class="feature-card">
    <h3>💡 Lighting Systems</h3>
    <p>Static, dynamic, and mixed lighting approaches.</p>
    <p><em>Coming soon</em></p>
  </div>
</div>

### System Comparisons

<div class="feature-grid">
  <div class="feature-card">
    <h3>🎭 Animation Systems</h3>
    <p>Skeletal animation, blending, and state machines.</p>
    <p><em>Coming soon</em></p>
  </div>
  
  <div class="feature-card">
    <h3>⚡ Physics Integration</h3>
    <p>How engines integrate physics libraries.</p>
    <p><em>Coming soon</em></p>
  </div>
  
  <div class="feature-card">
    <h3>🌐 Networking Models</h3>
    <p>Client-server, peer-to-peer, and deterministic approaches.</p>
    <p><em>Coming soon</em></p>
  </div>
</div>

## Comparison Format

Each comparison includes:

1. **Problem Statement** - What challenge is being solved?
2. **Approach A, B, C** - Different engine implementations
3. **Trade-offs** - Pros and cons of each
4. **Performance** - Benchmarks and metrics
5. **Use Cases** - When to choose each approach

## Featured Engines

### Praxis (Rust)
- **Architecture:** ECS with bevy_ecs
- **Graphics:** Vulkan via vulkano
- **Philosophy:** Data-oriented, safety-first

### Unreal Engine (C++)
- **Architecture:** Object-oriented with UObject
- **Graphics:** Custom renderer (forward/deferred)
- **Philosophy:** AAA quality, maximum control

### Unity (C#)
- **Architecture:** Component-based with GameObject
- **Graphics:** Universal RP / HDRP
- **Philosophy:** Accessibility, rapid prototyping

### Godot (GDScript/C++)
- **Architecture:** Node-based hierarchy
- **Graphics:** Forward+ renderer
- **Philosophy:** Open source, beginner-friendly

## Side-by-Side Examples

All comparisons use **identical examples** across engines:

=== "Praxis (Rust)"
    ```rust
    fn spawn_player(mut commands: Commands) {
        commands.spawn((
            Player,
            Transform::default(),
            Velocity::default(),
        ));
    }
    ```

=== "Unreal (C++)"
    ```cpp
    APlayerCharacter* Player = World->SpawnActor<APlayerCharacter>(
        APlayerCharacter::StaticClass(),
        FVector::ZeroVector,
        FRotator::ZeroRotator
    );
    ```

=== "Unity (C#)"
    ```csharp
    GameObject player = new GameObject("Player");
    player.AddComponent<PlayerController>();
    player.AddComponent<Rigidbody>();
    ```

=== "Godot (GDScript)"
    ```gdscript
    var player = CharacterBody3D.new()
    player.name = "Player"
    add_child(player)
    ```

## Performance Comparisons

We measure real-world performance across:

- **Rendering** - Draw calls, GPU time, frame rate
- **Physics** - Collision checks, simulation time
- **Memory** - Allocations, cache efficiency
- **Build Times** - Iteration speed

## Decision Matrices

Choose the right engine for your needs:

| Feature | Praxis | Unreal | Unity | Godot |
|---------|--------|--------|-------|-------|
| **Learning Curve** | Steep | Steep | Moderate | Gentle |
| **Performance** | Excellent | Excellent | Good | Good |
| **AAA Capability** | No | Yes | Yes | No |
| **Indie Friendly** | Yes | Moderate | Yes | Yes |
| **Open Source** | Yes | No | No | Yes |
| **2D Support** | No | Limited | Excellent | Excellent |
| **3D Support** | Good | Excellent | Excellent | Good |

## Real-World Case Studies

### When Unreal Won
- AAA photorealistic graphics
- Large development teams
- Console/PC focus

### When Unity Won
- Mobile and WebGL deployment
- Asset store ecosystem
- Rapid prototyping

### When Godot Won
- 2D games and pixel art
- Open source requirements
- Beginners and hobbyists

### When Custom Engines Won (like Praxis)
- Educational purposes
- Specific performance needs
- Full control requirements

## Contributing Comparisons

Help expand this section:

1. Use the **comparison template**
2. Provide **benchmarks** when possible
3. Include **code examples** in all languages
4. Focus on **trade-offs**, not "which is best"

## Further Reading

- [ECS vs OOP Detailed](../examples/ecs-vs-oop.md)
- [Rendering Patterns](../patterns/rendering-architecture-patterns.md)
- [Component Storage](../patterns/component-storage-strategies.md)

---

<div style="text-align: center; margin: 2rem 0;">
  <p><strong>Explore Comparisons:</strong></p>
  <p>
    <a href="../examples/ecs-vs-oop.html" class="md-button md-button--primary">ECS vs OOP</a>
    <a href="../patterns/" class="md-button">Universal Patterns</a>
  </p>
</div>
