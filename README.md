# Bench history

Bench records published by `praxis publish` and `praxis bench` (ADR 0024).
This branch holds data only and is never merged.

- `records/<adapter>-<backend>/<scene>/<source hash>.json`: the records on a
  Series, clean and release-profile runs of `main`'s source on a quiet machine.
- `tagged/<adapter>-<backend>/<scene>/<source hash>.json`: every other record,
  its `tags` saying why it is kept off the Series.
- `records/<adapter>-<backend>/release-build/<source hash>.json`: the release
  build whose binary ran that source's results, its binary's size and its wall
  time, published by CI beside lavapipe's records.
- `series/<adapter>-<backend>/<scene>.json`: one row per record in `records/`,
  in the order their commits were made, rebuilt in the commit that changes them.

lavapipe's Series read no GPU time: its counters and its binary's size, which
load cannot change, and its build time, which a shared runner's load and cache
hits move whatever a quiet test says. So its records join them from CI's
configuration whether or not the runner was quiet.

A record in `records/` holds its key against a tagged one. Otherwise a record
whose path exists replaces it only when its GPU frame p95 over its median is
lower, and a lavapipe or release build record never does: a source publishes
there once. A bad publish is undone with `git revert`.

`praxis history import` added the Baselines committed to `main` before the
history existed: each Baseline file a commit changed is a record of that
commit's source, its `provenance.source` `baseline-commit`. It holds what the
bench measured then, so a metric added later is absent from it and null in
its Series row.
