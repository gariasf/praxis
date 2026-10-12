# Frame data snapshot

The renderer never reads the ECS World mid-frame: a prepare pass builds Frame data (visible Renderables, lights, camera) and the renderer consumes only that. A render-thread split then becomes a channel send, and the renderer can be tested headless without a World. Planned; today `State::render` still queries the World directly.
