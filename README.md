# Bench history

Bench records published by `praxis publish` and `praxis bench` (ADR 0024).
This branch holds data only and is never merged.

- `records/<adapter>-<backend>/<scene>/<source hash>.json`: the records on a
  Series, clean and release-profile runs of `main`'s source on a quiet machine.
- `tagged/<adapter>-<backend>/<scene>/<source hash>.json`: every other record,
  its `tags` saying why it is kept off the Series.
- `series/<adapter>-<backend>/<scene>.json`: one row per record in `records/`,
  in the order their commits were made, rebuilt in the commit that changes them.

A record in `records/` holds its key against a tagged one. Otherwise a record
whose path exists replaces it only when its GPU frame p95 over its median is
lower. A bad publish is undone with `git revert`.
