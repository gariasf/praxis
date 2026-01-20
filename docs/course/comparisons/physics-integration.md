# Physics Integration: Multi-Engine Comparison

**Complexity**: Intermediate-Advanced  
**Curriculum Module**: [Module 5 - Physics Integration Strategies](../modules/05-physics-integration-strategies.md)

## Problem Statement

Game engines must integrate physics simulation with their core systems. Key challenges:

- How do we synchronize transforms between physics and rendering?
- How do we handle fixed timestep for deterministic simulation?
- How do we manage bidirectional communication (kinematic vs. dynamic)?
- How do we detect and respond to collisions?
- How do we optimize physics for large numbers of objects?

## Design Philosophy Comparison

| Engine | Physics Engine | Integration Approach | Timestep Model |
|--------|---------------|---------------------|----------------|
| **Unity** | PhysX (3D), Box2D (2D) | Component-based, automatic sync | Fixed timestep (default 50Hz) |
| **Unreal** | Chaos (UE5), PhysX (UE4) | Substepping, async simulation | Variable with substepping |
| **Godot** | Custom (Godot Physics), Bullet (optional) | Node-based, `_physics_process` | Fixed timestep (default 60Hz) |
| **Praxis** | Rapier3D | Manual sync systems, ECS-based | Fixed timestep (configurable) |

## Implementation Examples

### Basic Rigidbody Setup

#### Unity (C#)

```csharp
using UnityEngine;

public class PhysicsExample : MonoBehaviour
{
    void Start()
    {
        // Add Rigidbody component
        Rigidbody rb = gameObject.AddComponent<Rigidbody>();
        
        // Configure rigidbody
        rb.mass = 10f;
        rb.drag = 0.5f;
        rb.angularDrag = 0.05f;
        rb.useGravity = true;
        rb.isKinematic = false;  // Dynamic (physics-controlled)
        
        // Add collider
        BoxCollider collider = gameObject.AddComponent<BoxCollider>();
        collider.size = new Vector3(1, 1, 1);
        collider.isTrigger = false;  // Solid collision
        
        // Apply force
        rb.AddForce(Vector3.forward * 500f);
        rb.AddTorque(Vector3.up * 50f);
    }
    
    // Fixed timestep physics update
    void FixedUpdate()
    {
        Rigidbody rb = GetComponent<Rigidbody>();
        
        // Apply forces in FixedUpdate (not Update!)
        if (Input.GetKey(KeyCode.W))
        {
            rb.AddForce(transform.forward * 10f);
        }
        
        // Kinematic control (moves physics body)
        if (rb.isKinematic)
        {
            rb.MovePosition(rb.position + transform.forward * Time.fixedDeltaTime);
        }
    }
    
    // Collision callbacks
    void OnCollisionEnter(Collision collision)
    {
        Debug.Log("Collision with " + collision.gameObject.name);
        ContactPoint contact = collision.contacts[0];
        Vector3 normal = contact.normal;
        Vector3 point = contact.point;
    }
    
    void OnCollisionStay(Collision collision) { }
    void OnCollisionExit(Collision collision) { }
    
    // Trigger callbacks
    void OnTriggerEnter(Collider other) { }
    void OnTriggerStay(Collider other) { }
    void OnTriggerExit(Collider other) { }
}

// Physics settings (Project Settings > Physics)
// - Fixed Timestep: 0.02 (50 Hz)
// - Gravity: (0, -9.81, 0)
// - Default Solver Iterations: 6
// - Default Solver Velocity Iterations: 1
```

#### Unreal (C++)

