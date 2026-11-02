#!/usr/bin/env bash
# The capture gallery for the visual audit: every Reference scene at
# 2560x1440 at the Goldens' frame, `town` at the eight frames the Goldens
# test checks and `crowd` in both its shadow forms, written to
# target/tmp/gallery/<scene>-frame-<n>.png. Given an earlier gallery's
# directory, it also writes <scene>-frame-<n>-difference.png beside each
# image that directory holds too: the capture dimmed, with every pixel past
# the Goldens' tolerance in red. Prints one line per image; a failing
# capture prints the tail of its log, under target/tmp/gallery/logs/.
#
# With PRAXIS_BLENDER naming a Blender 5.2 executable, it first checks the
# Cycles reference's light units and display transform, then writes
# <scene>-frame-<n>-cycles.png beside each image of a scene that
# `praxis export-scene` exports, rendered by scripts/cycles-reference/, and
# <scene>-frame-<n>-cycles-difference.png: the capture dimmed, red where
# praxis is brighter than Cycles and blue where it is darker, by the
# channel that differs most. Each Cycles line carries the two figures of
# the capture's distance from its render, the mean 8-bit difference and the
# linear-light RMSE, and a scene with a Cycles Baseline
# (bench/cycles/<scene>.json) is judged against it at the Goldens' frame:
# a figure past its Baseline by compare.py's tolerance prints a FAIL line
# and the gallery goes on, exiting 1 at the end. With
# PRAXIS_RECORD_CYCLES_BASELINES set, it records each exported scene's
# Baseline from this run instead.
#
# Usage: [PRAXIS_BLENDER=<blender>] [PRAXIS_RECORD_CYCLES_BASELINES=1]
#        scripts/gallery.sh [<earlier gallery directory>]
set -euo pipefail

against="${1:-}"
if [[ -n "$against" ]]; then
  [[ -d "$against" ]] || { echo "no gallery at $against" >&2; exit 1; }
  against="$(cd "$against" && pwd)"
fi
cd "$(dirname "$0")/.." || exit 1
gallery=target/tmp/gallery
logs="$gallery/logs"
size=2560x1440
baseline_failures=0
# The Goldens test's GOLDEN_FRAME and TOWN_FRAMES (tests/goldens.rs).
golden_frame=59
town_frames=(59 300 500 700 900 1200 1500 1800)

[[ -z "$against" || "$against/" != "$(pwd)/$gallery/"* ]] ||
  { echo "copy the earlier gallery out of $gallery first" >&2; exit 1; }
[[ -z "${PRAXIS_RECORD_CYCLES_BASELINES:-}" || -n "${PRAXIS_BLENDER:-}" ]] ||
  { echo "PRAXIS_RECORD_CYCLES_BASELINES needs PRAXIS_BLENDER: Cycles Baselines come from its renders" >&2; exit 1; }
cargo build --release --quiet
rm -rf "$gallery"
mkdir -p "$logs"

# run_blender <log> <script> <argument>...: a Cycles reference script, its
# output appended to <log>; blender is the same, exiting on a failure after
# printing the log's tail.
run_blender() {
  local log="$1" script="$2"
  shift 2
  "$PRAXIS_BLENDER" -b --factory-startup --python-exit-code 1 \
    --python "scripts/cycles-reference/$script" -- "$@" >>"$log" 2>&1
}
blender() {
  if ! run_blender "$@"; then
    printf 'FAIL  %s  full log: %s\n' "$2" "$1"
    tail -n 20 "$1"
    exit 1
  fi
}

if [[ -n "${PRAXIS_BLENDER:-}" ]]; then
  blender "$logs/calibrate.log" calibrate.py
  # The check itself: point lights built in the wrong unit must fail it.
  if "$PRAXIS_BLENDER" -b --factory-startup --python-exit-code 1 --python     scripts/cycles-reference/calibrate.py -- --wrong-unit >>"$logs/calibrate.log" 2>&1; then
    printf 'FAIL  calibrate.py  passed point lights in the wrong unit: %s
