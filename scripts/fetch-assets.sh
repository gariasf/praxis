#!/usr/bin/env bash
# Downloads every `fetched` entry of assets/manifest.json to its path under
# assets/, from its pinned URL, and checks its SHA-256. A file already present
# with the right hash is kept. Needs curl, jq and shasum.
set -euo pipefail

assets="$(cd "$(dirname "$0")/../assets" && pwd)"
download="$(mktemp)"
trap 'rm -f "$download"' EXIT

jq -r '.[] | select(.fetched) | "\(.path) \(.source) \(.sha256)"' "$assets/manifest.json" |
  while read -r file source sha256; do
    target="$assets/$file"
    if [[ -f "$target" ]] && shasum -a 256 "$target" | grep -q "^$sha256 "; then
      echo "ok       $file"
      continue
    fi
    mkdir -p "$(dirname "$target")"
    curl --fail --silent --show-error --location --output "$download" "$source"
    actual="$(shasum -a 256 "$download" | cut -d ' ' -f 1)"
    if [[ "$actual" != "$sha256" ]]; then
      echo "error    $file: sha256 is $actual, manifest says $sha256" >&2
      exit 1
    fi
    mv "$download" "$target"
    echo "fetched  $file"
  done
