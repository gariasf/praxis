#!/usr/bin/env bash
# Copies into the checkout the lavapipe Goldens and Baselines a CI run
# recorded: the ones the checkout's would fail on. With no argument it has CI
# record the current branch's pushed head first, in a run that judges nothing,
# so a PR that moves them opens with them committed. With a run's id it copies
# that run's, as after a PR's run that failed on them.
#
# Usage: scripts/lavapipe-files.sh [<run id>]
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

newest_recording() {
  gh run list --workflow ci.yml --event workflow_dispatch --commit "$1" --limit 1 \
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
  earlier_recording=$(newest_recording "$head_commit")
  gh workflow run ci.yml --ref "$branch"
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
  echo "run $run_id recorded no lavapipe file: the checkout's all pass"
  exit 0
fi
download_directory=$(mktemp -d)
for artifact in $artifacts; do
  gh run download "$run_id" --name "$artifact" --dir "$download_directory/$artifact"
  cp -R "$download_directory/$artifact/." .
done
rm -r "$download_directory"
git status --short tests/goldens bench/baselines
