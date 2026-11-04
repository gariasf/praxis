#!/usr/bin/env bash
# The nightly sweep (ADR 0024): benches `main` on this machine when its
# source lacks a record on a Series here, the scenes it lacks, and only on a
# quiet machine. First it re-benches a Suspect that got slower beside the
# record before it, and bisects the commits between when the shift holds,
# each commit from a release binary built in a worktree of its own under
# target/sweep/trees, which a later night's bisect reuses. The records
# publish to `bench-history`, and each night's Verdict lands there too, as
# does a Baseline re-recorded since the night before. A
# commit a bisect pinned that no re-recorded Baseline covers opens one
# `needs-triage` issue per scene through `gh`, which must be installed and
# logged in (`gh auth login`); the Verdict records it. It runs in a
# clone of its own, which it moves to `main` each night, and appends what
# it did to target/sweep/sweep.log there. On the Mac the clone sits outside
# ~/Documents, ~/Desktop and ~/Downloads, which macOS keeps from a launchd
# job's bash ("Operation not permitted"); a worktree of a checkout there
# reaches into it, so the sweep has its own clone on both machines.
#
# Install on the M4 Pro (launchd, 04:00 local, or on the next wake when
# asleep then [ref:apple-launchd-start-calendar-interval-sleep]):
#   git clone https://github.com/gariasf/praxis.git ~/praxis-sweep
#   sweep=$(cd ~/praxis-sweep && pwd)
#   mkdir -p "$sweep/target/sweep"
#   sed "s|@SWEEP@|$sweep|g" scripts/sweep.plist \
#     >~/Library/LaunchAgents/com.github.gariasf.praxis.sweep.plist
#   launchctl bootstrap "gui/$(id -u)" \
#     ~/Library/LaunchAgents/com.github.gariasf.praxis.sweep.plist
# Install on the RTX 2070 SUPER (Task Scheduler, 04:00 local, waking the
# PC if the power plan allows wake timers; it runs only while the owner is
# logged on, as the bench needs the GPU), in Git Bash:
#   git clone https://github.com/gariasf/praxis.git ~/praxis-sweep
#   sweep=$(cd ~/praxis-sweep && pwd -W)
#   sed "s|@SWEEP@|$sweep|g; s|@BASH@|$(cygpath -m "$(command -v bash)")|g" \
#     scripts/sweep-task.xml >"$TEMP/praxis-sweep.xml"
#   powershell -NoProfile -Command "Register-ScheduledTask -TaskName praxis-sweep \
#     -Xml (Get-Content -Raw '$(cygpath -w "$TEMP/praxis-sweep.xml")')"
# The command path is cygpath's mixed form: sed would read the backslashes
# of the Windows form as escapes, and Git Bash's cygpath already ends it
# in .exe. Run once now with
# `launchctl kickstart gui/$(id -u)/com.github.gariasf.praxis.sweep` or
# `powershell -NoProfile -Command "Start-ScheduledTask praxis-sweep"`;
# remove with `launchctl bootout` or `powershell -NoProfile -Command
# "Unregister-ScheduledTask praxis-sweep -Confirm:\$false"` (Git Bash
# turns schtasks's /Run into a path).
# PRAXIS_SWEEP_CAP_SECONDS overrides the time cap, and WGPU_BACKEND the
# backend benched.
#
# The body is one block, so bash has read all of it before the checkout
# below replaces this file.
{
  set -uo pipefail
  cd "$(dirname "$0")/.." || exit 1
  export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
  export RUST_LOG="${RUST_LOG:-praxis=warn,wgpu=warn,naga=warn}" NO_COLOR=1
  mkdir -p target/sweep
  exec >>target/sweep/sweep.log 2>&1

  # Runs start until this many seconds of building, benching, and waiting
  # for the machine between runs have passed, the rest left to the next night:
  # about twice a full pass of the Reference scenes on the slower machine
  # (219 s on the M4 Pro, 155 s on the RTX 2070 SUPER, both measured
  # 2026-10-31).
  cap_seconds="${PRAXIS_SWEEP_CAP_SECONDS:-450}"

  log() { printf '%s sweep: %s\n' "$(date '+%Y-%m-%dT%H:%M:%S')" "$*"; }
  fail() {
    log "failed: $*"
    exit 1
  }

  git fetch --quiet origin main || fail "git fetch"
  if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
    git checkout --quiet --detach --force origin/main || fail "git checkout"
    # A newer sweep.sh runs from here on.
    [ -z "${PRAXIS_SWEEP_UPDATED:-}" ] && PRAXIS_SWEEP_UPDATED=1 exec "$BASH" scripts/sweep.sh
  fi
  log "main at $(git rev-parse --short HEAD)"

  cargo build --release --quiet || fail "cargo build --release"
  praxis=target/release/praxis
  # A Baseline re-recorded since the last night becomes its source's record,
  # which starts a line, before the night reads which scenes lack one.
  "$praxis" history import || log "praxis history import failed"
  scenes=$("$praxis" history unrecorded) || fail "praxis history unrecorded"
  "$praxis" fetch && "$praxis" cook || fail "fetch and cook"

  # Whether preflight warns of what the pattern $1 matches. Its output is
  # read whole, as preflight exits nonzero on a FAIL line and grep would
  # stop reading it.
  preflight_warns() {
    local checks
    checks=$("$praxis" preflight 2>/dev/null)
    grep -q "^warn *$1" <<<"$checks"
  }
  # The build and cook above raise the one-minute load average, which falls
  # by a factor of e a minute, so a few minutes pass before it is under
  # preflight's threshold again.
  load_wait_start=$SECONDS
  while preflight_warns 'cpu load' && [ $((SECONDS - load_wait_start)) -lt 300 ]; do
    sleep 5
  done

  # A FAIL line, such as a scene without a Golden here, spoils no bench, and
  # one that does, such as no adapter, fails the bench itself.
  preflight=$("$praxis" preflight)
  [ -n "$preflight" ] || fail "praxis preflight printed nothing"
  warnings=$(printf '%s\n' "$preflight" | grep '^warn' | sed 's/  */ /g')
  if [ -n "$warnings" ]; then
    log "skipped, the machine is not quiet:"
    printf '%s\n' "$warnings"
    exit 0
  fi

  # Each scene's bench reads the GPU load at its start, and a utilisation
  # reading lags the work it measures: nvidia-smi's covers the past 1/6 s
  # to 1 s [ref:nvidia-smi-utilization-sample-period], and on the RTX 2070
  # SUPER it still read 67% a third of a second after a scene's bench
  # exited. A scene's bench also raises the one-minute load average. So
  # each scene waits for preflight to see the GPU idle and the CPU load
  # under its threshold, and either still busy after a minute ends the night.
  machine_busy() { preflight_warns '[cg]pu load'; }
  # Whether the next run, $1, may start: within the time cap, once the
  # machine is idle; it logs why not.
  may_start() {
    if [ $((SECONDS - start)) -ge "$cap_seconds" ]; then
      log "time cap of ${cap_seconds}s reached; left for the next night: $1 and after"
      return 1
    fi
    local wait_start=$SECONDS
    while machine_busy; do
      if [ $((SECONDS - wait_start)) -ge 60 ]; then
        log "skipped, the GPU or CPU is still busy after a minute; left for the next night: $1 and after"
        return 1
      fi
      sleep 1
    done
  }
  # A worktree of commit $1 under target/sweep/trees, with its release
  # binary, the fetched sources and its Cooked cache, made once and kept
  # for later nights: prints its path. Every tree builds into one target
  # directory, so the dependencies build once; this clone's fetched sources
  # and Cooked cache are copied in, so only what differs is fetched and
  # cooked again.
  tree_at() {
    local tree="$PWD/target/sweep/trees/$1" build="$PWD/target/sweep/build" fetched
    if [ ! -x "$tree/praxis" ]; then
      rm -rf "$tree"
      git worktree prune
      git worktree add --quiet --detach "$tree" "$1" || return 1
      (cd "$tree" && CARGO_TARGET_DIR="$build" cargo build --release --quiet) || return 1
      # praxis.exe on Windows, which Git Bash runs as ./praxis; cargo's
      # dependency file and debug symbols are not needed.
      cp "$build/release/praxis"* "$tree/" 2>/dev/null
      rm -f "$tree/praxis.d" "$tree/praxis.pdb"
      git ls-files --others --ignored --exclude-standard --directory -- assets |
        while read -r fetched; do cp -R "$fetched" "$tree/$fetched"; done
      cp -R cooked "$tree/cooked" &&
        (cd "$tree" && ./praxis fetch && ./praxis cook) >/dev/null || return 1
    fi
    printf '%s\n' "$tree"
  }

  # The commit that added `bench --no-publish`: a binary of a commit before
  # it refuses the flag, and publishes nothing without it.
  no_publish_added=3c6b9d92
  start=$SECONDS
  # One Suspect a night: its ends interleaved, then the bisect's commits,
  # as `history bisect` names them. Each run's directory names its commit
  # in a file, for a binary too old to stamp its results with provenance.
  night="$PWD/target/sweep/bisect-$(date '+%Y%m%dT%H%M%S')"
  runs=0
  bisect_done=0
  while true; do
    job=$("$praxis" history bisect "$night") || {
      log "praxis history bisect failed"
      break
    }
    if [ -z "$job" ]; then
      bisect_done=1
      break
    fi
    read -r scene commit <<<"$job"
    may_start "the re-bench of $scene at $commit" || break
    tree=$(tree_at "$commit") || {
      log "no release binary of $commit"
      break
    }
    runs=$((runs + 1))
    run="$night/$(printf '%03d' "$runs")"
    mkdir -p "$run" && printf '%s\n' "$commit" >"$run/commit"
    git merge-base --is-ancestor "$no_publish_added" "$commit"
    case $? in
      0) publish_flag=--no-publish ;;
      1)
        publish_flag=
        log "re-bench of $scene at $commit without --no-publish: its binary predates the flag and publishes nothing"
        ;;
      *)
        log "no ancestry of $commit known"
        break
        ;;
    esac
    (cd "$tree" && ./praxis bench "$run" $publish_flag "$scene") || {
      log "re-bench of $scene at $commit failed"
      break
    }
  done
  if compgen -G "$night/[0-9]*/*.json" >/dev/null; then
    log "re-benched $runs runs of a Suspect into $night"
    # A night's runs publish together, each a file of its own, but for an
    # old binary's, which carry no provenance to make a record of.
    mkdir -p "$night/all"
    for result in "$night"/[0-9]*/*.json; do
      if ! grep -q '"provenance"' "$result"; then
        log "not published, no provenance: $result"
        continue
      fi
      cp "$result" "$night/all/$(basename "$(dirname "$result")")-$(basename "$result")"
    done
    if compgen -G "$night/all/*.json" >/dev/null; then
      "$praxis" publish "$night/all" || log "praxis publish $night/all failed"
    fi
  fi
  # No bisect waits for a later night: its trees go.
  if [ "$bisect_done" = 1 ] && compgen -G "target/sweep/trees/*/" >/dev/null; then
    for tree in target/sweep/trees/*/; do git worktree remove --force "$tree"; done
    git worktree prune
  fi

  # Every night, so one whose `gh` failed is tried again; the issue's number
  # on the Verdict keeps a later night from opening it twice.
  "$praxis" history regressions || log "praxis history regressions failed"

  [ -z "$scenes" ] && log "main's source has a record on every Series here"
  results="target/sweep/$(date '+%Y%m%dT%H%M%S')"
  for scene in $scenes; do
    may_start "$scene" || break
    "$praxis" bench "$results" --no-publish "$scene" || log "bench of $scene failed"
  done
  if compgen -G "$results/*.json" >/dev/null; then
    "$praxis" publish "$results" || fail "praxis publish $results"
  fi
  log "done in $((SECONDS - start))s"
  exit 0
}
