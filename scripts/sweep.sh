#!/usr/bin/env bash
# The nightly sweep (ADR 0024): benches `main` on this machine when its
# source lacks a record on a Series here, the scenes it lacks, and only on a
# quiet machine. The records publish to `bench-history`. It runs in a
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
# remove with `launchctl bootout` or `Unregister-ScheduledTask
# praxis-sweep` (Git Bash turns schtasks's /Run into a path).
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

  # Scenes start until this many seconds of benching have passed, the rest
  # left to the next night: about twice a full pass of the Reference scenes
  # on the slower machine (219 s on the M4 Pro, 155 s on the RTX 2070
  # SUPER, both measured 2026-10-31).
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
  scenes=$("$praxis" history unrecorded) || fail "praxis history unrecorded"
  if [ -z "$scenes" ]; then
    log "nothing to bench: main's source has a record on every Series here"
    exit 0
  fi
  "$praxis" fetch && "$praxis" cook || fail "fetch and cook"

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
  # reading lags the work it measures: right after the scene before, it
  # still counts that scene's frames. So each scene waits for preflight to
  # see the GPU idle, for up to a minute, after which its records say it
  # was busy.
  gpu_busy() { "$praxis" preflight | grep -q '^warn *gpu load'; }

  results="target/sweep/$(date '+%Y%m%dT%H%M%S')"
  start=$SECONDS
  for scene in $scenes; do
    if [ $((SECONDS - start)) -ge "$cap_seconds" ]; then
      log "time cap of ${cap_seconds}s reached; left for the next night: $scene and after"
      break
    fi
    waited=$SECONDS
    while gpu_busy; do
      if [ $((SECONDS - waited)) -ge 60 ]; then
        log "the GPU is still busy after a minute; benching $scene anyway"
        break
      fi
      sleep 1
    done
    "$praxis" bench "$results" --no-publish "$scene" || log "bench of $scene failed"
  done
  if compgen -G "$results/*.json" >/dev/null; then
    "$praxis" publish "$results" || fail "praxis publish $results"
  fi
  log "done in $((SECONDS - start))s"
  exit 0
}
