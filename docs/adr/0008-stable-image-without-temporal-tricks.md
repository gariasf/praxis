# Stable image without temporal tricks

Anti-aliasing is spatial (MSAA, SMAA); TAA, upscaling stacks and frame generation are not mandatory, and native resolution is the default. Lighting is ray-traced probe GI with hard ray-traced shadows (amended 2026-10-13, ADR 0020). The target is image stability and rendering correctness on cutting-edge hardware, not mass-market scalability; the main pass is clustered forward with 4x MSAA (amended 2026-10-12, ADR 0019).
