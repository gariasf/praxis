# Project 07: Multiplayer Arena

**Difficulty**: Advanced  
**Estimated Time**: 4-6 weeks  
**Core Learning**: Networking, client-server architecture, entity replication, lag compensation

## Overview

Build a multiplayer arena game with networked player movement, combat, and synchronization. This project teaches client-server architecture, entity state replication, client-side prediction, server reconciliation, and lag compensation techniques used in online games.

### Learning Objectives

- Implement client-server networking architecture
- Synchronize game state across network
- Apply client-side prediction and server reconciliation
- Implement lag compensation for fair gameplay
- Handle network conditions (latency, packet loss)
- Build lobby and matchmaking systems

## Feature Requirements

### Core Features (Minimum Viable)

1. **Basic Networking**
   - TCP or UDP socket communication
   - Client connection to server
   - Message serialization/deserialization
   - Connection management (connect, disconnect, timeout)

2. **Player Replication**
   - Spawn player entities on connection
   - Replicate player positions to all clients
   - Basic movement synchronization
   - Player list display

3. **Input Handling**
   - Client sends input to server
   - Server processes input and updates state
   - Server broadcasts state to clients
   - Basic movement (WASD)

4. **Simple Combat**
   - Shoot projectiles (raycasts or slow projectiles)
   - Hit detection on server
   - Health system
   - Respawn on death

### Extended Features (Recommended)

5. **Client-Side Prediction**
   - Client simulates own movement immediately
   - Server reconciliation (correct mispredictions)
   - Smooth corrections without rubber-banding
   - Input buffer with timestamps

6. **Interpolation**
   - Smooth remote player movement
   - Render delayed interpolated positions
   - Handle dropped packets gracefully
   - Extrapolation for brief disconnections

7. **Lag Compensation**
   - Server-side rewind for hit detection
   - Historical state buffer
   - Fair hit detection despite latency
   - "What you see is what you hit"

### Stretch Goals

8. **Advanced Features**
   - Lobby system (join/create rooms)
   - Matchmaking
   - Spectator mode
   - Chat system
   - Leaderboard/scoreboard

9. **Network Optimization**
   - Delta compression (send only changes)
   - Interest management (relevant entity filtering)
   - Bandwidth monitoring
   - Adaptive update rate

## Architecture Guidance

### System Components

```
MultiplayerArena
├── Networking
│   ├── NetworkServer (authoritative)
│   ├── NetworkClient
│   ├── MessageSerializer
│   └── ConnectionManager
├── GameState
│   ├── World (entities, physics)
│   ├── PlayerManager
│   ├── MatchState
│   └── ScoreTracker
├── Replication
│   ├── EntityReplicator
│   ├── StateSnapshot
│   └── DeltaCompressor
├── Prediction
│   ├── ClientPredictor
│   ├── ServerReconciler
│   └── InputBuffer
├── Interpolation
│   ├── RemoteEntityInterpolator
│   ├── StateHistory
│   └── Extrapolator
└── LagCompensation
    ├── HistoryBuffer
    ├── StateRewinder
    └── HitboxReconstructor
```

### Data Structures

**Network Message Types**
```
enum MessageType {
  # Connection
  Connect { player_name: string }
  Disconnect { player_id: uint }
  
  # Input
  PlayerInput { 
    timestamp: uint,
    sequence: uint,
    input: InputState
  }
  
  # State
  WorldState {
    timestamp: uint,
    entities: array of EntityState
  }
  
  # Events
  PlayerSpawn { player_id, position }
  PlayerDeath { player_id, killer_id }
  ProjectileFired { id, position, direction }
  Hit { target_id, damage, position }
}

InputState:
  - move_forward: float
  - move_right: float
  - look_yaw: float
  - look_pitch: float
  - shoot: bool
  - jump: bool
```

**Entity State (for replication)**
```
EntityState:
  - entity_id: uint
  - entity_type: Player | Projectile | etc.
  - position: vec3
  - rotation: quaternion
  - velocity: vec3 (optional)
  - health: int (optional)
  - animation_state: enum (optional)

NetworkedEntity:
  - entity_id: uint
  - authoritative_state: EntityState (server)
  - replicated_state: EntityState (client)
  - predicted_state: EntityState (client, local player)
  - last_update_time: timestamp
```

**Client-Side Prediction**
```
PredictionBuffer:
  - pending_inputs: queue of (sequence, InputState)
  - last_acked_sequence: uint
  - prediction_state: EntityState

Methods:
  - add_input(sequence, input)
  - reconcile(server_state, acked_sequence)
    # Replay inputs after acked_sequence
```

