-- Player controller script
-- Demonstrates basic entity manipulation and input handling

local move_speed = 5.0
local rotation_speed = 2.0
local player_entity = nil

function on_start()
    engine.log_info("Player controller initialized")
    
    player_entity = world.get_entity_by_name("Player")
    if not player_entity then
        engine.log_error("Player entity not found!")
        return
    end
    
    local transform = world.get_component_transform(player_entity)
    engine.log_info(string.format("Player starting position: (%.2f, %.2f, %.2f)", 
        transform.translation.x, 
        transform.translation.y, 
        transform.translation.z))
end

function on_update(delta_time)
    if not player_entity then return end
    
    local transform = world.get_component_transform(player_entity)
    
    -- Simple forward movement
    transform.translation.x = transform.translation.x + move_speed * delta_time
    
    -- Oscillate up and down
    local time_offset = transform.translation.x * 0.5
    transform.translation.y = math.sin(time_offset) * 2.0
    
    world.set_component_transform(player_entity, transform)
end

function on_destroy()
    engine.log_info("Player controller destroyed")
end