```cpp
#include "Components/StaticMeshComponent.h"
#include "PhysicsEngine/BodyInstance.h"

class APhysicsActor : public AActor
{
public:
    UPROPERTY(VisibleAnywhere)
    UStaticMeshComponent* MeshComponent;
    
    APhysicsActor()
    {
        MeshComponent = CreateDefaultSubobject<UStaticMeshComponent>(TEXT("Mesh"));
        RootComponent = MeshComponent;
        
        // Enable physics
        MeshComponent->SetSimulatePhysics(true);
        MeshComponent->SetMassOverrideInKg(NAME_None, 10.0f);
        MeshComponent->SetLinearDamping(0.5f);
        MeshComponent->SetAngularDamping(0.05f);
        MeshComponent->SetEnableGravity(true);
        
        // Collision setup
        MeshComponent->SetCollisionEnabled(ECollisionEnabled::QueryAndPhysics);
        MeshComponent->SetCollisionObjectType(ECollisionChannel::ECC_PhysicsBody);
        MeshComponent->SetCollisionResponseToAllChannels(ECollisionResponse::ECR_Block);
    }
    
    void BeginPlay() override
    {
        Super::BeginPlay();
        
        // Apply impulse
        MeshComponent->AddImpulse(FVector(0, 0, 1000));
        MeshComponent->AddTorqueInRadians(FVector(0, 0, 100));
    }
    
    void Tick(float DeltaTime) override
    {
        Super::Tick(DeltaTime);
        
        // For kinematic movement
        if (!MeshComponent->IsSimulatingPhysics())
        {
            FVector NewLocation = GetActorLocation() + FVector(0, 0, 100) * DeltaTime;
            MeshComponent->SetWorldLocation(NewLocation, true);  // Sweep for collisions
        }
    }
    
    // Collision events
    UFUNCTION()
    void OnHit(UPrimitiveComponent* HitComponent, AActor* OtherActor,
               UPrimitiveComponent* OtherComp, FVector NormalImpulse, const FHitResult& Hit)
    {
        UE_LOG(LogTemp, Warning, TEXT("Hit %s"), *OtherActor->GetName());
    }
    
    UFUNCTION()
    void OnBeginOverlap(UPrimitiveComponent* OverlappedComponent, AActor* OtherActor,
                        UPrimitiveComponent* OtherComp, int32 OtherBodyIndex,
                        bool bFromSweep, const FHitResult& SweepResult)
    {
        // Trigger overlap
    }
};

// Physics substep configuration (Project Settings)
// - Max Substep Delta Time: 0.0166 (60 Hz)
// - Max Substeps: 6
// - Enable Substepping: true
```

#### Godot (GDScript)

```gdscript
extends RigidBody3D

func _ready():
    # RigidBody3D properties
    mass = 10.0
    gravity_scale = 1.0
    linear_damp = 0.5
    angular_damp = 0.05
    
    # Collision layer/mask (bitflags)
    collision_layer = 1
    collision_mask = 1
    
    # Add collision shape
    var shape = BoxShape3D.new()
    shape.size = Vector3(1, 1, 1)
    
    var collision_shape = CollisionShape3D.new()
    collision_shape.shape = shape
    add_child(collision_shape)
    
    # Apply impulse
    apply_central_impulse(Vector3(0, 0, 10))
    apply_torque_impulse(Vector3(0, 1, 0))

# Physics processing (fixed timestep)
func _physics_process(delta):
    # Apply forces in _physics_process (not _process!)
    if Input.is_action_pressed("move_forward"):
        apply_central_force(global_transform.basis.z * -100)
    
    # Kinematic mode
    if freeze:  # Similar to kinematic
        global_position += global_transform.basis.z * delta

# Collision signals
func _on_body_entered(body: Node):
    print("Collision with ", body.name)

func _on_body_exited(body: Node):
    pass

# Setup signals in _ready
func _ready():
    body_entered.connect(_on_body_entered)
    body_exited.connect(_on_body_exited)

# Physics settings (Project Settings > Physics)
# - Default Gravity: 9.8
# - Physics Ticks Per Second: 60
# - Physics Jitter Fix: 0.5
```

#### Praxis (Rust)

