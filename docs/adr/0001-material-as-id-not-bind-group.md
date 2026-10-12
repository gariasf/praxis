# Material as ID, not bind group

Materials are entries in one structured GPU buffer, indexed by the Material ID that each Instance carries; the shader reads factors and Layers from it, so one bind group serves every material. The data flow stays "look up by ID" through bindless textures and GPU-driven rendering, which swap only how textures are sampled; per-material bind groups would be rewritten under every later phase. Implemented in Phase 6 Step 2.
