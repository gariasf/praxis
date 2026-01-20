# Fixed Timestep Physics

<span class="difficulty-badge difficulty-advanced">Advanced</span>

Physics simulations require consistent timestep for stability and determinism, regardless of frame rate. The fixed timestep accumulator pattern is essential for any serious physics implementation.

## Overview

The fixed timestep accumulator decouples physics updates from rendering:

1. Measure elapsed time since last frame (`delta_time`)
2. Accumulate time in a buffer
3. While accumulated time ≥ fixed step, run physics update and subtract step
4. Remaining time carries over to next frame

This ensures physics runs at constant rate (e.g., 60 Hz) even if rendering is 144 Hz or 30 Hz.

## Why Fixed Timestep?

| Variable Timestep | Fixed Timestep |
|-------------------|----------------|
| ❌ Non-deterministic | ✅ Deterministic |
| ❌ Unstable at low FPS | ✅ Stable physics |
| ❌ Different on each machine | ✅ Consistent simulation |
| ✅ Simple to implement | ⚠️ Requires accumulator |

## Algorithm

=== "Pseudocode"

    ```
    CONSTANT FIXED_TIMESTEP = 1.0 / 60.0  // 60 Hz physics

    GLOBAL accumulator = 0.0

    FUNCTION game_loop:
        last_time = current_time()
        
        LOOP:
            current_time = current_time()
            delta_time = current_time - last_time
            last_time = current_time
            
            // Clamp delta_time to prevent spiral of death
            IF delta_time > 0.25:
                delta_time = 0.25  // Max 4 missed frames
            
            accumulator += delta_time
            
            // Fixed timestep updates
            WHILE accumulator >= FIXED_TIMESTEP:
                update_physics(FIXED_TIMESTEP)
                accumulator -= FIXED_TIMESTEP
            
            // Variable timestep rendering
            render(delta_time)

    FUNCTION update_physics(dt):
        // Apply forces
        FOR EACH rigidbody:
            rigidbody.velocity += rigidbody.force * dt
            rigidbody.force = Vec3.ZERO
        
        // Integrate positions
        FOR EACH rigidbody:
            rigidbody.position += rigidbody.velocity * dt
        
        // Resolve collisions
        detect_and_resolve_collisions()
    ```

=== "Rust (Praxis)"

    ```rust
    use bevy_ecs::prelude::*;
    use praxis_math::Vec3;

    // Physics configuration resource
    #[derive(Resource)]
    pub struct PhysicsConfig {
        pub timestep: f32,        // Fixed timestep (e.g., 1/60)
        pub max_substeps: u32,    // Prevent spiral of death
    }

    impl Default for PhysicsConfig {
        fn default() -> Self {
            Self {
                timestep: 1.0 / 60.0,  // 60 Hz
                max_substeps: 4,        // Max 4 physics steps per frame
            }
        }
    }

    // Time accumulator resource
    #[derive(Resource)]
    pub struct PhysicsAccumulator {
        accumulator: f32,
    }

    impl Default for PhysicsAccumulator {
        fn default() -> Self {
            Self { accumulator: 0.0 }
        }
    }

    // Physics components
    #[derive(Component)]
    pub struct RigidBody {
        pub velocity: Vec3,
        pub force: Vec3,
        pub mass: f32,
    }

    #[derive(Component)]
    pub struct Position(pub Vec3);

    // Main physics system
    pub fn physics_system(
        time: Res<Time>,
        config: Res<PhysicsConfig>,
        mut accumulator: ResMut<PhysicsAccumulator>,
        mut rigidbodies: Query<(&mut RigidBody, &mut Position)>,
    ) {
        // Add frame time to accumulator
        let delta_time = time.delta_seconds().min(0.25); // Clamp to 250ms
        accumulator.accumulator += delta_time;
        
        // Run fixed timestep updates
        let mut substeps = 0;
        while accumulator.accumulator >= config.timestep {
            if substeps >= config.max_substeps {
                // Spiral of death prevention
                accumulator.accumulator = 0.0;
                break;
            }
            
            physics_step(config.timestep, &mut rigidbodies);
            accumulator.accumulator -= config.timestep;
            substeps += 1;
        }
    }

    fn physics_step(
        dt: f32,
        rigidbodies: &mut Query<(&mut RigidBody, &mut Position)>,
    ) {
        // Apply forces (F = ma, so a = F/m)
        for (mut rb, _) in rigidbodies.iter_mut() {
            let acceleration = rb.force / rb.mass;
            rb.velocity += acceleration * dt;
            rb.force = Vec3::ZERO;
        }
        
        // Integrate positions
        for (rb, mut pos) in rigidbodies.iter_mut() {
            pos.0 += rb.velocity * dt;
        }
    }

    // Example: applying forces
    pub fn apply_gravity(mut rigidbodies: Query<&mut RigidBody>) {
        const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);
        
        for mut rb in rigidbodies.iter_mut() {
            rb.force += GRAVITY * rb.mass;
        }
    }
    ```

    **Key Patterns**:
    
    - `PhysicsAccumulator` resource persists across frames
    - Clamping prevents "spiral of death"
    - Forces cleared after each substep
    - System schedule controls execution order