```rust
use bevy_ecs::prelude::*;
use rapier3d::prelude::*;
use praxis_physics::{PhysicsWorld, PhysicsConfig};

// Components
#[derive(Component)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
    pub mass: f32,
    pub linear_damping: f32,
    pub angular_damping: f32,
}

#[derive(Component)]
pub struct Collider {
    pub shape: ColliderShape,
    pub is_trigger: bool,
}

#[derive(Component)]
pub struct PhysicsVelocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

// Spawn entity with physics
fn spawn_physics_entity(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        GlobalTransform::default(),
        RigidBody {
            body_type: RigidBodyType::Dynamic,
            mass: 10.0,
            linear_damping: 0.5,
            angular_damping: 0.05,
        },
        Collider {
            shape: ColliderShape::Box(Vec3::new(0.5, 0.5, 0.5)),
            is_trigger: false,
        },
        PhysicsVelocity {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        },
    ));
}

// Physics systems (run in fixed timestep)
fn physics_system_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_transforms_to_physics,
        step_physics,
        sync_transforms_from_physics,
        handle_collision_events,
    ).chain());
    schedule
}

// 1. Sync transforms to physics (kinematic entities)
fn sync_transforms_to_physics(
    query: Query<(&Transform, &RigidBody), Changed<Transform>>,
    mut physics_world: ResMut<PhysicsWorld>,
) {
    for (transform, rigidbody) in query.iter() {
        if rigidbody.body_type == RigidBodyType::KinematicPositionBased {
            // Update physics body position from transform
            physics_world.set_body_position(entity, transform.translation, transform.rotation);
        }
    }
}

// 2. Step physics simulation
fn step_physics(
    mut physics_world: ResMut<PhysicsWorld>,
    config: Res<PhysicsConfig>,
) {
    physics_world.step(config.timestep);
}

// 3. Sync transforms from physics (dynamic entities)
fn sync_transforms_from_physics(
    mut query: Query<(&mut Transform, &RigidBody)>,
    physics_world: Res<PhysicsWorld>,
) {
    for (mut transform, rigidbody) in query.iter_mut() {
        if rigidbody.body_type == RigidBodyType::Dynamic {
            // Read physics body position into transform
            if let Some((position, rotation)) = physics_world.get_body_transform(entity) {
                transform.translation = position;
                transform.rotation = rotation;
            }
        }
    }
}

// 4. Handle collision events
fn handle_collision_events(
    physics_world: Res<PhysicsWorld>,
    query: Query<&Name>,
) {
    for event in physics_world.collision_events() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                println!("Collision started between {:?} and {:?}", e1, e2);
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                println!("Collision ended between {:?} and {:?}", e1, e2);
            }
        }
    }
}

// Apply forces
fn apply_forces(
    mut query: Query<&mut PhysicsVelocity>,
    keyboard: Res<Input<KeyCode>>,
) {
    for mut velocity in query.iter_mut() {
        if keyboard.pressed(KeyCode::W) {
            velocity.linear.z -= 10.0;
        }
    }
}

// Physics configuration
let physics_config = PhysicsConfig {
    timestep: 1.0 / 60.0,  // 60 Hz
    gravity: Vec3::new(0.0, -9.81, 0.0),
    iterations: 4,
    ..Default::default()
};
```

## Fixed Timestep Implementation

### Unity

```csharp
// Unity internal (conceptual)
class PhysicsManager
{
    float fixedDeltaTime = 0.02f;  // 50 Hz
    float accumulator = 0.0f;
    
    void Update(float deltaTime)
    {
        accumulator += deltaTime;
        
        // Fixed timestep loop
        while (accumulator >= fixedDeltaTime)
        {
            // Run FixedUpdate on all MonoBehaviours
            RunFixedUpdate();
            
            // Step physics
            Physics.Simulate(fixedDeltaTime);
            
            accumulator -= fixedDeltaTime;
        }
    }
}

// Time.fixedDeltaTime is constant (0.02)
// Time.deltaTime varies each frame
```

### Unreal

```cpp
// Unreal uses substepping (advanced fixed timestep)
void UWorld::Tick(ELevelTick TickType, float DeltaSeconds)
{
    // Physics substep configuration
    const float MaxSubstepDeltaTime = 0.0166f;  // 60 Hz
    const int32 MaxSubsteps = 6;
    
    float RemainingTime = DeltaSeconds;
    int32 NumSubsteps = 0;
    
    while (RemainingTime > SMALL_NUMBER && NumSubsteps < MaxSubsteps)
    {
        float SubDeltaTime = FMath::Min(RemainingTime, MaxSubstepDeltaTime);
        
        // Simulate physics for substep
        PhysicsScene->StepSimulation(SubDeltaTime);
        
        RemainingTime -= SubDeltaTime;
        NumSubsteps++;
    }
}
```

### Godot

```gdscript
# Godot internal (conceptual)
# _physics_process automatically called at fixed rate

# Engine configuration
var physics_fps = 60  # Default
var fixed_delta = 1.0 / physics_fps

# In _physics_process, delta is always 1/60 (0.0166...)
func _physics_process(delta):
    assert(delta == 1.0 / 60.0)  # Always true (approximately)
```

### Praxis

```rust
// Explicit fixed timestep accumulator
pub struct FixedTimestep {
    timestep: f32,
    accumulator: f32,
}

impl FixedTimestep {
    pub fn new(hz: f32) -> Self {
        Self {
            timestep: 1.0 / hz,
            accumulator: 0.0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32, mut physics_step: impl FnMut()) {
        self.accumulator += delta_time;
        
        while self.accumulator >= self.timestep {
            physics_step();
            self.accumulator -= self.timestep;
        }
    }
}

// Usage in game loop
fn game_loop(world: &mut World) {
    let mut fixed_timestep = FixedTimestep::new(60.0);  // 60 Hz
    
    loop {
        let delta_time = get_frame_time();
        
        // Fixed timestep physics
        fixed_timestep.update(delta_time, || {
            run_physics_systems(world);
        });
        
        // Variable timestep rendering
        run_render_systems(world);
    }
}
```

## Character Controller Implementation

### Unity

