# Game Loop Patterns

The game loop is the heartbeat of every game engine, controlling how time advances and when different subsystems update. The choice of game loop pattern fundamentally affects determinism, visual smoothness, and system complexity.

## The Core Problem

Games must:
1. **Process input** from players
2. **Update game state** (physics, AI, gameplay)
3. **Render frames** to the screen

These operations must happen continuously, but each frame takes a different amount of time to complete. The game loop pattern determines how to handle this variable frame time.

## Pattern Variants

### 1. Variable Timestep (Naive)

**Concept**: Update game state by however much time has passed since the last frame.

```
last_time = current_time()

loop:
    current = current_time()
    delta_time = current - last_time
    last_time = current
    
    process_input()
    update(delta_time)
    render()
```

**Trade-offs**:

✅ **Strengths**:
- Simple to implement
- Automatically adapts to frame rate
- No wasted CPU time

❌ **Weaknesses**:
- Non-deterministic (different results on different hardware)
- Physics instability with large delta times
- Replays and networking become extremely difficult
- "Spiral of death" (slow frames → larger delta → slower frames)

**When to use**:
- Simple games without physics
- Prototypes and tools
- Single-player games where determinism doesn't matter

**Real-world examples**:
- Early mobile games
- Many indie 2D games
- Non-physics-based puzzle games

### 2. Fixed Timestep

**Concept**: Update game state in fixed time increments, regardless of frame rate.

```
timestep = 1/60  # 16.67ms
accumulator = 0.0

loop:
    current = current_time()
    frame_time = current - last_time
    last_time = current
    accumulator += frame_time
    
    while accumulator >= timestep:
        process_input()
        update(timestep)
        accumulator -= timestep
    
    render()
```

**Trade-offs**:

✅ **Strengths**:
- Deterministic (same results regardless of frame rate)
- Stable physics simulation
- Replay-friendly
- Network-friendly (lockstep multiplayer)
- Predictable behavior for testing

❌ **Weaknesses**:
- Visual stutter when update rate ≠ render rate
- Wasted computation if rendering faster than updates
- Can fall behind if updates take too long

**When to use**:
- Physics-heavy games
- Multiplayer games (especially deterministic lockstep)
- Competitive games requiring consistency
- Games with replay systems

**Real-world examples**:
- Fighting games (Street Fighter, Mortal Kombat)
- RTS games with replay (StarCraft II)
- Physics simulators
- Many classic arcade games

### 3. Semi-Fixed Timestep (Fixed Update, Variable Render)

**Concept**: Separate update rate from render rate, interpolate between states for smooth rendering.

```
update_rate = 1/60
accumulator = 0.0

loop:
    current = current_time()
    frame_time = current - last_time
    last_time = current
    accumulator += frame_time
    
    while accumulator >= update_rate:
        previous_state = current_state.copy()
        process_input()
        update(update_rate)
        accumulator -= update_rate
    
    # Interpolation factor (0.0 to 1.0)
    alpha = accumulator / update_rate
    render_state = interpolate(previous_state, current_state, alpha)
    render(render_state)
```

**Trade-offs**:

✅ **Strengths**:
- Deterministic updates like fixed timestep
- Smooth rendering at any frame rate
- Physics stability
- Replay and network-friendly
- Best visual quality on high refresh displays

❌ **Weaknesses**:
- Most complex to implement
- Requires state interpolation logic
- Slightly increased memory (storing previous state)
- Input lag (up to one update frame)
- Not all state interpolates cleanly (discrete events, spawning)

**When to use**:
- Modern AAA games
- Games targeting multiple platforms/refresh rates
- VR games (need high, smooth frame rates)
- Competitive shooters balancing consistency and smoothness

**Real-world examples**:
- Source Engine games (Half-Life, Team Fortress)
- Unreal Engine default
- Unity with fixed update + interpolation
- Most modern multiplayer shooters

### 4. Capped Variable Timestep

