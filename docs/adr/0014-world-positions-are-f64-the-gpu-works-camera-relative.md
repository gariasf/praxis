# World positions are f64; the GPU works camera-relative

A Game world of 10 by 10 km or more must render, simulate and save without precision loss, and f32 positions lose it beyond 512 m from their origin. World positions are f64, right-handed, +Y up (gravity along −Y), in metres, with cameras looking down −Z: `Transform` holds translation as `DVec3` (relative to the parent for a child) with rotation (`Quat`) and scale (`Vec3`) in f32, 64 bytes like the `Affine3A` it replaces. The GPU never holds a World position in f32: it reads an integer anchor on a 64 m grid plus an f32 offset, and shaders compose positions relative to the camera. Accepted 2026-10-12. Implemented: `Transform` and the camera hold f64 positions, and prepare takes each point light relative to the camera in f64 before casting to f32 (#116). A child's Transform composes with its ancestors' into a World transform in f64, recomputed only when it or an ancestor moved (#120). Each Instance and the camera reach the GPU as an anchor on the 64 m grid plus an f32 offset, which the vertex shader composes camera-relative; a shader-function oracle holds the composition to the precision bar in every evaluation order within 100 km, and shows f32 World positions failing it (#119).

## Precision bar

Within 100 km of the origin: a point 0.3 m from the camera lands within 0.25 px at 1440p and a 45° vertical field of view (0.043 mm) of its exact camera-relative position; simulated positions resolve to 0.1 mm; saves round-trip positions bit-exactly.

## Considered options

- f32 positions fail the render bar beyond 512 m from the origin.
- A floating origin must shift whenever the camera strays 512 m from it, and each shift rewrites every position the engine holds (ECS, physics, GPU buffers, Lua state); entities simulated beyond 1024 m from the origin still fail.
- An integer cell plus an f32 position in the ECS meets the bar, but every system, script and save handles two-part positions and moves entities between cells.
- f64 rotation and scale buy nothing: the GPU applies rotation in f32 regardless.
- f64 on the GPU: Metal has no f64 shaders, and its default fast math may reorder the arithmetic that two-float emulation depends on.

## Consequences

- The Instance buffer (ADR 0003) and the per-Instance data (ADR 0005) store anchor plus offset, so camera motion dirties no Instance. The vertex shader composes camera-relative positions until draws are GPU-driven; then the culling pass does it once per Instance. Worst cases, measured separately, sum to 0.041 mm: the composition in every evaluation order the compiler may choose, plus f32 rotation of a vertex 128 m from its pivot. A 128 m grid sums to 0.045 mm and fails.
- Subsystems that compute in f32 (physics, audio, particles, ray-tracing acceleration structures) convert at their boundary into a local frame they own. The physics engine, not yet chosen, meets the simulation bar with f64 internals or f32 frames within 1 km of their origin.
- Lua receives positions as numbers, which are doubles. Saves store f64.
- A Chunk is a vertical column addressed by integer (x, z), with its origin at (x·edge, 0, z·edge). The cooked format picks the edge and encodes positions inside a Chunk so they meet the bar once promoted to f64: f32 heights alone fail the simulation bar above 1024 m, and cooked meshes stay within 128 m of their pivot.
- World-anchored shader effects (noise, wind, triplanar mapping) derive their periodic coordinates from the anchor.
- One Game world is loaded at a time, so a World position carries no Game world id.
