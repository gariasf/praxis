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
# Usage: scripts/gallery.sh [<earlier gallery directory>]
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
