# Project 03: Physics Playground

**Difficulty**: Intermediate  
**Estimated Time**: 2-3 weeks  
**Core Learning**: Physics simulation, rigid bodies, constraints, impulse-based dynamics

## Overview

Build an interactive physics sandbox where users can spawn objects, apply forces, and create constraints. This project teaches physics engine integration, rigid body dynamics, collision response, and interactive manipulation of physical objects.

### Learning Objectives

- Integrate a physics engine (Rapier, Bullet, PhysX, etc.)
- Understand rigid body dynamics and constraints
- Implement object spawning and manipulation
- Create joints and constraint systems
- Visualize physics debug information
- Optimize physics performance

## Feature Requirements

### Core Features (Minimum Viable)

1. **Rigid Body System**
   - Dynamic, static, and kinematic bodies
   - Basic shapes (box, sphere, capsule, cylinder)
   - Mass, friction, and restitution properties
   - Gravity simulation

2. **Object Spawning**
   - Click to spawn objects at cursor position
   - Selectable shape types (cube, sphere, etc.)
   - Adjustable spawn height
   - Multiple simultaneous objects (100+ objects)

3. **Physics Interaction**
   - Click and drag to apply forces
   - Impulse application on click
   - Object selection and deletion
   - Reset scene functionality

4. **Basic Constraints**
   - Fixed joints (weld objects together)
   - Hinge joints (door-like rotation)
   - Spring constraints
   - Distance constraints (rope-like)

### Extended Features (Recommended)

5. **Advanced Manipulation**
   - Grab and move objects with mouse (6-DOF)
   - Rotate grabbed objects
   - Freeze/unfreeze selected objects
   - Clone selected object

6. **Complex Shapes**
   - Convex hull meshes
   - Compound shapes (multiple colliders)
   - Mesh colliders (triangle mesh)
   - Terrain heightfield

7. **Advanced Constraints**
   - Ball-and-socket joint (ragdoll-style)
   - Slider/prismatic joint (piston-like)
   - Breakable constraints
   - Motor-driven joints
   - Chain/rope creation tool

### Stretch Goals

8. **Physics Tools**
   - Explosion force (radial impulse)
   - Zero-gravity zones
   - Force fields (attract/repel)
   - Buoyancy simulation (floating objects)

9. **Scenarios/Challenges**
   - Domino chain setup
   - Bridge building challenge
   - Tower stacking game
   - Rube Goldberg machine creator

## Architecture Guidance

### System Components

```
PhysicsPlayground
├── PhysicsWorld
│   ├── RigidBodyManager
│   ├── ColliderManager
│   ├── ConstraintSolver
│   └── ContactGenerator
├── ObjectSpawner
│   ├── ShapeFactory
│   ├── MaterialPresets
│   └── SpawnController
├── InteractionSystem
│   ├── ObjectSelector
│   ├── MouseGrabber (6-DOF constraint)
│   └── ForceApplicator
├── ConstraintBuilder
│   ├── JointFactory
│   ├── ConnectionVisualizer
│   └── ConstraintEditor
└── DebugRenderer
    ├── ColliderVisualizer
    ├── ContactPointDisplay
    └── ConstraintGizmos
```

### Data Structures

**Rigid Body**
```
RigidBody:
  - body_type: Dynamic | Static | Kinematic
  - mass: float (computed from density + volume)
  - position: vec3
  - rotation: quaternion
  - linear_velocity: vec3
  - angular_velocity: vec3
  - center_of_mass: vec3
  - inertia_tensor: mat3
  - material: PhysicsMaterial
  
PhysicsMaterial:
  - friction: float (0-1)
  - restitution: float (0-1, bounciness)
  - density: float
```

**Constraint/Joint**
```
Constraint:
  - type: Fixed | Hinge | BallSocket | Distance | etc.
  - body_a: RigidBody reference
  - body_b: RigidBody reference
  - anchor_a: vec3 (local space)
  - anchor_b: vec3 (local space)
  - axis: vec3 (for hinges, sliders)
  - limits: (min, max) angle or distance
  - motor_enabled: bool
  - motor_target: float
  - break_force: float (optional)

Methods:
  - solve_constraint(delta_time)
  - apply_impulse()
  - check_break_condition()
```

**Object Spawner State**
```
SpawnerConfig:
  - shape_type: Box | Sphere | Capsule | Cylinder
  - spawn_height: float
  - initial_velocity: vec3
  - material_preset: string
  - size: vec3 (dimensions)
  - color: vec3
```

### Physics Simulation Loop

```
fixed_update(delta_time):  // Called at fixed rate (e.g., 60 Hz)
  1. Apply external forces (gravity, user input)
  2. Integrate velocities (apply forces → acceleration → velocity)
  3. Detect collisions (broad phase → narrow phase)
  4. Generate contact constraints
  5. Solve constraints (iterative solver)
  6. Integrate positions (apply velocities → positions)
  7. Update transforms to ECS/scene graph
  8. Clear forces for next step
```

