#!/usr/bin/env bash
# The verify command: formatting, lint, build, tests (units, shader
# validation, layouts, oracles, Goldens) and the deterministic Budgets. Prints
# one line per step; a failing step prints the tail of its log and exits
# non-zero. Logs land in target/verify/.
set -uo pipefail

cd "$(dirname "$0")/.."
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

# Counters only: they repeat exactly on every run, times do not. The scenes
# are the ones with a Baseline, so a fetched helmet does not fail for lack of
# one.
budgets() {
  local scenes=()
  local baseline
  for baseline in bench/baselines/*.json; do
    scenes+=("$(basename "$baseline" .json)")
  done
  target/release/praxis bench "$results" "${scenes[@]}" &&
    target/release/praxis compare "$results" --ci
}

step fmt cargo fmt --all -- --check
step clippy cargo clippy --all-targets -- -D warnings
step build cargo build --release
step test env PRAXIS_REQUIRE_GOLDENS=1 cargo test
step budgets budgets
printf 'ok    verify  %4ss\n' "$SECONDS"
