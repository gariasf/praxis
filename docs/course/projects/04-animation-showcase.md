# Project 04: Animation Showcase

**Difficulty**: Intermediate  
**Estimated Time**: 3-4 weeks  
**Core Learning**: Skeletal animation, animation blending, state machines, IK

## Overview

Build a character animation system that loads skeletal animations, blends between states, and responds to user input. This project teaches animation pipeline concepts, skeletal hierarchies, interpolation techniques, and state management for animation systems.

### Learning Objectives

- Load and play skeletal animations from GLTF/FBX
- Implement animation blending and crossfading
- Build animation state machines
- Understand bone hierarchies and skinning
- Apply inverse kinematics (optional advanced)
- Optimize animation performance

## Feature Requirements

### Core Features (Minimum Viable)

1. **Skeletal Animation Playback**
   - Load animated GLTF/FBX models
   - Play animation clips
   - Loop animations
   - Control playback speed
   - Pause/resume functionality

2. **Bone Hierarchy Visualization**
   - Display skeleton wireframe
   - Highlight individual bones
   - Show bone names and hierarchy
   - Visualize joint rotations

3. **Multiple Animation Clips**
   - Load multiple animations (idle, walk, run, jump)
   - Switch between clips
   - Clip duration display
   - Animation list UI

4. **Basic Blending**
   - Crossfade between two animations
   - Adjustable blend time
   - Smooth transitions
   - No popping or artifacts

### Extended Features (Recommended)

5. **Animation State Machine**
   - Define states (Idle, Walk, Run, Jump)
   - Transition conditions (input-driven or automatic)
   - State entry/exit actions
   - Visual state graph display

6. **Layered Animation**
   - Upper body and lower body layers
   - Additive animations (wave, point)
   - Layer masking (bone filtering)
   - Per-layer blending

7. **Advanced Blending**
   - Blend trees (1D: walk-to-run, 2D: directional movement)
   - Multiple input blending
   - Parameter-driven blending
   - Smooth parameter interpolation

### Stretch Goals

8. **Inverse Kinematics**
   - Two-bone IK (arm/leg)
   - Look-at IK (head tracking)
   - Foot IK for uneven terrain
   - Hand IK for grabbing objects

9. **Procedural Animation**
   - Dynamic head look-at
   - Procedural idle variations
   - Spring-based bone simulation (tail, hair)
   - Ragdoll blending (death animation)

## Architecture Guidance

### System Components

```
AnimationShowcase
├── AnimationSystem
│   ├── AnimationLoader (GLTF/FBX)
│   ├── AnimationPlayer
│   ├── BlendTree
│   └── StateMachine
├── SkeletonManager
│   ├── BoneHierarchy
│   ├── PoseCalculator
│   └── SkinningRenderer
├── IKSolver (optional)
│   ├── TwoBoneIK
│   ├── LookAtIK
│   └── FABRIKSolver
└── AnimationEditor
    ├── ClipLibrary
    ├── StateMachineEditor
    └── DebugVisualizer
```

### Data Structures

**Skeleton**
```
Skeleton:
  - bones: array of Bone
  - root_bone: Bone reference
  - bone_name_map: map<string, bone_index>

Bone:
  - name: string
  - parent_index: int (-1 for root)
  - local_transform: Transform (position, rotation, scale)
  - inverse_bind_pose: mat4 (for skinning)
```

**Animation Clip**
```
AnimationClip:
  - name: string
  - duration: float (seconds)
  - tracks: array of BoneTrack
  - loop: bool
  - sample_rate: float (fps)

BoneTrack:
  - bone_index: int
  - position_keyframes: array of (time, vec3)
  - rotation_keyframes: array of (time, quaternion)
  - scale_keyframes: array of (time, vec3)

Methods:
  - sample(time) -> Transform
    // Interpolate between keyframes
```

**Animation Player**
```
AnimationPlayer:
  - current_clip: AnimationClip
  - current_time: float
  - play_speed: float
  - is_playing: bool
  - loop_mode: Once | Loop | PingPong

Methods:
  - play(clip)
  - update(delta_time)
  - get_current_pose() -> array of bone transforms
  - seek(time)
```

