# Deviations from the literature

Where the code should not match its source, and why, so that an audit's
reader or a review files no Finding against code that is right on purpose.
One row per Deviation, each at a code site. The image half of the same
list is `scripts/cycles-reference/DIFFERENCES.md`: what a Cycles
difference image shows that is not a defect. The audit skill
(`docs/agents/audit.md`) hands both files to every reader.

A Deviation lives here and nowhere else: not in a code comment, where no
number may sit and nobody reads across the tree, and not in
`docs/references.toml`, where a source has many uses and a Deviation
belongs to one site. A reader who finds the code matching a row files no
Finding; one who finds a row wrong files a Finding against the row, not
the code. A row is added when the owner accepts, at a triage, a deviation
an audit found past its tolerance, or when a PR deviates on purpose; it
goes when its trigger fires and the code changes, or when the source it
deviates from is superseded.

| Where | What deviates | Source | Why | Revisit when |
|---|---|---|---|---|

- **Where**: `path:item`, the function or constant that deviates.
- **What deviates**: the source's value or form and the code's, in one
  sentence.
- **Source**: the key in `docs/references.toml` the code cites for it.
- **Why**: the reason the code is right to differ: the WGSL accuracy floor
  of its expression chain, a measured cost, an ADR's clause, a backend's
  behaviour.
- **Revisit when**: the trigger that reopens it: a milestone, an ADR, a
  dependency version, a measurement crossing a number.