**Concept**: Variable timestep with maximum delta time limit.

```
max_delta = 1/30  # Cap at 30 FPS
last_time = current_time()

loop:
    current = current_time()
    delta_time = min(current - last_time, max_delta)
    last_time = current
    
    process_input()
    update(delta_time)
    render()
```

**Trade-offs**:

✅ **Strengths**:
- Prevents "spiral of death"
- Simpler than fixed timestep
- Smoother than fixed timestep at high frame rates

❌ **Weaknesses**:
- Still non-deterministic
- Game slows down in slow motion when capped
- Physics can still be unstable
- Not replay-friendly

**When to use**:
- Casual single-player games
- Games with soft physics (not simulation-critical)
- When simplicity trumps determinism

**Real-world examples**:
- Many mobile games
- Casual puzzle games with light physics
- Platformers without precise physics

## Advanced Variations

### 5. Multiple Update Rates

Some engines use different update rates for different systems:

```
physics_rate = 1/60
ai_rate = 1/30
animation_rate = 1/120

loop:
    # Each subsystem has its own accumulator
    if should_update(physics_accumulator, physics_rate):
        update_physics()
    
    if should_update(ai_accumulator, ai_rate):
        update_ai()
    
    if should_update(animation_accumulator, animation_rate):
        update_animations()
    
    render()
```

**Use cases**:
- AI doesn't need 60Hz updates (can save CPU)
- Physics requires fixed rate for stability
- Animations may benefit from higher rates for smoothness

**Examples**: Unreal Engine's tick groups, custom engines

### 6. Frame Pacing / Temporal Budgets

**Concept**: Dynamically adjust update work to maintain target frame rate.

```
target_frame_time = 1/60
work_budget = 0.8 * target_frame_time  # Reserve 80%

loop:
    frame_start = current_time()
    
    update_critical_systems()
    
    # Do as much optional work as budget allows
    while (current_time() - frame_start) < work_budget:
        update_one_optional_task()
    
    render()
```

**Use cases**:
- Open-world games with streaming
- Games with dynamic LOD systems
- VR (strict frame timing requirements)

**Examples**: Horizon Zero Dawn (2017 GDC talk), many modern open-world games

## Comparison Table

| Pattern | Deterministic | Visual Smoothness | Complexity | Network-Friendly | Replay-Friendly |
|---------|--------------|-------------------|------------|------------------|-----------------|
| Variable | ❌ | 🟡 Good* | ⭐ Simple | ❌ | ❌ |
| Fixed | ✅ | ❌ Stutter | ⭐⭐ Moderate | ✅ | ✅ |
| Semi-Fixed | ✅ | ✅ Excellent | ⭐⭐⭐ Complex | ✅ | ✅ |
| Capped Variable | ❌ | 🟡 Good* | ⭐ Simple | ❌ | ❌ |

*At high frame rates; stutters at low frame rates

## Input Handling Considerations

Different loop patterns affect input responsiveness:

**Variable Timestep**: Input processed every frame, best latency but inconsistent timing

**Fixed Timestep**: Input sampled at update rate (e.g., 60Hz)
- May miss fast inputs between updates
- Consistent, predictable timing
- Solution: Input buffering, separate input polling rate

**Semi-Fixed**: Input typically polled at update rate, rendered with interpolation
- Slight visual lag (1 update frame behind)
- Solution: Extrapolation for local player, interpolation for others

## Physics Integration

Physics engines typically require fixed timesteps for stability:

**Why**: Numerical integration (Euler, Verlet, RK4) assumes small, consistent time steps
- Large delta times → large integration errors → instability
- Variable delta times → different results (not deterministic)

**Solution patterns**:

1. **Fixed update for physics**: Always run physics at fixed rate (60Hz or 120Hz)
2. **Substeps**: Break large deltas into multiple small steps
3. **Solver iteration scaling**: Adjust constraint solver iterations based on delta time

## Common Pitfalls

