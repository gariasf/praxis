# One mega-buffer for vertices, one for indices

Meshes are (vertex offset, index offset, index count) sub-allocations of one shared vertex buffer and one shared index buffer in the Mesh pool. Indirect draws, BLAS construction and instancing all need geometry addressable by offset in a single buffer, and going from buffer-per-mesh later is expensive because every render path bakes in the smaller shape. Implemented in Phase 6 Step 1; growth by CPU re-upload is interim and is revisited with streaming.