=== "C++ (Unreal)"

    ```cpp
    // PhysicsEngine.h
    class UPhysicsEngine : public UObject {
    private:
        float FixedTimestep = 1.0f / 60.0f;  // 60 Hz
        float Accumulator = 0.0f;
        int32 MaxSubsteps = 4;

    public:
        void Tick(float DeltaTime);
        void PhysicsStep(float Dt);
    };

    // PhysicsEngine.cpp
    void UPhysicsEngine::Tick(float DeltaTime) {
        // Clamp delta time to prevent spiral of death
        DeltaTime = FMath::Min(DeltaTime, 0.25f);
        
        Accumulator += DeltaTime;
        
        int32 Substeps = 0;
        while (Accumulator >= FixedTimestep) {
            if (Substeps >= MaxSubsteps) {
                Accumulator = 0.0f;
                UE_LOG(LogPhysics, Warning, 
                    TEXT("Physics substep limit reached"));
                break;
            }
            
            PhysicsStep(FixedTimestep);
            Accumulator -= FixedTimestep;
            Substeps++;
        }
    }

    void UPhysicsEngine::PhysicsStep(float Dt) {
        TArray<UPrimitiveComponent*> PhysicsBodies;
        GetAllPhysicsBodies(PhysicsBodies);
        
        // Apply forces
        for (UPrimitiveComponent* Body : PhysicsBodies) {
            if (Body->IsSimulatingPhysics()) {
                FVector Acceleration = Body->GetForce() / Body->GetMass();
                FVector NewVelocity = Body->GetVelocity() + Acceleration * Dt;
                Body->SetVelocity(NewVelocity);
                Body->ClearForces();
            }
        }
        
        // Integrate positions
        for (UPrimitiveComponent* Body : PhysicsBodies) {
            if (Body->IsSimulatingPhysics()) {
                FVector NewPosition = Body->GetPosition() + 
                    Body->GetVelocity() * Dt;
                Body->SetPosition(NewPosition);
            }
        }
    }
    ```

    **Key Patterns**:
    
    - Accumulator stored as class member
    - Direct iteration over arrays
    - Unreal's `FMath` utilities for clamping

