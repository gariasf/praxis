# Project 02: First-Person Explorer

**Difficulty**: Beginner  
**Estimated Time**: 2-3 weeks  
**Core Learning**: Input handling, first-person camera, collision detection, character movement

## Overview

Build a first-person character controller that allows players to explore a 3D environment. This project teaches input systems, physics-based movement, collision detection, and interactive gameplay mechanics.

### Learning Objectives

- Implement WASD + mouse look controls
- Build character controller with physics
- Detect and resolve collisions
- Handle ground detection and jumping
- Create interactive environment elements
- Manage game state and UI overlays

## Feature Requirements

### Core Features (Minimum Viable)

1. **First-Person Camera**
   - Mouse look (yaw and pitch rotation)
   - Smooth camera movement
   - Constrained pitch (prevent over-rotation)
   - Adjustable sensitivity settings

2. **Character Movement**
   - WASD keyboard controls
   - Constant speed movement
   - Movement relative to camera direction
   - Ground-based locomotion

3. **Collision Detection**
   - Character collision capsule/cylinder
   - Static geometry collision (walls, floors)
   - Collision response (slide along walls)
   - Prevent walking through objects

4. **Basic Environment**
   - Simple level geometry (rooms, corridors)
   - Textured surfaces
   - Multiple rooms to explore
   - Lighting setup

### Extended Features (Recommended)

5. **Advanced Movement**
   - Sprint (shift to run faster)
   - Jump mechanics
   - Gravity and ground detection
   - Crouching (optional)

6. **Interactive Elements**
   - Doors (open/close on interaction)
   - Pickups (collect items)
   - Buttons/switches
   - Interaction raycast (highlight on hover)

7. **Enhanced Physics**
   - Stairs/ramp navigation
   - Slopes and uneven terrain
   - Moving platforms
   - Basic physics objects (push boxes)

### Stretch Goals

8. **Polish Features**
   - Head bobbing animation
   - Footstep sounds
   - Field-of-view effects (sprint FOV increase)
   - Camera shake on landing

9. **Gameplay Systems**
   - Inventory system (simple)
   - Health/stamina bars
   - Puzzle elements (keys, locked doors)
   - Minimap or compass

## Architecture Guidance

### System Components

```
FirstPersonExplorer
├── InputManager
│   ├── KeyboardInput
│   ├── MouseInput
│   └── InputMapping
├── CharacterController
│   ├── FirstPersonCamera
│   ├── MovementSystem
│   └── CollisionResolver
├── PhysicsWorld
│   ├── ColliderManager
│   ├── RaycastSystem
│   └── RigidBodySimulation
├── InteractionSystem
│   ├── InteractableObjects
│   ├── RaycastPicker
│   └── EventDispatcher
└── GameManager
    ├── LevelLoader
    ├── GameState
    └── UI
```

### Data Structures

**Character Controller**
```
CharacterController:
  - position: vec3
  - velocity: vec3
  - yaw: float (horizontal rotation)
  - pitch: float (vertical rotation)
  - height: float
  - radius: float (capsule radius)
  - is_grounded: bool
  - move_speed: float
  - sprint_multiplier: float
  - jump_force: float

Methods:
  - process_input(input_state, delta_time)
  - apply_gravity(delta_time)
  - check_ground()
  - resolve_collisions()
  - get_camera_transform() -> (position, rotation)
```

**Input State**
```
InputState:
  - move_forward: float (-1 to 1)
  - move_right: float (-1 to 1)
  - mouse_delta: vec2
  - jump_pressed: bool
  - sprint_held: bool
  - interact_pressed: bool

InputMapping:
  - key_bindings: map<Action, KeyCode>
  - mouse_sensitivity: float
  - invert_y: bool
```

**Interactable Object**
```
Interactable:
  - type: Door | Pickup | Button | etc.
  - state: Open | Closed | Active | etc.
  - interaction_radius: float
  - prompt_text: string

Methods:
  - on_interact(player)
  - can_interact() -> bool
  - get_interaction_prompt() -> string
```

### Movement Algorithm

```
update_character(delta_time):
  1. Read input state
  2. Calculate movement direction in world space:
     - Forward vector from camera yaw (ignore pitch)
     - Right vector perpendicular to forward
     - Combine WASD inputs
  3. Apply movement velocity
  4. Apply gravity if not grounded
  5. Tentatively update position
  6. Check collisions, adjust position
  7. Check if grounded (raycast down)
  8. Update camera based on mouse input
  9. Clamp pitch to prevent over-rotation
```

### Collision Detection Strategy

