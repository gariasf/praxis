# Praxis Roadmap

The long-range shape of the engine. Decisions live in `docs/adr/`, the
order of work is the milestone ladder (`docs/agents/workflow.md`, The
ladder), and topics not yet charted are fog on the map issue.

**Target shape:** a big-world, lore-heavy RPG (Skyrim-style) targeting
**next-generation hardware** the owner does not own yet, which the reference
machine stands in for (ADR 0013). Image quality and rendering correctness
come first. Current-generation hardware may be skipped for now; lower
settings for lesser devices come later, and a Steam Deck is the lowest bar
anyone would consider. Until then the Capability floor is the only floor,
and nothing in it has a fallback (ADR 0021). This anchors which
rendering features are core: PBR, ray-traced lighting, GPU-driven rendering,
streaming, LOD, scene hierarchy, long view distances. "Polish" is reserved
for genuinely optional things (HDR10 metadata, color grading) — not RT, not
bindless, not GI.

**Lighting model:** ray-traced irradiance probes and hard ray-traced shadows
(ADR 0020). Clustered forward rendering with 4x MSAA (ADR 0019) — no
mandatory TAA, no upscaling stack, no frame generation (ADR 0008). Native
resolution is the default.

**Graphics API:** wgpu, long-term. wgpu's experimental ray tracing (inline
ray queries) is sufficient for the planned RT features — no full ray
pipelines required. If a specific subsystem ever exceeds wgpu's safe API,
drop into `wgpu-hal` for that subsystem only. Migrating to raw Vulkan via
`ash` is not on the roadmap and should not influence design decisions.

The engine may grow to cover persistence, scripting, and AI later; the
foundation is rendering.

---

## Design Principles

How new systems are built.

1. **Handle-based resources.** Meshes and Materials, with their Textures,
   live in central Pools and are referenced by `u32` or generational Handles.
   ECS components carry Handles, never owned GPU buffers. Lets the renderer
   batch, instance and go bindless (ADR 0017) without rewriting components.

2. **Authoring data in components, not GPU byte layout.** `Transform` is an
   f64 translation with f32 rotation and scale (ADR 0014), not
   `[[f32; 4]; 4]`. The renderer packs GPU bytes on upload. Keeps ECS
   readable and independent of wgpu specifics.

3. **Renderer pulls, entities don't push.** No `entity.draw()`. The renderer
   side queries the ECS for renderables (prepare, into Frame data: ADR 0004),
   sorts and batches, then issues draws.
   Prerequisite for culling, instancing, and (eventually) GPU-driven rendering.

4. **Few pipelines; a feature's form follows its measured cost.** An effect
   joins an existing pass where it can, as a uniform or an input, and a
   variant only some runs need is a Render switch, chosen when the Renderer
   is created. A branch in a hot shader is not free: on the M4 Pro an `if`
   around one light's term cost the main pass 10 to 12%, and shadow rays
   cost less in a compute pass of their own than inline in the lit pass on
   eight of nine scenes (`docs/performance.md`). Where a branch, a switch or
   a pass of its own would each do, an A/B decides.

Inspired by modern AAA renderers (Spartan Engine, id Tech).

---

## Scripting plan

The engine is in Rust; gameplay logic — quests, dialogue, NPC behaviors,
encounter scripts, item effects, UI flow, content tuning — lives in
Lua. Same boundary Supergiant uses for Hades: a thin engine API, with
the content shape and reactive logic above it. Designers and modders
iterate on game content without recompiling.

**Choice: Lua via [mlua](https://github.com/mlua-rs/mlua/tree/v0.12.2) [ref:mlua-0.12.2-luajit], with the
LuaJIT backend on supported platforms.** Lua has a long track record in
shipped games as the gameplay/content layer above a native engine —
Hades and Hades II (Supergiant), World of Warcraft (UI and addons),
Roblox (the Luau dialect), Factorio, the Civilization series, the
classic Crytek and Far Cry titles. mlua is the most actively
maintained Rust binding and supports both PUC-Rio Lua and LuaJIT.
Rhai was considered (cleaner Rust integration, type-safe) but loses
on ecosystem size, designer familiarity, and runtime speed. WASM was
considered and rejected as overkill for gameplay scripting.

**Boundary shape:**

- **Engine in Rust:** ECS storage, simulation, rendering, physics
  (later), audio (later), asset loading. Per-frame hot paths.
- **Lua on top:** content data (entities, scenes, dialogue, items),
  reactive logic (event handlers), sequenced gameplay (coroutines for
  encounters, cutscenes, timed effects).
- **Lua does not author per-frame systems.** It runs in response to
  events the engine fires. Per-frame logic stays in Rust. This rule is
  what makes the boundary scale.

**Required engine subsystems** (built in M8, Gameplay core, on the
milestone ladder):

- **Event bus.** Engine systems fire events; Lua handlers subscribe
  and react. pcall-protected dispatch; tracing-integrated error
  logging so a broken script never crashes the engine.
- **Coroutine task scheduler.** Lua coroutines yield on engine
  timers, events and predicates; engine resumes them when
  conditions are satisfied. Frame-budget aware so a Lua loop can't
  starve the renderer.
- **Hot-reload.** File watcher reloads changed `.lua` modules; Lua VM
  is recreated; engine state (ECS components, GPU resources, scene)
  survives. All persistent Lua state lives in well-defined tables that
  serialize cleanly — engine never tries to snapshot arbitrary VM
  state.
- **Component bindings.** Core components (Transform, Health, etc.)
  get typed Rust → Lua accessors. A `LuaComponent` Rust type wraps an
  opaque Lua table for game-content components defined in Lua.
- **Lua REPL inside the debug UI** for live inspection.

**Hot-reload contract — the load-bearing rule:** components stay pure
data (ADR 0006). No closures, no behavior methods on
components. Behavior is always in systems or Lua handlers, never
captured inside component storage. Without this discipline, hot-reloading
a Lua script leaves dangling references inside ECS components and the
engine crashes.

**Save/load discipline:** all persistent Lua state is in well-defined
tables with stable shapes. Save files serialize the known set; the
engine never tries to snapshot the whole VM. Same approach Hades uses.
