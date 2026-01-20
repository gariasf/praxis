# Animation Systems: Multi-Engine Comparison

**Complexity**: Intermediate  
**Curriculum Module**: [Module 4 - Transform Hierarchies](../modules/04-transform-hierarchies.md)

## Problem Statement

Game engines must animate 3D characters and objects smoothly. Key challenges:

- How do we represent skeletal animation (bones, skin weights)?
- How do we blend between multiple animations?
- How do we support animation layers (additive, override)?
- How do we handle animation state machines and transitions?
- How do we optimize animation performance for many characters?

## Design Philosophy Comparison

| Engine | Animation Model | Blending System | State Machine |
|--------|----------------|-----------------|---------------|
| **Unity** | Mecanim Animator | Blend trees, layers | Visual Animator Controller |
| **Unreal** | Animation Blueprint | Blend spaces, layers | State machine graph |
| **Godot** | AnimationPlayer + AnimationTree | Blend tree nodes | AnimationNodeStateMachine |
| **Praxis** | Code-based AnimationPlayer | Manual blend implementation | DIY state machine |

## Implementation Examples

### Playing Simple Animation

#### Unity (C#)

```csharp
using UnityEngine;

public class SimpleAnimation : MonoBehaviour
{
    private Animator animator;
    
    void Start()
    {
        animator = GetComponent<Animator>();
        
        // Animator Controller must be assigned in Inspector
        // Controller contains animation states and transitions
    }
    
    void Update()
    {
        // Trigger animation via parameter
        if (Input.GetKeyDown(KeyCode.Space))
        {
            animator.SetTrigger("Jump");  // Transition to Jump state
        }
        
        // Set bool parameter
        bool isWalking = Input.GetKey(KeyCode::W);
        animator.SetBool("IsWalking", isWalking);
        
        // Set float parameter (for blend trees)
        float speed = GetComponent<Rigidbody>().velocity.magnitude;
        animator.SetFloat("Speed", speed);
        
        // Cross-fade to animation
        animator.CrossFade("Run", 0.2f);  // 0.2s transition
    }
}

// Animator Controller structure (visual in editor):
// States:
//   - Idle (default)
//   - Walk (IsWalking == true)
//   - Run (Speed > 2.0)
//   - Jump (Jump trigger)
// Transitions:
//   Idle -> Walk: IsWalking == true
//   Walk -> Idle: IsWalking == false
//   Any -> Jump: Jump trigger
```

#### Unreal (C++)

```cpp
#include "Animation/AnimInstance.h"
#include "Animation/AnimSequence.h"

// Animation Blueprint class (generated from C++)
class UMyAnimInstance : public UAnimInstance
{
    GENERATED_BODY()
    
public:
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Animation")
    float Speed;
    
    UPROPERTY(EditAnywhere, BlueprintReadWrite, Category = "Animation")
    bool bIsInAir;
    
    virtual void NativeUpdateAnimation(float DeltaSeconds) override
    {
        Super::NativeUpdateAnimation(DeltaSeconds);
        
        // Update animation variables
        APawn* Pawn = TryGetPawnOwner();
        if (Pawn)
        {
            Speed = Pawn->GetVelocity().Size();
            bIsInAir = Pawn->GetMovementComponent()->IsFalling();
        }
    }
};

// Playing animation directly (without Animation Blueprint)
class AMyCharacter : public ACharacter
{
public:
    UPROPERTY(EditAnywhere)
    UAnimSequence* JumpAnimation;
    
    void Jump()
    {
        // Play montage (animation with events, sections, etc.)
        UAnimInstance* AnimInstance = GetMesh()->GetAnimInstance();
        if (AnimInstance && JumpAnimation)
        {
            AnimInstance->Montage_Play(JumpAnimation, 1.0f);  // 1.0 = normal speed
        }
    }
};

// Animation Blueprint (visual node graph):
// - State Machine:
//   - Idle/Walk/Run (blend based on Speed)
//   - Jump (bIsInAir == true)
// - Blend Space: Walk-Run blending based on Speed (0-6 m/s)
```

#### Godot (GDScript)