**Blend Tree Node**
```
BlendNode:
  - type: Clip | Blend1D | Blend2D | Additive
  - children: array of BlendNode
  - blend_parameter: float or vec2
  
Blend1D:
  - clips: array of (AnimationClip, threshold)
  - parameter: float (e.g., speed)
  - output: blended pose

Blend2D:
  - clips: array of (AnimationClip, vec2_position)
  - parameter: vec2 (e.g., move_x, move_y)
  - output: blended pose using barycentric interpolation
```

**State Machine**
```
StateMachine:
  - states: map<string, State>
  - current_state: State
  - transitions: array of Transition
  
State:
  - name: string
  - animation: AnimationClip or BlendTree
  - on_enter: callback
  - on_exit: callback

Transition:
  - from_state: string
  - to_state: string
  - condition: predicate function
  - blend_time: float
```

### Animation Pipeline

```
update(delta_time):
  1. Update state machine (check transitions)
  2. Evaluate animation source:
     - If single clip: sample at current time
     - If blending: blend N poses with weights
     - If state machine: get current state's pose
  3. Compute local bone transforms for all bones
  4. Calculate global transforms (traverse hierarchy)
  5. Compute final skinning matrices:
     skinning_matrix = global_transform * inverse_bind_pose
  6. Upload matrices to GPU as uniform/storage buffer
  7. Render skinned mesh with matrices
```

### Blending Algorithm

**Linear Blend (Crossfade)**
```
blend_poses(pose_a, pose_b, weight):
  result = new Pose()
  for bone_index in skeleton.bones:
    pos = lerp(pose_a[bone_index].position, 
               pose_b[bone_index].position, 
               weight)
    rot = slerp(pose_a[bone_index].rotation, 
                pose_b[bone_index].rotation, 
                weight)
    scl = lerp(pose_a[bone_index].scale, 
               pose_b[bone_index].scale, 
               weight)
    result[bone_index] = Transform(pos, rot, scl)
  return result
```

**1D Blend Tree**
```
blend_1d(clips, thresholds, parameter):
  // Find two clips to blend between
  for i in 0..clips.length-1:
    if parameter >= thresholds[i] and parameter < thresholds[i+1]:
      weight = (parameter - thresholds[i]) / 
               (thresholds[i+1] - thresholds[i])
      return blend_poses(
        sample_clip(clips[i]), 
        sample_clip(clips[i+1]), 
        weight
      )
```

## Milestone Plan

### Milestone 1: Load and Play Single Animation (Week 1, Days 1-3)

**Goal**: Display animated character with single clip

**Tasks**:
- Load GLTF/FBX file with skeleton and animation
- Parse skeleton hierarchy (bones, parents, bind pose)
- Parse animation tracks (keyframes for each bone)
- Implement keyframe sampling (linear interpolation)
- Calculate global bone transforms from local
- Display animated character

**Deliverable**: Character playing single looping animation

### Milestone 2: Skeleton Visualization (Week 1, Days 4-5)

**Goal**: Debug visualization of skeleton

**Tasks**:
- Render bones as lines connecting joints
- Color code bones or number them
- Display bone names on hover
- Toggle skeleton overlay on/off
- Show bind pose vs. animated pose
- Add bone selection for inspection

**Deliverable**: Visual skeleton debugger

### Milestone 3: Multiple Clips and UI (Week 1, Days 6-7)

**Goal**: Load and switch between multiple animations

**Tasks**:
- Load multiple animation clips from file(s)
- Create clip library/manager
- Build UI for clip selection
- Implement play/pause/stop controls
- Add playback speed adjustment
- Display current time and duration

**Deliverable**: Animation player with multiple clips

### Milestone 4: Crossfade Blending (Week 2, Days 1-3)

**Goal**: Smooth transitions between animations

**Tasks**:
- Implement pose blending (lerp positions, slerp rotations)
- Add crossfade logic (blend two clips over time)
- Set transition duration (e.g., 0.3 seconds)
- Queue next animation during blend
- Handle edge cases (blend during blend)
- Visual indicator of blend progress