### Mouse Grabbing Implementation

**6-DOF Grab Constraint**
```
on_mouse_down(ray):
  hit = raycast_physics(ray)
  if hit.body.is_dynamic():
    grab_offset = hit.point - hit.body.position
    grab_constraint = create_constraint(
      body_a = hit.body,
      body_b = null,  // world space
      anchor_a = grab_offset,
      anchor_b = hit.point,
      type = BallSocket  // or Fixed
    )

on_mouse_move(ray):
  if grabbing:
    target_point = ray.origin + ray.direction * grab_distance
    grab_constraint.anchor_b = target_point

on_mouse_up():
  if grabbing:
    remove_constraint(grab_constraint)
    grab_constraint = null
```

## Milestone Plan

### Milestone 1: Physics Integration (Week 1, Days 1-3)

**Goal**: Integrate physics engine and spawn basic objects

**Tasks**:
- Set up physics engine (Rapier, Bullet, etc.)
- Create physics world with gravity
- Implement rigid body creation (box, sphere)
- Add static ground plane
- Spawn falling objects that collide
- Synchronize physics transforms to rendering

**Deliverable**: Objects fall and collide realistically

### Milestone 2: Object Spawning System (Week 1, Days 4-5)

**Goal**: Interactive spawning with UI controls

**Tasks**:
- Implement click-to-spawn at cursor position
- Add shape selection UI (buttons or dropdown)
- Create material presets (wood, metal, rubber)
- Add spawn height adjustment
- Display object count
- Implement scene reset

**Deliverable**: Spawn various objects at will

### Milestone 3: Force Application (Week 1, Days 6-7)

**Goal**: Apply forces to objects via interaction

**Tasks**:
- Implement raycast object selection
- Apply impulse on click (shoot objects)
- Click-and-drag to apply directional force
- Visual feedback (highlight selected object)
- Add force strength adjustment
- Display velocity vectors (debug mode)

**Deliverable**: Interact with objects via forces

### Milestone 4: Mouse Grabbing (Week 2, Days 1-3)

**Goal**: Grab and move objects with mouse

**Tasks**:
- Implement 6-DOF grab constraint
- Calculate grab point in object local space
- Update constraint target based on mouse position
- Handle depth control (mouse wheel while grabbing)
- Add rotation controls (keyboard while grabbing)
- Smooth motion (damping)

**Deliverable**: Pickup and move objects naturally

### Milestone 5: Constraints and Joints (Week 2, Days 4-5)

**Goal**: Create joints between objects

**Tasks**:
- Implement fixed joint (weld tool)
- Implement hinge joint (specify axis)
- Add distance constraint (rope)
- Visual joint creation UI (select two objects, choose type)
- Display joint gizmos/visualizations
- Constraint breaking threshold

**Deliverable**: Build connected structures

### Milestone 6: Polish and Scenarios (Week 2-3, Days 6-7+)

**Goal**: Add polish and fun scenarios

**Tasks**:
- Create preset scenarios (domino setup, tower, etc.)
- Add explosion tool (radial impulse)
- Implement compound shapes
- Performance optimization (spatial partitioning)
- Debug visualization improvements
- Sound effects (collision sounds)

**Deliverable**: Polished, fun physics sandbox

## Technical Challenges

### Challenge 1: Physics-Rendering Synchronization

**Problem**: Physics runs at fixed timestep, rendering at variable rate

**Approach**:
- Use fixed timestep accumulator pattern
- Interpolate rendered positions between physics steps
- Store previous and current physics transforms
- Render at interpolated position: `lerp(prev, current, alpha)`

**Code Pattern**:
```
accumulator = 0.0
fixed_dt = 1.0 / 60.0

game_loop():
  frame_time = get_frame_time()
  accumulator += frame_time
  
  while accumulator >= fixed_dt:
    physics_step(fixed_dt)
    accumulator -= fixed_dt
  
  alpha = accumulator / fixed_dt
  render_interpolated(alpha)
```

### Challenge 2: Stability with Many Constraints

**Problem**: Constraint solver struggles with large chains or stacks

