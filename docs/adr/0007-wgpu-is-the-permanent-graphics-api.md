# wgpu is the permanent graphics API

The engine stays on wgpu (Vulkan, DX12, Metal) for its lifetime. If one subsystem exceeds wgpu's safe API it drops to `wgpu-hal` for that subsystem only; a migration to raw Vulkan through `ash` is not planned and must not influence design. wgpu's experimental inline ray queries cover the planned RT features, and one portable API outweighs the last margin of control. Consoles are out of scope (0010).
