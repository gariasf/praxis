#!/usr/bin/env bash
# Copies into the checkout the lavapipe Goldens and Baselines a CI run
# recorded: a run it dispatches on the current branch, or the run given.
#
# Usage: scripts/lavapipe-files.sh [<run id>]
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

newest_recording() {
  local commit=$1
  gh run list --workflow lavapipe-files.yml --commit "$commit" --limit 1 \
    --json databaseId --jq '.[0].databaseId // empty'
}

if [ $# -eq 0 ]; then
  branch=$(git branch --show-current)
  head_commit=$(git rev-parse HEAD)
  pushed_commit=$(git ls-remote origin "refs/heads/$branch" | cut -f1)
  if [ "$pushed_commit" != "$head_commit" ]; then
    echo "push $branch first: CI records the commit origin holds" >&2
    exit 1
  fi
  git fetch --quiet origin main
  if ! git merge-base --is-ancestor origin/main "$head_commit"; then
    echo "rebase $branch onto origin/main first: a PR's run judges its merge with main" >&2
    exit 1
  fi
  earlier_recording=$(newest_recording "$head_commit")
  gh workflow run lavapipe-files.yml --ref "$branch"
  run_id=$earlier_recording
  for attempt in {1..30}; do
    run_id=$(newest_recording "$head_commit")
    [ "$run_id" != "$earlier_recording" ] && break
    sleep 2
  done
  if [ "$run_id" = "$earlier_recording" ]; then
    echo "the dispatched run did not appear within a minute" >&2
    exit 1
  fi
  gh run watch "$run_id" --exit-status --interval 30
else
  run_id=$1
fi

artifacts=$(gh api "repos/{owner}/{repo}/actions/runs/$run_id/artifacts" \
  --jq '.artifacts[].name | select(startswith("lavapipe-"))')
if [ -z "$artifacts" ]; then
  echo "run $run_id holds no lavapipe file"
  exit 0
fi
download_directory=$(mktemp -d)
for artifact in $artifacts; do
  gh run download "$run_id" --name "$artifact" --dir "$download_directory/$artifact"
  cp -R "$download_directory/$artifact/." .
done
rm -r "$download_directory"
git status --short tests/goldens bench/baselines