**Character Collision**
- Use capsule or cylinder collider for character
- Sweep test: cast shape along movement vector
- If collision detected, slide along surface
- Use iterative approach for complex geometry

**Slide Algorithm**
```
slide_movement(desired_velocity, collision_normal):
  1. Project velocity onto collision plane
  2. Remove component in normal direction
  3. Return parallel component
  4. Repeat for multiple collisions per frame
```

**Ground Detection**
```
check_grounded():
  1. Raycast from character center downward
  2. Distance = character height/2 + small epsilon
  3. If hit within distance: grounded = true
  4. Store ground normal for slope handling
```

## Milestone Plan

### Milestone 1: Basic First-Person Camera (Week 1, Days 1-2)

**Goal**: Implement mouse-look camera

**Tasks**:
- Set up input handling for mouse movement
- Implement yaw (horizontal) and pitch (vertical) rotation
- Convert rotation to view matrix
- Clamp pitch to prevent over-rotation (e.g., -85° to +85°)
- Add sensitivity adjustment
- Hide/lock mouse cursor

**Deliverable**: Camera that rotates with mouse movement

### Milestone 2: Keyboard Movement (Week 1, Days 3-4)

**Goal**: Add WASD movement without collision

**Tasks**:
- Handle keyboard input (WASD keys)
- Calculate movement direction from camera yaw
- Implement movement velocity application
- Add sprint modifier (hold Shift)
- Update character position each frame
- Display simple ground plane

**Deliverable**: Free-flying camera with WASD movement

### Milestone 3: Basic Collision (Week 1, Days 5-7)

**Goal**: Add collision with walls and floor

**Tasks**:
- Create simple level geometry (room with walls)
- Implement character capsule collider
- Add collision detection with level geometry
- Implement basic collision response (stop at walls)
- Add gravity and ground detection
- Prevent falling through floor

**Deliverable**: Character controller that collides with geometry

### Milestone 4: Polished Movement (Week 2, Days 1-3)

**Goal**: Improve collision response and add jumping

**Tasks**:
- Implement slide-along-wall collision response
- Add jumping mechanic (Space key)
- Improve ground detection (raycasting)
- Handle stairs/steps properly
- Add slope handling
- Tune movement feel (acceleration, friction)

**Deliverable**: Smooth character movement with proper collision

### Milestone 5: Interaction System (Week 2, Days 4-5)

**Goal**: Add interactive objects

**Tasks**:
- Implement raycast interaction detection
- Create door component (open/close)
- Add pickup items (collision-based or raycast)
- Display interaction prompts (UI overlay)
- Bind interaction key (E key)
- Visual feedback (highlight on hover)

**Deliverable**: Interactive doors and collectible items

### Milestone 6: Level Design and Polish (Week 2-3, Days 6-7+)

**Goal**: Create interesting level and polish

**Tasks**:
- Design multi-room level layout
- Add textures and lighting
- Place interactive elements strategically
- Implement UI (crosshair, item counter, hints)
- Add sound effects (footsteps, door sounds)
- Performance optimization
- Playtesting and tuning

**Deliverable**: Complete exploreable environment

## Technical Challenges

### Challenge 1: Smooth Mouse Look

**Problem**: Raw mouse input causes jittery or inconsistent rotation

**Approach**:
- Capture mouse delta each frame (not absolute position)
- Apply sensitivity multiplier
- Consider optional smoothing (moving average)
- Handle various DPI settings
- Lock cursor to window center

**Implementation Tips**:
```
on_mouse_move(delta_x, delta_y):
  yaw += delta_x * sensitivity * delta_time
  pitch -= delta_y * sensitivity * delta_time
  pitch = clamp(pitch, -max_pitch, max_pitch)
```

### Challenge 2: Collision Response Feel

**Problem**: Character gets stuck or stutters when hitting walls at angles

**Approach**:
- Use sliding collision (project velocity onto plane)
- Handle multiple collisions per frame iteratively
- Add small separation distance to prevent tunneling
- Use continuous collision detection for fast movement

**Sliding Algorithm**:
```
remaining_velocity = desired_velocity
for each collision in frame:
  slide_plane = collision.normal
  remaining_velocity = project_onto_plane(remaining_velocity, slide_plane)
  if length(remaining_velocity) < epsilon: break
```

### Challenge 3: Grounded Detection

**Problem**: Character "skips" on stairs or slopes, doesn't jump properly

**Approach**:
- Raycast downward from character center
- Check multiple points (center + cardinal directions) for reliable detection
- Use small tolerance (e.g., 0.1 units) above ground to count as grounded
- Separate "grounded" from "jumping" state
- Apply "coyote time" (grace period after leaving ground)

### Challenge 4: Movement Direction Calculation