=== "C# (Unity)"

    ```csharp
    using UnityEngine;

    public class PhysicsExample : MonoBehaviour {
        // FixedUpdate automatically runs at fixed timestep
        void FixedUpdate() {
            // Unity's physics runs here automatically
            // Time.fixedDeltaTime is the fixed timestep (default: 0.02)
            
            ApplyCustomForces();
        }
        
        // Update runs at variable frame rate
        void Update() {
            // Rendering and input handling here
        }
        
        void ApplyCustomForces() {
            Rigidbody rb = GetComponent<Rigidbody>();
            if (rb != null) {
                Vector3 gravity = Physics.gravity;
                rb.AddForce(gravity * rb.mass);
            }
        }
    }

    // Manual implementation for education
    public class ManualFixedTimestep : MonoBehaviour {
        public float fixedTimestep = 1.0f / 60.0f;
        private float accumulator = 0.0f;
        
        void Update() {
            float deltaTime = Mathf.Min(Time.deltaTime, 0.25f);
            accumulator += deltaTime;
            
            while (accumulator >= fixedTimestep) {
                PhysicsStep(fixedTimestep);
                accumulator -= fixedTimestep;
            }
        }
        
        void PhysicsStep(float dt) {
            // Custom physics simulation
        }
    }

    // Configure Unity's fixed timestep
    public class PhysicsConfiguration : MonoBehaviour {
        void Start() {
            // Set fixed timestep (default: 0.02 = 50 Hz)
            Time.fixedDeltaTime = 1.0f / 60.0f;  // 60 Hz
            
            // Maximum allowed timestep
            Time.maximumDeltaTime = 0.1f;  // 100ms max
        }
    }
    ```

    **Key Patterns**:
    
    - Unity provides `FixedUpdate()` callback
    - Accumulator pattern built into engine
    - `Time.fixedDeltaTime` configurable
    - PhysX runs automatically

## Comparison

| Aspect | Rust (Praxis) | C++ (Unreal) | C# (Unity) |
|--------|---------------|---------------|-------------|
| **Timestep Control** | Manual accumulator | Manual accumulator | Built-in `FixedUpdate()` |
| **Configuration** | `PhysicsConfig` resource | Engine settings | `Time.fixedDeltaTime` |
| **Spiral Prevention** | Manual clamping | Manual clamping | `Time.maximumDeltaTime` |
| **User Visibility** | Explicit | Explicit | Implicit |

## The Spiral of Death

!!! danger "What is it?"
    When physics can't keep up with real-time, each frame tries to run more substeps, making the next frame even slower. This creates a death spiral where the game freezes.

**Prevention strategies**:

1. **Clamp delta time** - Limit maximum time per frame
2. **Max substeps** - Discard remaining time after N steps
3. **Warn user** - Log when substeps are skipped
4. **Reduce physics load** - Simplify colliders, reduce object count

## Interpolation (Advanced)

For smooth rendering between physics steps:

```rust
// Store previous state
#[derive(Component)]
pub struct PreviousPosition(Vec3);

// Interpolate for rendering
fn interpolate_for_rendering(
    alpha: f32, // Accumulator / timestep (0.0 to 1.0)
    query: Query<(&Position, &PreviousPosition)>,
) {
    for (current, previous) in query.iter() {
        let render_pos = previous.0.lerp(current.0, alpha);
        // Use render_pos for display
    }
}
```

## Performance Tips

!!! tip "Typical Timesteps"
    - **30 Hz**: Minimum for stability
    - **60 Hz**: Standard for most games
    - **120 Hz**: High-precision simulations
    - **240 Hz**: Fighting games, physics-heavy titles

!!! tip "Optimization"
    - Use spatial partitioning to reduce collision checks
    - Sleep inactive objects
    - Use continuous collision detection only where needed
    - Profile to find actual bottlenecks

## Further Reading

- [Physics Guide](../../guides/physics.md)
- [Game Loop Patterns](../../patterns/game-loop-patterns.md)
- [Performance Optimization](../../learning-paths/performance.md)

## Exercises

1. **Implement basic physics** - Add gravity and collision detection
2. **Measure determinism** - Run same simulation twice, compare results
3. **Profile substeps** - Log how many substeps occur at different frame rates
4. **Add interpolation** - Smooth rendering between physics steps

---

<div style="text-align: center; margin: 2rem 0;">
  <a href="frustum-culling.html" class="md-button">← Previous: Frustum Culling</a>
  <a href="ecs-vs-oop.html" class="md-button">Next: ECS vs OOP →</a>
</div>