**Interpolation Buffer**
```
InterpolationBuffer:
  - state_history: circular buffer of (timestamp, EntityState)
  - interpolation_delay: float (e.g., 100ms)

Methods:
  - add_state(timestamp, state)
  - get_interpolated_state(render_time) -> EntityState
    # Interpolate between two states
```

**Lag Compensation History**
```
HistoryBuffer:
  - snapshots: circular buffer of (timestamp, WorldSnapshot)
  - max_history: float (e.g., 1 second)

WorldSnapshot:
  - timestamp: uint
  - entity_states: map<entity_id, EntityState>

Methods:
  - rewind_to(timestamp) -> WorldSnapshot
  - restore_current()
```

### Network Architecture

**Server (Authoritative)**
```
server_update_loop():
  1. Receive inputs from all clients
  2. Process inputs in order (timestamp/sequence)
  3. Simulate game world (physics, combat, etc.)
  4. Generate world state snapshot
  5. Send snapshot to all clients
  6. Store snapshot in history (for lag compensation)
  7. Repeat at fixed rate (e.g., 20-60 Hz)
```

**Client (Predicted)**
```
client_update_loop():
  1. Gather local input
  2. Send input to server (with timestamp/sequence)
  3. Apply input to predicted state immediately
  4. Store input in prediction buffer
  
  5. Receive world state from server
  6. For local player:
     - Check for misprediction
     - If misprediction: reconcile by replaying inputs
  7. For remote players:
     - Add state to interpolation buffer
     - Render interpolated state (delayed)
  
  8. Render world
```

### Client-Side Prediction Algorithm

```
on_local_input(input):
  sequence++
  timestamp = current_time()
  
  # Predict immediately
  apply_input_to_local_player(input)
  
  # Store for reconciliation
  prediction_buffer.add(sequence, timestamp, input)
  
  # Send to server
  send_to_server(PlayerInput{sequence, timestamp, input})

on_server_state_received(state, acked_sequence):
  # Find local player state in server message
  server_player_state = state.get_player(local_player_id)
  
  # Check for misprediction
  predicted_state = local_player.state
  error = distance(predicted_state.position, server_player_state.position)
  
  if error > threshold:
    # Rewind to server state
    local_player.state = server_player_state
    
    # Replay inputs after acked sequence
    for input in prediction_buffer.get_after(acked_sequence):
      apply_input_to_local_player(input)
  
  # Clean up acknowledged inputs
  prediction_buffer.remove_before(acked_sequence)
```

### Interpolation Algorithm

```
update_remote_player(player, delta_time):
  current_time = get_network_time()
  render_time = current_time - interpolation_delay
  
  # Find two states to interpolate between
  states = player.interpolation_buffer.get_states()
  state_before = null
  state_after = null
  
  for state in states:
    if state.timestamp <= render_time:
      state_before = state
    if state.timestamp > render_time and state_after == null:
      state_after = state
      break
  
  if state_before and state_after:
    # Interpolate
    t = (render_time - state_before.timestamp) / 
        (state_after.timestamp - state_before.timestamp)
    
    player.rendered_position = lerp(state_before.position, 
                                     state_after.position, t)
    player.rendered_rotation = slerp(state_before.rotation, 
                                      state_after.rotation, t)
  elif state_before:
    # Extrapolate (only state_before available)
    player.rendered_position = state_before.position + 
                               state_before.velocity * extrapolate_time
```

### Lag Compensation Hit Detection

```
process_shoot(shooter_id, target_id, shot_timestamp, aim_direction):
  # Rewind world to shooter's timestamp
  historical_world = history_buffer.rewind_to(shot_timestamp)
  
  # Get target position at that time
  target_state = historical_world.get_entity(target_id)
  
  # Perform hit detection in rewound state
  ray = Ray(shooter_position, aim_direction)
  hit = raycast(ray, target_state.hitbox)
  
  # Restore current world state
  history_buffer.restore_current()
  
  if hit:
    apply_damage(target_id, damage)
    send_hit_confirmation(shooter_id)
```

## Milestone Plan

### Milestone 1: Basic Client-Server (Week 1, Days 1-3)

**Goal**: Connect clients to server, send messages

**Tasks**:
- Set up server application (listens on port)
- Set up client application (connects to server)
- Implement basic message passing (ping/pong)
- Handle multiple client connections
- Display connected players
- Implement disconnect handling

**Deliverable**: Clients can connect and exchange messages

### Milestone 2: Player Spawning and Replication (Week 1, Days 4-7)

**Goal**: Spawn player entities, replicate positions

