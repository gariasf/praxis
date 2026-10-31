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
# Aliasing renders each scene twice more, once at three times the Goldens'
# size, so it is measured only when the diff against main can change what
# renders: the renderer and its shaders, the cooker or the assets.
skip_aliasing=--skip-aliasing
base=$(git merge-base HEAD origin/main 2>/dev/null || git merge-base HEAD main 2>/dev/null)
if [ -z "$base" ] ||
  [ -n "$(git diff --name-only "$base" -- src/render src/cook assets)" ] ||
  [ -n "$(git ls-files --others --exclude-standard -- src/render src/cook assets)" ]; then
  skip_aliasing=
fi

bench() {
  local scenes=()
  local scene
  shopt -s nullglob
  for scene in $(printf '%s\n' bench/baselines/*/*.json | sed 's|.*/||; s|\.json$||' | sort -u); do
    scenes+=("$scene")
  done
  target/release/praxis bench "$results" --counters --no-publish $skip_aliasing ${scenes[@]+"${scenes[@]}"} &&
    target/release/praxis compare "$results" --ci
}

# The Goldens test takes about half the step, so it runs beside every other
# test once both are built; its log is printed when it fails.
tests() {
  local goldens=reference_scenes_match_their_goldens
  cargo test --no-run || return
  PRAXIS_REQUIRE_GOLDENS=1 cargo test --test goldens "$goldens" >"$logs/goldens.log" 2>&1 &
  local goldens_job=$!
  cargo test -- --skip "$goldens"
  local others=$?
  wait "$goldens_job" || {
    cat "$logs/goldens.log"
    return 1
  }
  return "$others"
}

step fmt cargo fmt --all -- --check
step clippy cargo clippy --all-targets -- -D warnings
step build cargo build --release
step test tests
step bench bench
[ -z "$skip_aliasing" ] || printf 'ok    bench   aliasing skipped: no change under src/render, src/cook or assets\n'
printf 'ok    verify  %4ss\n' "$SECONDS"
