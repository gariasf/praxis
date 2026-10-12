# `bevy_ecs` as a standalone crate

The ECS is `bevy_ecs` used on its own, not the Bevy engine, and the engine keeps its own `Transform` (a `glam::Affine3A` newtype) instead of `bevy_transform`. A hand-written sparse-set ECS was considered and rejected on 2026-04-28: rendering is the project's prize and the ECS is plumbing for it.
