# Frame data snapshot

The renderer never reads the ECS World mid-frame: a prepare pass builds Frame data (visible Renderables, lights, camera) and the renderer consumes only that. Frame data is an owned, `Send` value with no borrow into the World (amended 2026-10-12, ADR 0016). A render-thread split then becomes a channel send, and the renderer can be tested headless without a World. Today prepare (`src/render/prepare.rs`) fills Frame data from the World on the main thread, and the `Renderer`, which owns the Mesh and Material pools, draws it without reading the World.
