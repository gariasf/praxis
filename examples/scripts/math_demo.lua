-- Math API demonstration
-- Shows vector and math operations

function demonstrate_vectors()
    engine.log_info("=== Vector Operations ===")
    
    local v1 = math.Vec3(1, 0, 0)
    local v2 = math.Vec3(0, 1, 0)
    
    engine.log_info("Created v1: (" .. v1.x .. ", " .. v1.y .. ", " .. v1.z .. ")")
    engine.log_info("Created v2: (" .. v2.x .. ", " .. v2.y .. ", " .. v2.z .. ")")
end

function calculate_distance(x1, y1, z1, x2, y2, z2)
    local dx = x2 - x1
    local dy = y2 - y1
    local dz = z2 - z1
    
    return math.sqrt(dx*dx + dy*dy + dz*dz)
end

function interpolate_position(from_x, from_y, from_z, to_x, to_y, to_z, t)
    local x = from_x + (to_x - from_x) * t
    local y = from_y + (to_y - from_y) * t
    local z = from_z + (to_z - from_z) * t
    
    return x, y, z
end

function rotate_angle(angle_degrees)
    local angle_radians = angle_degrees * (math.pi / 180.0)
    
    local cos_val = math.cos(angle_radians)
    local sin_val = math.sin(angle_radians)
    
    return cos_val, sin_val
end

-- Run demonstrations
function on_start()
    demonstrate_vectors()
    
    engine.log_info("\n=== Distance Calculation ===")
    local dist = calculate_distance(0, 0, 0, 3, 4, 0)
    engine.log_info("Distance: " .. dist)
    
    engine.log_info("\n=== Position Interpolation ===")
    local mid_x, mid_y, mid_z = interpolate_position(0, 0, 0, 10, 10, 10, 0.5)
    engine.log_info("Midpoint: (" .. mid_x .. ", " .. mid_y .. ", " .. mid_z .. ")")
    
    engine.log_info("\n=== Rotation Demo ===")
    local cos_val, sin_val = rotate_angle(45)
    engine.log_info("45 degrees: cos=" .. cos_val .. ", sin=" .. sin_val)
end
