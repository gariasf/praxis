#!/usr/bin/env bash
# The verify command: formatting, lint, build, tests (units, shader
# validation, layouts, oracles, Goldens) and the deterministic bench compare.
# Prints one line per step; a failing step prints the tail of its log and
# exits non-zero. Logs land in target/verify/.
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1
logs=target/verify
results="$logs/bench"
rm -rf "$logs"
mkdir -p "$results"

step() {
  local name="$1"
  shift
  local log="$logs/$name.log"
  local start=$SECONDS
  if "$@" >"$log" 2>&1; then
    printf 'ok    %-7s %4ss\n' "$name" $((SECONDS - start))
  else
    printf 'FAIL  %-7s %4ss  full log: %s\n' "$name" $((SECONDS - start)) "$log"
    tail -n 40 "$log"
    exit 1
  fi
}

# Counters against their Baselines and the Budgets: counters repeat exactly
# on every run, times do not, so each scene renders only the counter frames
# (`--counters`), which compare with a full run's Baseline. The scenes are
# the ones with a Baseline on any adapter, so a fetched helmet does not fail
# for lack of one; a scene without one for this adapter fails compare, and
# with no Baseline at all, every scene runs and compare fails for each.
bench() {
  local scenes=()
  local scene
  shopt -s nullglob
  for scene in $(printf '%s\n' bench/baselines/*/*.json | sed 's|.*/||; s|\.json$||' | sort -u); do
    scenes+=("$scene")
  done
  target/release/praxis bench "$results" --counters ${scenes[@]+"${scenes[@]}"} &&
    target/release/praxis compare "$results" --ci
}

step fmt cargo fmt --all -- --check
step clippy cargo clippy --all-targets -- -D warnings
step build cargo build --release
step test env PRAXIS_REQUIRE_GOLDENS=1 cargo test
step bench bench
printf 'ok    verify  %4ss\n' "$SECONDS"
