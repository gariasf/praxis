-- Entity spawner script
-- Demonstrates dynamic entity creation from Lua

local spawn_timer = 0
local spawn_interval = 2.0
local spawn_count = 0
local max_spawns = 5
local spawn_radius = 10.0

function on_start()
    engine.log_info("Spawner initialized")
    engine.log_info(string.format("Will spawn %d entities at %.1f second intervals", 
        max_spawns, spawn_interval))
end

function on_update(delta_time)
    spawn_timer = spawn_timer + delta_time
    
    if spawn_timer >= spawn_interval and spawn_count < max_spawns then
        spawn_entity()
        spawn_timer = 0
        spawn_count = spawn_count + 1
    end
    
    if spawn_count >= max_spawns then
        engine.log_info("Spawner completed all spawns")
    end
end

function spawn_entity()
    local entity = world.spawn()
    
    -- Generate random position in a circle
    local angle = math.random() * 2 * math.pi
    local distance = math.random() * spawn_radius
    local x = math.cos(angle) * distance
    local z = math.sin(angle) * distance
    
    world.add_component_transform(entity, x, 2.0, z)
    world.add_component_name(entity, string.format("Spawned_%d", spawn_count + 1))
    
    engine.log_info(string.format("Spawned entity at (%.2f, %.2f, %.2f)", x, 2.0, z))
end

function on_destroy()
    engine.log_info(string.format("Spawner destroyed after spawning %d entities", spawn_count))
end
