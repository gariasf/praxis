-- Enemy AI script
-- Demonstrates patrol behavior and state management

local patrol_points = {
    {x = 0, y = 0, z = 0},
    {x = 10, y = 0, z = 0},
    {x = 10, y = 0, z = 10},
    {x = 0, y = 0, z = 10}
}

local state = {
    current_point = 1,
    entity = nil,
    patrol_speed = 3.0,
    arrive_threshold = 0.5
}

function on_start()
    engine.log_info("Enemy AI initialized")
    
    state.entity = world.get_entity_by_name("Enemy")
    if not state.entity then
        engine.log_warn("Enemy entity not found, AI will not function")
        return
    end
end

function on_update(delta_time)
    if not state.entity then return end
    
    patrol_behavior(delta_time)
end

function patrol_behavior(delta_time)
    local transform = world.get_component_transform(state.entity)
    local target = patrol_points[state.current_point]
    
    -- Calculate direction to target
    local dx = target.x - transform.translation.x
    local dy = target.y - transform.translation.y
    local dz = target.z - transform.translation.z
    
    local distance = math.sqrt(dx * dx + dy * dy + dz * dz)
    
    if distance < state.arrive_threshold then
        -- Reached patrol point, move to next
        state.current_point = (state.current_point % #patrol_points) + 1
        engine.log_debug(string.format("Enemy reached patrol point %d", state.current_point - 1))
    else
        -- Move towards target
        local dir_x = dx / distance
        local dir_y = dy / distance
        local dir_z = dz / distance
        
        transform.translation.x = transform.translation.x + dir_x * state.patrol_speed * delta_time
        transform.translation.y = transform.translation.y + dir_y * state.patrol_speed * delta_time
        transform.translation.z = transform.translation.z + dir_z * state.patrol_speed * delta_time
        
        world.set_component_transform(state.entity, transform)
    end
end

function on_destroy()
    engine.log_info("Enemy AI destroyed")
end
