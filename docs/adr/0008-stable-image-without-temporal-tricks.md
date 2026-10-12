# Stable image without temporal tricks

Anti-aliasing is spatial (MSAA, SMAA); TAA, upscaling stacks and frame generation are not mandatory, and native resolution is the default. Lighting is hybrid: baked GI for static environments, ray-traced direct shadows and GI for dynamic objects on the high preset, cascaded shadow maps as the baseline. The target is image stability and rendering correctness on cutting-edge hardware, not mass-market scalability; the main pass is clustered forward with 4x MSAA (amended 2026-10-12, ADR 0018).
