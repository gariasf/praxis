#!/usr/bin/env bash
# Usage: scripts/probe-899.sh <output directory> <rounds> <scene>...
set -euo pipefail

output=$1
rounds=$2
shift 2
cd "$(dirname "$0")/.."
mkdir -p "$output"
output=$(cd "$output" && pwd)

variants=(main checked no-bounds no-loop no-int-div unchecked)
reversed=()
for variant in "${variants[@]}"; do
  reversed=("$variant" ${reversed[@]+"${reversed[@]}"})
done

for round in $(seq 1 "$rounds"); do
  if ((round % 2)); then order=("${variants[@]}"); else order=("${reversed[@]}"); fi
  for variant in "${order[@]}"; do
    directory="$output/$variant/round-$round"
    mkdir -p "$directory"
    probe=$variant
    [[ $variant == main ]] && probe=
    echo "$(date +%T) round $round $variant"
    PRAXIS_PROBE_899=$probe target/release/praxis bench "$directory" \
      --skip-aliasing --no-publish "$@" >"$directory/log.txt" 2>&1 ||
      { echo "FAILED $variant"; tail -5 "$directory/log.txt"; }
  done
done
