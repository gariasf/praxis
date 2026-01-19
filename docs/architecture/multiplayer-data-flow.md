# Multiplayer Data Flow

This document provides a comprehensive visual guide to the client-server networking architecture in Praxis, showing entity replication, lag compensation, interpolation, and the complete data flow for multiplayer games.

## Overview

Praxis uses a **client-server** architecture for multiplayer networking:

- **Server**: Authoritative game state, runs physics and game logic
- **Clients**: Render state, predict inputs, interpolate remote entities
- **Transport**: TCP for reliability, UDP for low-latency state updates

## Network Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                      CLIENT-SERVER TOPOLOGY                               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│                         SERVER (Authoritative)                            │
│                    ┌───────────────────────────┐                         │
│                    │  Game State (ECS World)   │                         │
│                    │  - Physics Simulation     │                         │
│                    │  - Game Logic             │                         │
│                    │  - Entity Authority       │                         │
│                    └─────────┬─────────────────┘                         │
│                              │                                            │
│                    ┌─────────┴─────────┐                                 │
│                    │ Network Server    │                                 │
│                    │ - Client Manager  │                                 │
│                    │ - Replication     │                                 │
│                    │ - Lag Comp        │                                 │
│                    └─────────┬─────────┘                                 │
│                              │                                            │
│           ┌──────────────────┼──────────────────┐                        │
│           │                  │                  │                        │
│           ▼                  ▼                  ▼                        │
│    ┌──────────────┐   ┌──────────────┐   ┌──────────────┐              │
│    │  Client 1    │   │  Client 2    │   │  Client N    │              │
│    │              │   │              │   │              │              │
│    │ ┌──────────┐ │   │ ┌──────────┐ │   │ ┌──────────┐ │              │
│    │ │ Network  │ │   │ │ Network  │ │   │ │ Network  │ │              │
│    │ │ Client   │ │   │ │ Client   │ │   │ │ Client   │ │              │
│    │ └────┬─────┘ │   │ └────┬─────┘ │   │ └────┬─────┘ │              │
│    │      │       │   │      │       │   │      │       │              │
│    │ ┌────▼─────┐ │   │ ┌────▼─────┐ │   │ ┌────▼─────┐ │              │
│    │ │ECS World │ │   │ │ECS World │ │   │ │ECS World │ │              │
│    │ │(Predict) │ │   │ │(Predict) │ │   │ │(Predict) │ │              │
│    │ └──────────┘ │   │ └──────────┘ │   │ └──────────┘ │              │
│    │      │       │   │      │       │   │      │       │              │
│    │ ┌────▼─────┐ │   │ ┌────▼─────┐ │   │ ┌────▼─────┐ │              │
│    │ │Renderer  │ │   │ │Renderer  │ │   │ │Renderer  │ │              │
│    │ └──────────┘ │   │ └──────────┘ │   │ └──────────┘ │              │
│    └──────────────┘   └──────────────┘   └──────────────┘              │
│                                                                           │
│  Data Flow:                                                              │
│    Client → Server: Input commands, player actions                       │
│    Server → Client: Entity state snapshots, events                       │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Entity Replication Flow

