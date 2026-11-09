# Vendored crates

Copies of published crates that carry upstream fixes Praxis needs before
their release. Each is the crate exactly as crates.io serves it, plus the
upstream commits listed below, wired in through `[patch.crates-io]` in
`Cargo.toml`. Until wgpu 31 is published, a fix Praxis needs from wgpu's
trunk is carried here ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6749721020));
the wgpu 31 bump deletes each copy whose fixes 31 carries, with its
`[patch.crates-io]` entry.

## wgpu-core 30.0.1

- **Source:** crates.io `wgpu-core` 30.0.1, the `.crate` whose SHA-256,
  `14c018fce9b6270aa203c2fdd56f3cce996713534bd757e4ea58c8560b121f14`,
  `Cargo.lock` recorded before the patch; built from gfx-rs/wgpu at
  `40f4a34ebaf56f9a046231f54125ad046239d3f3` (`.cargo_vcs_info.json`).
- **Licence:** MIT OR Apache-2.0, `LICENSE.MIT` and `LICENSE.APACHE`
  beside it.
- **Applied:** gfx-rs/wgpu
  [`59209f276143212cc0aa3298a694d9c959a20e3b`](https://github.com/gfx-rs/wgpu/commit/59209f276143212cc0aa3298a694d9c959a20e3b),
  "Deduplicate TLAS build dependencies by unique BLAS" (gfx-rs/wgpu#9835,
  merged into trunk 2026-07-17), +12/−3 in `src/command/ray_tracing.rs`,
  applied unchanged with `git apply`. A TLAS build records one dependency
  per distinct BLAS rather than per Instance, so each submit's walk of a
  TLAS's dependencies and Metal's residency set for them shrink from the
  Instances to the distinct BLASes. Measured in `docs/performance.md`,
  "`world`'s CPU p99" (2026-11-09).

To check the copy against its source, extract the `.crate` from crates.io
and diff it against this directory: the only difference is the commit
above.