**Approach**:
- Use iterative constraint solver (Sequential Impulse)
- Increase solver iterations for better accuracy
- Use warm-starting (reuse previous frame's impulses)
- Implement constraint sleeping for stable structures
- Limit maximum chain length

**Tuning Parameters**:
- Solver iterations: 10-20 for typical scenes
- Position correction: small bias to prevent drift
- Warm starting: 0.8-0.95 damping factor

### Challenge 3: Continuous Collision Detection

**Problem**: Fast-moving objects tunnel through geometry

**Approach**:
- Enable CCD for small, fast objects
- Use swept collision tests (cast shape along velocity)
- Time-of-impact (TOI) calculation
- Conservative advancement algorithm
- Performance trade-off: enable selectively

**When to Use**:
- Bullets, projectiles
- Small objects moving > size per frame
- Critical gameplay objects (not decorative)

### Challenge 4: Performance with Many Objects

**Problem**: Simulation slows with 500+ objects

**Approach**:
- Broad-phase collision detection (spatial partitioning)
- Use BVH or grid for collision culling
- Implement sleeping (deactivate stable objects)
- LOD physics (simplify distant objects)
- Object pooling (reuse memory)

**Optimization Checklist**:
- Profile physics time per frame
- Monitor active body count
- Use simple collision shapes where possible
- Batch similar objects

### Challenge 5: Constraint Stability

**Problem**: Constraints wobble, separate, or explode

**Approach**:
- Ensure rigid bodies have appropriate masses (avoid huge ratios)
- Use higher solver iterations
- Add damping to constraints (soft constraints)
- Limit maximum impulse magnitude
- Verify constraint anchors in correct local space

## Reference Implementations

### Praxis Engine (Rust)
- **File**: `examples/physics_demo.rs` (if exists, or create new)
- **Crates**: `praxis_physics` (Rapier3D integration)
- **Concepts**: Rigid bodies, collision, constraints

### Other Engines/Frameworks

**Unity (C#)**
- Example: Physics Materials demo
- Key APIs: `Rigidbody`, `ConfigurableJoint`, `Physics.AddForce()`

**Unreal Engine (C++)**
- Tutorial: "Physics Constraint Component"
- Key APIs: `UPrimitiveComponent`, `UPhysicsConstraintComponent`

**Godot (GDScript)**
- Example: RigidBody playground
- Key Nodes: `RigidBody`, `Generic6DOFJoint`, `PhysicsMaterial`

**Three.js + Cannon.js (JavaScript)**
- Example: Cannon.js demos (e.g., "Constraints", "Compound")
- Key APIs: `CANNON.World`, `CANNON.Body`, `CANNON.Constraint`

**Bevy + Rapier (Rust)**
- Plugin: `bevy_rapier3d`
- Example: Rapier testbed demos
- Pattern: ECS components for rigid bodies and joints

**PyBullet (Python)**
- Example: pybullet quickstart guide
- Key Functions: `createMultiBody()`, `createConstraint()`, `applyExternalForce()`

## Extension Ideas

### Beginner Extensions
- Color objects based on velocity (heat map)
- Trail renderer for moving objects
- Slow-motion mode
- Save/load scenes

### Intermediate Extensions
- Soft body simulation (cloth, jelly)
- Destructible objects (fracture on impact)
- Fluid simulation (simplified)
- Ragdoll character

### Advanced Extensions
- Vehicle physics (wheels, suspension)
- Cloth tearing
- Real-time fracturing
- Custom constraint types

## Success Criteria

Your physics playground should:

1. ✅ Simulate 100+ objects at 60 FPS (fixed timestep)
2. ✅ Provide stable, realistic collision response
3. ✅ Allow intuitive object manipulation (grab, move, apply forces)
4. ✅ Support multiple constraint types without instability
5. ✅ Handle edge cases (stacks, chains, fast motion) gracefully
6. ✅ Provide clear visual feedback (selection, constraints)
7. ✅ Enable creative experimentation (fun factor!)

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Physics Quality** | Basic collision, simple shapes | + Stable constraints, CCD | + Soft bodies, fracture, advanced features |
| **Interaction** | Click to spawn, apply forces | + Mouse grab, constraint builder | + Advanced tools, explosion, fields |
| **Performance** | 50 objects stable | 200+ objects at 60 FPS | 500+ objects, optimized broad-phase |
| **Scenarios** | Random spawning | + Preset setups, basic challenges | + Complex machines, puzzle levels |

## Common Pitfalls

1. **Variable Timestep**: Always use fixed timestep for physics simulation
2. **Mass Ratios**: Avoid bodies with vastly different masses (e.g., 0.1 kg vs 1000 kg)
3. **Over-Constraining**: Too many constraints on one body causes instability
4. **Ignoring CCD**: Fast objects need continuous collision detection
5. **Incorrect Units**: Use consistent unit scale (e.g., 1 unit = 1 meter)
6. **No Sleeping**: Implement sleeping for stable structures to improve performance
7. **Force vs Impulse**: Know when to use `add_force()` vs `apply_impulse()`

## Next Steps

After completing this project, you're ready for:
- **Project 02**: First-Person Explorer (character physics integration)
- **Project 06**: Particle Effects System (physics-driven particles)
- **Project 10**: Mini Game Engine (full physics subsystem integration)
