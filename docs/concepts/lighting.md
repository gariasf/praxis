# Lighting System

Dynamic lighting in Praxis with directional and point lights.

## Light Types

### Directional Light
Simulates distant light sources (sun, moon):

```rust
#[derive(Component)]
pub struct DirectionalLight {
    pub direction: Vec3,   // Normalized direction
    pub color: Vec3,       // RGB color
    pub intensity: f32,    // Brightness multiplier
}
```

**Characteristics:**
- Position doesn't matter, only direction
- No distance attenuation
- Affects all objects equally
- Casts shadows via cascaded shadow maps

```rust
// Sun-like light
world.spawn(DirectionalLight {
    direction: Vec3::new(0.3, -0.8, 0.5).normalize(),
    color: Vec3::new(1.0, 0.95, 0.85),  // Warm white
    intensity: 1.0,
});
```

### Point Light
Omnidirectional light with attenuation:

```rust
#[derive(Component)]
pub struct PointLight {
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,  // Maximum effect distance
}
```

**Characteristics:**
- Position from Transform component
- Radiates in all directions
- Intensity falls off with distance
- Culled beyond range for performance

```rust
// Light bulb
world.spawn((
    Transform::from_xyz(0.0, 3.0, 0.0),
    PointLight {
        color: Vec3::new(1.0, 0.9, 0.7),  // Warm
        intensity: 10.0,
        range: 15.0,
    },
));
```

## Lighting Data Flow

```
ECS Components → gather_lighting_system → LightingData Resource
                                              ↓
                         LightingUniforms (GPU Buffer) → Fragment Shader
```

1. **ECS Layer**: Light components on entities
2. **Collection**: `gather_lighting_system` queries all lights
3. **GPU Layer**: `LightingUniforms` uploaded to descriptor set 0, binding 2

## Shader Integration

Lights are processed in the fragment shader:

```glsl
layout(set = 0, binding = 2) uniform LightingData {
    DirectionalLight directional_lights[8];
    PointLight point_lights[32];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
};

// Per-pixel lighting
vec3 final_color = ambient_color.rgb * albedo;

for (uint i = 0; i < directional_light_count; i++) {
    final_color += compute_directional(directional_lights[i], normal, view_dir);
}

for (uint i = 0; i < point_light_count; i++) {
    final_color += compute_point(point_lights[i], world_pos, normal, view_dir);
}
```

## Light Limits

| Type | Maximum | Reason |
|------|---------|--------|
| Directional | 8 | Few needed, expensive shadows |
| Point | 32 | Forward rendering limit |

For more lights, use deferred rendering (O(lights × pixels) instead of O(lights × triangles)).

## Attenuation

Point lights use inverse square falloff:

```
attenuation = 1.0 / (distance² + 1.0)
```

Smooth falloff at range boundary prevents popping.

## Transform Integration

Lights respect the transform hierarchy:

```rust
// Directional light with rotatable transform
world.spawn((
    DirectionalLight::new(Vec3::NEG_Y, Vec3::ONE, 1.0),
    Transform::from_rotation(Quat::from_rotation_x(-0.5)),
));

// Point light attached to moving entity
world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    PointLight::new(Vec3::ONE, 5.0, 10.0),
    Parent(player_entity),  // Moves with player
));
```

## Ambient Light

Global fill light preventing pure black shadows:

```rust
let mut lighting_data = world.resource_mut::<LightingData>();
lighting_data.ambient_color = Vec3::new(0.1, 0.1, 0.15);  // Slight blue
```

## Usage Example

```rust
use praxis_ecs::{DirectionalLight, PointLight};
use praxis_graphics::LightingData;

// Initialize lighting resource
world.insert_resource(LightingData::default());

// Sun
world.spawn(DirectionalLight::new(
    Vec3::new(0.5, -0.8, 0.3).normalize(),
    Vec3::splat(1.0),
    1.2,
));

// Room lights
for x in [-5.0, 5.0] {
    for z in [-5.0, 5.0] {
        world.spawn((
            Transform::from_xyz(x, 3.0, z),
            PointLight::new(Vec3::new(1.0, 0.9, 0.8), 8.0, 12.0),
        ));
    }
}

// Schedule light gathering
schedule.add_systems(gather_lighting_system);
```

## See Also

- [Beginner's Guide: Lighting System Architecture](../beginners-guide.md#lighting-system-architecture) - Deep dive with data flow diagrams
- [Rendering Learning Path](../learning-paths/rendering.md) - Structured learning for lighting
- [PBR Materials](pbr-materials.md) - How lights interact with materials
- [Rendering Guide](../guides/rendering.md) - Practical lighting implementation
