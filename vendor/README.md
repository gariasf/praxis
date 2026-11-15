# Vendored crates

Copies of published crates that carry upstream fixes Praxis needs before
their release. Each is the crate exactly as crates.io serves it, plus the
upstream commits listed below, wired in through `[patch.crates-io]` in
`Cargo.toml`. Until wgpu 31 is published, a fix Praxis needs from wgpu's
trunk is carried here ([decision](https://github.com/gariasf/praxis/issues/17#issuecomment-6749721020));
the wgpu 31 bump deletes each copy whose fixes 31 carries, with its
`[patch.crates-io]` entry, and rewrites what names the copy: CLAUDE.md's
Stack and tree, and ADR 0020's static TLAS clause.

## wgpu-core 30.0.2

- **Source:** crates.io `wgpu-core` 30.0.2, the `.crate` whose SHA-256 in
  the crates.io index is
  `ad661da2ee99f208c30090b585284b2ab5b8bf2358ddf2d2215592808ec62f1a`. It
  carries no `.cargo_vcs_info.json`; its `src/`, `build.rs` and
  `Cargo.toml.orig` equal gfx-rs/wgpu's `wgpu-core` at the `v30.0.2` tag,
  `2b38d994b38dd001157d31fd3321111103a8dd80`.
- **Licence:** MIT OR Apache-2.0, `LICENSE.MIT` and `LICENSE.APACHE`
  beside it.
- **Applied:** gfx-rs/wgpu
  [`59209f276143212cc0aa3298a694d9c959a20e3b`](https://github.com/gfx-rs/wgpu/commit/59209f276143212cc0aa3298a694d9c959a20e3b),
  "Deduplicate TLAS build dependencies by unique BLAS" (gfx-rs/wgpu#9835,
  merged into trunk 2026-07-17), +12/−3 in `src/command/ray_tracing.rs`,
  applied unchanged: 30.0.2 leaves that file as 30.0.1 had it. A TLAS build records one dependency
  per distinct BLAS rather than per Instance, so each submit's walk of a
  TLAS's dependencies and Metal's residency set for them shrink from the
  Instances to the distinct BLASes. Measured in `docs/performance.md`,
  "`world`'s CPU p99" (2026-11-09).

To check the copy against its source, extract the `.crate` from crates.io
and diff it against this directory: the only difference is the commit
above.
