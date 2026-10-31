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
# praxis is brighter than Cycles and blue where it is darker.
#
# Usage: [PRAXIS_BLENDER=<blender>] scripts/gallery.sh [<earlier gallery directory>]
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
# The Goldens test's GOLDEN_FRAME and TOWN_FRAMES (tests/goldens.rs).
golden_frame=59
town_frames=(59 300 500 700 900 1200 1500 1800)

[[ -z "$against" || "$against/" != "$(pwd)/$gallery/"* ]] ||
  { echo "copy the earlier gallery out of $gallery first" >&2; exit 1; }
cargo build --release --quiet
rm -rf "$gallery"
mkdir -p "$logs"

# blender <log> <script> <argument>...: a Cycles reference script, its
# output appended to <log>, whose tail a failure prints.
blender() {
  local log="$1" script="$2"
  shift 2
  if ! "$PRAXIS_BLENDER" -b --factory-startup --python-exit-code 1 \
    --python "scripts/cycles-reference/$script" -- "$@" >>"$log" 2>&1; then
    printf 'FAIL  %s  full log: %s\n' "$script" "$log"
    tail -n 20 "$log"
    exit 1
  fi
}

if [[ -n "${PRAXIS_BLENDER:-}" ]]; then
  blender "$logs/calibrate.log" calibrate.py
  blender "$logs/calibrate.log" compare.py ramp
  printf 'ok    %-34s light units and display transform\n' cycles-reference
fi

# reference <image name> <scene> <frame>: the Cycles render of a capture,
# and its difference from it.
reference() {
  local name="$1-frame-$3"
  local export="$gallery/cycles/$name" log="$logs/$name-cycles.log"
  if ! target/release/praxis export-scene "$2" "$export" --frame "$3" >"$log" 2>&1; then
    printf 'FAIL  %s  full log: %s\n' "$name-cycles.png" "$log"
    tail -n 20 "$log"
    exit 1
  fi
  blender "$log" render.py "$export" "$export/cycles.exr" "$size"
  blender "$log" compare.py compare "$gallery/$name.png" "$export/cycles.exr" \
    "$gallery/$name-cycles.png" "$gallery/$name-cycles-difference.png"
  printf 'ok    %-34s %s\n' "$name-cycles.png" "$(grep '^ok *mean difference' "$log" | sed 's/^ok *//')"
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
