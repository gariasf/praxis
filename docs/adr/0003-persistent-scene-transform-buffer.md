# Persistent scene transform buffer

Transform-propagation results live in one GPU buffer indexed by renderable ID and are updated incrementally through dirty flags; there are no per-entity uniform buffers. GPU-driven rendering, ray-tracing TLAS rebuilds and instancing all consume exactly this shape. Planned for Phase 7; today the instance buffer is rewritten in full every frame.