```gdscript
extends Node3D

@onready var animation_player = $AnimationPlayer

func _ready():
    # AnimationPlayer has animations added in editor
    # or created via code
    pass

func _process(delta):
    # Play animation
    if Input.is_action_just_pressed("jump"):
        animation_player.play("jump")
    
    # Check if animation is playing
    if animation_player.is_playing():
        var current = animation_player.current_animation
        print("Playing:", current)
    
    # Set speed multiplier
    animation_player.speed_scale = 1.5  # Play 1.5x faster
    
    # Seek to specific time
    animation_player.seek(0.5, true)  # 0.5 seconds, update immediately
    
    # Blend to animation
    animation_player.play("walk")
    animation_player.advance(delta)

# Animation defined in editor or code:
# - Animation "walk": 
#   - Track 0: Transform of Bone "Leg_L"
#   - Track 1: Transform of Bone "Leg_R"
#   - Duration: 1.0 second
#   - Loop: true
```

#### Praxis (Rust)

```rust
use praxis_scene::animation::{AnimationPlayer, AnimationClip, Skeleton};

#[derive(Component)]
pub struct AnimationPlayer {
    current_clip: Option<Handle<AnimationClip>>,
    time: f32,
    speed: f32,
    looping: bool,
}

impl AnimationPlayer {
    pub fn play(&mut self, clip: Handle<AnimationClip>) {
        self.current_clip = Some(clip);
        self.time = 0.0;
    }
    
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}

// Animation update system
fn animation_update_system(
    mut query: Query<(&mut AnimationPlayer, &Skeleton)>,
    clips: Res<Assets<AnimationClip>>,
    time: Res<Time>,
) {
    for (mut player, skeleton) in query.iter_mut() {
        if let Some(clip_handle) = &player.current_clip {
            if let Some(clip) = clips.get(clip_handle) {
                // Advance time
                player.time += time.delta_seconds() * player.speed;
                
                // Loop or clamp
                if player.looping {
                    player.time %= clip.duration;
                } else {
                    player.time = player.time.min(clip.duration);
                }
                
                // Sample animation at current time
                let pose = clip.sample(player.time);
                
                // Apply pose to skeleton
                apply_pose_to_skeleton(skeleton, &pose);
            }
        }
    }
}

// Playing animation
fn play_animation(
    mut query: Query<&mut AnimationPlayer>,
    input: Res<Input<KeyCode>>,
    animations: Res<AnimationAssets>,
) {
    for mut player in query.iter_mut() {
        if input.just_pressed(KeyCode::Space) {
            player.play(animations.jump_clip.clone());
        }
    }
}
```

### Animation Blending

#### Unity (Blend Trees)

```csharp
// Blend Tree created in Animator Controller (visual editor)
// Example: Locomotion Blend Tree
// - Parameter: "Speed" (0 to 6)
// - Blend Type: 1D
// - Motions:
//   - Idle (Speed: 0)
//   - Walk (Speed: 2)
//   - Run (Speed: 6)
// - Blending: Linear between keyframes

public class BlendTreeController : MonoBehaviour
{
    private Animator animator;
    
    void Update()
    {
        float speed = CalculateSpeed();
        animator.SetFloat("Speed", speed);  // Automatically blends Idle/Walk/Run
    }
}

// 2D Blend Tree (e.g., directional movement)
// - Parameters: "MoveX", "MoveY"
// - Blend Type: 2D Freeform Directional
// - Motions:
//   - Walk Forward (X: 0, Y: 1)
//   - Walk Back (X: 0, Y: -1)
//   - Strafe Left (X: -1, Y: 0)
//   - Strafe Right (X: 1, Y: 0)
```

#### Unreal (Blend Spaces)

```cpp
// Blend Space created in editor (UBlendSpace asset)
// Example: 1D Blend Space for locomotion
// - Horizontal Axis: Speed (0 to 600)
// - Sample Points:
//   - Idle (Speed: 0)
//   - Walk (Speed: 200)
//   - Run (Speed: 600)

// Used in Animation Blueprint
void UMyAnimInstance::NativeUpdateAnimation(float DeltaSeconds)
{
    // Speed automatically feeds into Blend Space in anim graph
    Speed = GetOwningActor()->GetVelocity().Size();
    
    // Blend Space outputs blended pose
}

// Animation Blueprint (visual):
// - Blend Space Node: "LocomotionBlendSpace"
//   - Input: Speed (linked to Speed variable)
//   - Output: Blended animation pose
```

