# Reference

API documentation, specifications, and configuration reference.

## Core Reference

- [Crates](crates.md) - All workspace crates, their purposes, and dependencies
- [Components](components.md) - ECS components reference
- [Shaders](shaders.md) - Shader bindings and conventions
- [Configuration](configuration.md) - Configurable constants and settings

## File Formats

- [Scene Format](scene-format.md) - Scene file specification and serialization

## System APIs

Comprehensive API reference for each subsystem:

- [Animation API](animation-api.md) - Skeletal animation, blending, IK, and root motion
- [Audio API](audio-api.md) - Audio playback and spatial audio
- [Camera API](camera-api.md) - Camera types and controllers
- [GUI API](gui-api.md) - GUI system and widgets
- [Input API](input-api.md) - Keyboard, mouse, and gamepad handling
- [Mesh API](mesh-api.md) - Geometry loading and management

## Rustdoc API Documentation

For complete API documentation with all types and methods:

```bash
cargo doc --workspace --no-deps --open
```

## Related

- [Guides](../guides/README.md) - How to use these APIs
- [Concepts](../concepts/README.md) - Theory behind the APIs
- [Internals](../internals/) - Implementation details for contributors
