# Instance data in a structured buffer

Per-Instance data (model matrix, normal matrix, Material ID) sits in a structured buffer and each draw looks it up by `@builtin(instance_index)`; there is no bind group per mesh or per instance. Indirect and GPU-driven draws need exactly this lookup key. Implemented as the instance buffer in Phase 5.
