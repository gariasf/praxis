-- Simple demonstration script
-- Shows basic Lua functionality

local counter = 0

function on_start()
    engine.log_info("Script initialized")
    counter = 0
end

function on_update(delta_time)
    counter = counter + 1
    
    if counter % 100 == 0 then
        engine.log_info("Update called " .. counter .. " times")
    end
end

function greet(name)
    return "Hello, " .. name .. "!"
end

function calculate_circle_area(radius)
    return math.pi * radius * radius
end

function on_destroy()
    engine.log_info("Script destroyed after " .. counter .. " updates")
end
