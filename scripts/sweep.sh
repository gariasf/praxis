#!/usr/bin/env bash
# The nightly sweep (ADR 0024): benches `main` on this machine when its
# source lacks a record on a Series here, the scenes it lacks, and only on a
# quiet machine. First it re-benches a Suspect that got slower beside the
# record before it, and bisects the commits between when the shift holds,
# each commit from a release binary built in a worktree of its own under
# target/sweep/trees, which a later night's bisect reuses. `praxis history
# bisect` keeps the Night's directory and names each run, and `history
# publish-night` publishes its runs; this script builds and runs the
# binaries, waits for a quiet machine and keeps to the time cap and the
# allowance for preparing. The records
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
# logged on, as the bench needs the GPU; a night the PC is off is skipped,
# not caught up in the day), in Git Bash:
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
# PRAXIS_SWEEP_CAP_SECONDS overrides the time cap,
# PRAXIS_SWEEP_PREPARATION_SECONDS the allowance for preparing, and
# WGPU_BACKEND the backend benched.
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

  # Runs start until this many seconds of benching, and of waiting for the
  # GPU and CPU between benches, have passed, the rest left to the next
  # night: a full pass of the Reference scenes and a bisect's re-benches on
  # the slower machine, the M4 Pro, whose sweep of 27d88d5c on 2026-11-11
  # benched all 11 scenes in 299 s, waits included, after 8 re-benches of
  # town in 495 s, 794 s in all.
  cap_seconds="${PRAXIS_SWEEP_CAP_SECONDS:-900}"
  # Preparing, which is building, cooking, and waiting for the machine to
  # settle after them, counts against this allowance instead: nights that
  # benched from 2026-11-02 to 2026-11-06 spent 17 to 77 s of it.
  preparation_limit_seconds="${PRAXIS_SWEEP_PREPARATION_SECONDS:-1800}"

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

  preparation_start_seconds=$SECONDS
  cargo build --release --quiet || fail "cargo build --release"
  praxis=target/release/praxis
  # A Baseline re-recorded since the last night becomes its source's record,
  # which starts a line, before the night reads which scenes lack one.
  "$praxis" history import || log "praxis history import failed"
  scenes=$("$praxis" history unrecorded) || fail "praxis history unrecorded"
  "$praxis" fetch && "$praxis" cook || fail "fetch and cook"
  log "built and cooked in $((SECONDS - preparation_start_seconds))s"

  # Preflight's warn lines, without their verdict. A FAIL line, such as a
  # scene without a Golden here, spoils no bench, and one that does, such as
  # no adapter, fails the bench itself; preflight then exits nonzero, which
  # under pipefail would read as a quiet machine below, so its output is
  # captured before it is read.
  warnings() {
    local checks
    checks=$("$praxis" preflight 2>/dev/null)
    sed -n 's/  */ /g; s/^warn //p' <<<"$checks"
  }
  # Waits until preflight warns of nothing the pattern $2 matches, logging
  # its warnings at each check; fails once SECONDS has reached $1.
  quiet_by() {
    local deadline_seconds=$1 pattern=$2 busy
    while busy=$(warnings | grep "^$pattern"); do
      log "busy: ${busy//$'\n'/; }"
      [ "$SECONDS" -lt "$deadline_seconds" ] || return 1
      sleep 5
    done
  }
  [ -n "$("$praxis" preflight)" ] || fail "praxis preflight printed nothing"
  # The build and cook above raise the one-minute load average, which falls
  # by a factor of e a minute, so a few minutes pass before it is under
  # preflight's threshold again; any other warning may clear meanwhile.
  if ! quiet_by $((preparation_start_seconds + preparation_limit_seconds)) ''; then
    log "skipped, the machine is not quiet; the busiest processes:"
    ps -A -o pcpu=,comm= 2>/dev/null | sort -rn | head -3
    exit 0
  fi
  preparation_seconds=$((SECONDS - preparation_start_seconds))

  # Each scene's bench reads the GPU load at its start, and a utilisation
  # reading lags the work it measures: nvidia-smi's covers the past 1/6 s
  # to 1 s [ref:nvidia-smi-utilization-sample-period], and on the RTX 2070
  # SUPER it still read 67% a third of a second after a scene's bench
  # exited. A bench, and a tree's build, also raise the one-minute load
  # average, which takes minutes to fall, as above. So each bench waits for
  # preflight to see the GPU idle and the CPU load under its threshold, for
  # as long as the time cap allows, or after a tree's build the allowance
  # for preparing. Whether the next run, $1, may start: within the time cap,
  # once the machine is idle; it logs why not.
  may_start() {
    [ $((SECONDS - benching_start_seconds)) -lt "$cap_seconds" ] &&
      quiet_by $((benching_start_seconds + cap_seconds)) '[cg]pu load' && return 0
    log "time cap of ${cap_seconds}s of benching reached; left for the next night: $1 and after"
    return 1
  }
  # Makes the worktree $1, of commit $2, with its release binary, the
  # fetched sources and its Cooked cache, kept for later nights. Every tree
  # builds into one target directory, so the dependencies build once; this
  # clone's fetched sources and Cooked cache are copied in, so only what
  # differs is fetched and cooked again.
  make_tree() {
    local tree=$1 build="$PWD/target/sweep/build" fetched
    rm -rf "$tree"
    git worktree prune
    git worktree add --quiet --detach "$tree" "$2" || return 1
    (cd "$tree" && CARGO_TARGET_DIR="$build" cargo build --release --quiet) || return 1
    # praxis.exe on Windows, which Git Bash runs as ./praxis; cargo's
    # dependency file and debug symbols are not needed.
    cp "$build/release/praxis"* "$tree/" 2>/dev/null
    rm -f "$tree/praxis.d" "$tree/praxis.pdb"
    git ls-files --others --ignored --exclude-standard --directory -- assets |
      while read -r fetched; do cp -R "$fetched" "$tree/$fetched"; done
    cp -R cooked "$tree/cooked" &&
      (cd "$tree" && ./praxis fetch && ./praxis cook) >/dev/null
  }

  benching_start_seconds=$SECONDS
  # Not mapfile: launchd's bash 3.2 lacks it [ref:gnu-bash-news-4.0-mapfile].
  night="$PWD/target/sweep/bisect-$(date '+%Y%m%dT%H%M%S')"
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
    {
      read -r commit
      read -r run
      read -r arguments
    } <<<"$job"
    tree="$PWD/target/sweep/trees/$commit"
    if [ ! -x "$tree/praxis" ]; then
      [ "$preparation_seconds" -lt "$preparation_limit_seconds" ] || {
        log "allowance of ${preparation_limit_seconds}s for preparing reached; left for the next night: the re-bench at $commit and after"
        break
      }
      tree_start_seconds=$SECONDS
      tree_problem=""
      if ! make_tree "$tree" "$commit"; then
        tree_problem="no release binary of $commit"
      elif ! quiet_by $((tree_start_seconds + preparation_limit_seconds - preparation_seconds)) '[cg]pu load'; then
        tree_problem="allowance of ${preparation_limit_seconds}s for preparing reached while the GPU or CPU was busy; left for the next night: the re-bench at $commit and after"
      fi
      preparation_seconds=$((preparation_seconds + SECONDS - tree_start_seconds))
      benching_start_seconds=$((benching_start_seconds + SECONDS - tree_start_seconds))
      [ -z "$tree_problem" ] || {
        log "$tree_problem"
        break
      }
    fi
    may_start "the re-bench at $commit" || break
    (cd "$tree" && ./praxis bench "$run" $arguments) || {
      log "re-bench at $commit failed"
      break
    }
  done
  "$praxis" history publish-night "$night" || log "praxis history publish-night failed"
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
  log "done in $((SECONDS - benching_start_seconds))s of benching and ${preparation_seconds}s of preparing"
  exit 0
}
