# texture-wall kernel scheduling probes

Measurement patches behind `docs/performance.md`'s note of 2026-11-04 on
texture-wall's untimed stretch. Not for merging.

- `src/`: `PRAXIS_PROBE_DUMP=<csv>` writes every frame's raw timestamps,
  in ns from culling phase one's start; `PRAXIS_PROBE_DISTINCT_LIT=<n>` and
  `PRAXIS_PROBE_DISTINCT_DEPTH=<n>` make the views past `n` in each
  Channel repeat its Layer 0, in the lit pass's and the depth passes' bind
  groups; `PRAXIS_PROBE_KEEP_SAMPLED=1` keeps the Layers Materials sample.
- `wgpu-hal-command-buffer-times.patch`, against wgpu-hal 30.0.1's
  `src/metal/mod.rs`, used through `[patch.crates-io]`:
  `PRAXIS_PROBE_ALL_COMMAND_BUFFERS=<csv>` writes each Metal command
  buffer's index in its submission, their count, its commit time,
  `kernelStartTime`, `kernelEndTime`, `GPUStartTime` and `GPUEndTime`.
- `run-variant.sh <name> <depth n> <lit n> <round>` runs one variant;
  `timeline.py`, `per_buffer.py`, `alignment.py` and `correlate.py` read
  the CSVs.
