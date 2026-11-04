# Bench history

Bench records published by `praxis publish` and `praxis bench`, and the
Baselines `praxis history import` added from `main`'s history (ADR 0024).
This branch holds data only and is never merged.

- `records/<adapter>-<backend>/<scene>/<source hash>.json`: the records on a
  Series, clean and release-profile runs of `main`'s source in their Series'
  configuration: 2560x1440 over 1800 frames on a quiet machine, or on lavapipe
  CI's 640x360 over 64 frames, quiet or not.
- `tagged/<adapter>-<backend>/<scene>/<source hash>.json`: every other record,
  its `tags` saying why it is kept off the Series.
- `records/<adapter>-<backend>/release-build/<source hash>.json`: the release
  build whose binary ran that source's results, its binary's size and its wall
  time, published by CI beside lavapipe's records.
- `series/<adapter>-<backend>/<scene>.json`: one row per record in `records/`,
  in the order their commits were made, every one rebuilt by each publish. A
  row holds the counter frames' counters by name and the measured frames'
  under `measured_counters`.
- `experiments/<name>/<adapter>-<backend>/<scene>/<source hash>.json`: an
  Experiment's records, published with `--experiment <name>` from any branch,
  on no Series. Each names its `experiment.merge_base` with `main` and that
  commit's source hash, whose record on the Series is what it compares with.

A machine is quiet when preflight warned of no GPU load over 10%, one-minute
load average over 2, other praxis process, battery power or busy display at
the run's start, and the run's GPU frame p95 over its median stayed
under 1.06.

lavapipe's records hold CPU, GPU and pass times like any other, but lavapipe
renders on a shared runner's CPU, so the page draws only its counters and its
binary's size, which load cannot change, and its build time, which a shared
runner's load and cache hits move whatever a quiet test says. So its records
join its Series from CI's configuration whether or not the runner was quiet.

A Series row's `suspects` names each frame median, CPU or GPU, that makes its
record a suspect, with its shift as a share of the median of up to 8 records
before it on its line: it lies outside that median ± the larger of 4 median
absolute deviations and 3%, unless it only continues a suspect among those 8:
on the same side, within that suspect's tolerance of its median. A new Harness
version or scene content, an imported or re-recorded Baseline, or a commit a
Verdict pinned starts a line. lavapipe's records are never suspects.

- `verdicts/<adapter>-<backend>/<scene>/<source hash>.json`: what the sweep
  found of the slower suspect of that source: the night re-benched it and the
  record before it on its line, interleaved, from release binaries of their
  commits, and the shift `held` when their mean medians still differed in its
  direction by half its marked shift and by 3%, else `cleared`. A held shift
  is bisected along `main`'s first-parent commits of other source hashes
  between `good` and `bad`, each commit's median judged by which end's median
  of that night it lies nearer, over as many nights as it takes, until
  `pinned` names the commit that changed it, whether that commit changed the
  scene's Baseline for the adapter (`covered_by_baseline`), and the mean
  medians of that night's ends (`measured`). A pinned commit no Baseline
  covers is a Regression: it opens one `needs-triage` issue per scene, whose
  number `issue` records. A suspect from the record after a Verdict's
  predecessor to its suspect has that Verdict.

A record in `records/` holds its key against a tagged one. Otherwise a record
whose path exists replaces it only when its GPU frame p95 over its median is
lower, and a lavapipe or release build record never does: a source publishes
there once. A bad publish is undone with `git revert`.

`praxis history import` added the Baselines committed to `main` before the
history existed, each a record of its commit's source, its `provenance.source`
`baseline-commit`. A Baseline file a commit on `main`'s first-parent line
added or changed is a record, unless the commit only renamed it or gave it new
fields without changing a value it held, which measured nothing again. Two
Baselines of one source meet the rules above with the later first: it stays
unless the earlier goes on a Series and it does not, or the earlier's GPU frame
spread is lower and neither is lavapipe's. A Baseline counts as quiet unless the
GPU frame spread it recorded reached 1.06: one that recorded none counts as
quiet unchecked. Each import rewrites the records an earlier one wrote, with
the tags the rules of its day give them. A record holds what the bench
measured then, so a metric added later is absent from it: its Series row
holds null for a time, Aliasing figure or scene content hash it lacks, and
leaves out a counter or pass it lacks. A Baseline `main` re-recorded since
is a record the same way, its `provenance.source` `baseline-rerecorded` and its
Harness version the one its commit defined; the sweep imports each night.
Its mark stays with its key: a quieter record of its source that holds the key
in its place carries `rerecorded_baseline`, and starts the line in its place.