```csharp
public class CharacterController : MonoBehaviour
{
    public float speed = 5f;
    public float jumpForce = 5f;
    
    private Rigidbody rb;
    private bool isGrounded;
    
    void Start()
    {
        rb = GetComponent<Rigidbody>();
        rb.constraints = RigidbodyConstraints.FreezeRotation;  // Prevent tipping
    }
    
    void FixedUpdate()
    {
        // Ground check
        isGrounded = Physics.Raycast(transform.position, Vector3.down, 1.1f);
        
        // Movement
        float horizontal = Input.GetAxis("Horizontal");
        float vertical = Input.GetAxis("Vertical");
        
        Vector3 movement = new Vector3(horizontal, 0, vertical) * speed;
        rb.velocity = new Vector3(movement.x, rb.velocity.y, movement.z);
        
        // Jump
        if (Input.GetButton("Jump") && isGrounded)
        {
            rb.AddForce(Vector3.up * jumpForce, ForceMode.Impulse);
        }
    }
}

// Or use built-in CharacterController component (capsule-based)
public class CharacterControllerExample : MonoBehaviour
{
    private CharacterController controller;
    
    void Update()
    {
        float horizontal = Input.GetAxis("Horizontal");
        float vertical = Input.GetAxis("Vertical");
        
        Vector3 move = transform.right * horizontal + transform.forward * vertical;
        controller.Move(move * speed * Time.deltaTime);
        
        // Gravity
        if (!controller.isGrounded)
        {
            velocity.y += gravity * Time.deltaTime;
        }
        controller.Move(velocity * Time.deltaTime);
    }
}
```

### Unreal

```cpp
// Use UCharacterMovementComponent (built-in)
class AMyCharacter : public ACharacter
{
public:
    AMyCharacter()
    {
        // CharacterMovementComponent is created by ACharacter
        UCharacterMovementComponent* Movement = GetCharacterMovement();
        Movement->MaxWalkSpeed = 600.0f;
        Movement->JumpZVelocity = 600.0f;
        Movement->GravityScale = 1.0f;
        Movement->GroundFriction = 8.0f;
    }
    
    void SetupPlayerInputComponent(UInputComponent* PlayerInputComponent) override
    {
        PlayerInputComponent->BindAxis("MoveForward", this, &AMyCharacter::MoveForward);
        PlayerInputComponent->BindAxis("MoveRight", this, &AMyCharacter::MoveRight);
        PlayerInputComponent->BindAction("Jump", IE_Pressed, this, &ACharacter::Jump);
    }
    
    void MoveForward(float Value)
    {
        AddMovementInput(GetActorForwardVector(), Value);
    }
    
    void MoveRight(float Value)
    {
        AddMovementInput(GetActorRightVector(), Value);
    }
};
```

### Godot

```gdscript
extends CharacterBody3D

var speed = 5.0
var jump_velocity = 5.0
var gravity = 9.8

func _physics_process(delta):
    # Gravity
    if not is_on_floor():
        velocity.y -= gravity * delta
    
    # Jump
    if Input.is_action_just_pressed("jump") and is_on_floor():
        velocity.y = jump_velocity
    
    # Movement
    var input_dir = Input.get_vector("left", "right", "forward", "back")
    var direction = (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
    
    if direction:
        velocity.x = direction.x * speed
        velocity.z = direction.z * speed
    else:
        velocity.x = move_toward(velocity.x, 0, speed)
        velocity.z = move_toward(velocity.z, 0, speed)
    
    move_and_slide()  # Built-in kinematic movement with collision
```

### Praxis

```rust
#[derive(Component)]
pub struct CharacterController {
    pub speed: f32,
    pub jump_force: f32,
    pub is_grounded: bool,
}

fn character_movement_system(
    mut query: Query<(&mut Transform, &mut PhysicsVelocity, &CharacterController)>,
    keyboard: Res<Input<KeyCode>>,
    physics_world: Res<PhysicsWorld>,
) {
    for (mut transform, mut velocity, controller) in query.iter_mut() {
        // Ground check (raycast)
        let is_grounded = physics_world.raycast(
            transform.translation,
            Vec3::NEG_Y,
            1.1,
            QueryFilter::default(),
        ).is_some();
        
        // Movement input
        let mut movement = Vec3::ZERO;
        if keyboard.pressed(KeyCode::W) { movement.z -= 1.0; }
        if keyboard.pressed(KeyCode::S) { movement.z += 1.0; }
        if keyboard.pressed(KeyCode::A) { movement.x -= 1.0; }
        if keyboard.pressed(KeyCode::D) { movement.x += 1.0; }
        
        movement = movement.normalize_or_zero() * controller.speed;
        velocity.linear.x = movement.x;
        velocity.linear.z = movement.z;
        
        // Jump
        if keyboard.just_pressed(KeyCode::Space) && is_grounded {
            velocity.linear.y = controller.jump_force;
        }
    }
}
```

