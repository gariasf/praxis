// Light Linking Helper Functions
// Include this in shaders that need light linking support

// Light linking uniform (per-object)
layout(set = 2, binding = 0, std140) uniform LightLinking {
    uint light_mask;
    uint _padding[3];
} light_linking;

// Check if a light on a given channel can affect this object
bool can_light_affect_object(uint light_channel) {
    return (light_linking.light_mask & (1u << light_channel)) != 0u;
}

// Check if a light with a bitmask can affect this object
bool can_light_affect_object_mask(uint light_channel_mask) {
    return (light_linking.light_mask & light_channel_mask) != 0u;
}

// Common channel definitions (for consistency)
const uint CHANNEL_HERO = 0u;
const uint CHANNEL_ENVIRONMENT = 1u;
const uint CHANNEL_ACCENT = 2u;
const uint CHANNEL_EFFECTS = 3u;
const uint CHANNEL_UI = 4u;

// Helper to create channel mask from channel index
uint channel_to_mask(uint channel) {
    return 1u << channel;
}

// Example usage in lighting loop:
/*
for (uint i = 0u; i < lighting.directional_light_count; i++) {
    DirectionalLight light = lighting.directional_lights[i];
    
    // Check if this light affects the current object
    if (!can_light_affect_object(light.channel)) {
        continue;
    }
    
    // Calculate lighting...
}
*/
