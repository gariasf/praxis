# wgpu is the permanent graphics API

The engine stays on wgpu (Vulkan, DX12, Metal) for its lifetime, using `wgpu-hal` only for a subsystem that exceeds wgpu's safe API; migration to raw Vulkan through `ash` is not planned and must not influence design. wgpu's experimental inline ray queries [ref:wgpu-types-30-ray-query-experimental] cover the planned RT features, and one portable API outweighs the last margin of control. Consoles are out of scope (0010).

## Consequences

A reopen trigger and an exit, added 2026-11-14 ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6878038353), from queue item 4 of `docs/audit/strategy/2026-11.md`). The choice of wgpu's API, WGSL and its three backends stays; what can reopen is whether the engine keeps following wgpu's releases.

- Each milestone exit's dependency review (`docs/agents/workflow.md`, The ladder) checks three things, and this decision reopens when any holds. The newest wgpu release cannot meet the Capability floor (ADR 0021) on Metal, Vulkan or DX12, as when one drops or reshapes ray queries, which wgpu calls experimental and "expected to be subject to breaking changes" [ref:wgpu-types-30-ray-query-experimental]. `vendor/` carries more than five upstream commits, which `vendor/README.md` lists, or a fix upstream has declined. Or the wgpu that `Cargo.toml` pins is more than two major releases behind the newest published.
- The exit is an owned fork of wgpu's crates, built from `vendor/`, where on 2026-11-14 a copy of wgpu-core 30.0.1 carries one upstream fix: the engine keeps wgpu's API and WGSL, takes upstream's changes by choice and carries what upstream lacks. Raw Vulkan through `ash` stays unplanned.
