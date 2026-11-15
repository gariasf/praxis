# Sessions on one machine

How sessions running side by side on the owner's machine keep out of each other's tickets, builds and timings. Read before the first build, bench or worktree removal while other sessions run; a session alone on the machine needs only *Timed runs*.

## Claims

- A ticket's state is on the tracker: its assignee holds it and its comments say what is left. A session keeps no status log anywhere else. A session that stops with work left (a PR waiting on the owner, an unmerged branch, a worktree kept) says so in a comment on the ticket: the branch, the worktree, what waits on whom, and the command that resumes it.
- An assigned ticket, its worktree and its running jobs belong to the session that claimed it. A fresh session, or one after `/clear`, does not adopt them from a note or a worktree it finds: it asks the sessions running whose the ticket is, and takes it only when its holder hands it over or the owner says so.
- A message or a comment names its session (`praxis-b4`), never "this session".

## Worktrees

- One worktree per ticket, `../praxis-<ticket>`, on its own branch; the main checkout stays on `main`. A command reaches another worktree by absolute path or `git -C`: a `cd` inside a command moves the session's working directory.
- Each worktree has its own `target/`, never a link to another's: a test binary holds its worktree's path, so a shared one runs another session's code. A finished ticket's built `target/` goes to the next worktree by `mv`; an APFS clone (`cp -c -R`) is the fallback.
- A worktree's `cooked/` is an APFS clone of the main checkout's (`cp -c -R ../praxis/cooked cooked`), not a link: a link shares one cache across Blob formats, and `git add -A` commits it.
- After the merge the session removes its worktree and branch. Cloning or deleting a `target/` or a `cooked/` sets the machine's indexer and its virus scanner working for minutes, so neither happens during a quiet turn.

## Quiet turns

A timed run (a bench whose figures are quoted or recorded as Baselines, an A/B, a traversal) has the machine to itself (`workflow.md`, PRs). Sessions take turns by message.

- Before its first build, and before any timed run, a session asks every other `praxis-*` session whether a quiet turn is held, and waits for the answers. A session that was cleared no longer knows what it held or granted.
- To take a turn: send `<ticket> quiet` with its expected length to every `praxis-*` session, wait until each has answered, run, then send `<ticket> done`. Turns go in the order they were asked for.
- While another session's turn is held: no build, test, `scripts/verify.sh`, bench, gallery or capture, and no clone or removal of a `target/` or `cooked/`. Edits, reading, review sub-agents told to run no cargo, and waits on CI go on. A short untimed slot between two turns is asked for by name (`<ticket> verify`) and closed the same way (`<ticket> verify done`).
- A turn ends with its `done` message, not when a process exits or a session goes idle.
- The design and the review are settled before the turn: a change under `src/` after a Baseline run needs another one.

## Timed runs

- `cargo run --release -- preflight` comes first; a `warn` line holds off a bench (`workflow.md`, Session start). It reads the machine once, so right before each run also check:
  - mains power: `pmset -g batt` reads `AC Power`. On battery the GPU slows run after run. After a run, `pmset -g log | grep -E "Using (AC|Batt)|Sleep"` over its window; a round that overlaps a switch or a sleep is discarded.
  - no indexing or scan: `ps -Ao pcpu,comm -r | head` shows neither `mds_stores` nor `BDLDaemon` near a core. Either inflates CPU medians while GPU medians hold, so the binary is built and the worktree made well before the run.
- An A/B interleaves its variants over at least three rounds, alternating which runs first, from binaries built before the turn.
- A script never runs `praxis` with no arguments: that opens the window and renders until closed. `praxis help` prints the usage; `praxis bench --help` benches every scene into a directory named `--help`.
