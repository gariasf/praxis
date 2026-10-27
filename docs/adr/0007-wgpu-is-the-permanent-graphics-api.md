# wgpu is the permanent graphics API

The engine stays on wgpu (Vulkan, DX12, Metal) for its lifetime, using `wgpu-hal` only for a subsystem that exceeds wgpu's safe API; migration to raw Vulkan through `ash` is not planned and must not influence design. wgpu's experimental inline ray queries [ref:wgpu-types-30-ray-query-experimental] cover the planned RT features, and one portable API outweighs the last margin of control. Consoles are out of scope (0010).