**Problem**: Combining camera rotation with WASD input correctly

**Approach**:
- Get forward vector from camera yaw only (ignore pitch)
- Calculate right vector (perpendicular to forward)
- Combine: `velocity = forward * input.w + right * input.d`
- Normalize combined vector before applying speed
- Keep vertical component separate (gravity/jumping)

**Code Pattern**:
```
forward = vec3(sin(yaw), 0, cos(yaw))  // XZ plane only
right = vec3(forward.z, 0, -forward.x)  // Perpendicular
move_dir = normalize(forward * input_forward + right * input_right)
horizontal_velocity = move_dir * move_speed
```

### Challenge 5: Stair Climbing

**Problem**: Character bumps into stair edges instead of climbing smoothly

**Approach**:
- Step-up detection: if blocked, try moving up by step height
- Raycast from elevated position to see if clear
- If successful, adjust character position upward
- Limit maximum step height (e.g., 0.5 units)
- Ensure stairs have proper collision geometry

## Reference Implementations

### Praxis Engine (Rust)
- **File**: `examples/fps_camera_controller.rs`
- **Concepts**: First-person camera, input handling, movement
- **Crates**: `praxis_input`, `praxis_physics`, `praxis_scene`

### Other Engines/Frameworks

**Unity (C#)**
- Tutorial: "First Person Movement" (Brackeys)
- Key APIs: `CharacterController`, `Input.GetAxis()`, `Cursor.lockState`

**Unreal Engine (C++)**
- Template: "First Person Template"
- Key APIs: `ACharacter`, `UCharacterMovementComponent`, `AddControllerInput`

**Godot (GDScript)**
- Tutorial: "Your First 3D Game" (official docs)
- Key Nodes: `KinematicBody`, `Camera`, `RayCast`
- Method: `move_and_slide()`

**Three.js + Cannon.js (JavaScript)**
- Example: First-person controls with physics
- Libraries: `PointerLockControls`, `Cannon.World`, `Cannon.Body`

**OpenGL + Bullet Physics (C++)**
- Tutorial: "Character Controller with Bullet"
- Components: GLM for math, GLFW for input, Bullet `btKinematicCharacterController`

**Bevy (Rust)**
- Plugin: `bevy_fps_controller`
- Pattern: ECS-based character controller with Rapier physics

## Extension Ideas

### Beginner Extensions
- Adjustable movement speed (UI slider)
- Flashlight toggle (spotlight attached to camera)
- Simple particle effects (door opening dust)
- Pause menu

### Intermediate Extensions
- Climbing ladders
- Swimming mechanics
- Vaulting over low obstacles
- Damage system (fall damage, hazards)

### Advanced Extensions
- Multiplayer support (network other players)
- Save/load player position
- Cutscene system (disable controls, move camera)
- Advanced AI NPCs to interact with

## Success Criteria

Your first-person explorer should:

1. ✅ Respond immediately to mouse and keyboard input
2. ✅ Move smoothly without stuttering or glitches
3. ✅ Collide properly with all environment geometry
4. ✅ Handle stairs and slopes correctly
5. ✅ Provide clear interaction feedback (prompts, highlights)
6. ✅ Run at stable 60+ FPS
7. ✅ Feel responsive and intuitive to control
8. ✅ Handle edge cases (corners, tight spaces, fast movement)

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Controls** | Functional mouse look + WASD | + Smooth feel, sprint, jump | + Crouching, stamina, advanced movement |
| **Collision** | Basic blocking collision | + Slide response, ground detection | + Stairs, slopes, complex geometry |
| **Interaction** | Simple pickups | + Doors, buttons, UI prompts | + Inventory, puzzles, complex mechanics |
| **Polish** | Working prototype | + Sound, visual feedback | + Head bob, FOV effects, particles |

## Common Pitfalls

1. **Input Lag**: Process input before physics simulation each frame
2. **Collision Tunneling**: Fast movement can pass through thin walls (use continuous collision detection)
3. **Inconsistent Frame Rate**: Use delta time for movement calculations, fixed timestep for physics
4. **Getting Stuck**: Ensure collision resolver doesn't push character into geometry
5. **Camera Over-Rotation**: Always clamp pitch angle
6. **Floating on Slopes**: Ground detection needs proper tolerance and normal checking
7. **Unnatural Feel**: Tune movement speed, acceleration, mouse sensitivity carefully

## Next Steps

After completing this project, you're ready for:
- **Project 03**: Physics Playground (advanced physics interactions)
- **Project 07**: Multiplayer Arena (networked character controllers)
- **Project 08**: Scene Editor (camera systems, selection via raycasting)