### Pitfall 1: Unbounded Accumulator

```
# BAD: Can run updates forever if frame takes too long
while accumulator >= timestep:
    update(timestep)
    accumulator -= timestep
```

**Solution**: Cap maximum updates per frame

```
# GOOD: Limit to prevent spiral of death
max_updates_per_frame = 5
updates = 0

while accumulator >= timestep and updates < max_updates_per_frame:
    update(timestep)
    accumulator -= timestep
    updates += 1

if updates >= max_updates_per_frame:
    accumulator = 0  # Discard time, accept slowdown
```

### Pitfall 2: Delta Time in Milliseconds vs Seconds

Be consistent! Mixing units causes confusion and bugs.

```
# Choose one convention and stick to it
delta_time = 0.016  # Seconds (common)
delta_time = 16.67  # Milliseconds (also common)

# Document clearly in code
velocity += acceleration * delta_time_seconds
```

### Pitfall 3: Not Accounting for Pause/Background Time

```
# BAD: Huge delta after resuming from pause
delta_time = current_time() - last_time

# GOOD: Clamp or reset after pause
if was_paused:
    last_time = current_time()  # Reset clock
else:
    delta_time = min(current_time() - last_time, max_delta)
```

## Practical Recommendations

**For beginners**: Start with variable timestep, understand its limitations

**For physics games**: Use fixed timestep (60Hz is standard)

**For modern production**: Use semi-fixed with interpolation
- Fixed 60Hz or 120Hz updates
- Uncapped rendering
- Interpolate visual state

**For VR/competitive**: Semi-fixed with strict frame timing
- May need 90Hz/120Hz/144Hz updates
- Frame pacing critical
- Consider input prediction/extrapolation

**For networked games**: Fixed timestep with client-side prediction and server reconciliation

## Implementation Checklist

When implementing a game loop:

- [ ] Decide determinism requirements
- [ ] Choose pattern based on needs
- [ ] Handle pause/resume correctly
- [ ] Cap maximum updates per frame (if using accumulator)
- [ ] Test at various frame rates (30, 60, 144+ FPS)
- [ ] Test slowdown scenarios (background tabs, low-end hardware)
- [ ] Document delta time units (seconds vs milliseconds)
- [ ] Profile update costs to ensure they fit in frame budget
- [ ] Consider platform-specific timing APIs (high precision)

## Further Reading

### Classic Articles
- **"Fix Your Timestep!"** by Glenn Fiedler (canonical resource)
- **"The Game Loop"** from Game Programming Patterns by Robert Nystrom

### Academic Papers
- **"Numerical Integration in Game Physics"** - explains why fixed timesteps matter
- **"Timestep Independence Using Blended Physics"** - advanced interpolation techniques

### GDC Talks
- **"It IS Rocket Science! The Physics of 'Rocket League'"** - fixed timestep in competitive game
- **"Networking for Physics Programmers"** - determinism and timesteps
- **"Overwatch Gameplay Architecture and Netcode"** - semi-fixed with prediction

### Engine Documentation
- **Unreal Engine**: "Tick Groups" and "Delta Time"
- **Unity**: "Fixed Update vs Update"
- **Source Engine**: "Tick Rate and Interpolation"
- **Godot**: "Process and Physics Process"

### Books
- **Game Engine Architecture** by Jason Gregory - Chapter on the game loop
- **Game Physics Engine Development** by Ian Millington - Time integration
- **Real-Time Collision Detection** by Christer Ericson - Temporal coherence

## Summary

The game loop pattern is a foundational choice that affects every system in your engine:

- **Variable timestep**: Simple but non-deterministic
- **Fixed timestep**: Deterministic but can stutter visually
- **Semi-fixed timestep**: Best of both worlds, most complex
- **Capped variable**: Compromise for casual games

Choose based on your requirements for determinism, visual smoothness, and implementation complexity. Most modern engines converge on semi-fixed timestep with interpolation as the best general-purpose solution.