#### Godot (AnimationTree)

```gdscript
extends Node3D

@onready var animation_tree = $AnimationTree
@onready var state_machine = animation_tree.get("parameters/playback")

func _ready():
    animation_tree.active = true

func _process(delta):
    # Blend between animations using blend amount
    var speed = calculate_speed()
    
    # Set blend parameter (0 = idle, 1 = run)
    animation_tree.set("parameters/IdleRunBlend/blend_amount", speed / max_speed)
    
    # Or use state machine
    if speed > 0.1:
        state_machine.travel("Run")
    else:
        state_machine.travel("Idle")

# AnimationTree structure (created in editor):
# - Root: BlendTree
#   - BlendSpace1D node:
#     - 0.0: Idle animation
#     - 1.0: Run animation
#     - Parameter: "IdleRunBlend/blend_amount"
```

#### Praxis (Manual Blending)

```rust
pub struct AnimationBlender {
    clip_a: Handle<AnimationClip>,
    clip_b: Handle<AnimationClip>,
    blend_weight: f32,  // 0.0 = full A, 1.0 = full B
}

fn blend_animations(
    clip_a: &AnimationClip,
    clip_b: &AnimationClip,
    time_a: f32,
    time_b: f32,
    weight: f32,
) -> AnimationPose {
    let pose_a = clip_a.sample(time_a);
    let pose_b = clip_b.sample(time_b);
    
    // Blend each bone transform
    let mut blended_pose = AnimationPose::default();
    for (bone_idx, (transform_a, transform_b)) in pose_a.iter().zip(pose_b.iter()).enumerate() {
        blended_pose.bone_transforms[bone_idx] = Transform {
            translation: transform_a.translation.lerp(transform_b.translation, weight),
            rotation: transform_a.rotation.slerp(transform_b.rotation, weight),
            scale: transform_a.scale.lerp(transform_b.scale, weight),
        };
    }
    
    blended_pose
}

// Blend tree system
fn animation_blend_system(
    mut query: Query<(&AnimationBlender, &mut Skeleton)>,
    clips: Res<Assets<AnimationClip>>,
    time: Res<Time>,
) {
    for (blender, mut skeleton) in query.iter_mut() {
        let clip_a = clips.get(&blender.clip_a).unwrap();
        let clip_b = clips.get(&blender.clip_b).unwrap();
        
        let pose = blend_animations(clip_a, clip_b, time.elapsed(), time.elapsed(), blender.blend_weight);
        
        apply_pose_to_skeleton(&mut skeleton, &pose);
    }
}
```

### Animation Layers and Masking

#### Unity

```csharp
// Layers defined in Animator Controller
// - Base Layer: Full body locomotion
// - Upper Body Layer: Shooting/reloading (only affects upper body bones)
//   - Avatar Mask: Only spine, arms, head
//   - Blending: Additive or Override
//   - Weight: 0.0 to 1.0

public class LayeredAnimation : MonoBehaviour
{
    private Animator animator;
    
    void Update()
    {
        // Control layer weight
        if (Input.GetKey(KeyCode.Mouse0))  // Shooting
        {
            animator.SetLayerWeight(1, 1.0f);  // Full upper body override
            animator.SetTrigger("Shoot");
        }
        else
        {
            animator.SetLayerWeight(1, 0.0f);  // Blend out
        }
    }
}
```

#### Unreal

```cpp
// Animation Blueprint: Layered blend per bone
// - Base: Full body locomotion
// - Blend: Upper body animation
// - Bone Name: "Spine1" (affects spine and all children)
// - Blend Depth: 1.0 (full replacement)

void UMyAnimInstance::NativeUpdateAnimation(float DeltaSeconds)
{
    // Control blend alpha in blueprint
    bIsAiming = /* ... */;
    UpperBodyBlendAlpha = bIsAiming ? 1.0f : 0.0f;
}

// Animation Blueprint (visual):
// - Locomotion State Machine -> Output Pose
// - Aiming Animation -> Layered Blend Per Bone
//   - Base Pose: Locomotion output
//   - Blend Pose: Aiming animation
//   - Blend Alpha: UpperBodyBlendAlpha
//   - Branch Filters: Spine, Arms
```

