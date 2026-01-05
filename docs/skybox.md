# Skybox System

The Praxis engine provides a complete skybox rendering system for creating realistic background environments. Skyboxes use cubemap textures to simulate distant surroundings like skies, space, or indoor environments.

## Overview

A skybox is a large cube textured with a cubemap that surrounds the entire scene, creating the illusion of a distant environment. The skybox renderer uses specialized techniques to ensure it always appears at infinite distance:

- **Reversed Depth**: Skybox always renders behind all other geometry
- **Camera-Centered**: Skybox follows camera rotation but not translation
- **Cubemap Sampling**: 6-face texture for seamless panoramic views

## Components

### Skybox Component (ECS)

```rust
use praxis_ecs::Skybox;

// Spawn a skybox entity
world.spawn(Skybox::new("day_sky"));
```

### Cubemap Texture

Cubemaps consist of 6 square textures representing the faces of a cube:
- +X (Right)
- -X (Left)
- +Y (Top)
- -Y (Bottom)
- +Z (Front)
- -Z (Back)

## Loading Cubemaps

### From 6 Face Images

```rust
use praxis_graphics::TextureManager;

let face_paths = [
    "assets/skybox/right.png",   // +X
    "assets/skybox/left.png",    // -X
    "assets/skybox/top.png",     // +Y
    "assets/skybox/bottom.png",  // -Y
    "assets/skybox/front.png",   // +Z
    "assets/skybox/back.png",    // -Z
];

texture_manager.load_cubemap("day_sky", face_paths)?;
```

### From Equirectangular Image

```rust
// Convert a 360° panorama to a cubemap
texture_manager.load_cubemap_from_equirectangular(
    "sky",
    "assets/panorama.jpg",
    512  // Face size
)?;
```

### From Raw Face Data

```rust
use praxis_graphics::Cubemap;

let face_size = 512;
let face_data: [Vec<u8>; 6] = /* ... RGBA8 data for each face ... */;

let cubemap = Cubemap::from_faces(
    memory_allocator,
    command_buffer_allocator,
    queue,
    face_size,
    face_data,
)?;
```

## Rendering a Skybox

### 1. Create SkyboxRenderer

```rust
use praxis_graphics::SkyboxRenderer;

let skybox_renderer = SkyboxRenderer::new(
    device.clone(),
    render_pass.clone(),
    viewport,
    memory_allocator.clone(),
)?;
```

### 2. Create Descriptor Set

```rust
let cubemap_texture = texture_manager.get_texture("day_sky").unwrap();

let descriptor_set = skybox_renderer.create_descriptor_set(
    descriptor_set_allocator,
    view_proj_buffer.clone(),
    cubemap_texture.view.clone(),
    cubemap_texture.sampler.clone(),
)?;
```

### 3. Record Draw Commands

```rust
command_buffer
    .bind_pipeline_graphics(skybox_renderer.pipeline().clone())?
    .bind_vertex_buffers(0, skybox_renderer.vertex_buffer().clone())?
    .bind_index_buffer(skybox_renderer.index_buffer().clone())?
    .bind_descriptor_sets(
        PipelineBindPoint::Graphics,
        skybox_renderer.pipeline().layout().clone(),
        0,
        descriptor_set,
    )?
    .draw_indexed(skybox_renderer.index_count(), 1, 0, 0, 0)?;
```

## Technical Details

### Reversed Depth

The skybox uses a special depth trick to ensure it always renders behind all geometry:

```glsl
// In vertex shader
gl_Position = clip_pos.xyww;  // Set z = w
```

After perspective division (z/w), this results in depth = 1.0, which with reversed depth testing (LessOrEqual) ensures the skybox is always behind everything else.

### Camera-Centered Transform

The skybox removes translation from the view matrix to keep it centered on the camera:

```glsl
// Remove translation from view matrix
mat4 view_no_translation = mat4(mat3(view));
```

This makes the skybox appear infinitely distant, as it doesn't move when the camera translates.

### Cubemap Sampling

The fragment shader samples the cubemap using a direction vector:

```glsl
vec3 color = texture(skybox_cubemap, direction).rgb;
```

The direction vector points from the camera origin toward the skybox surface, and the GPU automatically handles face selection.

## Equirectangular Conversion

The system can convert equirectangular (spherical panorama) images to cubemaps. This is useful for:
- HDR environment maps
- 360° photos
- Procedurally generated skies

The conversion algorithm:
1. For each pixel on each cube face:
2. Calculate the 3D direction vector for that pixel
3. Convert direction to spherical coordinates (theta, phi)
4. Map spherical coordinates to UV coordinates in the equirectangular image
5. Sample the equirectangular image at those coordinates

## Performance Considerations

- **Face Size**: Typical sizes are 512, 1024, or 2048 pixels per face
  - Larger = better quality but more memory
  - Smaller = faster loading and less memory
  
- **Compression**: Consider using compressed texture formats (DXT/BC) for production

- **Mipmaps**: Not currently implemented but would improve quality at glancing angles

- **Draw Order**: Render skybox last (or first with reversed depth) to minimize overdraw

## Example Usage

See `examples/skybox_demo.rs` for a complete working example demonstrating:
- Loading a procedural cubemap
- Creating a skybox renderer
- Rendering the skybox with camera movement
- First-person camera controls

## Asset Requirements

When creating cubemap assets:

1. **Face Orientation**: Use the standard +X, -X, +Y, -Y, +Z, -Z order
2. **Square Images**: All faces must be square and the same size
3. **Seamless Edges**: Ensure adjacent faces tile seamlessly
4. **Format**: PNG or JPEG supported (PNG recommended for quality)

## Future Enhancements

Potential improvements for the skybox system:
- Mipmap generation for better quality
- HDR cubemap support
- Dynamic skybox blending (day/night transitions)
- Environment map reflections on objects
- Atmospheric scattering simulation
