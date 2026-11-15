#!/usr/bin/env bash
# Interleaved rounds of the shadow mask module's runtime-check variants.
# Usage: scripts/probe-873.sh <output directory> <rounds> <scene>...
# Each variant benches into <output directory>/<variant>/round-<n>; odd
# rounds run the variants in order, even rounds in reverse.
set -euo pipefail

output=$1
rounds=$2
shift 2
cd "$(dirname "$0")/.."
mkdir -p "$output"
output=$(cd "$output" && pwd)

variants=(main checked no-bounds no-loop no-ray no-int-div unchecked)
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
    PRAXIS_PROBE_873=$probe target/release/praxis bench "$directory" \
      --skip-aliasing --no-publish "$@" >"$directory/log.txt" 2>&1 ||
      { echo "FAILED $variant"; tail -5 "$directory/log.txt"; }
  done
done