**Deliverable**: Smooth animation transitions

### Milestone 5: Animation State Machine (Week 2-3, Days 4-7)

**Goal**: Input-driven animation states

**Tasks**:
- Define state machine structure
- Create states (Idle, Walk, Run, Jump)
- Define transitions with conditions (e.g., speed > 0.5)
- Implement state evaluation each frame
- Bind keyboard input to state parameters
- Visualize current state in UI

**Deliverable**: Character responds to input with state changes

### Milestone 6: Advanced Blending (Week 3-4, Days 1-7+)

**Goal**: Blend trees and layering

**Tasks**:
- Implement 1D blend tree (walk-run blend by speed)
- Add blend parameter UI slider
- Implement layered animation (upper/lower body)
- Add additive animation (wave gesture)
- Create bone masking system
- Tune blend weights and parameters

**Deliverable**: Sophisticated blending system

### Optional Milestone 7: Inverse Kinematics

**Goal**: Procedural IK for feet/hands

**Tasks**:
- Implement two-bone IK solver (CCD or analytical)
- Add IK targets (ground plane for feet)
- Blend IK result with animation
- Implement look-at IK for head
- Add debug visualization for IK chains
- Handle edge cases (unreachable targets)

**Deliverable**: IK-enhanced animations

## Technical Challenges

### Challenge 1: Quaternion Interpolation

**Problem**: Lerping quaternions produces incorrect rotations

**Approach**:
- Use `slerp()` (spherical linear interpolation) for quaternions
- Handle quaternion double-cover (q and -q represent same rotation)
- Choose shortest path: if dot(q1, q2) < 0, negate one quaternion
- Understand gimbal lock and why quaternions avoid it

**Implementation**:
```
slerp(q1, q2, t):
  dot = q1.dot(q2)
  if dot < 0:
    q2 = -q2
    dot = -dot
  
  if dot > 0.9995:  // Very close, use lerp
    return normalize(lerp(q1, q2, t))
  
  angle = acos(dot)
  return (sin((1-t)*angle)*q1 + sin(t*angle)*q2) / sin(angle)
```

### Challenge 2: Bone Hierarchy Traversal

**Problem**: Computing global transforms from local transforms

**Approach**:
- Traverse skeleton depth-first from root
- Each bone's global transform = parent_global * bone_local
- Cache results to avoid recomputation
- Handle root bone specially (no parent)

**Algorithm**:
```
compute_global_transforms(skeleton, local_transforms):
  global_transforms = new array[skeleton.bones.length]
  
  for bone in skeleton.bones (in hierarchy order):
    if bone.parent_index == -1:
      global_transforms[bone.index] = local_transforms[bone.index]
    else:
      global_transforms[bone.index] = 
        global_transforms[bone.parent_index] * local_transforms[bone.index]
  
  return global_transforms
```

### Challenge 3: Skinning Matrix Calculation

**Problem**: Converting bone transforms to skinning matrices

**Approach**:
- Skinning matrix = global_transform * inverse_bind_pose
- Inverse bind pose stored in mesh data (from modeling tool)
- Upload as array of mat4 to GPU
- Vertex shader blends up to 4 bone influences per vertex

**Vertex Shader Pattern**:
```glsl
uniform mat4 bone_matrices[MAX_BONES];

attribute vec4 bone_indices;  // Which bones affect this vertex
attribute vec4 bone_weights;  // Influence weights (sum to 1.0)

void main() {
  mat4 skin_matrix = 
    bone_matrices[int(bone_indices.x)] * bone_weights.x +
    bone_matrices[int(bone_indices.y)] * bone_weights.y +
    bone_matrices[int(bone_indices.z)] * bone_weights.z +
    bone_matrices[int(bone_indices.w)] * bone_weights.w;
  
  vec4 skinned_position = skin_matrix * vec4(position, 1.0);
  gl_Position = projection * view * skinned_position;
}
```

### Challenge 4: State Machine Edge Cases

**Problem**: Multiple valid transitions, conflicting conditions

