# Scripting Learning Path

Master Lua scripting integration for runtime game logic and rapid iteration.

## Path Overview

**Time Investment**: 1-2 weeks  
**Prerequisites**: Basic programming knowledge  
**Final Goal**: Build hot-reloadable, performant game logic in Lua

## Progression Map

```
Beginner (3-4 days)
├── Lua basics
├── Script loading
├── Function calls
└── Data passing
    ↓
Intermediate (4-5 days)
├── ECS access
├── Component manipulation
├── Game logic patterns
└── Event handling
    ↓
Advanced (4-5 days)
├── Hot-reload
├── Sandboxing
├── Performance monitoring
└── Advanced patterns
```

---

## Beginner: Lua Basics

**Goal**: Setup scripting system and master basic Lua integration.

### Prerequisites

- ✓ Basic programming knowledge (any language)
- ✓ Familiarity with ECS concepts (recommended)
- ✓ Completed [Getting Started](../getting-started/README.md)

### Step 1: Lua Language Basics

**Theory** (2-3 hours):
1. If new to Lua, learn basics:
   - Variables and types
   - Functions
   - Tables (Lua's main data structure)
   - Control flow

**Quick Lua Reference**:
```lua
-- Variables
local x = 10
local name = "Player"
local active = true

-- Tables (arrays and maps)
local array = {1, 2, 3}
local map = {x = 10, y = 20}

-- Functions
function greet(name)
    return "Hello, " .. name
end

-- Control flow
if health > 0 then
    print("Alive")
else
    print("Dead")
end

for i = 1, 10 do
    print(i)
end
```

**External Resource**: [Lua in 15 Minutes](http://tylerneylon.com/a/learn-lua/)

### Step 2: Scripting Context Setup

**Theory** (1-2 hours):
1. Read [Scripting Guide](../guides/scripting.md) - Overview
2. Understand Praxis scripting architecture
3. Review `crates/praxis_scripting/README.md`

**Practice** (2-3 hours):
1. Initialize scripting context
2. Configure basic settings

**Code Pattern**:
```rust
use praxis_scripting::{ScriptingContext, ScriptingConfig};

// Create context
let config = ScriptingConfig::default();
let mut context = ScriptingContext::new(config)?;

// Load script from file
context.load_script("game_logic", "scripts/game.lua")?;

// Load script from string
context.load_string("test", r#"
    function double(x)
        return x * 2
    end
"#)?;
```

**Exercises**:
1. Create scripting context
2. Load simple Lua script
3. Verify script loads without errors

### Step 3: Calling Lua Functions

**Practice** (3-4 hours):
1. Continue [Scripting Guide: Quick Start](../guides/scripting.md#quick-start)
2. Call Lua functions from Rust
3. Pass arguments and receive returns

**Code Patterns**:
```rust
// Call function with no arguments
let result: i32 = context.call_function("test", "get_value", ())?;

// Call with single argument
let greeting: String = context.call_function(
    "game_logic",
    "greet",
    "Hero"
)?;

// Call with multiple arguments
let sum: i32 = context.call_function(
    "math_utils",
    "add",
    (5, 10)
)?;

// Call with complex types
let pos: Vec3 = context.call_function(
    "game_logic",
    "get_spawn_point",
    "player"
)?;
```

**Exercises**:
1. Create Lua function that returns player name
2. Create Lua function that calculates distance
3. Call from Rust with different parameters
4. Handle return values

### Step 4: Passing Data Between Rust and Lua

**Theory** (1-2 hours):
1. Understand type conversion
2. Learn about UserData
3. Table passing

**Practice** (3-4 hours):
1. Pass Rust data to Lua
2. Return Lua data to Rust
3. Use tables for complex data

**Type Conversion**:
```rust
// Primitive types (automatic)
let x: f32 = context.get_global("speed")?;
context.set_global("health", 100)?;

// Tables
let table = lua.create_table()?;
table.set("x", 10.0)?;
table.set("y", 20.0)?;
context.set_global("position", table)?;

// Custom types (via UserData)
#[derive(Clone)]
struct Player {
    name: String,
    health: i32,
}

// Register and use in Lua
context.register_userdata::<Player>()?;
```

**Exercises**:
1. Pass Vec3 to Lua as table
2. Return table from Lua as Vec3
3. Store game configuration in Lua
4. Read configuration from Rust

### Step 5: Basic Game Logic

**Practice** (4-5 hours):
1. Write simple game logic in Lua
2. Call from game loop
3. Update game state

**Example: Health System**:
```lua
-- scripts/health.lua
local health_system = {}

function health_system.apply_damage(entity_health, damage)
    entity_health.current = entity_health.current - damage
    
    if entity_health.current <= 0 then
        entity_health.current = 0
        return "dead"
    elseif entity_health.current < entity_health.max * 0.3 then
        return "critical"
    else
        return "alive"
    end
end

function health_system.heal(entity_health, amount)
    entity_health.current = math.min(
        entity_health.current + amount,
        entity_health.max
    )
end

return health_system
```

**From Rust**:
```rust
let status: String = context.call_function(
    "health",
    "apply_damage",
    (player_health, 25)
)?;
```

**Exercises**:
1. Implement damage system in Lua
2. Implement inventory system
3. Implement quest tracking
4. Call from Rust game loop

### Beginner Checkpoint

**Self-Assessment**:
- [ ] Understand Lua syntax basics
- [ ] Can load and execute scripts
- [ ] Can call Lua functions from Rust
- [ ] Know how to pass data between Rust and Lua
- [ ] Written simple game logic in Lua

**Capstone Project**: Create a simple game system in Lua:
- Health/damage calculations
- Inventory management
- Level progression logic
- Callable from Rust

**Time to Complete**: 15-20 hours

---

## Intermediate: ECS Integration

**Goal**: Access and manipulate ECS entities and components from Lua.

### Prerequisites

- ✓ Completed Beginner section
- ✓ Understanding of ECS architecture
- ✓ Read [ECS Concepts](../concepts/ecs-architecture.md)

### Step 1: ECS Access Fundamentals

**Theory** (2-3 hours):
1. Read [Scripting Guide: ECS Integration](../guides/scripting.md#ecs-integration)
2. Understand world access from Lua
3. Learn entity manipulation

**Key Concepts**:
- World is passed to Lua context
- Entities are accessed by ID or name
- Components are read/written via API

### Step 2: Entity Operations

**Practice** (3-4 hours):
1. Find entities from Lua
2. Spawn new entities
3. Despawn entities

**Lua API**:
```lua
-- Find entity by name
local player = world.get_entity_by_name("Player")
local enemy = world.get_entity_by_name("Enemy_01")

-- Spawn new entity
local projectile = world.spawn()

-- Despawn entity
world.despawn(enemy)

-- Check if entity exists
if world.entity_exists(player) then
    print("Player exists")
end
```

**Exercises**:
1. Find player entity from Lua
2. Spawn enemy entities
3. Despawn defeated enemies
4. List all entities with specific tag

### Step 3: Component Access

**Practice** (4-5 hours):
1. Read components
2. Modify components
3. Add/remove components

**Lua API**:
```lua
-- Get Transform component
local transform = world.get_component_transform(player)
print("Position:", transform.translation.x, transform.translation.y, transform.translation.z)

-- Modify Transform
transform.translation.x = 10.0
transform.translation.y = 5.0
world.set_component_transform(player, transform)

-- Custom components
local health = world.get_component(player, "Health")
health.current = health.current - 10
world.set_component(player, "Health", health)
```

**Exercises**:
1. Move entity from Lua
2. Rotate entity
3. Modify health component
4. Update velocity

### Step 4: Game Logic Patterns

**Practice** (5-6 hours):
1. Implement common patterns
2. Event-driven logic
3. State management

**Pattern: Enemy AI**:
```lua
-- scripts/enemy_ai.lua
local enemy_ai = {}

function enemy_ai.update(enemy_id, player_id, delta_time)
    local enemy_pos = world.get_component_transform(enemy_id).translation
    local player_pos = world.get_component_transform(player_id).translation
    
    -- Calculate direction to player
    local dir_x = player_pos.x - enemy_pos.x
    local dir_z = player_pos.z - enemy_pos.z
    local distance = math.sqrt(dir_x * dir_x + dir_z * dir_z)
    
    -- Chase player if in range
    if distance < 10.0 and distance > 1.0 then
        local speed = 2.0
        local norm_x = dir_x / distance
        local norm_z = dir_z / distance
        
        local transform = world.get_component_transform(enemy_id)
        transform.translation.x = transform.translation.x + norm_x * speed * delta_time
        transform.translation.z = transform.translation.z + norm_z * speed * delta_time
        world.set_component_transform(enemy_id, transform)
    end
end

return enemy_ai
```

**Pattern: Pickup System**:
```lua
-- scripts/pickups.lua
function collect_pickup(player_id, pickup_id)
    local pickup = world.get_component(pickup_id, "Pickup")
    
    if pickup.type == "health" then
        local health = world.get_component(player_id, "Health")
        health.current = math.min(health.current + pickup.value, health.max)
        world.set_component(player_id, "Health", health)
        world.despawn(pickup_id)
        return "health_restored"
    elseif pickup.type == "ammo" then
        local inventory = world.get_component(player_id, "Inventory")
        inventory.ammo = inventory.ammo + pickup.value
        world.set_component(player_id, "Inventory", inventory)
        world.despawn(pickup_id)
        return "ammo_collected"
    end
end
```

**Exercises**:
1. Implement enemy patrol AI
2. Create pickup collection system
3. Build simple quest system
4. Implement door/lever interaction

### Step 5: Integration with Systems

**Practice** (4-5 hours):
1. Call scripts from Rust systems
2. Update multiple entities
3. Query entities in Lua

**Rust System**:
```rust
fn lua_update_system(
    mut scripting: ResMut<ScriptingContext>,
    world: &World,
    time: Res<Time>,
) {
    // Give Lua access to world
    scripting.with_world(world, |lua| {
        // Update all enemies
        let enemy_ids: Vec<Entity> = /* query enemies */;
        let player_id: Entity = /* find player */;
        
        for enemy_id in enemy_ids {
            let _result: () = lua.call_function(
                "enemy_ai",
                "update",
                (enemy_id, player_id, time.delta_seconds())
            )?;
        }
        
        Ok(())
    })?;
}
```

**Exercises**:
1. Create system that calls Lua update
2. Process all enemies in Lua
3. Handle player input in Lua
4. Implement turn-based combat in Lua

### Intermediate Checkpoint

**Self-Assessment**:
- [ ] Can access entities from Lua
- [ ] Can read and modify components
- [ ] Implemented game logic with ECS access
- [ ] Integrated Lua scripts with Rust systems
- [ ] Built functional game systems in Lua

**Capstone Project**: Build a complete game subsystem in Lua:
- Enemy AI with chase/patrol behavior
- Combat system with damage calculation
- Pickup/collectible system
- Player progression system
- Integrated with ECS

**Time to Complete**: 20-25 hours

---

## Advanced: Hot-Reload and Performance

**Goal**: Maximize development velocity and optimize script performance.

### Prerequisites

- ✓ Completed Intermediate section
- ✓ Built functional Lua systems
- ✓ Comfortable with Lua-Rust integration

### Step 1: Hot-Reload Setup

**Theory** (1-2 hours):
1. Continue [Scripting Guide: Hot-Reload](../guides/scripting.md#hot-reload)
2. Understand file watching
3. Learn reload strategies

**Practice** (3-4 hours):
1. Enable hot-reload
2. Configure watch directories
3. Handle reload events

**Code Pattern**:
```rust
use praxis_scripting::ScriptingContext;

// Enable hot-reload
context.enable_hot_reload("scripts")?;

// In game loop
context.check_for_changes()?;

// Scripts automatically reload when files change!
```

**Exercises**:
1. Enable hot-reload for scripts directory
2. Modify Lua script while game runs
3. Observe instant changes
4. Handle reload errors gracefully

**Development Workflow**:
1. Run game
2. Edit Lua file in text editor
3. Save file
4. See changes immediately (no restart!)

### Step 2: Sandboxing Configuration

**Theory** (2-3 hours):
1. Continue [Scripting Guide: Sandboxing](../guides/scripting.md#sandboxing)
2. Understand security levels
3. Learn restriction mechanisms

**Security Levels**:
```rust
use praxis_scripting::{ScriptingConfig, SandboxLevel};

// No restrictions (development)
let config = ScriptingConfig {
    sandbox_level: SandboxLevel::None,
    ..Default::default()
};

// Moderate restrictions (testing)
let config = ScriptingConfig {
    sandbox_level: SandboxLevel::Moderate,  // No file I/O
    ..Default::default()
};

// Strict restrictions (production)
let config = ScriptingConfig {
    sandbox_level: SandboxLevel::Strict,  // Minimal Lua features
    ..Default::default()
};
```

**Practice** (2-3 hours):
1. Test different sandbox levels
2. Understand restricted operations
3. Configure for your game

**Exercises**:
1. Test script with file I/O (should fail in Moderate+)
2. Test script with network access (should fail in Strict)
3. Configure appropriate level for game

### Step 3: Performance Monitoring

**Theory** (2-3 hours):
1. Continue [Scripting Guide: Performance Monitoring](../guides/scripting.md#performance-monitoring)
2. Understand performance tracking
3. Learn to identify slow scripts

**Practice** (4-5 hours):
1. Enable performance monitoring
2. Profile script execution
3. Identify bottlenecks
4. Optimize slow scripts

**Monitoring Setup**:
```rust
let config = ScriptingConfig {
    enable_performance_monitoring: true,
    warn_threshold_ms: 5.0,  // Warn if script takes > 5ms
    ..Default::default()
};

// Get performance stats
let stats = context.get_performance_stats();
for (script_name, duration) in stats {
    println!("{}: {:.2}ms", script_name, duration);
}
```

**Optimization Techniques**:
```lua
-- BAD: Expensive every frame
function update(dt)
    local enemies = world.get_all_entities_with("Enemy")
    for _, enemy in ipairs(enemies) do
        -- Complex calculation
        update_enemy(enemy, dt)
    end
end

-- GOOD: Cache results, update less frequently
local enemy_cache = {}
local cache_timer = 0

function update(dt)
    cache_timer = cache_timer + dt
    
    if cache_timer > 0.5 then  -- Update cache every 0.5s
        enemy_cache = world.get_all_entities_with("Enemy")
        cache_timer = 0
    end
    
    for _, enemy in ipairs(enemy_cache) do
        update_enemy(enemy, dt)
    end
end
```

**Exercises**:
1. Profile existing Lua scripts
2. Identify slowest operations
3. Optimize with caching
4. Measure improvement

### Step 4: Advanced Patterns

**Practice** (5-6 hours):
1. Implement advanced patterns
2. Script composition
3. Event system

**Pattern: Behavior Trees**:
```lua
-- Behavior tree implementation
local BehaviorTree = {}

function BehaviorTree:new()
    local bt = {root = nil}
    setmetatable(bt, {__index = BehaviorTree})
    return bt
end

function BehaviorTree:sequence(...)
    local children = {...}
    return function(entity, dt)
        for _, child in ipairs(children) do
            local result = child(entity, dt)
            if result ~= "success" then
                return result
            end
        end
        return "success"
    end
end

function BehaviorTree:selector(...)
    local children = {...}
    return function(entity, dt)
        for _, child in ipairs(children) do
            local result = child(entity, dt)
            if result == "success" then
                return "success"
            end
        end
        return "failure"
    end
end

-- Usage
local enemy_behavior = BehaviorTree:new()
enemy_behavior.root = enemy_behavior:selector(
    attack_player,
    chase_player,
    patrol
)
```

**Pattern: State Machine**:
```lua
local StateMachine = {}

function StateMachine:new(states)
    local sm = {
        states = states,
        current = nil
    }
    setmetatable(sm, {__index = StateMachine})
    return sm
end

function StateMachine:transition(state_name)
    if self.current and self.states[self.current].exit then
        self.states[self.current].exit()
    end
    
    self.current = state_name
    
    if self.states[state_name].enter then
        self.states[state_name].enter()
    end
end

function StateMachine:update(dt)
    if self.current and self.states[self.current].update then
        self.states[self.current].update(dt)
    end
end
```

**Pattern: Event System**:
```lua
local EventSystem = {}
EventSystem.listeners = {}

function EventSystem:subscribe(event_name, callback)
    if not self.listeners[event_name] then
        self.listeners[event_name] = {}
    end
    table.insert(self.listeners[event_name], callback)
end

function EventSystem:publish(event_name, ...)
    if self.listeners[event_name] then
        for _, callback in ipairs(self.listeners[event_name]) do
            callback(...)
        end
    end
end

-- Usage
EventSystem:subscribe("enemy_died", function(enemy_id, killer_id)
    world.spawn_loot(enemy_id)
    increment_kill_count(killer_id)
end)

EventSystem:publish("enemy_died", enemy_id, player_id)
```

**Exercises**:
1. Implement behavior tree for AI
2. Create state machine for player states
3. Build event system for game events
4. Combine patterns for complex behavior

### Step 5: Production Integration

**Practice** (4-5 hours):
1. Organize script files
2. Implement module system
3. Error handling
4. Debug tools

**Project Structure**:
```
scripts/
├── core/
│   ├── utils.lua
│   ├── math_utils.lua
│   └── events.lua
├── entities/
│   ├── player.lua
│   ├── enemy.lua
│   └── pickup.lua
├── systems/
│   ├── combat.lua
│   ├── inventory.lua
│   └── quest.lua
└── main.lua
```

**Module Pattern**:
```lua
-- core/utils.lua
local utils = {}

function utils.clamp(value, min, max)
    return math.max(min, math.min(max, value))
end

return utils

-- entities/player.lua
local utils = require("core.utils")

local player = {}

function player.take_damage(player_id, damage)
    local health = world.get_component(player_id, "Health")
    health.current = utils.clamp(health.current - damage, 0, health.max)
    world.set_component(player_id, "Health", health)
end

return player
```

**Exercises**:
1. Organize scripts into modules
2. Implement require system
3. Add comprehensive error handling
4. Create debug console

### Advanced Checkpoint

**Self-Assessment**:
- [ ] Hot-reload working for rapid iteration
- [ ] Appropriate sandboxing configured
- [ ] Scripts profiled and optimized
- [ ] Advanced patterns implemented
- [ ] Production-ready script organization

**Capstone Project**: Production-ready game systems:
- Complete AI system with behavior trees
- Event-driven architecture
- Hot-reload for all gameplay code
- Performance-optimized (< 5ms per frame)
- Organized module structure
- Debug tools and console

**Time to Complete**: 20-25 hours

---

## Cross-References

### Related Systems
- [Physics Path](physics.md) - Script physics behaviors
- [Animation Path](animation.md) - Control animations from Lua
- [Networking Path](networking.md) - Script multiplayer logic

### Integration
- [Input Guide](../guides/input.md) - Handle input in Lua
- [Audio Guide](../guides/audio.md) - Trigger sounds from Lua
- [ECS Concepts](../concepts/ecs-architecture.md) - Understand ECS

---

## Practice Resources

### Examples
```bash
cargo run --example scripting_demo
cargo run --example scripting_advanced_demo
```

### External Resources
- Lua 5.4 Reference Manual
- Programming in Lua (book)
- Love2D tutorials (similar Lua game development)

---

## Next Steps

After completing this path:

1. **Specialize**: Complex AI, modding support
2. **Integrate**: Script all gameplay systems
3. **Optimize**: Minimize script overhead
4. **Create**: Full game in Lua + Rust

---

[← Back to Learning Paths](README.md) | [Next: Networking Path →](networking.md)