## Trade-Off Analysis

### Unity

**Pros**:
- PhysX is industry-standard and well-optimized
- Automatic transform synchronization
- FixedUpdate pattern is intuitive
- Built-in CharacterController
- Good collision callbacks

**Cons**:
- PhysX closed-source (less customizable)
- Fixed timestep can cause spiral of death if slow
- Component lookups have overhead
- 2D and 3D physics are separate (different APIs)

**Performance**: Excellent for typical game physics

### Unreal

**Pros**:
- Chaos physics in UE5 (highly advanced)
- Substepping provides stability with variable framerate
- CharacterMovementComponent is production-ready
- Async physics simulation option
- Excellent profiling tools

**Cons**:
- Chaos more complex to understand
- Heavier runtime than simpler engines
- C++ recompilation for physics changes
- Substepping can be expensive

**Performance**: AAA-quality, scales to large worlds

### Godot

**Pros**:
- Godot Physics is open-source and customizable
- `_physics_process` is simple and clear
- Built-in CharacterBody3D for kinematic controllers
- Lightweight overhead
- Good for 2D and 3D

**Cons**:
- Godot Physics less mature than PhysX/Chaos
- Fewer advanced features (CCD, complex constraints)
- Bullet integration optional but not default
- Performance not as optimized as commercial engines

**Performance**: Good for indie/medium-scale games

### Praxis

**Pros**:
- Rapier3D is pure Rust (cross-platform, safe)
- Full control over sync and timestep
- Can optimize sync for specific use cases
- Deterministic (useful for networking)
- No GC pauses

**Cons**:
- Manual synchronization required
- More boilerplate code
- No built-in character controller
- Rapier less battle-tested than PhysX
- Must implement collision callbacks manually

**Performance**: Excellent, especially for deterministic simulation

## Key Takeaways

### Universal Principles

1. **Fixed Timestep is Essential**: Physics simulation requires constant time steps for stability
2. **Bidirectional Sync**: Kinematic bodies drive physics, dynamic bodies drive transforms
3. **Collision Layers**: Use layers/masks to filter collisions efficiently
4. **Character Controllers**: Specialized kinematic movement for player characters
5. **Continuous Collision Detection**: Prevent fast objects tunneling through geometry

### Design Patterns to Steal

- **Accumulator Pattern**: Buffer frame time, step physics in fixed increments
- **Substepping**: Break large timesteps into smaller substeps (Unreal approach)
- **Collision Callbacks**: Events for started/stayed/ended collisions
- **Raycast Filtering**: Query filter with layers, ignoreentities, etc.
- **Deferred Physics Operations**: Buffer adds/removes, apply between steps

### Common Pitfalls

- **Applying Forces in Variable Timestep**: Always use fixed timestep for forces
- **Reading Physics Data Mid-Step**: Can cause race conditions
- **Forgetting to Sync**: Manual systems must sync transforms both ways
- **Spiral of Death**: Fixed timestep accumulator grows unbounded (cap iterations!)
- **Kinematic-Dynamic Interaction**: Moving kinematic bodies can push dynamic objects

## Further Reading

### Unity
- [Physics Overview](https://docs.unity3d.com/Manual/PhysicsOverview.html)
- [Rigidbody](https://docs.unity3d.com/ScriptReference/Rigidbody.html)
- [CharacterController](https://docs.unity3d.com/ScriptReference/CharacterController.html)

### Unreal
- [Physics](https://docs.unrealengine.com/5.0/en-US/physics-in-unreal-engine/)
- [Chaos Physics](https://docs.unrealengine.com/5.0/en-US/chaos-physics-in-unreal-engine/)
- [CharacterMovementComponent](https://docs.unrealengine.com/5.0/en-US/API/Runtime/Engine/GameFramework/UCharacterMovementComponent/)

### Godot
- [Physics Introduction](https://docs.godotengine.org/en/stable/tutorials/physics/physics_introduction.html)
- [Rigidbody3D](https://docs.godotengine.org/en/stable/classes/class_rigidbody3d.html)
- [CharacterBody3D](https://docs.godotengine.org/en/stable/classes/class_characterbody3d.html)

### Praxis
- [Praxis Physics](../../../crates/praxis_physics/README.md)
- [Rapier Documentation](https://rapier.rs/)

### General
- [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/)
- [Game Physics](https://www.gamephysics.com/)
