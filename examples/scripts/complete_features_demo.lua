-- Complete Features Demo - Game Logic Script
-- This script demonstrates Lua scripting integration with the Praxis engine
-- Edit this file while the demo is running to see hot-reload in action!

-- Called every frame with delta_time (in seconds)
function update(delta_time)
    -- Frame update logic can go here
    -- This is called continuously, so keep it fast!
end

-- Called when terrain system is loaded
function on_terrain_loaded()
    print("Terrain system loaded!")
end

-- Called when network connection is established
function on_network_connected()
    print("Network connected!")
end

-- Calculate camera influence for LOD based on distance
-- Returns a value between 0.0 and 1.0
function calculate_camera_influence(distance)
    if distance < 50 then
        return 1.0  -- Full detail
    elseif distance < 100 then
        return 0.75 -- High detail
    elseif distance < 200 then
        return 0.5  -- Medium detail
    else
        return 0.25 -- Low detail
    end
end

-- Demo function - can be called from the engine
function get_demo_info()
    return {
        name = "Complete Features Demo",
        version = "1.0",
        features = {
            "Terrain Rendering",
            "Networking",
            "Scripting",
            "TAA/SSR/SSAO",
            "Deferred Rendering"
        }
    }
end

print("Complete Features Demo script loaded!")
print("You can edit this file and press 'L' to reload, or just save and hot-reload will apply changes automatically")