**Tasks**:
- Server spawns player entity on connection
- Server sends player list to all clients
- Clients render all players
- Server broadcasts player positions (e.g., 20 Hz)
- Clients update remote player positions
- Display player names/IDs

**Deliverable**: See other players move in world

### Milestone 3: Input Processing (Week 2, Days 1-3)

**Goal**: Client input controls server-side player

**Tasks**:
- Client captures WASD input
- Client sends input to server
- Server processes input, updates player position
- Implement basic character controller on server
- Add simple arena geometry
- Collision detection server-side

**Deliverable**: Control your player via server

### Milestone 4: Client-Side Prediction (Week 2, Days 4-7)

**Goal**: Instant local movement response

**Tasks**:
- Client applies input locally (prediction)
- Implement prediction buffer
- Server sends acknowledged input sequence
- Client reconciles on mismatch
- Add sequence numbers and timestamps
- Handle corrections smoothly

**Deliverable**: Responsive local movement

### Milestone 5: Interpolation for Remote Players (Week 3, Days 1-3)

**Goal**: Smooth remote player movement

**Tasks**:
- Implement interpolation buffer per remote player
- Store received states with timestamps
- Render interpolated state (delayed by 100-150ms)
- Handle dropped packets (extrapolation)
- Tune interpolation delay
- Debug visualizations (show actual vs rendered)

**Deliverable**: Smooth remote players despite network jitter

### Milestone 6: Combat System (Week 3-4, Days 4-7)

**Goal**: Shoot and hit other players

**Tasks**:
- Implement shoot input
- Server-side raycast hit detection
- Apply damage and track health
- Respawn on death
- Display health bars
- Send hit events to clients (feedback)

**Deliverable**: Working combat mechanics

### Milestone 7: Lag Compensation (Week 4-5, Days 1-5+)

**Goal**: Fair hit detection despite latency

**Tasks**:
- Implement historical state buffer on server
- Store world snapshots each frame
- On hit detection, rewind to shooter's timestamp
- Perform raycast in historical state
- Restore current state
- Test with artificial latency

**Deliverable**: Hits register correctly despite lag

### Milestone 8: Polish and Optimization (Week 5-6, Days 6+)

**Goal**: Production-ready multiplayer experience

**Tasks**:
- Add lobby/room system
- Implement scoreboard
- Add chat (optional)
- Optimize bandwidth (delta compression)
- Handle edge cases (reconnect, timeout)
- Network statistics display (ping, packet loss)
- Playtesting and tuning

**Deliverable**: Polished multiplayer arena

## Technical Challenges

### Challenge 1: Clock Synchronization

**Problem**: Client and server clocks differ, causing timestamp issues

**Approach**:
- Measure round-trip time (RTT) at connection
- Estimate server time: `server_time = client_time + offset`
- Periodically resync (send ping, measure RTT)
- Use monotonic time to avoid system clock changes

**Implementation**:
```
on_connect():
  send_ping(client_timestamp)

on_ping_received(server):
  send_pong(client_timestamp, server_timestamp)

on_pong_received(client, client_sent, server_time):
  rtt = current_time - client_sent
  offset = server_time + rtt/2 - current_time
  update_clock_offset(offset)
```

### Challenge 2: Rubber-Banding

**Problem**: Harsh corrections when client mispredicts

**Approach**:
- Use small error threshold before correcting
- Smooth corrections over multiple frames
- Improve prediction accuracy (match server simulation)
- Reduce latency where possible

**Smooth Correction**:
```
on_misprediction(server_pos, predicted_pos):
  error = server_pos - predicted_pos
  if length(error) > snap_threshold:
    # Large error: snap immediately
    position = server_pos
  else:
    # Small error: smooth over time
    correction_speed = 5.0  # units per second
    position += error * correction_speed * delta_time
```

### Challenge 3: Packet Loss Handling

**Problem**: Lost packets cause stuttering or missing updates

**Approach**:
- Use UDP with application-level reliability for critical messages
- Extrapolate movement briefly during loss
- Detect prolonged loss, show disconnection warning
- Redundant state (include multiple past states in packet)

**Extrapolation**:
```
if time_since_last_update > interpolation_delay * 2:
  # Extrapolate using last known velocity
  extrapolate_time = time_since_last_update - interpolation_delay
  position = last_known_position + last_known_velocity * extrapolate_time
  
  if extrapolate_time > max_extrapolate_time:
    show_disconnection_indicator()
```

### Challenge 4: Bandwidth Management

**Problem**: Too much data sent causes congestion

**Approach**:
- Send only changed data (delta compression)
- Prioritize important entities (closer, visible)
- Adaptive update rate (lower rate when bandwidth limited)
- Quantize positions (e.g., 16-bit instead of 32-bit floats)