### Server-Side Entity Spawning

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    SERVER: Entity Spawning                                │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Game Logic (e.g., player joins)                                        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ spawn_player_entity(world, player_id)                       │        │
│  │                                                              │        │
│  │  let entity = world.spawn((                                 │        │
│  │      NetworkId::new(generate_id()),  // Unique network ID   │        │
│  │      Replicated::new()                // Mark for replication│       │
│  │          .with_priority(255)          // High priority       │        │
│  │          .with_owner(client_id),      // Ownership          │        │
│  │      Transform::default(),            // Local transform     │        │
│  │      ReplicatedTransform::default(),  // Network transform   │        │
│  │      Velocity::default(),             // Physics             │        │
│  │      Health { current: 100.0, max: 100.0 },                │        │
│  │      PlayerController::default(),                            │        │
│  │  ));                                                         │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                         ↓                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Replication System (runs every frame)                       │        │
│  │                                                              │        │
│  │  1. Query entities with (NetworkId, Replicated)            │        │
│  │  2. For each new entity:                                    │        │
│  │     - Serialize all replicated components                   │        │
│  │     - Create SpawnEntity message                            │        │
│  │     - Add to outgoing message queue                         │        │
│  │  3. For each changed entity:                                │        │
│  │     - Use change detection (Changed<T>)                     │        │
│  │     - Serialize only changed components                     │        │
│  │     - Create UpdateEntity message                           │        │
│  │     - Add to outgoing queue                                 │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                         ↓                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Network Server (server.update())                            │        │
│  │                                                              │        │
│  │  1. Batch messages by client                                │        │
│  │  2. Serialize to binary (bincode)                           │        │
│  │  3. Send via UDP (state) or TCP (events)                   │        │
│  │     - SpawnEntity → TCP (reliable)                          │        │
│  │     - UpdateEntity → UDP (unreliable, frequent)             │        │
│  │     - DespawnEntity → TCP (reliable)                        │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                         ↓                                                 │
│            Network (TCP/UDP packets)                                     │
│                         ↓                                                 │
└──────────────────────────────────────────────────────────────────────────┘
```

### Client-Side Entity Replication

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    CLIENT: Entity Replication                             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│            Network (Receive TCP/UDP packets)                             │
│                         ↓                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Network Client (client.update())                            │        │
│  │                                                              │        │
│  │  1. Receive packets                                         │        │
│  │  2. Deserialize messages (bincode)                          │        │
│  │  3. Categorize by type                                      │        │
│  │     - SpawnEntity                                            │        │
│  │     - UpdateEntity                                           │        │
│  │     - DespawnEntity                                          │        │
│  │     - Event                                                  │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                         ↓                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Message Processing                                          │        │
│  ├─────────────────────────────────────────────────────────────┤        │
│  │                                                              │        │
│  │ SpawnEntity Message:                                        │        │
│  │   1. Check if entity already exists (by NetworkId)         │        │
│  │   2. If not, spawn new entity:                              │        │
│  │      world.spawn((                                          │        │
│  │          NetworkId(msg.network_id),                         │        │
│  │          RemoteEntity,  // Mark as server-controlled        │        │
│  │          msg.components...                                  │        │
│  │      ))                                                      │        │
│  │   3. Add to NetworkId → Entity mapping                     │        │
│  │                                                              │        │
│  │ UpdateEntity Message:                                       │        │
│  │   1. Lookup local entity by NetworkId                      │        │
│  │   2. If found:                                              │        │
│  │      - Deserialize component updates                        │        │
│  │      - Add to interpolation buffer                          │        │
│  │      - Schedule for interpolation/extrapolation             │        │
│  │   3. If not found, request full spawn                       │        │
│  │                                                              │        │
│  │ DespawnEntity Message:                                      │        │
│  │   1. Lookup local entity by NetworkId                      │        │
│  │   2. If found:                                              │        │
│  │      - Remove from NetworkId mapping                        │        │
│  │      - Despawn entity and children                          │        │
│  │                                                              │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                         ↓                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Interpolation System (runs every frame)                    │        │
│  │                                                              │        │
│  │  For each RemoteEntity with buffered updates:              │        │
│  │    1. Get current time and render delay (100ms)            │        │
│  │    2. Find two snapshots around target time                │        │
│  │    3. Interpolate between snapshots:                        │        │
│  │       position = lerp(snapshot1.pos, snapshot2.pos, t)     │        │
│  │       rotation = slerp(snapshot1.rot, snapshot2.rot, t)    │        │
│  │    4. Apply interpolated transform                          │        │
│  │                                                              │        │
│  │  If no future snapshots (extrapolation):                   │        │
│  │    1. Use last known velocity                               │        │
│  │    2. Extrapolate forward:                                  │        │
│  │       position += velocity * delta_time                     │        │
│  │    3. Apply with reduced confidence (fade out velocity)    │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                         ↓                                                 │
│                  Render (smooth, interpolated)                           │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Client Input Flow

### Input to Server

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    CLIENT: Input Processing                               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  User Input (keyboard, mouse, gamepad)                                   │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Input System                                                │        │
│  │                                                              │        │
│  │  Collect input state:                                       │        │
│  │    - Movement: WASD, joystick                               │        │
│  │    - Actions: Jump, shoot, interact                         │        │
│  │    - View: Mouse delta, look direction                      │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Create Input Command                                        │        │
│  │                                                              │        │
│  │  let command = InputCommand {                               │        │
│  │      sequence: next_sequence_number(),                      │        │
│  │      timestamp: current_time(),                             │        │
│  │      movement: Vec2::new(x, y),                             │        │
│  │      actions: pressed_buttons,                              │        │
│  │      view_direction: camera_forward,                        │        │
│  │  };                                                          │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Client-Side Prediction                                      │        │
│  │                                                              │        │
│  │  1. Store command in history buffer                         │        │
│  │  2. Apply command locally (predict):                        │        │
│  │     - Update player transform                               │        │
│  │     - Apply movement physics                                │        │
│  │     - Play animations                                        │        │
│  │  3. Render predicted state immediately                      │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Send to Server                                              │        │
│  │                                                              │        │
│  │  1. Serialize command (bincode)                             │        │
│  │  2. Send via UDP (low latency)                              │        │
│  │  3. Include sequence number for ordering                    │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│            Network → Server                                              │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Server Processing

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    SERVER: Input Processing                               │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│            Network ← Client                                              │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Receive Input Command                                       │        │
│  │                                                              │        │
│  │  1. Deserialize command                                     │        │
│  │  2. Validate:                                                │        │
│  │     - Is client authorized?                                 │        │
│  │     - Is sequence number valid?                             │        │
│  │     - Is timestamp reasonable?                              │        │
│  │  3. Add to client input queue                               │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Process Input (in Update system set)                       │        │
│  │                                                              │        │
│  │  For each client's input queue:                             │        │
│  │    1. Dequeue oldest unprocessed command                    │        │
│  │    2. Find player entity (by client_id)                     │        │
│  │    3. Apply input to player:                                │        │
│  │       - Update velocity from movement                       │        │
│  │       - Trigger actions (jump, shoot)                       │        │
│  │       - Update view direction                               │        │
│  │    4. Mark command as processed (sequence number)           │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Physics & Game Logic                                        │        │
│  │                                                              │        │
│  │  1. Step physics simulation (60 Hz)                         │        │
│  │  2. Apply game rules                                        │        │
│  │  3. Detect collisions                                        │        │
│  │  4. Update all entity states                                │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Replication System                                          │        │
│  │                                                              │        │
│  │  1. Detect changed components (Change Detection)           │        │
│  │  2. Create state snapshot                                   │        │
│  │  3. Send to all clients (broadcast)                         │        │
│  │  4. Include last processed sequence number per client       │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│            Network → All Clients                                         │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

### Client Reconciliation

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    CLIENT: Server Reconciliation                          │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│            Network ← Server (State Update)                               │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Receive State Update                                        │        │
│  │                                                              │        │
│  │  State message contains:                                    │        │
│  │    - Server state for player entity                         │        │
│  │    - Last acknowledged input sequence                       │        │
│  │    - Server timestamp                                        │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Reconciliation Check                                        │        │
│  │                                                              │        │
│  │  1. Find acknowledged input in history                      │        │
│  │  2. Get predicted state at that time                        │        │
│  │  3. Compare with server state:                              │        │
│  │     if distance(predicted, server) > threshold:             │        │
│  │       MISPREDICTION - need reconciliation                   │        │
│  │     else:                                                    │        │
│  │       GOOD - discard old inputs                             │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Replay Inputs (if misprediction)                           │        │
│  │                                                              │        │
│  │  1. Reset player to server state                            │        │
│  │  2. Replay all unacknowledged inputs:                       │        │
│  │     for each input after last_ack:                          │        │
│  │       apply_input(player, input)                            │        │
│  │       step_physics(fixed_timestep)                          │        │
│  │  3. Result is corrected prediction                          │        │
│  │                                                              │        │
│  │  Visual smoothing:                                          │        │
│  │    - Interpolate from old position to corrected position   │        │
│  │    - Duration: 100-200ms                                    │        │
│  │    - Prevents jarring "teleport" effect                     │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Cleanup                                                     │        │
│  │                                                              │        │
│  │  1. Remove acknowledged inputs from history                 │        │
│  │  2. Update last server timestamp                            │        │
│  │  3. Continue with new predictions                           │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Lag Compensation

Server-side lag compensation for fair hit detection:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    LAG COMPENSATION (Server-Side)                         │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Client fires weapon (t=0, but arrives at server at t=50ms)             │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Receive Shoot Command                                       │        │
│  │                                                              │        │
│  │  Command contains:                                          │        │
│  │    - Player ID                                              │        │
│  │    - Client timestamp (when shot was fired)                 │        │
│  │    - Target position (raycast hit)                          │        │
│  │    - View direction                                          │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Calculate Rewind Time                                       │        │
│  │                                                              │        │
│  │  1. Estimate client latency:                                │        │
│  │     latency = (current_server_time - client_timestamp)     │        │
│  │              + measured_rtt / 2                             │        │
│  │                                                              │        │
│  │  2. Calculate rewind target:                                │        │
│  │     rewind_time = current_time - latency                    │        │
│  │                                                              │        │
│  │  Example: 50ms latency → rewind 50ms into past             │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Rewind World State                                          │        │
│  │                                                              │        │
│  │  For each entity with NetworkHistory:                      │        │
│  │    1. Find snapshot closest to rewind_time                  │        │
│  │    2. Restore entity transform/state                        │        │
│  │    3. Store current state for later restoration             │        │
│  │                                                              │        │
│  │  History buffer example (per entity):                       │        │
│  │    [t-200ms] → snapshot                                     │        │
│  │    [t-150ms] → snapshot                                     │        │
│  │    [t-100ms] → snapshot  ← Rewind to here                  │        │
│  │    [t-50ms]  → snapshot                                     │        │
│  │    [t-0ms]   → current (saved)                             │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Perform Hit Detection                                       │        │
│  │                                                              │        │
│  │  1. Cast ray from player position                          │        │
│  │  2. Test against rewound entity positions                   │        │
│  │  3. Check for hits                                          │        │
│  │                                                              │        │
│  │  World state now represents what client saw when firing    │        │
│  │  Hit detection is "fair" from client perspective           │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Restore Current State                                       │        │
│  │                                                              │        │
│  │  1. Restore all entities to current positions               │        │
│  │  2. Apply hit results (damage, etc.)                        │        │
│  │  3. Continue simulation from current time                   │        │
│  └─────────────────────────────────────────────────────────────┘        │
│         ↓                                                                 │
│  ┌─────────────────────────────────────────────────────────────┐        │
│  │ Replicate Results                                           │        │
│  │                                                              │        │
│  │  - Send hit confirmation to shooter                         │        │
│  │  - Send damage event to victim                              │        │
│  │  - Replicate to all observers                               │        │
│  └─────────────────────────────────────────────────────────────┘        │
│                                                                           │
│  Why This Works:                                                         │
│  - Server recreates world state as client saw it                         │
│  - Compensates for network delay automatically                           │
│  - Client gets instant feedback (prediction)                             │
│  - Server validation prevents cheating                                    │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Complete Frame Timeline

```
┌──────────────────────────────────────────────────────────────────────────┐
│                    MULTIPLAYER FRAME TIMELINE                             │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Time: t = 0ms                                                           │
│  ┌──────────────────────────────────────────────────────────┐           │
│  │ CLIENT                                                    │           │
│  │ - User presses 'W' (move forward)                        │           │
│  │ - Create input command (sequence #42)                    │           │
│  │ - Predict: move player forward locally                   │           │
│  │ - Render predicted position                              │           │
│  │ - Send input to server (UDP)                             │           │
│  └──────────────────────────────────────────────────────────┘           │
│                        ↓                                                  │
│  Time: t = 50ms (network delay)                                         │
│  ┌──────────────────────────────────────────────────────────┐           │
│  │ SERVER                                                    │           │
│  │ - Receive input command #42                              │           │
│  │ - Validate and queue                                     │           │
│  │                                                           │           │
│  │ Next tick (16.67ms @ 60Hz):                             │           │
│  │ - Process input #42                                      │           │
│  │ - Update player velocity                                 │           │
│  │ - Step physics simulation                                │           │
│  │ - Detect state changes                                   │           │
│  │ - Create state snapshot                                  │           │
│  │ - Send snapshot to all clients (UDP)                     │           │
│  │   • Includes ack for sequence #42                        │           │
│  └──────────────────────────────────────────────────────────┘           │
│                        ↓                                                  │
│  Time: t = 100ms (50ms back to client)                                  │
│  ┌──────────────────────────────────────────────────────────┐           │
│  │ CLIENT                                                    │           │
│  │ - Receive server state (ack #42)                         │           │
│  │ - Compare predicted vs server position                   │           │
│  │                                                           │           │
│  │ If positions match (within threshold):                   │           │
│  │   - Discard inputs up to #42                             │           │
│  │   - Continue predicting new inputs                       │           │
│  │                                                           │           │
│  │ If mismatch detected:                                    │           │
│  │   - Reset to server position                             │           │
│  │   - Replay inputs #43, #44, #45... (not yet acked)      │           │
│  │   - Interpolate visually to hide correction              │           │
│  └──────────────────────────────────────────────────────────┘           │
│                        ↓                                                  │
│  Meanwhile, for OTHER players:                                           │
│  ┌──────────────────────────────────────────────────────────┐           │
│  │ CLIENT (Remote Entity Interpolation)                     │           │
│  │                                                           │           │
│  │ t=0ms:   Snapshot A (position, rotation)                │           │
│  │ t=50ms:  Snapshot B received                             │           │
│  │ t=100ms: Snapshot C received                             │           │
│  │                                                           │           │
│  │ Render time = t - 100ms (interpolation delay)           │           │
│  │                                                           │           │
│  │ Current frame (t=100ms):                                 │           │
│  │   Render at interpolation time = t-100ms = 0ms          │           │
│  │   Interpolate between snapshots A and B                  │           │
│  │   position = lerp(A.pos, B.pos, 0.5)                    │           │
│  │                                                           │           │
│  │ Result: Smooth motion, always 100ms behind server        │           │
│  └──────────────────────────────────────────────────────────┘           │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

## Network Messages

### Message Types

```rust
#[derive(Serialize, Deserialize)]
pub enum NetworkMessage {
    // Connection
    Connect { player_name: String },
    Disconnect { reason: String },
    
    // Entity Lifecycle
    SpawnEntity {
        network_id: u64,
        owner: ClientId,
        components: Vec<ComponentData>,
    },
    DespawnEntity {
        network_id: u64,
    },
    
    // State Updates
    UpdateEntity {
        network_id: u64,
        changes: Vec<ComponentChange>,
        timestamp: f64,
    },
    
    // Input
    InputCommand {
        sequence: u32,
        timestamp: f64,
        movement: Vec2,
        actions: u32,  // Bitfield
        view_direction: Vec3,
    },
    
    // Events
    GameEvent {
        event_type: EventType,
        data: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct ComponentData {
    type_id: ComponentTypeId,
    data: Vec<u8>,  // Serialized component
}
```

## Bandwidth Optimization

### Priority-Based Replication

```
High Priority (255):
  - Player entities
  - Projectiles near players
  - Interactive objects

Medium Priority (128):
  - NPCs in view
  - Vehicles
  - Props

Low Priority (64):
  - Static objects
  - Distant entities
  - Environmental details

Update Rate by Priority:
  High:   60 Hz (every frame)
  Medium: 20 Hz (every 3rd frame)
  Low:    10 Hz (every 6th frame)
```

### Delta Compression

```
Only send changed components:
  Frame N:   Full snapshot (all components)
  Frame N+1: Only Transform (changed)
  Frame N+2: Only Velocity (changed)
  Frame N+3: Only Health (changed)

Reduces bandwidth by 70-90% in typical scenarios
```

## Related Documentation

- [Networking Guide](../guides/systems/networking.md) - Complete networking implementation guide
- [Networking Learning Path](../learning-paths/networking.md) - Progressive networking tutorials
- [praxis_networking Crate](../../crates/praxis_networking/README.md) - Crate documentation
- [ECS System Execution](ecs-system-execution-order.md) - System scheduling for networking
- [Physics Guide](../guides/physics.md) - Physics integration with networking
- [Scripting Guide](../guides/scripting.md) - Scripting in multiplayer context