' "$logs/calibrate.log"
    exit 1
  fi
  blender "$logs/calibrate.log" compare.py ramp
  printf 'ok    %-34s light units and display transform\n' cycles-reference
fi

# reference <image name> <scene> <frame>: the Cycles render of a capture,
# and its difference from it.
reference() {
  local scene="$2" frame="$3" name="$1-frame-$3"
  local export="$gallery/cycles/$name" log="$logs/$name-cycles.log"
  if ! target/release/praxis export-scene "$scene" "$export" --frame "$frame" >"$log" 2>&1; then
    printf 'FAIL  %s  full log: %s\n' "$name-cycles.png" "$log"
    tail -n 20 "$log"
    exit 1
  fi
  blender "$log" render.py "$export" "$export/cycles.exr" "$size"
  # The scene's Cycles Baseline, judged or recorded at the Goldens' frame.
  local baseline_path="bench/cycles/$scene.json" baseline=()
  if [[ "$frame" == "$golden_frame" ]]; then
    if [[ -n "${PRAXIS_RECORD_CYCLES_BASELINES:-}" ]]; then
      baseline=(--baseline "$scene" "$frame" "$baseline_path" --record)
    elif [[ -f "$baseline_path" ]]; then
      baseline=(--baseline "$scene" "$frame" "$baseline_path")
    fi
  fi
  if ! run_blender "$log" compare.py compare "$gallery/$name.png" "$export/cycles.exr" \
    "$gallery/$name-cycles.png" "$gallery/$name-cycles-difference.png" \
    ${baseline[@]+"${baseline[@]}"}; then
    if grep -q '^FAIL  .*Baseline' "$log"; then
      printf 'FAIL  %-34s %s\n' "$name-cycles.png" "$(grep '^FAIL  .*Baseline' "$log" | sed 's/^FAIL *//')"
      baseline_failures=$((baseline_failures + 1))
      return
    fi
    printf 'FAIL  compare.py  full log: %s\n' "$log"
    tail -n 20 "$log"
    exit 1
  fi
  printf 'ok    %-34s %s\n' "$name-cycles.png" "$(grep '^ok ' "$log" | tail -n 1 | sed 's/^ok *//')"
}

# capture <image name> <scene> <frame> [<capture flag>...]
capture() {
  local name="$1" scene="$2" frame="$3"
  shift 3
  local image="$name-frame-$frame.png"
  local log="$logs/$name-frame-$frame.log"
  if ! target/release/praxis capture "$scene" "$gallery/$image" "$size" \
    --frame "$frame" "$@" >"$log" 2>&1; then
    printf 'FAIL  %s  full log: %s\n' "$image" "$log"
    tail -n 20 "$log"
    exit 1
  fi
  if [[ -z "$against" ]]; then
    printf 'ok    %s\n' "$image"
  elif [[ ! -f "$against/$image" ]]; then
    printf 'ok    %-34s no earlier image\n' "$image"
  else
    local difference
    difference="$(RUST_LOG=error target/release/praxis difference "$against/$image" \
      "$gallery/$image" "$gallery/${image%.png}-difference.png" 2>&1)" ||
      { printf 'FAIL  %s  %s\n' "$image" "$difference"; exit 1; }
    printf 'ok    %-34s %s\n' "$image" "$difference"
  fi
  # export-scene refuses crowd, whose figures the GPU bends.
  if [[ -n "${PRAXIS_BLENDER:-}" && "$scene" != crowd ]]; then
    reference "$name" "$scene" "$frame"
  fi
}

for frame in "${town_frames[@]}"; do
  capture town town "$frame"
done
for scene in helmet pbr-sweep uv-grid normal-tangent far-coplanar orrery \
  minification texture-wall shadows; do
  capture "$scene" "$scene" "$golden_frame"
done
for form in proxies refit; do
  capture "crowd-$form" crowd "$golden_frame" --crowd-form "$form"
done
[[ "$baseline_failures" -eq 0 ]] || exit 1