**Delta Compression**:
```
send_state_update(clients):
  for client in clients:
    delta = compute_delta(client.last_sent_state, current_state)
    
    message = DeltaState {
      base_sequence: client.last_sequence,
      changed_entities: delta.changed,
      removed_entities: delta.removed
    }
    
    send(client, message)
    client.last_sent_state = current_state
    client.last_sequence++
```

### Challenge 5: Cheating Prevention

**Problem**: Clients can be modified to cheat

**Approach**:
- Server authority: never trust client
- Validate all client inputs (bounds check, rate limits)
- Use lag compensation carefully (limit rewind time)
- Detect impossible actions (teleportation, super speed)
- Rate limit inputs and actions

**Server Validation**:
```
on_player_input(player_id, input):
  player = get_player(player_id)
  
  # Validate move distance
  max_move = player.max_speed * delta_time
  if length(input.move_vector) > max_move:
    reject_input()  # Or clamp
  
  # Validate timing (prevent input spam)
  if current_time - player.last_input_time < min_input_interval:
    reject_input()
  
  # Validate action cooldowns
  if input.shoot and player.shoot_cooldown > 0:
    input.shoot = false
```

## Reference Implementations

### Praxis Engine (Rust)
- **File**: `examples/networking_demo.rs`
- **Crates**: `praxis_networking`
- **Concepts**: Client-server, replication, interpolation, lag compensation

### Other Engines/Frameworks

**Unity (C#)**
- Library: Mirror Networking, Netcode for GameObjects
- Tutorial: "Multiplayer FPS" tutorials (various)
- Concepts: SyncVar, NetworkTransform, client-side prediction

**Unreal Engine (C++)**
- System: Unreal's replication system
- Tutorial: "Multiplayer Shooter" (official)
- Key APIs: `APlayerController`, `APlayerState`, replication graphs

**Godot (GDScript)**
- Tutorial: "High-level multiplayer" (official docs)
- APIs: `NetworkedMultiplayerENet`, RPC, `rset`

**Bevy (Rust)**
- Plugin: `bevy_renet` (networking)
- Pattern: ECS-based networking with Rapier physics sync

**Source Engine (C++)**
- Documentation: Valve's networking articles
- Concepts: Lag compensation, interpolation (gold standard reference)

**Photon / PlayFab**
- Platform: Managed multiplayer services
- Languages: C#, C++, JavaScript, Unity/Unreal plugins

## Extension Ideas

### Beginner Extensions
- Team-based gameplay (red vs blue)
- Capture the flag mode
- Power-ups (health, ammo)
- Spectator camera

### Intermediate Extensions
- Voice chat integration
- Replays (record state history)
- Anti-cheat measures
- Server browser

### Advanced Extensions
- Interest management (send only relevant entities)
- Dedicated server hosting (headless mode)
- Cross-play (multiple platforms)
- Matchmaking with skill ratings

## Success Criteria

Your multiplayer arena should:

1. ✅ Support 4+ simultaneous players smoothly
2. ✅ Provide responsive controls (< 100ms perceived latency)
3. ✅ Handle 100-200ms network latency gracefully
4. ✅ Implement fair hit detection (lag compensation)
5. ✅ Synchronize all players without rubber-banding
6. ✅ Handle disconnections and reconnections
7. ✅ Prevent common exploits (server authority)

## Assessment Rubric

| Category | Beginner | Intermediate | Advanced |
|----------|----------|--------------|----------|
| **Architecture** | Basic client-server | + Client prediction, interpolation | + Lag compensation, optimization |
| **Features** | Connect, move, chat | + Combat, health, respawn | + Lobby, matchmaking, modes |
| **Network Quality** | Works on LAN | Handles 50-100ms latency | Handles 200ms+, packet loss |
| **Polish** | Functional prototype | Smooth gameplay, UI | Production-ready, anti-cheat |

## Common Pitfalls

1. **Client Authority**: Never let client dictate authoritative state
2. **No Prediction**: Without prediction, controls feel laggy
3. **No Interpolation**: Remote players teleport without smooth movement
4. **Fixed Delay**: Interpolation delay should adapt to network conditions
5. **Ignoring Packet Loss**: Always handle missing updates gracefully
6. **Synchronous Network I/O**: Use async/non-blocking sockets
7. **Unbounded Rewind**: Limit lag compensation rewind time (e.g., 500ms max)

## Next Steps

After completing this project, you're ready for:
- **Project 02**: First-Person Explorer (combine with networking)
- **Project 08**: Scene Editor (networked collaborative editing)
- **Project 10**: Mini Game Engine (networking as core subsystem)