#### Godot

```gdscript
# AnimationTree with BlendSpace or Add nodes
func _process(delta):
    # Additive animation (e.g., breathing on top of idle)
    animation_tree.set("parameters/AddBreathing/add_amount", 0.5)
    
    # Or use blend filter (bone mask)
    # Set in AnimationTree node properties:
    # - Filter: Enabled
    # - Tracks: Select only upper body bones
```

#### Praxis

```rust
pub struct AnimationLayer {
    clip: Handle<AnimationClip>,
    bone_mask: Vec<bool>,  // Which bones this layer affects
    weight: f32,
    blend_mode: BlendMode,
}

enum BlendMode {
    Override,
    Additive,
}

fn layered_animation_system(
    mut query: Query<(&AnimationLayers, &mut Skeleton)>,
    clips: Res<Assets<AnimationClip>>,
) {
    for (layers, mut skeleton) in query.iter_mut() {
        let mut final_pose = AnimationPose::default();
        
        // Apply each layer in order
        for layer in &layers.0 {
            let clip = clips.get(&layer.clip).unwrap();
            let layer_pose = clip.sample(layer.time);
            
            for (bone_idx, &is_affected) in layer.bone_mask.iter().enumerate() {
                if is_affected {
                    match layer.blend_mode {
                        BlendMode::Override => {
                            final_pose.bone_transforms[bone_idx] = 
                                final_pose.bone_transforms[bone_idx]
                                    .lerp(layer_pose.bone_transforms[bone_idx], layer.weight);
                        }
                        BlendMode::Additive => {
                            final_pose.bone_transforms[bone_idx] = 
                                final_pose.bone_transforms[bone_idx]
                                    .add(layer_pose.bone_transforms[bone_idx] * layer.weight);
                        }
                    }
                }
            }
        }
        
        apply_pose_to_skeleton(&mut skeleton, &final_pose);
    }
}
```

## State Machines

### Unity (Animator Controller)

```csharp
// Visual state machine in Animator Controller
// States:
//   - Idle
//   - Walk
//   - Run
//   - Jump
// Transitions:
//   - Idle -> Walk: Speed > 0.1
//   - Walk -> Run: Speed > 2.0
//   - Any -> Jump: Jump trigger
//   - Jump -> Idle: ExitTime + IsGrounded
```

### Unreal (Animation Blueprint)

```cpp
// Animation Blueprint State Machine (visual)
// Similar structure to Unity
// Can also use AnimNotify for events:

UCLASS()
class UMyAnimNotify : public UAnimNotify
{
public:
    virtual void Notify(USkeletalMeshComponent* MeshComp, UAnimSequenceBase* Animation) override
    {
        // Called at specific point in animation
        // E.g., footstep sound, spawn particle effect
    }
};
```

### Godot (AnimationNodeStateMachine)

```gdscript
# AnimationTree with StateMachine node
# States defined in editor, transitions set programmatically

func _process(delta):
    var state_machine = animation_tree.get("parameters/StateMachine/playback")
    
    if speed > 0.1:
        state_machine.travel("Walk")
    else:
        state_machine.travel("Idle")
    
    if is_jumping:
        state_machine.travel("Jump")
```

### Praxis (DIY State Machine)

```rust
enum AnimationState {
    Idle,
    Walk,
    Run,
    Jump,
}

pub struct AnimationStateMachine {
    current_state: AnimationState,
    transitions: HashMap<(AnimationState, AnimationState), TransitionCondition>,
}

impl AnimationStateMachine {
    pub fn update(&mut self, context: &AnimationContext) {
        // Check for valid transitions from current state
        for ((from, to), condition) in &self.transitions {
            if *from == self.current_state && condition.evaluate(context) {
                self.current_state = *to;
                break;
            }
        }
    }
    
    pub fn get_animation(&self) -> Handle<AnimationClip> {
        match self.current_state {
            AnimationState::Idle => self.idle_clip,
            AnimationState::Walk => self.walk_clip,
            AnimationState::Run => self.run_clip,
            AnimationState::Jump => self.jump_clip,
        }
    }
}
```