**Approach**:
- Prioritize transitions (order matters or explicit priority)
- Evaluate all conditions first, then choose highest priority
- Prevent ping-ponging (add hysteresis or cooldown)
- Use one-frame delay for condition changes
- Log transition events for debugging

### Challenge 5: Performance with Many Bones

**Problem**: Large skeletons (100+ bones) slow down update

**Approach**:
- Use SIMD for matrix multiplication
- Compute only visible characters
- LOD: use simpler skeletons for distant characters
- GPU skinning (always, don't do CPU skinning)
- Cache unchanged bone transforms

## Reference Implementations

### Praxis Engine (Rust)
- **Files**: 
  - `examples/skeletal_animation_demo.rs`
  - `examples/animation_blending_demo.rs`
  - `examples/animation_advanced_demo.rs`
- **Crates**: `praxis_scene` (animation system)
- **Concepts**: Skeletal animation, blend trees, state machines

### Other Engines/Frameworks

**Unity (C#)**
- Tutorial: "Animator Controller" (official docs)
- Key APIs: `Animator`, `AnimatorController`, `AnimationClip`, blend trees

**Unreal Engine (C++)**
- Tutorial: "Animation Blueprint"
- Key APIs: `UAnimInstance`, `UAnimSequence`, blend spaces, state machines

**Godot (GDScript)**
- Tutorial: "3D Skeletal Animation"
- Key Nodes: `AnimationPlayer`, `AnimationTree`, `Skeleton`

**Three.js (JavaScript)**
- Example: [three.js skinning](https://threejs.org/examples/#webgl_animation_skinning_blending)
- Key APIs: `AnimationMixer`, `AnimationClip`, `SkinnedMesh`

**Bevy (Rust)**
- Example: Bevy animation examples
- Pattern: ECS-based animation with `AnimationPlayer` component

**Ozz-animation (C++)**
- Library: Ozz is production-quality animation library
- Features: Runtime, blending, IK, optimized for performance

## Extension Ideas

### Beginner Extensions
- Animation speed multiplier per clip
- Mirror animations (flip left/right)
- Root motion extraction (move character by animation)
- Animation events (footstep sounds at specific frames)

### Intermediate Extensions
- Blend tree editor (visual node graph)
- Animation compression
- Foot placement IK
- Aim-space blending (upper body tracks target)

### Advanced Extensions
- Motion matching (data-driven animation)
- Muscle-based control
- Full-body IK
- Animation retargeting (transfer animations between skeletons)

## Success Criteria

Your animation showcase should:

1. ✅ Play skeletal animations smoothly at 60 FPS
2. ✅ Blend between animations without popping or artifacts
3. ✅ Respond to input with appropriate state transitions
4. ✅ Handle complex skeletons (50+ bones) efficiently
5. ✅ Provide clear visualization of skeleton and state
6. ✅ Support multiple simultaneous animated characters
7. ✅ Feel natural and responsive to user input

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Playback** | Single clip loops correctly | Multiple clips, smooth crossfade | State machine, blend trees |
| **Blending** | Basic crossfade | 1D blend tree, layering | 2D blend trees, additive |
| **Performance** | 1 character, 30 bones | 10 characters, 50 bones | 50+ characters, LOD, optimization |
| **Features** | Play/pause/speed control | State machine, bone masking | IK, procedural, advanced |

## Common Pitfalls

1. **Lerping Quaternions**: Use `slerp()`, not component-wise `lerp()`
2. **Incorrect Bind Pose**: Ensure inverse bind pose matches rest pose
3. **Missing Normalization**: Normalize quaternions after blending
4. **Wrong Parent Order**: Traverse skeleton in parent-to-child order
5. **Ignoring Double Cover**: Handle quaternion negation for shortest path
6. **Fixed Timestep Mismatch**: Use delta time, don't assume 60 FPS
7. **GPU Upload Overhead**: Minimize bone matrix uploads (use storage buffers)

## Next Steps

After completing this project, you're ready for:
- **Project 01**: 3D Model Viewer (integrate animated models)
- **Project 02**: First-Person Explorer (character controller with animations)
- **Project 07**: Multiplayer Arena (networked animation replication)