## Trade-Off Analysis

### Unity (Mecanim)

**Pros**:
- Visual Animator Controller (designer-friendly)
- Powerful blend trees (1D, 2D, freeform)
- Animation layers with masking
- State machine transitions
- Humanoid retargeting (share animations between characters)

**Cons**:
- Animator Controller can become complex
- Parameter synchronization overhead
- Less performant than code-only approaches
- Runtime animation creation limited

### Unreal (Animation Blueprints)

**Pros**:
- Most powerful visual animation system
- Control rigs for procedural animation
- Animation montages (events, sections)
- Pose blending very sophisticated
- Animation compression excellent

**Cons**:
- Blueprint performance overhead
- Complex learning curve
- C++ animation harder to debug
- Heavy for simple use cases

### Godot (AnimationPlayer/AnimationTree)

**Pros**:
- Simple AnimationPlayer for basic needs
- AnimationTree adds blend trees and state machines
- Can animate any property (not just bones)
- Lightweight
- Easy to learn

**Cons**:
- Less sophisticated than Unity/Unreal
- Fewer built-in blend types
- Performance not as optimized
- Retargeting less robust

### Praxis (Code-Based)

**Pros**:
- Full control over implementation
- Can optimize for specific needs
- No editor dependency
- Educational (see all details)
- Deterministic

**Cons**:
- Must implement everything
- No visual tools
- More code to write
- Designer-unfriendly

## Key Takeaways

### Universal Principles

1. **Skeletal Animation**: Bones + skin weights = deformed mesh
2. **Linear Interpolation (Lerp)**: Blend translations/scales linearly
3. **Spherical Interpolation (Slerp)**: Blend rotations with quaternions
4. **Layering**: Combine full-body + upper-body animations
5. **State Machines**: Manage animation transitions logically

### Design Patterns to Steal

- **Blend Trees**: Smooth transitions based on continuous parameters
- **Animation Layers**: Separate concerns (locomotion vs. actions)
- **Bone Masking**: Apply animations only to specific bones
- **Animation Events**: Trigger logic at keyframes (footsteps, VFX)
- **Retargeting**: Share animations across different skeletons

### Common Pitfalls

- **No Transition Blending**: Abrupt animation switches look bad
- **Ignoring Root Motion**: Animation movement vs. code movement mismatch
- **Too Many Layers**: Overhead from excessive layering
- **Poor Bone Hierarchy**: Deep hierarchies slow down skinning
- **No Animation Compression**: Uncompressed animations waste memory

## Further Reading

### Unity
- [Animation Overview](https://docs.unity3d.com/Manual/AnimationOverview.html)
- [Animator Controller](https://docs.unity3d.com/Manual/class-AnimatorController.html)
- [Blend Trees](https://docs.unity3d.com/Manual/class-BlendTree.html)

### Unreal
- [Animation System](https://docs.unrealengine.com/5.0/en-US/animation-system-in-unreal-engine/)
- [Animation Blueprints](https://docs.unrealengine.com/5.0/en-US/animation-blueprints-in-unreal-engine/)
- [Blend Spaces](https://docs.unrealengine.com/5.0/en-US/blend-spaces-in-unreal-engine/)

### Godot
- [Animation](https://docs.godotengine.org/en/stable/tutorials/animation/index.html)
- [AnimationPlayer](https://docs.godotengine.org/en/stable/classes/class_animationplayer.html)
- [AnimationTree](https://docs.godotengine.org/en/stable/classes/class_animationtree.html)

### Praxis
- [Praxis Animation](../../guides/animation/)
- [Skeletal Animation Guide](../../guides/animation/skeletal-basics.md)

### General
- [Skeletal Animation (Wikipedia)](https://en.wikipedia.org/wiki/Skeletal_animation)
- [Animation Blending](https://www.gamedeveloper.com/programming/animation-blending-achieving-inverse-kinematics-and-more)
